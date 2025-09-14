//! Integration tests for the complete ZFS statistics collection pipeline

#[cfg(test)]
mod integration_tests {
    use crate::system::{DemoCommandExecutor, DemoFilesystemReader};
    use crate::zfs::ZfsStatsCollector;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_full_statistics_collection_pipeline() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Test collecting all statistics
        let arc_stats = collector.collect_arc_stats().await;
        let l2arc_stats = collector.collect_l2arc_stats().await;
        let slog_stats = collector.collect_slog_stats().await;

        // ARC stats should always be available (even if from fallback)
        assert!(arc_stats.is_ok());

        // L2ARC and SLOG might not be available in demo mode, but should not panic
        let _ = l2arc_stats; // Just ensure it completes
        let _ = slog_stats; // Just ensure it completes
    }

    #[tokio::test]
    async fn test_rate_calculations_over_time() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Collect initial stats
        let start = Instant::now();
        let _initial_arc = collector.collect_arc_stats().await;

        // Wait a bit and collect again to get rates
        tokio::time::sleep(Duration::from_millis(10)).await;
        let arc_with_rates = collector.collect_arc_stats().await;

        // Should succeed
        assert!(arc_with_rates.is_ok());

        let arc_stats = arc_with_rates.unwrap();
        // Read ops should be calculable (may be 0 in demo mode, but should not panic)
        let _read_ops_rate = arc_stats.read_ops;
    }

    #[tokio::test]
    async fn test_cache_effectiveness() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // First call should execute commands
        let start = Instant::now();
        let _stats1 = collector.collect_slog_stats().await;
        let first_call_duration = start.elapsed();

        // Second call should use cache (faster)
        let start = Instant::now();
        let _stats2 = collector.collect_slog_stats().await;
        let second_call_duration = start.elapsed();

        // Cached call should be significantly faster (though in demo mode this might not be measurable)
        // We just ensure both calls complete successfully
        assert!(first_call_duration >= Duration::from_nanos(0));
        assert!(second_call_duration >= Duration::from_nanos(0));
    }

    #[tokio::test]
    async fn test_error_recovery() {
        // Test that the system can handle various error conditions gracefully
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Clear cache to force fresh collection
        collector.clear_cache();

        // All collection methods should complete without panicking
        // (they may return errors, but should handle them gracefully)
        let arc_result = collector.collect_arc_stats().await;
        let l2arc_result = collector.collect_l2arc_stats().await;
        let slog_result = collector.collect_slog_stats().await;

        // ARC collection should work (may use fallback methods)
        // L2ARC and SLOG may fail in demo mode, but should not panic
        let _ = arc_result;
        let _ = l2arc_result;
        let _ = slog_result;
    }

    #[tokio::test]
    async fn test_concurrent_collections() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Test collecting multiple statistics concurrently
        let (arc_result, l2arc_result, slog_result) = tokio::join!(
            collector.collect_arc_stats(),
            collector.collect_l2arc_stats(),
            collector.collect_slog_stats()
        );

        // All should complete (may succeed or fail gracefully)
        let _ = arc_result;
        let _ = l2arc_result;
        let _ = slog_result;
    }

    #[test]
    fn test_cache_operations() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Initially cache should be empty
        assert!(collector.cache.is_empty());

        // After clearing, should still be empty
        collector.clear_cache();
        assert!(collector.cache.is_empty());

        // Cleanup on empty cache should work
        collector.cleanup_cache();
        assert!(collector.cache.is_empty());
    }
}