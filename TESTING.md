# Testing Guide

This document explains how to run and write tests for the Ferrum Minecraft client.

## Quick Start

### Running All Tests

```bash
# Set required environment variable
export RUSTC_BOOTSTRAP=1  # Linux/macOS
# or
$env:RUSTC_BOOTSTRAP=1    # Windows PowerShell

# Run all tests (may timeout due to Bevy compilation)
cargo test --workspace
```

**Note**: Running all tests at once may timeout during Bevy compilation (120+ seconds). For faster feedback, test individual crates as shown below.

### Running Individual Crate Tests

```bash
# Physics tests (17 tests)
cargo test --package ferrum-physics

# World interaction tests (9 tests)
cargo test --package ferrum-world

# Inventory system tests (47 tests)
cargo test --package ferrum-inventory

# CPU meshing tests (10 tests)
cargo test --package ferrum-meshing-cpu

# Asset loading tests (4 tests)
cargo test --package ferrum-assets

# Configuration tests (6 tests)
cargo test --package ferrum-config

# Protocol tests (10 tests)
cargo test --package ferrum-protocol

# Subprocess management tests (5 tests)
cargo test --package ferrum-subprocess
```

**Total Test Coverage**: 83+ tests across 8 crates

## Known Issues

### RUSTC_BOOTSTRAP Required

The `azalea-protocol` dependency uses nightly Rust features. You must set the `RUSTC_BOOTSTRAP=1` environment variable before running tests:

```bash
# Linux/macOS
export RUSTC_BOOTSTRAP=1
cargo test

# Windows PowerShell
$env:RUSTC_BOOTSTRAP=1
cargo test

# Windows CMD
set RUSTC_BOOTSTRAP=1
cargo test
```

### Bevy Compilation Timeout

Bevy takes 120+ seconds to compile, which can cause test timeouts when running the full workspace. Workarounds:

1. **Test individual crates** (recommended for development)
2. **Increase timeout** if your test runner supports it
3. **Use cached builds** by running `cargo build` first

### Lighting System Tests

The `ferrum-render` crate currently has 4 failing tests out of 15 in the lighting system. These are known issues documented in `HANDOFF.md`. The core gameplay systems are unaffected.

## Test Organization

### Unit Tests

Unit tests are located in each crate's `tests/` directory:

```
ferrum-physics/tests/physics.rs       # Player movement, collision, gravity
ferrum-world/tests/interaction.rs     # Block breaking/placing
ferrum-inventory/tests/inventory.rs   # Item management
ferrum-inventory/tests/crafting.rs    # Recipe system
ferrum-inventory/tests/combat.rs      # Damage, durability
ferrum-meshing-cpu/tests/binary_greedy.rs  # Mesh generation
ferrum-protocol/tests/packets.rs      # Network protocol
ferrum-config/tests/config.rs         # TOML parsing
ferrum-assets/tests/asset_manager.rs  # Asset loading
ferrum-subprocess/tests/lifecycle.rs  # Pumpkin server management
```

### Integration Tests

Integration tests are in the main `ferrum` crate:

```
ferrum/tests/integration.rs      # Full system integration
ferrum/tests/network.rs          # Network connectivity
ferrum/tests/chunk_loading.rs    # Chunk streaming
ferrum/tests/entity_sync.rs      # Entity tracking
ferrum/tests/player_position.rs  # Position synchronization
```

### Test Structure

Tests follow standard Rust conventions:

```rust
#[test]
fn test_player_movement_wasd() {
    // Arrange
    let mut player = Player::new(Vec3::ZERO);
    player.set_on_ground(true);

    // Act
    let input = MovementInput {
        forward: true,
        backward: false,
        left: false,
        right: false,
        jump: false,
        sprint: false,
    };
    player.apply_movement(input, 0.05);

    // Assert
    assert!(player.velocity().z < 0.0);
}
```

## Writing New Tests

### Adding Unit Tests

1. Create or open the test file in `<crate>/tests/`
2. Import the module you're testing
3. Write test functions with `#[test]` attribute
4. Run with `cargo test --package <crate-name>`

Example:

```rust
use ferrum_physics::{Player, MovementInput};
use glam::Vec3;

#[test]
fn test_new_feature() {
    let player = Player::new(Vec3::ZERO);
    // Test your feature
    assert_eq!(player.position(), Vec3::ZERO);
}
```

### Test-Driven Development

This project follows TDD practices:

1. **Write the test first** - Define expected behavior
2. **Run the test** - Verify it fails
3. **Implement the feature** - Make the test pass
4. **Refactor** - Clean up while keeping tests green

Example workflow:

```bash
# 1. Write test in ferrum-physics/tests/physics.rs
# 2. Run test (should fail)
cargo test --package ferrum-physics test_new_feature

# 3. Implement feature in ferrum-physics/src/
# 4. Run test again (should pass)
cargo test --package ferrum-physics test_new_feature

# 5. Run all physics tests to ensure no regressions
cargo test --package ferrum-physics
```

### Testing Best Practices

1. **Test one thing per test** - Keep tests focused and atomic
2. **Use descriptive names** - `test_player_jumps_when_on_ground` not `test1`
3. **Arrange-Act-Assert** - Structure tests clearly
4. **Avoid test interdependencies** - Each test should be independent
5. **Test edge cases** - Empty inventories, full stacks, boundary conditions
6. **Use realistic values** - Match Minecraft's actual physics/mechanics

## Benchmarking

### Running Benchmarks

Benchmarks use the Criterion framework:

```bash
# Run all benchmarks
cargo bench

# Run specific crate benchmarks
cargo bench --package ferrum-meshing-cpu
cargo bench --package ferrum-meshing-gpu
```

### Available Benchmarks

**CPU Meshing** (`ferrum-meshing-cpu/benches/chunk_meshing.rs`):
- `cpu_mesh_uniform_air` - Empty chunks (6.5µs)
- `cpu_mesh_uniform_stone` - Solid chunks (44µs)
- `cpu_mesh_checkerboard` - Worst case (506µs)
- `cpu_mesh_terrain` - Realistic terrain (64µs)

**GPU Meshing** (`ferrum-meshing-gpu/benches/chunk_meshing.rs`):
- GPU compute shader benchmarks

### Interpreting Results

Criterion outputs detailed statistics:

```
cpu_mesh_terrain        time:   [63.891 µs 64.127 µs 64.389 µs]
                        change: [-2.1% -1.5% -0.9%] (p = 0.00 < 0.05)
                        Performance has improved.
```

Target performance (Phase 1):
- Realistic terrain: <100µs per chunk
- 32 chunk render distance at 144 FPS

### Writing Benchmarks

Add benchmarks to `<crate>/benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferrum_meshing_cpu::CpuMesher;

fn bench_new_algorithm(c: &mut Criterion) {
    let mesher = CpuMesher::new();
    let chunk = create_test_chunk();
    
    c.bench_function("new_algorithm", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

criterion_group!(benches, bench_new_algorithm);
criterion_main!(benches);
```

## Continuous Integration

Tests run automatically on GitHub Actions for:
- Linux (Ubuntu latest)
- Windows (latest)

CI configuration: `.github/workflows/ci.yml`

### CI Test Commands

```bash
# What CI runs
export RUSTC_BOOTSTRAP=1
cargo test --workspace --verbose
cargo clippy -- -D warnings
cargo fmt --check
```

## Troubleshooting

### Tests Fail with "nightly feature" Error

**Solution**: Set `RUSTC_BOOTSTRAP=1` environment variable.

### Tests Timeout During Compilation

**Solution**: Test individual crates instead of `--workspace`.

### "Cannot find crate" Errors

**Solution**: Build dependencies first:
```bash
cargo build --workspace
cargo test --package <crate-name>
```

### Bevy-Related Test Failures

**Symptoms**: Tests involving Bevy systems fail or hang.

**Solution**: Bevy tests require special setup. Check `ferrum/tests/` for examples of proper Bevy test app initialization.

### Lighting Tests Failing

**Expected**: 4/15 lighting tests currently fail (known issue).

**Impact**: Does not affect core gameplay. See `HANDOFF.md` for details on fixing.

## Test Coverage Summary

| Crate | Tests | Status |
|-------|-------|--------|
| ferrum-physics | 17 | Passing |
| ferrum-world | 9 | Passing |
| ferrum-inventory | 47 | Passing |
| ferrum-meshing-cpu | 10 | Passing |
| ferrum-assets | 4 | Passing |
| ferrum-config | 6 | Passing |
| ferrum-protocol | 10 | Passing |
| ferrum-subprocess | 5 | Passing |
| ferrum-render | 11/15 | 4 failing (lighting) |
| **Total** | **83+** | **Core systems passing** |

## Additional Resources

- **HANDOFF.md** - Development guide and architecture overview
- **README.md** - Quick start and project overview
- **Cargo.toml** - Workspace configuration and dependencies
- **.sisyphus/notepads/** - Implementation notes and learnings

## Getting Help

If tests fail unexpectedly:

1. Check this document for known issues
2. Review `HANDOFF.md` for system-specific details
3. Examine test output for specific error messages
4. Run with `--verbose` for detailed output: `cargo test --verbose`
5. Check recent commits for related changes
