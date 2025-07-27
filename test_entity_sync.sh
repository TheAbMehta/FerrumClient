#!/usr/bin/env bash
set -e

echo "Testing ferrum-entity..."
RUSTC_BOOTSTRAP=1 cargo test --package ferrum-entity

echo ""
echo "Testing entity_sync integration tests..."
RUSTC_BOOTSTRAP=1 cargo test --package ferrum --test entity_sync

echo ""
echo "All entity synchronization tests passed!"
