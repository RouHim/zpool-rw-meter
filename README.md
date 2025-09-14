# ZFS Cache Performance Monitor (Rust)

A high-performance, real-time monitoring tool for ZFS cache layers (ARC, L2ARC, SLOG) written in Rust with visual progress bars and performance indicators.

## Features

- **Real-time monitoring** of all ZFS cache layers with accurate rate calculations
- **Visual progress bars** with Unicode characters
- **Color-coded performance indicators** (excellent/good/fair/poor)
- **Automatic pool detection** or manual selection
- **Configurable refresh intervals**
- **Comprehensive error handling** and graceful fallbacks
- **Async I/O** for high performance and responsiveness
- **Demo mode** for testing without ZFS installation

## Cache Layers Monitored

### 📊 ARC (Adaptive Replacement Cache)
- Primary RAM-based cache
- Hit/miss rates and performance rating
- Cache size vs target size with utilization
- Read operations per second (calculated rates)

### 💾 L2ARC (Level 2 ARC)
- Secondary SSD-based read cache
- Hit/miss rates for L2 cache
- Cache size and read throughput
- Operations per second (calculated rates)

### 🟡 SLOG (Synchronous Write Log)
- Dedicated write cache device
- Device utilization and write operations
- Write throughput and latency metrics
- Performance assessment based on utilization/latency

## Requirements

- **Rust toolchain** (1.70+ recommended)
- **ZFS utilities**: `zpool`, `arcstat` (for live mode)
- **System access**: `/proc/spl/kstat/zfs/arcstats` (for live mode)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd zpool-rw-meter

# Build the project
cargo build --release

# Run in demo mode (no ZFS required)
DEMO_MODE=true cargo run

# Or run live mode on ZFS systems
cargo run
```

### System Dependencies (for live mode)

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install zfsutils-linux
```

#### RHEL/CentOS/Fedora
```bash
sudo dnf install zfs
# or on older systems:
sudo yum install zfs
```

## Usage

```bash
# Build and run with default settings (demo mode)
DEMO_MODE=true cargo run

# Monitor specific pool with default 2-second refresh
cargo run pool_name

# Custom refresh interval (1 second)
cargo run pool_name 1

# Live mode on ZFS systems (requires ZFS installation)
cargo run

# Show help
cargo run -- --help
```

## Environment Variables

- **`DEMO_MODE=true`** - Run with realistic sample data (useful for testing or demo purposes without ZFS)

## Example Output

```
======================= 🔍 ZFS Cache Performance Monitor ========================
Pool: data | Refresh: 2s | Time: 2025-09-14 17:10:08

📊 ARC (Primary RAM Cache)
    Hit Rate:    100 (Excellent) [####################] 100.0%
    Cache Size:  46.3G/46.5G [####################] 99.6%
    Read Ops:    0/s

💾 L2ARC (Secondary SSD Cache)
    Hit Rate:    73.4304932735426 (Good) [###############.....] 73.4%
    Cache Size:  553.7G
    Read Rate:   0 B/s
    Operations:  0/s

🟡 SLOG (Synchronous Write Log)
    Device:      mirror-1
    Utilization: 0 (Excellent) [....................] 0.0%
    Write Ops:   0/s
    Write Rate:  0 B/s
    Latency:     0.0ms

================================================================================
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
Error: Failed to collect ARC statistics from all sources
```
This error occurs when running in live mode without ZFS installed. Use demo mode instead:
```bash
DEMO_MODE=true cargo run
```

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

### Compilation Issues
If you encounter compilation errors:
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### Testing Without ZFS
The application includes comprehensive demo mode for testing:
```bash
# Run demo mode
DEMO_MODE=true cargo run

# Test with different pools/intervals
DEMO_MODE=true cargo run testpool 1
```

**Demo mode features:**
- Realistic ZFS statistics simulation
- All cache layers displayed
- Rate calculations working
- Full UI functionality

## Implementation Details

### Architecture
- **Async runtime**: Tokio for high-performance concurrent I/O
- **Modular design**: Separate modules for display, ZFS parsing, system abstraction
- **Trait-based abstractions**: `CommandExecutor` and `FilesystemReader` for testing
- **Rate calculation**: Custom `RateCalculator` for accurate ops/second metrics

### Data Sources
- **ARC stats**: `arcstat` utility and `/proc/spl/kstat/zfs/arcstats` parsing
- **L2ARC stats**: Direct parsing from ZFS kernel statistics
- **SLOG stats**: Combined `zpool status` and `zpool iostat` data
- **Visual elements**: Unicode progress bars with terminal control sequences
- **Error handling**: Comprehensive fallbacks and graceful degradation

### Performance Features
- **Real-time rates**: Accurate per-second calculations for all metrics
- **Flicker-free display**: Terminal control for smooth updates
- **Signal handling**: Graceful Ctrl+C shutdown
- **Timeout protection**: Prevents hanging on slow commands

## Migration Status

This project has been successfully migrated from shell script to Rust:

- ✅ **Phase 1**: Foundation & Demo Mode - Complete
- ✅ **Phase 2**: System Abstractions - Complete
- ✅ **Phase 3**: ZFS Statistics Engine - Complete
- 🔄 **Phase 4**: Caching & Error Handling - In Progress
- 🔄 **Phase 5**: Unit Tests & Real System Testing - Pending

### Key Improvements in Rust Version
- **Performance**: Async I/O with Tokio runtime
- **Reliability**: Strong typing and comprehensive error handling
- **Maintainability**: Modular architecture with clear separation of concerns
- **Testing**: Trait-based abstractions enable thorough unit testing
- **Accuracy**: Precise rate calculations for real-time metrics

## Contributing

Feel free to submit issues and enhancement requests! The codebase is written in Rust with a focus on performance and reliability.

## License

This project is open source and available under the MIT License.