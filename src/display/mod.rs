//! Display module for terminal output and formatting

pub mod formatter;
pub mod progress;
pub mod terminal;

// Re-export commonly used items
pub use formatter::{
    format_bytes, format_bytes_ratio, format_latency_ms, format_ops_per_second, format_rate,
};
pub use progress::ProgressBar;
pub use terminal::Terminal;
