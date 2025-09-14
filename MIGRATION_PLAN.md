# ZFS Cache Monitor - Rust Migration Plan

## Project Overview

Migration of the existing ZFS cache monitor shell script (751 lines) to a modern Rust application. The goal is to create a fast, reliable, and maintainable ZFS monitoring tool while preserving all existing functionality.

## Current Shell Script Analysis

### Core Functionality
- **Real-time monitoring** of ARC, L2ARC, and SLOG performance
- **Visual progress bars** with ASCII characters for compatibility
- **Color-coded status** indicators (green/yellow/red)
- **Automatic pool detection** with fallback options
- **Demo mode** for testing without ZFS
- **Robust error handling** with multiple fallback methods
- **Terminal-optimized display** with minimal flicker

### Key Features to Preserve
- Command-line interface: `zfs-cache-monitor [POOL] [INTERVAL]`
- Real-time updates with configurable refresh intervals
- Three-tier cache monitoring (ARC → L2ARC → SLOG)
- Human-readable byte formatting (B/K/M/G/T/P)
- Performance ratings (Excellent/Good/Fair/Poor)
- ANSI color support with fallback
- Ctrl+C graceful shutdown

### Data Sources Analyzed
1. **ARC Statistics**: `/proc/spl/kstat/zfs/arcstats` (primary), `arcstat` command (fallback)
2. **L2ARC Statistics**: ZFS kstats for L2 cache metrics
3. **SLOG Statistics**: `zpool status` and `zpool iostat` commands
4. **Pool Information**: `zpool list` and `zpool status` commands

## Rust Implementation Strategy

### Dependencies (Minimal Approach)
```toml
[dependencies]
procfs = "0.16"       # For /proc filesystem access
nix = "0.29"          # For system calls and process management  
console = "0.15"      # For terminal control and colors
```

**Rationale**: Following the user's preference for minimal dependencies, avoiding heavyweight crates like `clap`, `tokio`, `anyhow`, or `serde`.

### Project Structure
```
src/
├── main.rs                 # CLI entry point and argument parsing
├── monitor.rs              # Main monitoring loop and display coordination
├── display/
│   ├── mod.rs              # Display module exports
│   ├── terminal.rs         # Terminal control and ANSI handling
│   ├── progress.rs         # Progress bar rendering
│   └── formatter.rs        # Human-readable formatting (bytes, etc.)
├── zfs/
│   ├── mod.rs              # ZFS module exports
│   ├── stats.rs            # Statistics collection and parsing
│   ├── pools.rs            # Pool detection and validation
│   └── types.rs            # ZFS data structures
├── system/
│   ├── mod.rs              # System interface exports
│   ├── commands.rs         # Command execution abstraction
│   └── filesystem.rs       # File system access abstraction
└── demo/
    ├── mod.rs              # Demo mode implementation
    └── data.rs             # Realistic sample data
```

## Implementation Phases

### Phase 1: Foundation & Demo Mode (Week 1)
**Goal**: Create basic project structure with working demo mode

#### 1.1 Project Setup
- [ ] Initialize Cargo project with minimal dependencies
- [ ] Set up module structure and exports
- [ ] Configure clippy and rustfmt settings
- [ ] Create basic CLI argument parsing (manual, no clap)

#### 1.2 Demo Mode Implementation
- [ ] Create realistic demo data based on actual ZFS system:
  ```rust
  // Based on real system data provided
  pub const DEMO_ARC_DATA: ArcStats = ArcStats {
      hit_rate: 100.0,           // Excellent performance observed
      miss_rate: 0.0,
      size: 49_720_066_048,      // ~46.3G current usage
      target: 49_910_562_816,    // ~46.5G target
      read_ops: 1247,
  };
  
  pub const DEMO_L2ARC_DATA: L2ArcStats = L2ArcStats {
      hit_rate: 73.5,            // Good L2ARC performance
      miss_rate: 26.5,
      size: 594_542_387_200,     // ~554GB L2ARC cache
      read_bytes: 245_760_000,
      total_ops: 892,
  };
  
  pub const DEMO_SLOG_DATA: SlogStats = SlogStats {
      device: "mirror-1".to_string(),  // UUID-based mirror observed
      write_ops: 156,
      write_bw: 12_582_912,
      utilization: 28.7,
      latency: 2.1,
  };
  ```

#### 1.3 Basic Display System
- [ ] Implement terminal control (clear screen, cursor positioning)
- [ ] Create ANSI color support with fallback to plain text
- [ ] Build ASCII progress bar renderer matching shell script
- [ ] Implement human-readable byte formatting
- [ ] Create header/footer display functions

### Phase 2: System Abstractions (Week 2)
**Goal**: Create abstraction layers for development without ZFS

#### 2.1 Command Execution Abstraction
```rust
pub trait CommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<String, SystemError>;
    fn execute_with_timeout(&self, command: &str, args: &[&str], timeout: Duration) -> Result<String, SystemError>;
}

pub struct RealCommandExecutor;  // Uses std::process::Command
pub struct DemoCommandExecutor;  // Returns predefined demo data
```

#### 2.2 Filesystem Access Abstraction
```rust
pub trait FilesystemReader {
    fn read_to_string(&self, path: &str) -> Result<String, SystemError>;
    fn exists(&self, path: &str) -> bool;
}

pub struct RealFilesystemReader;  // Uses std::fs
pub struct DemoFilesystemReader;  // Returns demo /proc/spl/kstat/zfs/arcstats data
```

#### 2.3 ZFS Data Structures
```rust
#[derive(Debug, Clone)]
pub struct ArcStats {
    pub hit_rate: f64,
    pub miss_rate: f64, 
    pub size: u64,
    pub target: u64,
    pub read_ops: u64,
}

#[derive(Debug, Clone)]
pub struct L2ArcStats {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub size: u64,
    pub read_bytes: u64,
    pub total_ops: u64,
}

#[derive(Debug, Clone)] 
pub struct SlogStats {
    pub device: String,
    pub write_ops: u64,
    pub write_bw: u64,
    pub utilization: f64,
    pub latency: f64,
}

#[derive(Debug)]
pub enum CacheStatus {
    Excellent, Good, Fair, Poor
}
```

### Phase 3: ZFS Statistics Engine (Week 3)
**Goal**: Implement robust ZFS data collection matching shell script behavior

#### 3.1 ARC Statistics Collection
- [ ] Implement `/proc/spl/kstat/zfs/arcstats` parser
- [ ] Add `arcstat` command fallback with multiple methods:
  - `arcstat -f hit%,miss%,read,arcsz,c 1 1`
  - `arcstat 1 1` 
  - Single-shot `echo | arcstat`
- [ ] Handle timeout scenarios (3-second limit observed in shell script)
- [ ] Implement robust parsing for varying `arcstat` output formats

#### 3.2 L2ARC Statistics Collection  
- [ ] Parse L2ARC kstats from `/proc/spl/kstat/zfs/arcstats`
- [ ] Detect L2ARC device presence via `zpool status`
- [ ] Calculate hit/miss rates and throughput metrics
- [ ] Handle inactive L2ARC scenarios

#### 3.3 SLOG Statistics Collection
- [ ] Implement SLOG device detection with caching (mirrors shell script)
- [ ] Support multiple detection methods:
  - Dedicated `logs` section parsing
  - Mirror device identification (`mirror-X` pattern)
  - Health-based activity indicators
- [ ] Integrate with `zpool iostat` for performance metrics
- [ ] Ensure consistent device visibility (prevent alternating display)

#### 3.4 Pool Management
- [ ] Implement pool auto-detection (`zpool list -H -o name`)
- [ ] Add pool validation and error handling
- [ ] Support pool-specific monitoring

### Phase 4: Display Engine (Week 4)
**Goal**: Create rich terminal display matching shell script aesthetics

#### 4.1 Progress Bar System
- [ ] Implement ASCII progress bars with configurable width
- [ ] Support filled (`#`) and empty (`.`) characters for compatibility
- [ ] Add percentage-based rendering (0-100%)
- [ ] Create color-coded progress bars

#### 4.2 Color and Status System
- [ ] Implement performance-based color coding:
  - Green: Excellent (≥80-85%)
  - Yellow: Good (≥60-70%) 
  - Red: Poor (<60%)
- [ ] Add performance rating text ("Excellent", "Good", "Fair", "Poor")
- [ ] Support ANSI color with graceful fallback

#### 4.3 Layout and Formatting
- [ ] Create section-based display layout:
  ```
  ═══════════════════════════════════════════════════════════════════
  🔍 ZFS Cache Performance Monitor
  ═══════════════════════════════════════════════════════════════════
  Pool: data | Refresh: 2s | Time: 2025-01-15 14:30:22
  
  📊 ARC (Primary RAM Cache)
    Hit Rate:    #################### 100.0% (Excellent)
    Cache Size:  ##################.. 98.5% (46.3G/46.5G)
    Read Ops:    1247/s
  
  💾 L2ARC (Secondary SSD Cache)  
    Hit Rate:    ##############...... 73.5% (Good)
    Cache Size:  554.0G
    Read Rate:   234.5M/s
    Operations:  892/s
  
  🟡 SLOG (Synchronous Write Log)
    Device:      mirror-1
    Utilization: #####............... 28.7%
    Write Ops:   156/s
    Write Rate:  12.0M/s
    Latency:     2.1ms (Excellent)
  
  ═══════════════════════════════════════════════════════════════════
  Press Ctrl+C to exit | Data refreshes every 2s
  ```

#### 4.4 Terminal Control
- [ ] Implement flicker-free updates using cursor positioning
- [ ] Add graceful cursor management (hide during updates)
- [ ] Support Ctrl+C signal handling for clean shutdown
- [ ] Handle terminal resize scenarios

### Phase 5: Integration & Polish (Week 5)
**Goal**: Complete integration with comprehensive testing

#### 5.1 Main Application Loop
- [ ] Implement non-blocking refresh loop with configurable intervals
- [ ] Add signal handling for graceful shutdown
- [ ] Integrate all display sections with proper error handling
- [ ] Support both demo and live modes seamlessly

#### 5.2 Error Handling & Robustness
- [ ] Implement comprehensive error types:
  ```rust
  #[derive(Debug)]
  pub enum MonitorError {
      ZfsUnavailable,
      PoolNotFound(String),
      InvalidInterval(String),
      SystemError(String),
      ParseError(String),
  }
  ```
- [ ] Add graceful degradation for missing components
- [ ] Implement retry logic for transient command failures
- [ ] Support partial data display when some metrics unavailable

#### 5.3 CLI Interface Completion
- [ ] Complete argument parsing (`[POOL] [INTERVAL]`)
- [ ] Add help text matching shell script (`-h`, `--help`)
- [ ] Support environment variables (`DEBUG=true`, `DEMO_MODE=true`)
- [ ] Validate input parameters with helpful error messages

#### 5.4 Performance Optimization
- [ ] Optimize kstat parsing for minimal allocations
- [ ] Cache expensive operations (SLOG device detection)
- [ ] Minimize system calls during refresh loops
- [ ] Profile memory usage and optimize hot paths

## Testing Strategy

### Unit Testing
- [ ] Test ZFS kstat parsers with real data samples
- [ ] Test progress bar rendering with various percentages
- [ ] Test byte formatting across all size ranges
- [ ] Test color code generation for different performance levels

### Integration Testing
- [ ] Test demo mode end-to-end functionality
- [ ] Test graceful degradation when ZFS unavailable
- [ ] Test signal handling and cleanup
- [ ] Test various pool configurations

### Real System Testing
- [ ] Test on actual ZFS systems with different configurations:
  - Single pool systems
  - Multi-pool systems  
  - Systems with/without L2ARC
  - Systems with/without SLOG
  - Various ZFS versions

### Compatibility Testing
- [ ] Test on different terminal emulators
- [ ] Test with various `TERM` environment settings
- [ ] Test color support detection and fallback
- [ ] Test with different ZFS tool versions

## Migration Validation

### Functional Parity Checklist
- [ ] All command-line options work identically
- [ ] Display layout matches shell script aesthetics  
- [ ] Performance thresholds and color coding identical
- [ ] Error messages and handling behavior preserved
- [ ] Demo mode produces comparable output
- [ ] Memory usage reasonable for continuous operation

### Performance Benchmarks
- [ ] Startup time comparison (should be faster than shell)
- [ ] Memory usage measurement (should be stable)
- [ ] CPU usage during monitoring (should be minimal)
- [ ] Refresh rate accuracy (should match specified interval)

### User Experience Validation
- [ ] Terminal responsiveness during updates
- [ ] Graceful handling of window resizing
- [ ] Clean shutdown behavior
- [ ] Error message clarity and helpfulness

## Deployment Strategy

### Build Configuration
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Installation Options
1. **Direct binary replacement**: `zfs-cache-monitor` → `zfs-cache-monitor-rs`
2. **Cargo installation**: `cargo install --path .` for development
3. **System package**: Future distribution via package managers
4. **Docker container**: For containerized ZFS monitoring

### Backwards Compatibility
- [ ] Preserve all command-line interface compatibility
- [ ] Support same environment variables
- [ ] Maintain identical output format for scripting
- [ ] Provide migration guide for any behavioral changes

## Risk Mitigation

### Technical Risks
1. **ZFS version compatibility**: Test with multiple ZFS versions
2. **Performance degradation**: Continuous profiling and benchmarking
3. **Memory leaks**: Extensive testing of long-running scenarios
4. **Terminal compatibility**: Test across various terminal types

### Development Risks  
1. **Scope creep**: Stick to exact shell script feature parity
2. **Over-engineering**: Maintain simplicity and focus on core functionality
3. **Testing complexity**: Prioritize real-system testing over mocking

### Mitigation Strategies
- Incremental development with working demo mode first
- Extensive real-system testing throughout development
- Clear feature parity validation at each phase
- Performance regression testing with each major change

## Success Criteria

### Primary Goals
- [ ] **Complete functional parity** with existing shell script
- [ ] **Improved performance**: Faster startup, lower resource usage
- [ ] **Enhanced reliability**: Better error handling and recovery
- [ ] **Maintainable codebase**: Clear structure for future enhancements

### Secondary Goals  
- [ ] **Cross-platform potential**: Foundation for future Windows/macOS support
- [ ] **Extensibility**: Clean architecture for adding new ZFS metrics
- [ ] **Documentation**: Comprehensive code documentation and user guide
- [ ] **Community adoption**: Positive feedback from ZFS community

## Timeline Summary

- **Week 1**: Foundation & Demo Mode (basic structure, demo functionality)
- **Week 2**: System Abstractions (command/filesystem interfaces)
- **Week 3**: ZFS Statistics Engine (robust data collection)
- **Week 4**: Display Engine (rich terminal interface)
- **Week 5**: Integration & Polish (testing, optimization, deployment)

**Total Duration**: 5 weeks for complete migration with comprehensive testing

This migration plan ensures a systematic approach to recreating the shell script's functionality in Rust while maintaining its reliability, performance characteristics, and user experience.