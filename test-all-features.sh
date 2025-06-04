#!/bin/bash
# Burn-In Tool Feature Test Script

echo "=== Burn-In Tool Feature Test ==="
echo ""

# Check if release build exists
if [ ! -f "./target/release/burnin" ]; then
    echo "Building release version..."
    cargo build --release
fi

echo "1. Hardware Information"
echo "----------------------"
./target/release/burnin hardware
echo ""

echo "2. Quick CPU Test (5 seconds)"
echo "-----------------------------"
./target/release/burnin custom --duration 5s --components cpu
echo ""

echo "3. Help Information"
echo "-------------------"
./target/release/burnin --help
echo ""

echo "4. Version Information"
echo "----------------------"
./target/release/burnin --version
echo ""

echo "Test complete!"
