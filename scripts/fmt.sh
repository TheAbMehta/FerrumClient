#!/usr/bin/env bash
set -e

echo "Formatting Rust code..."
echo ""

cargo fmt --all

echo ""
echo "Format complete!"
