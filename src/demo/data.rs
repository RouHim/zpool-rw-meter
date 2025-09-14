



/// Demo zpool status output
#[allow(dead_code)]
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


