#!/usr/bin/env bash
# Benchmark script for Ferrum Minecraft Client

set -e

echo "Running Ferrum benchmarks..."
echo ""

# Set required environment variable
export RUSTC_BOOTSTRAP=1

# Run CPU meshing benchmarks
echo "=== CPU Meshing Benchmarks ==="
cargo bench --package ferrum-meshing-cpu

echo ""
echo "Benchmarks complete!"
echo "Results saved to target/criterion/"
