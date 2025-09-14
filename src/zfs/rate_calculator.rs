use std::collections::HashMap;
use std::time::Instant;

/// Tracks metrics over time to calculate rates (operations per second)
#[derive(Debug)]
pub struct RateCalculator {
    previous_values: HashMap<String, u64>,
    previous_timestamps: HashMap<String, Instant>,
}

impl RateCalculator {
    pub fn new() -> Self {
        Self {
            previous_values: HashMap::new(),
            previous_timestamps: HashMap::new(),
        }
    }

    /// Calculate rate for a metric (value per second)
    /// Returns None for the first measurement (no previous value to compare)
    pub fn calculate_rate(
        &mut self,
        key: &str,
        current_value: u64,
        current_time: Instant,
    ) -> Option<f64> {
        if let (Some(prev_value), Some(prev_time)) = (
            self.previous_values.get(key),
            self.previous_timestamps.get(key),
        ) {
            let value_delta = current_value.saturating_sub(*prev_value);
            let time_delta = current_time.duration_since(*prev_time);

            if time_delta.as_secs_f64() > 0.0 {
                let rate = value_delta as f64 / time_delta.as_secs_f64();
                Some(rate)
            } else {
                // Time hasn't changed, return 0 rate
                Some(0.0)
            }
        } else {
            // First measurement, store and return None
            None
        }
    }

    /// Update the stored values for a metric
    pub fn update(&mut self, key: &str, value: u64, timestamp: Instant) {
        self.previous_values.insert(key.to_string(), value);
        self.previous_timestamps.insert(key.to_string(), timestamp);
    }

    /// Calculate rate and update in one operation
    /// Returns the calculated rate (or None for first measurement)
    pub fn calculate_and_update(
        &mut self,
        key: &str,
        current_value: u64,
        current_time: Instant,
    ) -> Option<f64> {
        let rate = self.calculate_rate(key, current_value, current_time);
        self.update(key, current_value, current_time);
        rate
    }


}

impl Default for RateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_first_measurement_returns_none() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        let rate = calculator.calculate_rate("test", 100, now);
        assert!(rate.is_none());
    }

    #[test]
    fn test_rate_calculation() {
        let mut calculator = RateCalculator::new();
        let start = Instant::now();

        // First measurement
        calculator.update("ops", 1000, start);
        thread::sleep(Duration::from_millis(100));

        // Second measurement
        let now = Instant::now();
        let rate = calculator.calculate_rate("ops", 1100, now).unwrap();

        // Should be approximately 1000 ops per second (100 ops / 0.1 second)
        assert!((rate - 1000.0).abs() < 200.0); // Allow reasonable tolerance for timing
    }

    #[test]
    fn test_calculate_and_update() {
        let mut calculator = RateCalculator::new();
        let start = Instant::now();

        // First call should return None
        let rate1 = calculator.calculate_and_update("test", 100, start);
        assert!(rate1.is_none());

        thread::sleep(Duration::from_millis(50));

        // Second call should return a rate
        let now = Instant::now();
        let rate2 = calculator.calculate_and_update("test", 150, now).unwrap();

        // Should be approximately 1000 ops per second (50 ops / 0.05 seconds)
        assert!((rate2 - 1000.0).abs() < 100.0);
    }

    #[test]
    fn test_zero_time_delta() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        calculator.update("test", 100, now);
        let rate = calculator.calculate_rate("test", 200, now).unwrap();

        // Should return 0 when time hasn't changed
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_different_keys() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Different keys should be tracked independently
        calculator.update("key1", 100, now);
        calculator.update("key2", 200, now);

        let rate1 = calculator
            .calculate_rate("key1", 150, now + Duration::from_secs(1))
            .unwrap();
        let rate2 = calculator
            .calculate_rate("key2", 250, now + Duration::from_secs(1))
            .unwrap();

        assert_eq!(rate1, 50.0);
        assert_eq!(rate2, 50.0);
    }

    #[test]
    fn test_zero_rate_calculation() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Same value should result in zero rate
        calculator.update("test", 100, now);
        let rate = calculator
            .calculate_rate("test", 100, now + Duration::from_secs(1))
            .unwrap();

        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_decreasing_rate_calculation() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Decreasing value should result in zero rate (due to saturating_sub protection)
        calculator.update("test", 200, now);
        let rate = calculator
            .calculate_rate("test", 100, now + Duration::from_secs(1))
            .unwrap();

        // saturating_sub prevents negative rates, so we get 0 instead of -100
        assert_eq!(rate, 0.0);
    }







    #[test]
    fn test_precision_with_small_deltas() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Test with very small time deltas (microseconds)
        calculator.update("precise", 1000, now);
        let later = now + Duration::from_micros(500); // 0.0005 seconds
        let rate = calculator.calculate_rate("precise", 1001, later).unwrap();

        // Should be 2 ops per second (1 op / 0.0005 seconds)
        assert!((rate - 2000.0).abs() < 100.0); // Allow some tolerance
    }

    #[test]
    fn test_very_large_rates() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Test with very large value changes
        calculator.update("bandwidth", 0, now);
        let rate = calculator
            .calculate_rate("bandwidth", 1_000_000_000, now + Duration::from_millis(1))
            .unwrap();

        // Should be 1e12 bytes per second
        assert!((rate - 1_000_000_000_000.0).abs() < 1_000_000.0);
    }

    #[test]
    fn test_rate_calculation_with_fractional_seconds() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        calculator.update("test", 0, now);

        // Test with 0.5 seconds
        let half_second = now + Duration::from_millis(500);
        let rate = calculator.calculate_rate("test", 100, half_second).unwrap();

        // Should be 200 ops per second (100 ops / 0.5 seconds)
        assert!((rate - 200.0).abs() < 10.0);
    }

    #[test]
    fn test_calculate_rate_without_update() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Manually update without using calculate_and_update
        calculator.update("manual", 100, now);

        // Calculate rate without updating
        let later = now + Duration::from_secs(2);
        let rate = calculator.calculate_rate("manual", 300, later).unwrap();

        // Should be 100 ops per second (200 ops / 2 seconds)
        assert!((rate - 100.0).abs() < 5.0);

        // Value should still be 100 (not updated)
        assert_eq!(*calculator.previous_values.get("manual").unwrap(), 100);
    }

    #[test]
    fn test_update_overwrites_previous_values() {
        let mut calculator = RateCalculator::new();
        let time1 = Instant::now();
        let time2 = time1 + Duration::from_secs(1);

        // First update
        calculator.update("test", 100, time1);
        assert_eq!(*calculator.previous_values.get("test").unwrap(), 100);
        assert_eq!(*calculator.previous_timestamps.get("test").unwrap(), time1);

        // Second update should overwrite
        calculator.update("test", 200, time2);
        assert_eq!(*calculator.previous_values.get("test").unwrap(), 200);
        assert_eq!(*calculator.previous_timestamps.get("test").unwrap(), time2);
    }

    #[test]
    fn test_empty_key_handling() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Empty string should work as a valid key
        let rate = calculator.calculate_and_update("", 100, now);
        assert!(rate.is_none());

        let rate2 = calculator
            .calculate_and_update("", 200, now + Duration::from_secs(1))
            .unwrap();
        assert_eq!(rate2, 100.0);
    }

    #[test]
    fn test_unicode_keys() {
        let mut calculator = RateCalculator::new();
        let now = Instant::now();

        // Unicode keys should work
        let key = "测试_key_🚀";
        let rate = calculator.calculate_and_update(key, 50, now);
        assert!(rate.is_none());

        let rate2 = calculator
            .calculate_and_update(key, 100, now + Duration::from_secs(1))
            .unwrap();
        assert_eq!(rate2, 50.0);
    }
}
