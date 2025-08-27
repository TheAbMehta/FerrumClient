#!/usr/bin/env bash
set -e

echo "Running cargo check..."
echo ""

export RUSTC_BOOTSTRAP=1

cargo check --workspace

echo ""
echo "Check complete!"
