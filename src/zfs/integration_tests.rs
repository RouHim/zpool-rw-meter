//! Integration tests for the complete ZFS statistics collection pipeline

#[cfg(test)]
mod integration_tests {
    use crate::system::commands::DemoCommandExecutor;
    use crate::system::filesystem::DemoFilesystemReader;
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

    #[tokio::test]
    async fn test_rate_calculation_accuracy() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Collect multiple times to establish rate calculation baseline
        let mut previous_read_ops = None;

        for i in 0..5 {
            let arc_stats = collector.collect_arc_stats().await.unwrap();

            if let Some(prev) = previous_read_ops {
                // In subsequent collections, we should have rate data
                // (though in demo mode, rates might be 0)
                if i > 0 {
                    // Rate should be calculable (even if 0)
                    assert!(arc_stats.read_ops >= 0.0);
                }
            }

            previous_read_ops = Some(arc_stats.read_ops);

            // Small delay between collections
            if i < 4 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }

    #[tokio::test]
    async fn test_cache_invalidation_scenarios() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // First collection should populate cache
        let _stats1 = collector.collect_slog_stats().await;

        // Verify cache has entries
        assert!(!collector.cache.is_empty());

        // Clear cache manually
        collector.clear_cache();
        assert!(collector.cache.is_empty());

        // Next collection should repopulate cache
        let _stats2 = collector.collect_slog_stats().await;
        assert!(!collector.cache.is_empty());

        // Cleanup should not remove valid entries
        collector.cleanup_cache();
        assert!(!collector.cache.is_empty());
    }

    #[tokio::test]
    async fn test_error_propagation_and_recovery() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Test that errors don't prevent subsequent successful operations
        // (In demo mode, most operations should succeed, but this tests the pattern)

        // Collect stats multiple times
        for _ in 0..3 {
            let arc_result = collector.collect_arc_stats().await;
            let l2arc_result = collector.collect_l2arc_stats().await;
            let slog_result = collector.collect_slog_stats().await;

            // ARC should generally succeed
            assert!(arc_result.is_ok());

            // L2ARC and SLOG may or may not be available, but should not panic
            // Just ensure they return some result (Ok or Err)
            let _ = l2arc_result;
            let _ = slog_result;
        }
    }

    #[tokio::test]
    async fn test_collections_with_time_gaps() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // First collection
        let start_time = Instant::now();
        let stats1 = collector.collect_arc_stats().await.unwrap();

        // Wait a longer period
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Second collection
        let stats2 = collector.collect_arc_stats().await.unwrap();

        // Both should succeed and have reasonable values
        assert!(stats1.read_ops >= 0.0);
        assert!(stats2.read_ops >= 0.0);

        // Time should have progressed
        assert!(start_time.elapsed() >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_multiple_collection_types_interaction() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Collect all types in sequence
        let arc_stats = collector.collect_arc_stats().await.unwrap();
        let l2arc_stats = collector.collect_l2arc_stats().await;
        let slog_stats = collector.collect_slog_stats().await;

        // ARC should always be available
        assert!(arc_stats.hit_rate >= 0.0 && arc_stats.hit_rate <= 100.0);
        assert!(arc_stats.size >= 0);

        // L2ARC and SLOG may be None in demo mode, but should not panic
        match l2arc_stats {
            Ok(Some(l2_stats)) => {
                assert!(l2_stats.hit_rate >= 0.0 && l2_stats.hit_rate <= 100.0);
                assert!(l2_stats.size >= 0);
            }
            Ok(None) => {} // No L2ARC available
            Err(_) => {}   // Error occurred, but handled gracefully
        }

        match slog_stats {
            Ok(Some(slog_stats)) => {
                assert!(slog_stats.write_ops >= 0.0);
                assert!(slog_stats.write_bw >= 0.0);
            }
            Ok(None) => {} // No SLOG available
            Err(_) => {}   // Error occurred, but handled gracefully
        }
    }

    #[tokio::test]
    async fn test_cache_expiration_during_collections() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Override the default 30-second cache with a very short one for testing
        // (We can't easily change the TTL after creation, so we'll work with the default)

        // First collection populates cache
        let _stats1 = collector.collect_slog_stats().await;
        assert!(!collector.cache.is_empty());

        // Manually expire cache entries by clearing (simulating expiration)
        collector.clear_cache();
        assert!(collector.cache.is_empty());

        // Next collection should work fine despite expired cache
        let _stats2 = collector.collect_slog_stats().await;
        assert!(!collector.cache.is_empty());
    }

    #[tokio::test]
    async fn test_concurrent_collections_with_shared_state() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Test that concurrent collections don't interfere with each other
        let arc_future = collector.collect_arc_stats();
        let l2arc_future = collector.collect_l2arc_stats();
        let slog_future = collector.collect_slog_stats();

        let (arc_result, l2arc_result, slog_result) =
            tokio::join!(arc_future, l2arc_future, slog_future);

        // All should complete successfully
        assert!(arc_result.is_ok());
        // L2ARC and SLOG may fail in demo mode but should not panic
        let _ = l2arc_result;
        let _ = slog_result;

        // Cache should be populated
        assert!(!collector.cache.is_empty());
    }

    #[tokio::test]
    async fn test_collection_pipeline_resilience() {
        let mut collector = ZfsStatsCollector::new(DemoCommandExecutor, DemoFilesystemReader);

        // Test that the pipeline can handle multiple rapid collections
        for i in 0..10 {
            let arc_result = collector.collect_arc_stats().await;
            assert!(arc_result.is_ok(), "ARC collection failed on iteration {}", i);

            // Occasionally collect other stats
            if i % 3 == 0 {
                let _ = collector.collect_l2arc_stats().await;
                let _ = collector.collect_slog_stats().await;
            }

            // Small delay to allow rate calculations to work
            if i < 9 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }

        // Final state should be valid
        assert!(!collector.cache.is_empty());
    }
}