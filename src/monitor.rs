// Demo data imports are no longer needed since we parse from files
use crate::display::{
    ProgressBar, Terminal, format_bytes, format_bytes_ratio, format_latency_ms,
    format_ops_per_second, format_rate,
};
use crate::system::commands::{DemoCommandExecutor, RealCommandExecutor};
use crate::system::filesystem::{DemoFilesystemReader, RealFilesystemReader};
use crate::zfs::{CacheStatus, ZfsStatsCollector};
use std::error::Error;
use std::fmt;
use std::io::Write;

/// Main monitoring loop and display coordination
pub async fn run(demo_mode: bool) -> Result<(), Box<dyn Error>> {
    run_with_args(demo_mode, None, 2).await
}

/// Main monitoring loop with arguments
pub async fn run_with_args(
    demo_mode: bool,
    pool: Option<&str>,
    interval: u32,
) -> Result<(), Box<dyn Error>> {
    let terminal = Terminal::new();
    let pool_name = pool.unwrap_or("data"); // Default pool

    if demo_mode {
        run_demo_mode(&terminal, pool_name, interval).await
    } else {
        run_live_mode(&terminal, pool_name, interval).await
    }
}

async fn run_demo_mode(
    terminal: &Terminal,
    pool_name: &str,
    interval: u32,
) -> Result<(), Box<dyn Error>> {
    let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

    // Set up signal handler for Ctrl+C
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        let _ = tx_clone.send(()).await;
    });

    loop {
        tokio::select! {
            _ = rx.recv() => {
                // Ctrl+C received, exit gracefully
                terminal.show_cursor()?;
                println!("\nMonitoring stopped.");
                return Ok(());
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)) => {
                // Time to refresh
            }
        }

        // Clear screen and hide cursor for flicker-free updates
        terminal.clear_screen()?;
        terminal.hide_cursor()?;

        // Collect stats
        let arc_stats = collector.collect_arc_stats().await?;
        let l2arc_stats = collector.collect_l2arc_stats().await?;
        let slog_stats = collector.collect_slog_stats().await?;

        // Display all sections
        display_header(terminal, pool_name, interval)?;
        display_arc_section(terminal, &arc_stats)?;
        if let Some(l2arc) = l2arc_stats {
            display_l2arc_section(terminal, &l2arc)?;
        }
        if let Some(slog) = slog_stats {
            display_slog_section(terminal, &slog)?;
        }
        display_footer(terminal)?;

        // Flush output
        std::io::stdout().flush()?;
    }
}

async fn run_live_mode(
    terminal: &Terminal,
    pool_name: &str,
    interval: u32,
) -> Result<(), Box<dyn Error>> {
    let mut collector = ZfsStatsCollector::new(RealCommandExecutor, RealFilesystemReader);

    // Set up signal handler for Ctrl+C
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        let _ = tx_clone.send(()).await;
    });

    loop {
        tokio::select! {
            _ = rx.recv() => {
                // Ctrl+C received, exit gracefully
                terminal.show_cursor()?;
                println!("\nMonitoring stopped.");
                return Ok(());
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)) => {
                // Time to refresh
            }
        }

        // Clear screen and hide cursor for flicker-free updates
        terminal.clear_screen()?;
        terminal.hide_cursor()?;

        // Collect stats
        let arc_stats = collector.collect_arc_stats().await?;
        let l2arc_stats = collector.collect_l2arc_stats().await?;
        let slog_stats = collector.collect_slog_stats().await?;

        // Display all sections
        display_header(terminal, pool_name, interval)?;
        display_arc_section(terminal, &arc_stats)?;
        if let Some(l2arc) = l2arc_stats {
            display_l2arc_section(terminal, &l2arc)?;
        }
        if let Some(slog) = slog_stats {
            display_slog_section(terminal, &slog)?;
        }
        display_footer(terminal)?;

        // Flush output
        std::io::stdout().flush()?;
    }
}

fn display_header(_terminal: &Terminal, pool: &str, interval: u32) -> Result<(), Box<dyn Error>> {
    println!("{:=^80}", " 🔍 ZFS Cache Performance Monitor ");
    println!(
        "Pool: {} | Refresh: {}s | Time: {}",
        pool,
        interval,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!();
    Ok(())
}

fn display_arc_section(
    _terminal: &Terminal,
    arc: &crate::zfs::ArcStats,
) -> Result<(), Box<dyn Error>> {
    println!("📊 ARC (Primary RAM Cache)");
    let progress_bar = ProgressBar::new(20);
    let usage_percent = (arc.size as f64 / arc.target as f64) * 100.0;

    println!(
        "    Hit Rate:    {}",
        progress_bar.render(
            arc.hit_rate,
            Some(&format!(
                "{} ({})",
                arc.hit_rate,
                CacheStatus::from_hit_rate(arc.hit_rate)
            ))
        )
    );
    println!(
        "    Cache Size:  {}",
        progress_bar.render(
            usage_percent,
            Some(&format_bytes_ratio(arc.size, arc.target))
        )
    );
    println!("    Read Ops:    {}", format_ops_per_second(arc.read_ops));
    println!();
    Ok(())
}

fn display_l2arc_section(
    _terminal: &Terminal,
    l2arc: &crate::zfs::L2ArcStats,
) -> Result<(), Box<dyn Error>> {
    println!("💾 L2ARC (Secondary SSD Cache)");
    let progress_bar = ProgressBar::new(20);

    println!(
        "    Hit Rate:    {}",
        progress_bar.render(
            l2arc.hit_rate,
            Some(&format!(
                "{} ({})",
                l2arc.hit_rate,
                CacheStatus::from_hit_rate(l2arc.hit_rate)
            ))
        )
    );
    println!("    Cache Size:  {}", format_bytes(l2arc.size));
    println!("    Read Rate:   {}", format_rate(l2arc.read_bytes));
    println!(
        "    Operations:  {}",
        format_ops_per_second(l2arc.total_ops)
    );
    println!();
    Ok(())
}

fn display_slog_section(
    _terminal: &Terminal,
    slog: &crate::zfs::SlogStats,
) -> Result<(), Box<dyn Error>> {
    println!("🟡 SLOG (Synchronous Write Log)");
    let progress_bar = ProgressBar::new(20);

    println!("    Device:      {}", slog.device);
    println!(
        "    Utilization: {}",
        progress_bar.render(
            slog.utilization,
            Some(&format!(
                "{} ({})",
                slog.utilization,
                CacheStatus::from_hit_rate(100.0 - slog.utilization)
            ))
        )
    );
    println!("    Write Ops:   {}", format_ops_per_second(slog.write_ops));
    println!("    Write Rate:  {}", format_rate(slog.write_bw));
    println!("    Latency:     {}", format_latency_ms(slog.latency));
    println!();
    Ok(())
}

fn display_footer(_terminal: &Terminal) -> Result<(), Box<dyn Error>> {
    println!("{:=^80}", "");
    println!("Press Ctrl+C to exit | Data refreshes every 2s");
    Ok(())
}

/// Error types for the monitor
#[derive(Debug)]
pub enum MonitorError {
    ZfsUnavailable,
    PoolNotFound(String),
    InvalidInterval(String),
    SystemError(String),
    ParseError(String),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MonitorError::ZfsUnavailable => write!(f, "ZFS is not available on this system"),
            MonitorError::PoolNotFound(pool) => write!(f, "Pool '{}' not found", pool),
            MonitorError::InvalidInterval(interval) => write!(f, "Invalid interval: {}", interval),
            MonitorError::SystemError(msg) => write!(f, "System error: {}", msg),
            MonitorError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for MonitorError {}
