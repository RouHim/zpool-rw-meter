/// ARC (Adaptive Replacement Cache) statistics
#[derive(Debug, Clone)]
pub struct ArcStats {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub size: u64,     // Current cache size in bytes
    pub target: u64,   // Target cache size in bytes
    pub read_ops: u64, // Read operations per second
}

/// L2ARC (Level 2 ARC) statistics
#[derive(Debug, Clone)]
pub struct L2ArcStats {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub size: u64,       // Cache size in bytes
    pub read_bytes: u64, // Bytes read per second
    pub total_ops: u64,  // Total operations per second
}

/// SLOG (Separate Intent Log) statistics
#[derive(Debug, Clone)]
pub struct SlogStats {
    pub device: String,   // Device identifier (e.g., "mirror-1")
    pub write_ops: u64,   // Write operations per second
    pub write_bw: u64,    // Write bandwidth in bytes per second
    pub utilization: f64, // Device utilization percentage
    pub latency: f64,     // Average latency in milliseconds
}

/// Overall cache performance status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheStatus {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl CacheStatus {
    /// Determine status based on hit rate percentage
    pub fn from_hit_rate(hit_rate: f64) -> Self {
        if hit_rate >= 85.0 {
            CacheStatus::Excellent
        } else if hit_rate >= 70.0 {
            CacheStatus::Good
        } else if hit_rate >= 50.0 {
            CacheStatus::Fair
        } else {
            CacheStatus::Poor
        }
    }
}

impl std::fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CacheStatus::Excellent => write!(f, "Excellent"),
            CacheStatus::Good => write!(f, "Good"),
            CacheStatus::Fair => write!(f, "Fair"),
            CacheStatus::Poor => write!(f, "Poor"),
        }
    }
}
