/// Human-readable byte formatting (B/K/M/G/T/P)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}

/// Format bytes with both current and total (e.g., "46.3G/46.5G")
pub fn format_bytes_ratio(current: u64, total: u64) -> String {
    format!("{}/{}", format_bytes(current), format_bytes(total))
}

/// Format rate (bytes per second)
pub fn format_rate(bytes_per_second: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_second))
}

/// Format operations per second
pub fn format_ops_per_second(ops: u64) -> String {
    format!("{}/s", ops)
}

/// Format latency in milliseconds
pub fn format_latency_ms(latency: f64) -> String {
    format!("{:.1}ms", latency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0K");
        assert_eq!(format_bytes(1536), "1.5K");
        assert_eq!(format_bytes(1024 * 1024), "1.0M");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0G");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.0T");
    }

    #[test]
    fn test_format_bytes_ratio() {
        assert_eq!(format_bytes_ratio(1024, 2048), "1.0K/2.0K");
        assert_eq!(
            format_bytes_ratio(46_301_224_960, 49_910_562_816),
            "43.1G/46.5G"
        );
    }

    #[test]
    fn test_format_rate() {
        assert_eq!(format_rate(1024), "1.0K/s");
        assert_eq!(format_rate(1024 * 1024), "1.0M/s");
    }

    #[test]
    fn test_format_ops_per_second() {
        assert_eq!(format_ops_per_second(1000), "1000/s");
        assert_eq!(format_ops_per_second(50), "50/s");
    }

    #[test]
    fn test_format_latency_ms() {
        assert_eq!(format_latency_ms(2.1), "2.1ms");
        assert_eq!(format_latency_ms(0.5), "0.5ms");
    }
}
