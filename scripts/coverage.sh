#!/usr/bin/env bash
set -e

echo "Generating test coverage..."
echo ""

export RUSTC_BOOTSTRAP=1

cargo install cargo-tarpaulin || true

cargo tarpaulin --workspace --out Html --output-dir target/coverage

echo ""
echo "Coverage report generated at target/coverage/index.html"
