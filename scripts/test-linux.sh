#!/bin/bash
set -e

echo "=== Running tests on Linux (Vulkan) ==="

# Set environment variables for Vulkan
export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/intel_icd.x86_64.json
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/intel_icd.x86_64.json

# Run tests with workspace flag
echo "Running cargo test..."
cargo test --workspace --verbose

echo "=== All tests passed on Linux ==="
