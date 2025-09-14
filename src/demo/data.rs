use crate::zfs::types::{ArcStats, L2ArcStats, SlogStats};

/// Demo ARC data based on real system: 100% hit rate, 46.3G/46.5G usage
pub const DEMO_ARC_DATA: ArcStats = ArcStats {
    hit_rate: 100.0,
    miss_rate: 0.0,
    size: 49_720_066_048,   // ~46.3G current usage
    target: 49_910_562_816, // ~46.5G target
    read_ops: 1247,
};

/// Demo L2ARC data based on real system: 73.5% hit rate, ~554GB cache
pub const DEMO_L2ARC_DATA: L2ArcStats = L2ArcStats {
    hit_rate: 73.5,
    miss_rate: 26.5,
    size: 594_542_387_200,   // ~554GB L2ARC cache
    read_bytes: 245_760_000, // ~234MB/s read rate
    total_ops: 892,          // 892 operations per second
};

/// Demo SLOG data based on real system: mirror-1 device, 28.7% utilization
pub fn demo_slog_data() -> SlogStats {
    SlogStats {
        device: "mirror-1".to_string(),
        write_ops: 156,
        write_bw: 12_582_912, // ~12MB/s write bandwidth
        utilization: 28.7,
        latency: 2.1,
    }
}

/// Demo zpool status output
pub const DEMO_ZPOOL_STATUS: &str = r#"  pool: data
 state: ONLINE
  scan: scrub repaired 0B in 00:00:02 with 0 errors on Sun Sep 14 16:00:03 2025
config:

	NAME        STATE     READ WRITE CKSUM
	data        ONLINE       0     0     0
	  mirror-0  ONLINE       0     0     0
	    ata-WDC_WD80EMAZ-00WJTA0_9RK3VYJD  ONLINE       0     0     0
	    ata-WDC_WD80EMAZ-00WJTA0_9RK8VYJD  ONLINE       0     0     0
	  mirror-1  ONLINE       0     0     0
	    ata-WDC_WD80EMAZ-00WJTA0_9RKAVYJD  ONLINE       0     0     0
	    ata-WDC_WD80EMAZ-00WJTA0_9RKDVYJD  ONLINE       0     0     0
	logs
	  mirror-1  ONLINE       0     0     0
	    ata-Samsung_SSD_860_EVO_250GB_S3YJNX0N1234567  ONLINE       0     0     0
	    ata-Samsung_SSD_860_EVO_250GB_S3YJNX0N7654321  ONLINE       0     0     0

errors: No known data errors
"#;

/// Demo zpool iostat output
pub const DEMO_ZPOOL_IOSTAT: &str = r#"               capacity     operations     bandwidth
pool        alloc   free   read  write   read  write
----------  -----  -----  -----  -----  -----  -----
data        3.45T  2.55T     47     23   234M  12.0M
logs            -      -      -      -      -      -
  mirror-1     0B  1.82T      0     23      0  12.0M
----------  -----  -----  -----  -----  -----  -----
"#;

/// Demo /proc/spl/kstat/zfs/arcstats content
pub const DEMO_ARCSTATS: &str = r#"7 1 1 91 6144 31927403520 18446744073709551615
name                            type data
hits                            4    18446744073709551615
misses                          4    0
demand_data_hits                4    18446744073709551615
demand_data_misses              4    0
demand_metadata_hits            4    18446744073709551615
demand_metadata_misses          4    0
prefetch_data_hits              4    0
prefetch_data_misses            4    0
prefetch_metadata_hits          4    0
prefetch_metadata_misses        4    0
mru_hits                        4    9223372036854775807
mru_ghost_hits                  4    0
mfu_hits                        4    9223372036854775807
mfu_ghost_hits                  4    0
deleted                         4    0
mutex_miss                      4    0
evict_skip                      4    0
evict_not_enough                4    0
evict_l2_cached                 4    0
evict_l2_eligible               4    0
evict_l2_ineligible             4    0
evict_l2_skip                   4    0
hash_elements                   4    1000
hash_elements_max               4    10000
hash_collisions                 4    0
hash_chains                     4    500
hash_chain_max                  4    10
p                               4    50
c                               4    49720066048
c_min                           4    4194304
c_max                           4    49910562816
size                            4    49720066048
hdr_size                        4    1000000
data_size                       4    48000000000
metadata_size                   4    1700000000
other_size                      4    2006648
anon_size                       4    0
anon_evictable_data             4    0
anon_evictable_metadata         4    0
mru_size                        4    24000000000
mru_evictable_data              4    24000000000
mru_evictable_metadata          4    0
mru_ghost_size                  4    0
mru_ghost_evictable_data        4    0
mru_ghost_evictable_metadata    4    0
mfu_size                        4    24000000000
mfu_evictable_data              4    24000000000
mfu_evictable_metadata          4    0
mfu_ghost_size                  4    0
mfu_ghost_evictable_data        4    0
mfu_ghost_evictable_metadata    4    0
l2_hits                         4    655000
l2_misses                       4    237000
l2_feeds                        4    1000
l2_rw_clash                     4    0
l2_read_bytes                   4    245760000
l2_write_bytes                  4    10000000
l2_writes_sent                  4    1000
l2_writes_done                  4    1000
l2_writes_error                 4    0
l2_writes_lock_retry            4    0
l2_evict_lock_retry             4    0
l2_evict_reading                4    0
l2_evict_l1cached               4    0
l2_free_on_write                4    0
l2_cdata_free_on_write          4    0
l2_abort_lowmem                 4    0
l2_cksum_bad                    4    0
l2_io_error                     4    0
l2_size                         4    594542387200
l2_asize                        4    594542387200
l2_hdr_size                     4    10000000
l2_compress_successes           4    500
l2_compress_zeros               4    200
l2_compress_failures            4    0
l2_write_trylock_fail           4    0
l2_write_passed_head            4    0
l2_write_spa_mismatch           4    0
l2_write_in_l2                  4    0
l2_write_io_in_progress         4    0
l2_write_not_cacheable          4    0
l2_write_full                    4    0
l2_write_buffer_iter            4    0
l2_write_pios                    4    0
l2_write_buffer_bytes_scanned    4    0
l2_write_buffer_list_iter        4    0
l2_write_buffer_list_null_iter   4    0
read_ops                        4    1247
write_ops                       4    23
"#;
