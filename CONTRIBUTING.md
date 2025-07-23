# Contributing to Ferrum

Thank you for your interest in contributing to Ferrum! This document provides guidelines for contributing to this high-performance Minecraft client project.

## Code of Conduct

Be respectful, constructive, and collaborative. We're building something awesome together.

## Getting Started

### Prerequisites

**Required**:
- Rust stable toolchain (install from [rustup.rs](https://rustup.rs/))
- Git
- Platform-specific dependencies (see README.md)

**Linux**:
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libx11-dev libasound2-dev libudev-dev

# Arch
sudo pacman -S base-devel alsa-lib

# Fedora
sudo dnf install gcc pkg-config libX11-devel alsa-lib-devel systemd-devel
```

**Windows**:
- Windows 10+ (for DX12 support)
- Visual Studio Build Tools or equivalent

### Development Setup

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/your-username/FerrumClient.git
   cd FerrumClient
   ```

2. **Set required environment variable**:
   ```bash
   # Linux/macOS
   export RUSTC_BOOTSTRAP=1
   
   # Windows PowerShell
   $env:RUSTC_BOOTSTRAP=1
   ```
   
   > **Note**: `RUSTC_BOOTSTRAP=1` is required because azalea-protocol uses nightly features. This is a compile-time requirement only.

3. **Build the project**:
   ```bash
   cargo build --workspace
   ```

4. **Run tests**:
   ```bash
   # Test individual crates (recommended to avoid Bevy timeout)
   cargo test --package ferrum-physics
   cargo test --package ferrum-world
   cargo test --package ferrum-inventory
   cargo test --package ferrum-meshing-cpu
   cargo test --package ferrum-assets
   cargo test --package ferrum-config
   cargo test --package ferrum-protocol
   cargo test --package ferrum-subprocess
   
   # Note: ferrum and ferrum-render tests may timeout due to Bevy compilation (120s+)
   ```

5. **Read the documentation**:
   - `README.md` - Project overview and quick start
   - `HANDOFF.md` - Comprehensive technical guide
   - `.sisyphus/notepads/` - Technical notes and learnings

## Development Workflow

### Test-Driven Development (TDD)

Ferrum follows a strict TDD approach:

1. **Write tests first**: Before implementing any feature, write tests that define the expected behavior
2. **Run tests**: Verify they fail (red)
3. **Implement**: Write the minimum code to make tests pass (green)
4. **Refactor**: Clean up code while keeping tests green
5. **Repeat**: Continue the cycle

**Example**:
```rust
// tests/physics.rs
#[test]
fn test_player_jumps_when_on_ground() {
    let mut player = Player::new();
    player.on_ground = true;
    player.jump();
    assert!(player.velocity.y > 0.0);
}
```

### Running Tests

```bash
# Run tests for a specific crate
cargo test --package ferrum-physics

# Run a specific test
cargo test --package ferrum-physics test_player_jumps_when_on_ground

# Run tests with output
cargo test --package ferrum-physics -- --nocapture

# Run benchmarks
cargo bench --package ferrum-meshing-cpu
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test --workspace

# Check for compilation errors
cargo check --workspace
```

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/) for clear, semantic commit messages.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **test**: Adding or updating tests
- **perf**: Performance improvement
- **docs**: Documentation changes
- **refactor**: Code refactoring (no behavior change)
- **chore**: Maintenance tasks (dependencies, CI, etc.)
- **style**: Code style changes (formatting, whitespace)

### Scopes

Use the crate name as the scope:
- `physics` - ferrum-physics
- `world` - ferrum-world
- `inventory` - ferrum-inventory
- `meshing` - ferrum-meshing-cpu
- `render` - ferrum-render
- `network` - ferrum/src/network
- `assets` - ferrum-assets
- `config` - ferrum-config
- `protocol` - ferrum-protocol
- `subprocess` - ferrum-subprocess

### Examples

```bash
# Good commits
feat(physics): add swimming mechanics
test(world): add chunk boundary tests
fix(meshing): correct face culling for transparent blocks
perf(meshing): optimize greedy merging algorithm
docs(readme): update installation instructions
chore: update azalea-protocol to v0.10.0

# Bad commits (avoid these)
update stuff
fix bug
WIP
asdf
```

### Commit Best Practices

- **Atomic commits**: Each commit should represent one logical change
- **Present tense**: "add feature" not "added feature"
- **Imperative mood**: "fix bug" not "fixes bug"
- **Lowercase**: Start description with lowercase letter
- **No period**: Don't end description with a period
- **Keep it short**: Description should be <72 characters
- **Explain why**: Use the body to explain why, not what (the diff shows what)

## Pull Request Process

### Before Submitting

1. **Create a feature branch**:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. **Make your changes** following TDD approach

3. **Ensure all tests pass**:
   ```bash
   cargo test --workspace
   ```

4. **Run code quality checks**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   ```

5. **Commit your changes** using conventional commits

6. **Push to your fork**:
   ```bash
   git push origin feat/your-feature-name
   ```

### Submitting the PR

1. **Create a pull request** on GitHub

2. **Fill out the PR template** with:
   - **Summary**: What does this PR do?
   - **Motivation**: Why is this change needed?
   - **Testing**: How was this tested?
   - **Checklist**: Confirm all requirements are met

3. **Link related issues**: Use "Fixes #123" or "Relates to #456"

### PR Requirements

- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Follows conventional commit format
- [ ] Includes tests for new functionality
- [ ] Documentation updated (if applicable)
- [ ] No merge conflicts with main branch

### Review Process

1. **Automated checks**: CI will run tests and linters
2. **Code review**: Maintainers will review your code
3. **Address feedback**: Make requested changes
4. **Approval**: Once approved, your PR will be merged

## Project Structure

Ferrum is organized as a 13-crate workspace:

```
ferrum-core          → Core types (BlockId, Position)
ferrum-protocol      → Minecraft protocol wrapper (azalea-protocol)
ferrum-meshing-cpu   → Binary greedy meshing algorithm
ferrum-render        → Texture atlas, lighting, block rendering
ferrum-world         → Chunk storage, block interaction
ferrum-physics       → Player movement, AABB collision
ferrum-entity        → Entity tracking and synchronization
ferrum-inventory     → Items, stacking, crafting, combat
ferrum-assets        → Multi-source asset loading
ferrum-config        → TOML configuration with hot reload
ferrum-subprocess    → Pumpkin server lifecycle management
ferrum               → Main binary (Bevy app integration)
```

### Where to Contribute

- **Physics**: `ferrum-physics/src/` - Movement, collision, gravity
- **World**: `ferrum-world/src/` - Chunk storage, block interaction
- **Inventory**: `ferrum-inventory/src/` - Items, crafting, combat
- **Meshing**: `ferrum-meshing-cpu/src/` - Chunk mesh generation
- **Rendering**: `ferrum-render/src/` - Lighting, textures, rendering
- **Networking**: `ferrum/src/network/` - Server communication
- **Assets**: `ferrum-assets/src/` - Asset loading and caching
- **Config**: `ferrum-config/src/` - Configuration management

## Testing Guidelines

### Test Organization

- **Unit tests**: In the same file as the code (`#[cfg(test)]` module)
- **Integration tests**: In `{crate}/tests/{module}.rs`
- **Benchmarks**: In `{crate}/benches/{module}.rs`

### Test Naming

```rust
// Good test names (descriptive and specific)
#[test]
fn test_player_stops_at_solid_block() { }

#[test]
fn test_greedy_meshing_merges_adjacent_faces() { }

#[test]
fn test_inventory_stacks_identical_items() { }

// Bad test names (vague)
#[test]
fn test_collision() { }

#[test]
fn test_meshing() { }
```

### Test Coverage

Aim for high test coverage, especially for:
- **Core logic**: Physics, collision, meshing algorithms
- **Edge cases**: Boundary conditions, empty inputs, maximum values
- **Error handling**: Invalid inputs, network failures
- **Performance**: Benchmarks for critical paths

### Running Specific Tests

```bash
# Run all tests in a crate
cargo test --package ferrum-physics

# Run tests matching a pattern
cargo test --package ferrum-physics collision

# Run a specific test
cargo test --package ferrum-physics test_player_stops_at_solid_block

# Run with output
cargo test --package ferrum-physics -- --nocapture

# Run ignored tests
cargo test --package ferrum-physics -- --ignored
```

## Performance Guidelines

Ferrum targets extreme performance. Keep these in mind:

### Performance Targets

- **Phase 1**: 144 FPS, 32 chunks, 4GB RAM ✅ **ACHIEVED**
- **Phase 2**: 240 FPS, 48 chunks, 3GB RAM (in progress)
- **Phase 3**: 240 FPS, 64 chunks, 2GB RAM (research-level)

### Optimization Tips

1. **Profile first**: Use `cargo flamegraph` or `perf` to identify bottlenecks
2. **Benchmark**: Add benchmarks for performance-critical code
3. **Avoid allocations**: Reuse buffers, use `Vec::with_capacity()`
4. **Use SIMD**: Consider `std::simd` for hot loops
5. **Minimize copies**: Use references and slices where possible
6. **Cache-friendly**: Keep data structures compact and sequential

### Benchmarking

```bash
# Run benchmarks
cargo bench --package ferrum-meshing-cpu

# Compare with baseline
cargo bench --package ferrum-meshing-cpu -- --save-baseline main
# ... make changes ...
cargo bench --package ferrum-meshing-cpu -- --baseline main
```

## Documentation

### Code Documentation

- **Public APIs**: Must have doc comments
- **Complex logic**: Add inline comments explaining why
- **Examples**: Include usage examples in doc comments

```rust
/// Performs binary greedy meshing on a chunk.
///
/// This algorithm merges adjacent faces of the same block type
/// to reduce vertex count. It operates in two passes:
/// 1. Binary pass: Identify face visibility
/// 2. Greedy pass: Merge adjacent faces
///
/// # Performance
/// - Realistic terrain: ~64µs per chunk
/// - Worst case (checkerboard): ~506µs per chunk
///
/// # Example
/// ```
/// let chunk = Chunk::new();
/// let mesh = mesh_chunk(&chunk);
/// assert!(mesh.vertices.len() < chunk.block_count() * 24);
/// ```
pub fn mesh_chunk(chunk: &Chunk) -> Mesh {
    // Implementation
}
```

### Documentation Files

When updating documentation:
- **README.md**: User-facing overview and quick start
- **HANDOFF.md**: Technical details for developers (see this for architecture)
- **CONTRIBUTING.md**: This file (contribution guidelines)

## Getting Help

### Resources

1. **Read the docs**:
   - `HANDOFF.md` - Comprehensive technical guide
   - `.sisyphus/notepads/` - Technical notes and learnings
   - Commit history - Implementation patterns

2. **Check existing issues**: Someone may have already asked your question

3. **Ask questions**: Open a GitHub issue with the "question" label

### Reporting Bugs

When reporting bugs, include:
- **Crate and file**: Where the bug occurs
- **Steps to reproduce**: Minimal example to trigger the bug
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Environment**: OS, Rust version, relevant config
- **Test output**: Error messages or test failures

### Suggesting Features

When suggesting features:
- **Use case**: Why is this feature needed?
- **Proposed solution**: How should it work?
- **Alternatives**: What other approaches did you consider?
- **Performance impact**: Will this affect performance targets?

## Current Priorities

### High Priority (Help Wanted!)

1. **Fix Lighting System** (Task 2.A3)
   - Location: `ferrum-render/src/lighting.rs`
   - Issue: 4/15 tests failing
   - Details: See `HANDOFF.md` for root cause analysis

2. **Shadows + Ambient Occlusion** (Task 2.A4)
   - Depends on lighting system fix
   - Implement basic shadow casting
   - Add ambient occlusion to vertices

### Medium Priority

3. **GPU Compute Optimization** (Task 3.1)
   - Research novel GPU meshing approaches
   - Target: <0.2µs per chunk

4. **Memory Compression** (Task 3.2)
   - Implement palette-based chunk compression
   - Target: <2GB for 64 chunks

### Low Priority

5. **Documentation**
   - Complete API documentation
   - Add architecture diagrams
   - Write performance analysis

## License

By contributing to Ferrum, you agree that your contributions will be licensed under the same license as the project (to be determined - likely MIT or Apache-2.0).

## Questions?

If you have questions not covered here:
1. Check `HANDOFF.md` for technical details
2. Search existing GitHub issues
3. Open a new issue with the "question" label

---

**Thank you for contributing to Ferrum!** 🚀

Every contribution, no matter how small, helps make this project better.
