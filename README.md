# Burnin

A lightweight, portable CLI tool for comprehensive system burn-in testing across bare metal, VMs, and containers.

## Features

- **Zero Dependencies**: Single statically-linked binary (~50MB) for easy deployment
- **Comprehensive Testing**: Tests CPU, memory, storage, network, and thermal components
- **Adaptive Testing**: Automatically detects hardware and adjusts tests accordingly
- **Multiple Output Formats**: Human-readable text, JSON for automation, CSV for reporting
- **Configurable**: Quick, standard, and full test presets with customizable parameters
- **Portable**: Works on bare metal, VMs, and containers

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/burnin.git
cd burnin

# Build release version (optimized)
cargo build --release

# Install to system path (optional)
sudo cp target/release/burnin /usr/local/bin/

# Or run directly
./target/release/burnin --help
```

### Binary Size
- Debug build: ~39MB
- Release build: ~1.5MB (96% smaller)

### Performance
- Release builds are approximately 2.3x faster than debug builds
- Use `cargo build --release` for production deployments

## Performance Considerations

### Optimization
The release build provides significant performance improvements:
- **2.3x faster** CPU test execution
- **96% smaller** binary size (1.5MB vs 39MB)
- Lower memory footprint
- Better compiler optimizations

### Build Commands
```bash
# Debug build (for development)
cargo build

# Release build (for production)
cargo build --release

# Run with optimizations
./target/release/burnin quick
```

### Resource Usage
- CPU tests will use all available cores by default
- Memory tests allocate 75% of available RAM by default
- Adjust `--threads` and memory settings for resource-constrained systems

## Usage

### Test Presets

```bash
# Quick Test (1-2 minutes)
burnin quick

# Standard Test (5-15 minutes)
burnin standard

# Full Test (30+ minutes)
burnin full
```

### Custom Tests

```bash
# Run specific components with custom duration
burnin custom --duration 20m --components cpu memory --stress 8

# Run all tests with higher stress level
burnin custom --stress 9 --duration 1h

# Run only storage tests with specific test path
burnin custom --components storage --storage-path /mnt/test --storage-size 2GB

# Run memory tests with specific size
burnin custom --components memory --memory-size 80%

# Run network tests with custom thresholds
burnin custom --components network --latency-ms 50 --bandwidth-mbps 100
```

### Hardware Information

```bash
# Show detailed hardware information
burnin hardware

# Export hardware information to JSON
burnin hardware --format json --output hardware.json
```

### Output Formats and Reporting

```bash
# Default text output with progress bars
burnin quick

# Quiet mode (minimal output)
burnin quick --quiet

# Verbose mode (detailed output)
burnin standard --verbose

# JSON output for automation
burnin quick --format json --output results.json

# CSV output for spreadsheets
burnin quick --format csv --output results.csv

# Combined formats
burnin full --format text --format json --output results.json
```

### Advanced Options

```bash
# Run with specific thread count
burnin custom --threads 4

# Set custom thermal thresholds
burnin custom --thermal-warning 75 --thermal-critical 85

# Load configuration from file
burnin custom --config my-config.toml

# Specify custom test timeout
burnin custom --timeout 2h
```

## Test Components

- **CPU**: Multi-threaded stress tests including prime number generation, matrix multiplication, floating point operations
- **Memory**: Sequential and random access patterns, pattern testing, walking bit patterns
- **Storage**: Sequential and random read/write, mixed workloads, filesystem metadata operations
- **Network**: Latency, bandwidth, and packet loss tests
- **Thermal**: Temperature monitoring during other tests

## Configuration

You can customize tests using command-line options or a configuration file:

```bash
burnin custom --config my-config.toml
```

Example configuration file:

```toml
# Global settings
duration = "30m"
threads = 0  # Auto-detect
stress_level = 8
output_format = "text"

# Component settings
[cpu]
enabled = true
workload = "mixed"

[memory]
enabled = true
test_size_percent = 80

[storage]
enabled = true
test_path = "/tmp/burnin-test"
test_size_mb = 1024

[network]
enabled = true

[thermal]
enabled = true
warning_threshold = 80.0
critical_threshold = 90.0
```

## Troubleshooting

### Common Issues

- **Permission Errors**: Some tests (especially storage and network) may require elevated permissions. Run with `sudo` if needed.
  ```bash
  sudo burnin standard
  ```

- **Resource Limitations**: In containers or VMs, some tests might fail due to resource limitations. Use `--components` to select only relevant tests.
  ```bash
  burnin custom --components cpu memory
  ```

- **Test Failures**: If a test fails, check the detailed output with `--verbose` flag to identify the specific issue.
  ```bash
  burnin quick --verbose
  ```

- **High Temperature Warnings**: If you receive thermal warnings, ensure proper cooling and consider lowering the stress level.
  ```bash
  burnin custom --stress 5
  ```

### Exit Codes

- `0`: All tests passed successfully
- `1`: One or more tests failed
- `2`: Configuration or parameter error
- `3`: Permission denied or resource unavailable
- `4`: Test was interrupted by user (Ctrl+C)

## Development

### Running Tests

```bash
# Run all tests
cargo test --release

# Run specific test module
cargo test core::config::tests --release

# Run with output
cargo test -- --nocapture
```

### Running Benchmarks

```bash
# Install cargo-criterion (optional, for better output)
cargo install cargo-criterion

# Run benchmarks
cargo bench

# View benchmark results
open target/criterion/report/index.html
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit
```

## Docker Support

### Building Docker Image

```bash
# Build the image
docker build -t burnin:latest .

# Run a quick test
docker run --privileged burnin:latest quick

# Run with custom parameters
docker run --privileged burnin:latest custom --duration 30m --components cpu,memory
```

### Using Docker Compose

```bash
# Run default quick test
docker-compose up burnin

# Run and save JSON results
docker-compose up burnin-json

# View results
cat test-results/report.json
```

## CI/CD

The project includes GitHub Actions workflows for:

- **CI Pipeline** (`ci.yml`): Runs on every push and PR
  - Code formatting check
  - Clippy linting
  - Unit tests
  - Cross-platform builds (Linux, macOS)
  
- **Release Pipeline** (`release.yml`): Runs on version tags
  - Creates GitHub releases
  - Builds binaries for multiple platforms
  - Uploads compressed artifacts

### Creating a Release

```bash
# Tag a new version
git tag -a v1.0.0 -m "Release version 1.0.0"

# Push the tag
git push origin v1.0.0
```

## Project Structure

```
src/
├── core/           # Core functionality
│   ├── config.rs   # Configuration handling
│   ├── error.rs    # Error types
│   ├── hardware.rs # Hardware detection
│   ├── mod.rs      # Module exports
│   ├── runner.rs   # Test execution
│   └── test.rs     # Test traits and types
├── reporters/      # Output formatters
│   ├── csv.rs      # CSV reporter
│   ├── json.rs     # JSON reporter
│   ├── mod.rs      # Module exports
│   └── text.rs     # Human-readable text reporter
├── tests/          # Test implementations
│   ├── cpu.rs      # CPU stress tests
│   ├── memory.rs   # Memory tests
│   ├── mod.rs      # Module exports
│   ├── network.rs  # Network tests
│   ├── storage.rs  # Storage I/O tests
│   └── thermal.rs  # Thermal monitoring
└── main.rs         # CLI entry point
```

### Adding a New Test

1. Create a new file in `src/tests/`
2. Implement the `BurnInTest` trait
3. Register the test in `src/tests/mod.rs`
4. Add configuration options in `src/core/config.rs`

Example implementation:

```rust
pub struct MyNewTest;

impl BurnInTest for MyNewTest {
    fn name(&self) -> &'static str {
        "my-new-test"
    }
    
    fn detect_hardware(&self) -> Result<HardwareInfo> {
        // Detect relevant hardware
        Ok(HardwareInfo::default())
    }
    
    fn estimate_duration(&self, config: &TestConfig) -> Duration {
        // Estimate test duration
        Duration::from_secs(60 * 5) // 5 minutes
    }
    
    fn execute(&self, config: &TestConfig) -> Result<TestResult> {
        // Implement test logic
        Ok(TestResult::pass(self.name()))
    }
    
    fn cleanup(&self) -> Result<()> {
        // Clean up resources
        Ok(())
    }
}
```

## License

MIT License
