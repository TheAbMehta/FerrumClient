#!/usr/bin/env bash
set -e

echo "Running Ferrum Minecraft Client..."
echo ""

export RUSTC_BOOTSTRAP=1

cargo run --release
