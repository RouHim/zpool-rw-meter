use super::rate_calculator::RateCalculator;
use super::types::{ArcStats, L2ArcStats, SlogStats};
use crate::system::{CommandExecutor, FilesystemReader};
// async_trait is used via the derive macro
use std::error::Error;
use std::time::Instant;

/// ZFS statistics collector with rate calculation
pub struct ZfsStatsCollector<E: CommandExecutor, F: FilesystemReader> {
    command_executor: E,
    filesystem_reader: F,
    rate_calculator: RateCalculator,
}

impl<E: CommandExecutor, F: FilesystemReader> ZfsStatsCollector<E, F> {
    pub fn new(command_executor: E, filesystem_reader: F) -> Self {
        Self {
            command_executor,
            filesystem_reader,
            rate_calculator: RateCalculator::new(),
        }
    }

    /// Collect ARC statistics
    pub async fn collect_arc_stats(&mut self) -> Result<ArcStats, Box<dyn Error>> {
        let now = Instant::now();

        // Try to get ARC stats from /proc/spl/kstat/zfs/arcstats first
        match self.collect_arc_stats_from_proc(now).await {
            Ok(stats) => Ok(stats),
            Err(_) => {
                // Fallback to arcstat command
                self.collect_arc_stats_from_arcstat(now).await
            }
        }
    }

    /// Collect ARC statistics from /proc/spl/kstat/zfs/arcstats
    async fn collect_arc_stats_from_proc(
        &mut self,
        now: Instant,
    ) -> Result<ArcStats, Box<dyn Error>> {
        let content = self
            .filesystem_reader
            .read_to_string("/proc/spl/kstat/zfs/arcstats")?;

        // Parse the kstat format
        let mut hits = 0u64;
        let mut misses = 0u64;
        let mut size = 0u64;
        let mut c_max = 0u64;
        let mut read_ops_total = 0u64;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                match parts[0] {
                    "hits" => hits = parts[2].parse().unwrap_or(0),
                    "misses" => misses = parts[2].parse().unwrap_or(0),
                    "size" => size = parts[2].parse().unwrap_or(0),
                    "c_max" => c_max = parts[2].parse().unwrap_or(0),
                    "read_ops" => read_ops_total = parts[2].parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        // Calculate hit/miss rates
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let miss_rate = if total > 0 {
            (misses as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Calculate read operations per second
        let read_ops_rate = self
            .rate_calculator
            .calculate_and_update("arc_read_ops", read_ops_total, now)
            .unwrap_or(0.0);

        Ok(ArcStats {
            hit_rate,
            miss_rate,
            size,
            target: c_max,
            read_ops: read_ops_rate as u64,
        })
    }

    /// Collect ARC statistics from arcstat command (fallback)
    async fn collect_arc_stats_from_arcstat(
        &mut self,
        now: Instant,
    ) -> Result<ArcStats, Box<dyn Error>> {
        // Try different arcstat command formats
        let commands = vec![
            ("arcstat", vec!["-f", "hit%,miss%,read,arcsz,c", "1", "1"]),
            ("arcstat", vec!["1", "1"]),
            ("echo", vec!["|", "arcstat"]),
        ];

        for (cmd, args) in commands {
            match self
                .command_executor
                .execute_with_timeout(cmd, &args, std::time::Duration::from_secs(3))
                .await
            {
                Ok(output) => {
                    if let Some(mut stats) = self.parse_arcstat_output(&output) {
                        // Calculate read operations rate
                        stats.read_ops = self
                            .rate_calculator
                            .calculate_and_update("arc_read_ops", stats.read_ops, now)
                            .unwrap_or(0.0) as u64;
                        return Ok(stats);
                    }
                }
                Err(_) => continue,
            }
        }

        Err("Failed to collect ARC statistics from all sources".into())
    }

    /// Parse arcstat command output
    fn parse_arcstat_output(&self, output: &str) -> Option<ArcStats> {
        // Parse the output format: "100.0 0.0 1247 49720066048 49910562816"
        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.len() >= 5 {
            let hit_rate = parts[0].parse().ok()?;
            let miss_rate = parts[1].parse().ok()?;
            let read_ops = parts[2].parse().ok()?;
            let size = parts[3].parse().ok()?;
            let target = parts[4].parse().ok()?;

            Some(ArcStats {
                hit_rate,
                miss_rate,
                size,
                target,
                read_ops,
            })
        } else {
            None
        }
    }

    /// Collect L2ARC statistics
    pub async fn collect_l2arc_stats(&mut self) -> Result<Option<L2ArcStats>, Box<dyn Error>> {
        let now = Instant::now();

        // Check if L2ARC is available by looking at arcstats
        let arc_content = self
            .filesystem_reader
            .read_to_string("/proc/spl/kstat/zfs/arcstats")?;

        // Check for L2ARC presence
        let has_l2arc = arc_content.lines().any(|line| line.starts_with("l2_size"));

        if !has_l2arc {
            return Ok(None);
        }

        // Parse L2ARC statistics from arcstats
        let mut l2_hits = 0u64;
        let mut l2_misses = 0u64;
        let mut l2_size = 0u64;
        let mut l2_read_bytes_total = 0u64;

        for line in arc_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                match parts[0] {
                    "l2_hits" => l2_hits = parts[2].parse().unwrap_or(0),
                    "l2_misses" => l2_misses = parts[2].parse().unwrap_or(0),
                    "l2_size" => l2_size = parts[2].parse().unwrap_or(0),
                    "l2_read_bytes" => l2_read_bytes_total = parts[2].parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        let total_l2_ops = l2_hits + l2_misses;
        let l2_hit_rate = if total_l2_ops > 0 {
            (l2_hits as f64 / total_l2_ops as f64) * 100.0
        } else {
            0.0
        };
        let l2_miss_rate = if total_l2_ops > 0 {
            (l2_misses as f64 / total_l2_ops as f64) * 100.0
        } else {
            0.0
        };

        // Calculate rates for operations and read bandwidth
        let l2_ops_rate = self
            .rate_calculator
            .calculate_and_update("l2_total_ops", total_l2_ops, now)
            .unwrap_or(0.0);
        let l2_read_bytes_rate = self
            .rate_calculator
            .calculate_and_update("l2_read_bytes", l2_read_bytes_total, now)
            .unwrap_or(0.0);

        Ok(Some(L2ArcStats {
            hit_rate: l2_hit_rate,
            miss_rate: l2_miss_rate,
            size: l2_size,
            read_bytes: l2_read_bytes_rate as u64,
            total_ops: l2_ops_rate as u64,
        }))
    }

    /// Collect SLOG statistics
    pub async fn collect_slog_stats(&mut self) -> Result<Option<SlogStats>, Box<dyn Error>> {
        let now = Instant::now();

        // Get zpool status to find SLOG devices
        let status_output = self.command_executor.execute("zpool", &["status"]).await?;
        let slog_device = self.parse_slog_device_from_status(&status_output)?;

        if slog_device.is_none() {
            return Ok(None);
        }

        let device_name = slog_device.unwrap();

        // Get I/O statistics for the SLOG device
        let iostat_output = self
            .command_executor
            .execute("zpool", &["iostat", "-v"])
            .await?;
        let (write_ops_total, write_bw_total) =
            self.parse_slog_stats_from_iostat(&iostat_output, &device_name)?;

        // Calculate rates
        let write_ops_rate = self
            .rate_calculator
            .calculate_and_update(
                &format!("slog_{}_write_ops", device_name),
                write_ops_total,
                now,
            )
            .unwrap_or(0.0);
        let write_bw_rate = self
            .rate_calculator
            .calculate_and_update(
                &format!("slog_{}_write_bw", device_name),
                write_bw_total,
                now,
            )
            .unwrap_or(0.0);

        Ok(Some(SlogStats {
            device: device_name,
            write_ops: write_ops_rate as u64,
            write_bw: write_bw_rate as u64,
            utilization: 0.0, // TODO: Calculate utilization
            latency: 0.0,     // TODO: Calculate latency
        }))
    }

    /// Parse SLOG device from zpool status output
    fn parse_slog_device_from_status(
        &self,
        status_output: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let mut in_logs_section = false;

        for line in status_output.lines() {
            let line = line.trim();

            if line.starts_with("logs") {
                in_logs_section = true;
                continue;
            }

            if in_logs_section {
                if line.is_empty() {
                    continue;
                }
                // Look for mirror or single device lines
                if line.starts_with("mirror-") || line.contains("ONLINE") {
                    // Extract device name from mirror-X pattern
                    if let Some(mirror_match) = line.split_whitespace().next() {
                        if mirror_match.starts_with("mirror-") {
                            return Ok(Some(mirror_match.to_string()));
                        }
                    }
                }
                // Exit logs section when we hit another section
                if line.starts_with(char::is_alphabetic)
                    && !line.contains("ONLINE")
                    && !line.starts_with("mirror-")
                {
                    break;
                }
            }
        }

        Ok(None)
    }

    /// Parse SLOG statistics from zpool iostat output
    fn parse_slog_stats_from_iostat(
        &self,
        iostat_output: &str,
        device_name: &str,
    ) -> Result<(u64, u64), Box<dyn Error>> {
        let mut in_device_section = false;
        let mut write_ops = 0u64;
        let mut write_bw = 0u64;

        for line in iostat_output.lines() {
            let line = line.trim();

            // Look for the device section
            if line.contains(device_name) {
                in_device_section = true;
                continue;
            }

            if in_device_section && !line.is_empty() && !line.starts_with('-') {
                // Parse the I/O stats line: "0B 1.82T      0     23      0  12.0M"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    write_ops = parts[4].parse().unwrap_or(0);
                    // Parse bandwidth (e.g., "12.0M" -> bytes)
                    write_bw = self.parse_bandwidth(parts[5])?;
                }
                break;
            }
        }

        Ok((write_ops, write_bw))
    }

    /// Parse bandwidth string (e.g., "12.0M" -> bytes)
    fn parse_bandwidth(&self, bw_str: &str) -> Result<u64, Box<dyn Error>> {
        if bw_str.is_empty() || bw_str == "-" {
            return Ok(0);
        }

        // Handle formats like "12.0M", "234M", "1.82T"
        let bw_str = bw_str.trim();
        let last_char = bw_str.chars().last().unwrap_or('B');
        let num_str = &bw_str[..bw_str.len().saturating_sub(1)];

        let multiplier: u64 = match last_char {
            'B' => 1,
            'K' => 1024,
            'M' => 1024 * 1024,
            'G' => 1024 * 1024 * 1024,
            'T' => 1024u64 * 1024 * 1024 * 1024,
            _ => {
                // If no unit, assume bytes
                return bw_str
                    .parse()
                    .map_err(|_| "Invalid bandwidth format".into());
            }
        };

        let num: f64 = num_str.parse().unwrap_or(0.0);
        Ok((num * multiplier as f64) as u64)
    }
}
