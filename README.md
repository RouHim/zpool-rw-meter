# ZFS Cache Performance Monitor

A real-time monitoring tool for ZFS cache layers (ARC, L2ARC, SLOG) with visual progress bars and performance indicators.

## Features

- **Real-time monitoring** of all ZFS cache layers
- **Visual progress bars** with Unicode characters
- **Color-coded performance indicators** (excellent/good/fair/poor)
- **Automatic pool detection** or manual selection
- **Configurable refresh intervals**
- **Comprehensive error handling** and graceful fallbacks

## Cache Layers Monitored

### 📊 ARC (Adaptive Replacement Cache)
- Primary RAM-based cache
- Hit/miss rates and performance rating
- Cache size vs target size with utilization
- Read operations per second

### 💾 L2ARC (Level 2 ARC)
- Secondary SSD-based read cache
- Hit/miss rates for L2 cache
- Cache size and read throughput
- Operations per second

### ✏️ SLOG (Synchronous Write Log)
- Dedicated write cache device
- Device utilization and write operations
- Write throughput and latency metrics
- Performance assessment based on utilization/latency

## Requirements

- **ZFS utilities**: `zpool`, `arcstat`
- **System tools**: `bc`, `awk`, `grep`
- **Optional**: `iostat` (for enhanced SLOG monitoring)

### Installation on Ubuntu/Debian
```bash
sudo apt update
sudo apt install zfsutils-linux bc
```

### Installation on RHEL/CentOS/Fedora
```bash
sudo dnf install zfs bc
# or on older systems:
sudo yum install zfs bc
```

## Usage

```bash
# Auto-detect pool, 2-second refresh
./zfs-cache-monitor.sh

# Monitor specific pool
./zfs-cache-monitor.sh tank

# Custom refresh interval
./zfs-cache-monitor.sh tank 1

# Show help
./zfs-cache-monitor.sh --help

# Enable debug mode for troubleshooting
DEBUG=true ./zfs-cache-monitor.sh

# Run in demo mode (for testing without ZFS)
DEMO_MODE=true ./zfs-cache-monitor.sh
```

## Environment Variables

- **`DEBUG=true`** - Enable verbose debugging output to troubleshoot `arcstat` issues
- **`DEMO_MODE=true`** - Run with realistic sample data (useful for testing or demo purposes)

## Example Output

```
═══════════════════════════════════════════════════════════════════
🔍 ZFS Cache Performance Monitor
═══════════════════════════════════════════════════════════════════
Pool: tank | Refresh: 2s | Time: 2025-09-14 15:30:45

📊 ARC (Primary RAM Cache)
  Hit Rate:    ████████████████████ 89.2% (Excellent)
  Cache Size:  ██████████████████░░ 87.3% (15.2G/17.4G)
  Read Ops:    1,247/s

💾 L2ARC (Secondary SSD Cache)
  Hit Rate:    ████████████████░░░░ 78.4% (Good)
  Cache Size:  45.8G
  Read Rate:   234.5M/s
  Operations:  892/s

✏️ SLOG (Synchronous Write Log)
  Device:      nvme1n1p3
  Utilization: ██████░░░░░░░░░░░░░░ 28.7%
  Write Ops:   156/s
  Write Rate:  12.3M/s
  Latency:     2.1ms (Excellent)

═══════════════════════════════════════════════════════════════════
Press Ctrl+C to exit | Data refreshes every 2s
```

## Performance Indicators

### Color Coding
- 🟢 **Green**: Excellent performance (80%+ hit rate)
- 🟡 **Yellow**: Good performance (60-80% hit rate)
- 🔴 **Red**: Poor performance (<60% hit rate)

### Rating Thresholds
- **ARC Hit Rate**: 85%+ excellent, 70%+ good
- **L2ARC Hit Rate**: 75%+ excellent, 50%+ good
- **SLOG Performance**: Based on utilization (<80%) and latency (<5ms)

## Troubleshooting

### Missing Dependencies
```bash
Error: Missing required tools: zpool arcstat
```
Install ZFS utilities for your distribution (see Requirements section).

### No ZFS Pools Found
```bash
Error: No ZFS pools found
```
Ensure ZFS is loaded and pools are imported:
```bash
sudo modprobe zfs
sudo zpool import -a
```

### Permission Issues
```bash
Permission denied
```
Run with appropriate privileges or add user to disk group:
```bash
sudo usermod -a -G disk $USER
```

### Script Hangs or Shows Only Header
If the script displays the header but hangs without showing data:

```bash
# Enable debug mode to see what's happening
DEBUG=true ./zfs-cache-monitor.sh

# Try demo mode to verify the script works
DEMO_MODE=true ./zfs-cache-monitor.sh

# Test arcstat manually
arcstat 1 1
```

**Common causes:**
- `arcstat` command hanging (fixed with timeout protection)
- Different `arcstat` versions with varying syntax
- Missing ZFS kernel modules

The script includes multiple fallback methods:
1. `arcstat -f fields 1 1` (specific fields)
2. `arcstat 1 1` (default output)  
3. Direct parsing from `/proc/spl/kstat/zfs/arcstats`

## Implementation Details

- **ARC stats**: Retrieved via `arcstat` utility and `/proc/spl/kstat/zfs/arcstats`
- **L2ARC stats**: Parsed from ZFS kernel statistics
- **SLOG stats**: Combined `iostat` and `zpool iostat` data
- **Visual elements**: Unicode progress bars with ANSI color codes
- **Error handling**: Graceful fallbacks for missing devices/tools

## Contributing

Feel free to submit issues and enhancement requests!

## License

This project is open source and available under the MIT License.