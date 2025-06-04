.PHONY: all build release test bench clean fmt lint audit help

# Default target
all: build

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run all tests
test: test-unit test-integration

# Run unit tests
test-unit:
	cargo test --lib --release

# Run integration tests
test-integration:
	cargo test --test integration_test --release

# Run benchmarks
bench:
	cargo bench

# Clean build artifacts
clean:
	cargo clean
	rm -rf test-results/
	rm -f *.csv *.json

# Format code
fmt:
	cargo fmt

# Run linter
lint:
	cargo clippy -- -D warnings

# Security audit
audit:
	cargo audit

# Build Docker image
docker-build:
	docker build -t burnin:latest .

# Run quick test in Docker
docker-test:
	docker run --privileged burnin:latest quick

# Install release binary to system
install: release
	sudo cp target/release/burnin /usr/local/bin/

# Uninstall from system
uninstall:
	sudo rm -f /usr/local/bin/burnin

# Display help
help:
	@echo "Burn-In Tool Makefile Commands:"
	@echo "  make build          - Build debug version"
	@echo "  make release        - Build optimized release version"
	@echo "  make test           - Run all tests"
	@echo "  make test-unit      - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make bench          - Run benchmarks"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run clippy linter"
	@echo "  make audit          - Run security audit"
	@echo "  make docker-build   - Build Docker image"
	@echo "  make docker-test    - Run quick test in Docker"
	@echo "  make install        - Install to /usr/local/bin"
	@echo "  make uninstall      - Uninstall from system"
	@echo "  make help           - Display this help message"
