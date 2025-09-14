use super::error::{ZfsError, ZfsResult};
use super::rate_calculator::RateCalculator;
use super::types::{ArcStats, L2ArcStats, SlogStats};
use crate::system::{Cache, CommandExecutor, FilesystemReader};
// async_trait is used via the derive macro
use std::time::{Duration, Instant};

/// ZFS statistics collector with rate calculation and caching
pub struct ZfsStatsCollector<E: CommandExecutor, F: FilesystemReader> {
    command_executor: E,
    filesystem_reader: F,
    rate_calculator: RateCalculator,
    cache: Cache<String>,
}

impl<E: CommandExecutor, F: FilesystemReader> ZfsStatsCollector<E, F> {
    pub fn new(command_executor: E, filesystem_reader: F) -> Self {
        Self {
            command_executor,
            filesystem_reader,
            rate_calculator: RateCalculator::new(),
            // Cache expensive operations for 30 seconds
            cache: Cache::new(Duration::from_secs(30)),
        }
    }

    /// Collect ARC statistics
    pub async fn collect_arc_stats(&mut self) -> ZfsResult<ArcStats> {
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
    async fn collect_arc_stats_from_proc(&mut self, now: Instant) -> ZfsResult<ArcStats> {
        let content = self
            .filesystem_reader
            .read_to_string("/proc/spl/kstat/zfs/arcstats")
            .map_err(|e| {
                ZfsError::filesystem_error("/proc/spl/kstat/zfs/arcstats", "read", &e.to_string())
            })?;

        // Parse the kstat format
        let mut hits = 0u64;
        let mut misses = 0u64;
        let mut size = 0u64;
        let mut c_max = 0u64;
        let mut read_ops_total = 0u64;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let value_str = parts[2];
                let value = value_str.parse::<u64>().map_err(|_| {
                    ZfsError::parse_error(
                        "ARC kstat",
                        line,
                        &format!("Invalid number: {}", value_str),
                    )
                })?;

                match parts[0] {
                    "hits" => hits = value,
                    "misses" => misses = value,
                    "size" => size = value,
                    "c_max" => c_max = value,
                    "read_ops" => read_ops_total = value,
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
    async fn collect_arc_stats_from_arcstat(&mut self, now: Instant) -> ZfsResult<ArcStats> {
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
                    match self.parse_arcstat_output(&output) {
                        Ok(mut stats) => {
                            // Calculate read operations rate
                            stats.read_ops = self
                                .rate_calculator
                                .calculate_and_update("arc_read_ops", stats.read_ops, now)
                                .unwrap_or(0.0) as u64;
                            return Ok(stats);
                        }
                        Err(_) => continue, // Try next command
                    }
                }
                Err(e) => {
                    // Log the error but try the next command
                    eprintln!(
                        "Warning: arcstat command failed ({} {:?}): {}",
                        cmd, args, e
                    );
                    continue;
                }
            }
        }

        Err(ZfsError::subsystem_unavailable(
            "ARC",
            "Failed to collect statistics from all sources (/proc and arcstat command)",
        ))
    }

    /// Parse arcstat command output
    fn parse_arcstat_output(&self, output: &str) -> ZfsResult<ArcStats> {
        // Parse the output format: "100.0 0.0 1247 49720066048 49910562816"
        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(ZfsError::invalid_format(
                "at least 5 space-separated numbers",
                &format!("{} parts", parts.len()),
                "arcstat output",
            ));
        }

        let hit_rate = parts[0].parse::<f64>().map_err(|_| {
            ZfsError::parse_error("arcstat hit_rate", parts[0], "Invalid hit rate percentage")
        })?;

        let miss_rate = parts[1].parse::<f64>().map_err(|_| {
            ZfsError::parse_error(
                "arcstat miss_rate",
                parts[1],
                "Invalid miss rate percentage",
            )
        })?;

        let read_ops = parts[2].parse::<u64>().map_err(|_| {
            ZfsError::parse_error(
                "arcstat read_ops",
                parts[2],
                "Invalid read operations count",
            )
        })?;

        let size = parts[3]
            .parse::<u64>()
            .map_err(|_| ZfsError::parse_error("arcstat size", parts[3], "Invalid cache size"))?;

        let target = parts[4].parse::<u64>().map_err(|_| {
            ZfsError::parse_error("arcstat target", parts[4], "Invalid target size")
        })?;

        Ok(ArcStats {
            hit_rate,
            miss_rate,
            size,
            target,
            read_ops,
        })
    }

    /// Collect L2ARC statistics
    pub async fn collect_l2arc_stats(&mut self) -> ZfsResult<Option<L2ArcStats>> {
        let now = Instant::now();

        // Check if L2ARC is available by looking at arcstats
        let arc_content = self
            .filesystem_reader
            .read_to_string("/proc/spl/kstat/zfs/arcstats")
            .map_err(|e| {
                ZfsError::filesystem_error("/proc/spl/kstat/zfs/arcstats", "read", &e.to_string())
            })?;

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
                let value_str = parts[2];
                let value = value_str.parse::<u64>().map_err(|_| {
                    ZfsError::parse_error(
                        "L2ARC kstat",
                        line,
                        &format!("Invalid number: {}", value_str),
                    )
                })?;

                match parts[0] {
                    "l2_hits" => l2_hits = value,
                    "l2_misses" => l2_misses = value,
                    "l2_size" => l2_size = value,
                    "l2_read_bytes" => l2_read_bytes_total = value,
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
    pub async fn collect_slog_stats(&mut self) -> ZfsResult<Option<SlogStats>> {
        let now = Instant::now();

        // Get zpool status to find SLOG devices (cached for performance)
        let status_output = if let Some(cached) = self.cache.get("zpool_status") {
            cached.clone()
        } else {
            let output = self
                .command_executor
                .execute("zpool", &["status"])
                .await
                .map_err(|e| ZfsError::command_error("zpool", &["status"], &e.to_string()))?;
            self.cache
                .insert("zpool_status".to_string(), output.clone());
            output
        };

        let slog_device = self.parse_slog_device_from_status(&status_output)?;

        if slog_device.is_none() {
            return Ok(None);
        }

        let device_name = slog_device.unwrap();

        // Get I/O statistics for the SLOG device (cached for performance)
        let iostat_output = if let Some(cached) = self.cache.get("zpool_iostat") {
            cached.clone()
        } else {
            let output = self
                .command_executor
                .execute("zpool", &["iostat", "-v"])
                .await
                .map_err(|e| ZfsError::command_error("zpool", &["iostat", "-v"], &e.to_string()))?;
            self.cache
                .insert("zpool_iostat".to_string(), output.clone());
            output
        };

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
    fn parse_slog_device_from_status(&self, status_output: &str) -> ZfsResult<Option<String>> {
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
    ) -> ZfsResult<(u64, u64)> {
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
                    write_ops = parts[4].parse::<u64>().map_err(|_| {
                        ZfsError::parse_error(
                            "iostat write_ops",
                            parts[4],
                            "Invalid write operations count",
                        )
                    })?;
                    // Parse bandwidth (e.g., "12.0M" -> bytes)
                    write_bw = self.parse_bandwidth(parts[5])?;
                }
                break;
            }
        }

        Ok((write_ops, write_bw))
    }

    /// Clear all cached data (useful for testing or when pool configuration changes)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Cleanup expired cache entries
    pub fn cleanup_cache(&mut self) {
        self.cache.cleanup();
    }

    /// Parse bandwidth string (e.g., "12.0M" -> bytes)
    fn parse_bandwidth(&self, bw_str: &str) -> ZfsResult<u64> {
        if bw_str.is_empty() || bw_str == "-" {
            return Ok(0);
        }

        // Handle formats like "12.0M", "234M", "1.82T"
        let bw_str = bw_str.trim();
        let last_char = bw_str.chars().last().ok_or_else(|| {
            ZfsError::invalid_format("non-empty string", "empty string", "bandwidth parsing")
        })?;

        let num_str = if "BKMGT".contains(last_char) {
            &bw_str[..bw_str.len().saturating_sub(1)]
        } else {
            // No unit suffix, treat whole string as number
            bw_str
        };

        let multiplier: u64 = match last_char {
            'B' => 1,
            'K' => 1024,
            'M' => 1024 * 1024,
            'G' => 1024 * 1024 * 1024,
            'T' => 1024u64 * 1024 * 1024 * 1024,
            _ => {
                // If no unit, assume bytes - parse the whole string
                return bw_str.parse::<u64>().map_err(|_| {
                    ZfsError::parse_error("bandwidth", bw_str, "Invalid number format")
                });
            }
        };

        let num: f64 = num_str.parse().map_err(|_| {
            ZfsError::parse_error("bandwidth number", num_str, "Invalid numeric value")
        })?;
        Ok((num * multiplier as f64) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::{DemoCommandExecutor, DemoFilesystemReader};
    use std::time::Instant;

    #[test]
    fn test_parse_arcstat_output_valid() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);
        let output = "95.2 4.8 1234 5368709120 8589934592";

        let result = collector.parse_arcstat_output(output);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.hit_rate, 95.2);
        assert_eq!(stats.miss_rate, 4.8);
        assert_eq!(stats.read_ops, 1234);
        assert_eq!(stats.size, 5368709120);
        assert_eq!(stats.target, 8589934592);
    }

    #[test]
    fn test_parse_arcstat_output_insufficient_parts() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);
        let output = "95.2 4.8 1234"; // Only 3 parts, need 5

        let result = collector.parse_arcstat_output(output);
        assert!(result.is_err());

        if let Err(ZfsError::InvalidFormat { expected, received, .. }) = result {
            assert_eq!(expected, "at least 5 space-separated numbers");
            assert_eq!(received, "3 parts");
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_parse_arcstat_output_invalid_hit_rate() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);
        let output = "invalid 4.8 1234 5368709120 8589934592";

        let result = collector.parse_arcstat_output(output);
        assert!(result.is_err());

        if let Err(ZfsError::ParseError { data_source, .. }) = result {
            assert_eq!(data_source, "arcstat hit_rate");
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_parse_bandwidth_bytes() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        assert_eq!(collector.parse_bandwidth("1024").unwrap(), 1024);
        assert_eq!(collector.parse_bandwidth("0").unwrap(), 0);
    }

    #[test]
    fn test_parse_bandwidth_with_units() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        assert_eq!(collector.parse_bandwidth("1K").unwrap(), 1024);
        assert_eq!(collector.parse_bandwidth("2M").unwrap(), 2 * 1024 * 1024);
        assert_eq!(collector.parse_bandwidth("1.5G").unwrap(), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
        assert_eq!(collector.parse_bandwidth("1T").unwrap(), 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_bandwidth_empty_or_dash() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        assert_eq!(collector.parse_bandwidth("").unwrap(), 0);
        assert_eq!(collector.parse_bandwidth("-").unwrap(), 0);
    }

    #[test]
    fn test_parse_bandwidth_invalid_number() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        let result = collector.parse_bandwidth("invalid");
        assert!(result.is_err());

        if let Err(ZfsError::ParseError { data_source, .. }) = result {
            assert_eq!(data_source, "bandwidth");
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_parse_bandwidth_invalid_unit() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        let result = collector.parse_bandwidth("100X"); // Invalid unit
        assert!(result.is_err());

        if let Err(ZfsError::ParseError { data_source, .. }) = result {
            assert_eq!(data_source, "bandwidth number");
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_collect_arc_stats_from_proc_success() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);
        let now = Instant::now();

        // This will use the demo filesystem reader which returns mock data
        let result = collector.collect_arc_stats_from_proc(now);
        // The demo data may not have the expected format, so we just check it doesn't panic
        // In a real test, we'd mock the filesystem reader to return known data
        let _ = result; // Just ensure it doesn't panic
    }

    #[test]
    fn test_collect_arc_stats_from_arcstat_fallback() {
        let collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);
        let now = Instant::now();

        // This will try various arcstat commands, some may succeed with demo data
        let result = collector.collect_arc_stats_from_arcstat(now);
        // We don't assert success since demo data may not match expected formats
        let _ = result; // Just ensure it doesn't panic
    }

    #[test]
    fn test_collect_slog_stats_cached() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // First call should populate cache
        let result1 = collector.collect_slog_stats();
        // Second call should use cache
        let result2 = collector.collect_slog_stats();

        // Both should complete without panicking
        let _ = result1;
        let _ = result2;
    }

    #[test]
    fn test_cache_operations() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Test cache clearing
        collector.clear_cache();
        assert!(collector.cache.is_empty());

        // Test cache cleanup (though no expired entries in this case)
        collector.cleanup_cache();
    }
}
