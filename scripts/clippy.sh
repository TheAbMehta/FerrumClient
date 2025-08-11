#!/usr/bin/env bash
set -e

echo "Running Clippy linter..."
echo ""

export RUSTC_BOOTSTRAP=1

cargo clippy --workspace -- -D warnings

echo ""
echo "Clippy complete!"
