#!/usr/bin/env bash
set -e

echo "Generating documentation..."
echo ""

export RUSTC_BOOTSTRAP=1

cargo doc --workspace --no-deps --open

echo ""
echo "Documentation generated!"
