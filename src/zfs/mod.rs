//! ZFS statistics collection and data structures

pub mod pools;
pub mod rate_calculator;
pub mod stats;
pub mod types;

// Re-export commonly used items
pub use stats::ZfsStatsCollector;
pub use types::{ArcStats, CacheStatus, L2ArcStats, SlogStats};
