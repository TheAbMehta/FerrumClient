#!/usr/bin/env bash
# Build script for Ferrum Minecraft Client

set -e

echo "Building Ferrum Minecraft Client..."
echo ""

# Set required environment variable
export RUSTC_BOOTSTRAP=1

# Build all crates
echo "Building workspace..."
cargo build --release --workspace

echo ""
echo "Build complete!"
echo "Binary location: target/release/ferrum"
