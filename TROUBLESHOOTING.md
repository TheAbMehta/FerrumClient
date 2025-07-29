# Troubleshooting Guide - Ferrum Minecraft Client

This guide documents common issues and their solutions when building, testing, and running the Ferrum Minecraft client.

## Table of Contents

- [Build Issues](#build-issues)
- [Runtime Issues](#runtime-issues)
- [Test Issues](#test-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Performance Issues](#performance-issues)

---

## Build Issues

### RUSTC_BOOTSTRAP Required

**Symptom**: Build fails with errors about unstable features in `simdnbt` or `azalea-protocol`.

**Cause**: The `azalea-protocol` dependency uses nightly Rust features (`portable_simd`, `allocator_api`) in its `simdnbt` dependency, but we're using stable Rust.

**Solution**: Set the `RUSTC_BOOTSTRAP=1` environment variable before building:

```bash
# Linux/macOS
export RUSTC_BOOTSTRAP=1
cargo build

# Windows PowerShell
$env:RUSTC_BOOTSTRAP=1
cargo build

# Windows CMD
set RUSTC_BOOTSTRAP=1
cargo build
```

**Permanent Solution**: Add to your shell profile:

```bash
# Linux/macOS (~/.bashrc or ~/.zshrc)
echo 'export RUSTC_BOOTSTRAP=1' >> ~/.bashrc

# Windows PowerShell (profile)
Add-Content $PROFILE 'Set-Item -Path Env:RUSTC_BOOTSTRAP -Value "1"'
```

**Note**: This is a workaround to enable nightly features on stable Rust. It's safe for this project but should not be used in production code without understanding the implications.

---

### Bevy Compilation Timeout

**Symptom**: Build hangs or times out after 120+ seconds when compiling Bevy or crates that depend on it (`ferrum`, `ferrum-render`).

**Cause**: Bevy is a large game engine with many dependencies and procedural macros. Compilation takes 2-3 minutes on first build.

**Solution**: Be patient. This is expected behavior.

**Workarounds**:
- Use incremental compilation (enabled by default in Cargo)
- Build individual crates that don't depend on Bevy:
  ```bash
  cargo build --package ferrum-physics
  cargo build --package ferrum-world
  cargo build --package ferrum-inventory
  ```
- Use `cargo check` instead of `cargo build` for faster feedback during development
- Consider using `sccache` or `mold` linker to speed up builds

**CI/CD**: Increase timeout to at least 5 minutes for Bevy-dependent crates.

---

### Missing System Dependencies (Linux)

**Symptom**: Build fails with errors about missing libraries like `X11`, `alsa`, or `udev`.

**Cause**: Bevy requires system libraries for windowing, audio, and input.

**Solution**: Install required dependencies for your distribution:

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libx11-dev libasound2-dev libudev-dev

# Arch Linux
sudo pacman -S base-devel alsa-lib

# Fedora
sudo dnf install gcc pkg-config libX11-devel alsa-lib-devel systemd-devel
```

---

### wgpu Version Mismatch

**Symptom**: Compilation errors about incompatible `wgpu` types or missing methods.

**Cause**: Bevy 0.18 uses `wgpu 27.0.1`, but other dependencies may pull in `wgpu 28.0.0`.

**Solution**: Ensure all crates use compatible wgpu versions. Check `Cargo.toml`:

```toml
[dependencies]
wgpu = "27.0.1"  # Match Bevy's version
```

**Diagnostic Command**:
```bash
cargo tree | grep wgpu
```

---

## Runtime Issues

### Pumpkin Server Fails to Start

**Symptom**: Client logs "Failed to start Pumpkin server" or "Pumpkin startup timeout".

**Cause**: Pumpkin binary not found, port already in use, or server crashed during startup.

**Solution**:

1. **Verify Pumpkin binary exists**:
   ```bash
   ls pumpkin/target/release/pumpkin  # Linux/macOS
   dir pumpkin\target\release\pumpkin.exe  # Windows
   ```

2. **Build Pumpkin manually**:
   ```bash
   cd pumpkin
   cargo build --release
   cd ..
   ```

3. **Check if port 25565 is in use**:
   ```bash
   # Linux/macOS
   lsof -i :25565
   
   # Windows
   netstat -ano | findstr :25565
   ```

4. **Disable auto-start** in `config.toml`:
   ```toml
   [server]
   auto_start = false
   ```
   Then start Pumpkin manually in a separate terminal.

---

### Orphaned Pumpkin Processes

**Symptom**: Multiple Pumpkin server processes remain running after client exits, consuming resources.

**Cause**: Process cleanup failed or client crashed before calling `Drop` on `PumpkinServer`.

**Solution**:

1. **Kill orphaned processes**:
   ```bash
   # Linux/macOS
   pkill -f pumpkin
   
   # Windows
   taskkill /F /IM pumpkin.exe
   ```

2. **Verify process groups** (Linux/macOS only):
   The subprocess manager uses `setpgid(0, 0)` to create process groups. If this fails, orphaned processes may occur.

**Prevention**: The `PumpkinServer` `Drop` implementation should handle cleanup, but panics or forced termination (Ctrl+C) may bypass it.

---

### Asset Loading Failures

**Symptom**: Client logs "Failed to load texture" or "Asset not found".

**Cause**: Asset sources (Mojang CDN, JAR, PrismarineJS) are unreachable or cache is corrupted.

**Solution**:

1. **Clear asset cache**:
   ```bash
   rm -rf ~/.ferrum/cache  # Linux/macOS
   rmdir /S %USERPROFILE%\.ferrum\cache  # Windows
   ```

2. **Check network connectivity**:
   ```bash
   curl -I https://resources.download.minecraft.net
   ```

3. **Verify JAR file exists** (if using JAR source):
   ```bash
   ls ~/.minecraft/versions/1.21.11/1.21.11.jar  # Linux/macOS
   dir %APPDATA%\.minecraft\versions\1.21.11\1.21.11.jar  # Windows
   ```

4. **Change asset source** in `config.toml`:
   ```toml
   [assets]
   source = "prismarine"  # Try alternative source
   ```

---

### Connection to Pumpkin Fails

**Symptom**: Client logs "Connection failed" or "Handshake failed".

**Cause**: Pumpkin server not running, wrong address, or firewall blocking connection.

**Solution**:

1. **Verify Pumpkin is running**:
   ```bash
   # Check if port 25565 is listening
   netstat -an | grep 25565  # Linux/macOS
   netstat -an | findstr 25565  # Windows
   ```

2. **Check server address** in `config.toml`:
   ```toml
   [server]
   address = "127.0.0.1:25565"  # Localhost for local server
   ```

3. **Test connection manually**:
   ```bash
   telnet 127.0.0.1 25565
   ```

4. **Check firewall rules** (if connecting to remote server):
   ```bash
   # Linux
   sudo ufw status
   
   # Windows
   netsh advfirewall show allprofiles
   ```

---

## Test Issues

### Workspace Tests Timeout

**Symptom**: `cargo test --workspace` hangs or times out after 120 seconds.

**Cause**: Bevy compilation takes too long for test timeout limits.

**Solution**: Test individual crates instead:

```bash
# Test non-Bevy crates (fast)
cargo test --package ferrum-physics
cargo test --package ferrum-world
cargo test --package ferrum-inventory
cargo test --package ferrum-meshing-cpu
cargo test --package ferrum-assets
cargo test --package ferrum-config
cargo test --package ferrum-protocol
cargo test --package ferrum-subprocess

# Test Bevy crates separately (slow)
cargo test --package ferrum-render
cargo test --package ferrum
```

**CI/CD**: Use separate test jobs for Bevy and non-Bevy crates.

---

### Lighting System Tests Fail

**Symptom**: 4 out of 15 lighting tests fail in `ferrum-render`:
- `test_opaque_blocks_stop_light`
- `test_sky_light_blocked_by_opaque`
- `test_smooth_lighting_averages_neighbors`
- `test_smooth_lighting_uses_max_of_block_and_sky`

**Cause**: Known bug in light propagation logic. Light passes through opaque blocks and smooth lighting returns incorrect values.

**Status**: BLOCKED - Requires manual debugging of `ferrum-render/src/lighting.rs`.

**Workaround**: Skip lighting tests:
```bash
cargo test --package ferrum-render -- --skip lighting
```

**Fix Instructions** (for developers):
1. Read `ferrum-render/src/lighting.rs` lines 51-175
2. Debug `propagate_block_light()` - ensure opaque blocks stop propagation
3. Debug `propagate_sky_light()` - ensure opaque blocks block sky light
4. Verify `get_smooth_light()` samples correct neighbors (lines 154-175)
5. Run: `cargo test --package ferrum-render`

See `HANDOFF.md` for detailed root cause analysis.

---

### Subprocess Tests Fail with File Conflicts

**Symptom**: Tests in `ferrum-subprocess` fail with "file already exists" or "permission denied" errors.

**Cause**: Parallel test execution creates conflicting temporary files.

**Solution**: Run tests serially:
```bash
cargo test --package ferrum-subprocess -- --test-threads=1
```

**Note**: This is already handled in the test suite with unique temp file names using process IDs.

---

## Platform-Specific Issues

### Linux: Missing Vulkan Support

**Symptom**: Client crashes with "Failed to create Vulkan instance" or "No suitable GPU found".

**Cause**: Missing Vulkan drivers or runtime libraries.

**Solution**:

```bash
# Ubuntu/Debian
sudo apt install mesa-vulkan-drivers vulkan-tools

# Arch Linux
sudo pacman -S vulkan-icd-loader vulkan-tools

# Fedora
sudo dnf install mesa-vulkan-drivers vulkan-tools

# Verify Vulkan support
vulkaninfo
```

---

### Windows: DX12 Not Available

**Symptom**: Client falls back to DX11 or crashes with "DirectX 12 not supported".

**Cause**: Windows 10 version too old or GPU drivers outdated.

**Solution**:

1. **Update Windows**: DX12 requires Windows 10 version 1909 or later.
2. **Update GPU drivers**:
   - NVIDIA: https://www.nvidia.com/Download/index.aspx
   - AMD: https://www.amd.com/en/support
   - Intel: https://www.intel.com/content/www/us/en/download-center/home.html

3. **Verify DX12 support**:
   ```powershell
   dxdiag
   ```
   Check "Feature Levels" includes "12_0" or higher.

---

### macOS: Metal Backend Issues

**Symptom**: Client crashes or renders incorrectly on macOS.

**Cause**: Bevy's Metal backend may have compatibility issues with older macOS versions.

**Solution**:

1. **Update macOS**: Requires macOS 10.15 (Catalina) or later for Metal 2.
2. **Check Metal support**:
   ```bash
   system_profiler SPDisplaysDataType | grep Metal
   ```

3. **Use OpenGL fallback** (if Metal fails):
   Set environment variable:
   ```bash
   export WGPU_BACKEND=gl
   cargo run
   ```

---

## Performance Issues

### Low FPS (Below 60 FPS)

**Symptom**: Client runs slower than expected, especially with high render distance.

**Diagnostic Commands**:
```bash
# Check CPU usage
top  # Linux/macOS
taskmgr  # Windows

# Check GPU usage (Linux)
nvidia-smi  # NVIDIA GPUs
radeontop   # AMD GPUs
```

**Solutions**:

1. **Reduce render distance** in `config.toml`:
   ```toml
   [client]
   render_distance = 16  # Lower from 32
   ```

2. **Enable VSync** (caps FPS to monitor refresh rate):
   ```toml
   [client]
   vsync = true
   ```

3. **Limit FPS** (reduce CPU/GPU load):
   ```toml
   [client]
   fps_limit = 60
   ```

4. **Check for debug build**:
   ```bash
   # Always use release builds for performance testing
   cargo build --release
   cargo run --release
   ```

---

### High Memory Usage (Above 4GB)

**Symptom**: Client consumes excessive RAM, especially with high render distance.

**Cause**: Chunk data not being unloaded or memory leaks.

**Diagnostic Commands**:
```bash
# Monitor memory usage
ps aux | grep ferrum  # Linux/macOS
tasklist /FI "IMAGENAME eq ferrum.exe" /FO LIST  # Windows
```

**Solutions**:

1. **Reduce render distance**:
   ```toml
   [client]
   render_distance = 16  # Each chunk is ~64KB
   ```

2. **Check for memory leaks**:
   ```bash
   # Run with Valgrind (Linux)
   valgrind --leak-check=full cargo run --release
   
   # Run with Instruments (macOS)
   instruments -t Leaks target/release/ferrum
   ```

3. **Profile memory usage**:
   ```bash
   cargo install cargo-profiler
   cargo profiler callgrind --release
   ```

---

### Slow Chunk Meshing (Above 200µs per chunk)

**Symptom**: Chunk loading is slow, causing stuttering or delayed rendering.

**Diagnostic**: Run benchmarks:
```bash
cargo bench --package ferrum-meshing-cpu
```

**Expected Results**:
- Realistic terrain: ~64µs per chunk
- Uniform chunks: 6.5-44µs
- Worst case (checkerboard): ~506µs

**Solutions**:

1. **Verify release build**:
   ```bash
   cargo bench --release
   ```

2. **Check CPU frequency scaling** (Linux):
   ```bash
   cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
   # Should be "performance" not "powersave"
   ```

3. **Profile meshing code**:
   ```bash
   cargo install cargo-flamegraph
   cargo flamegraph --bench meshing_benchmarks
   ```

---

## Getting Help

If your issue is not covered here:

1. **Read documentation**:
   - `README.md` - Quick start and overview
   - `HANDOFF.md` - Detailed technical documentation
   - `PROJECT_STATUS.md` - Current project status

2. **Check logs**:
   - Client logs are printed to stdout/stderr
   - Pumpkin server logs are in `pumpkin/logs/`

3. **Search commit history**:
   ```bash
   git log --all --grep="<keyword>"
   ```

4. **Review test cases**:
   - Tests often document expected behavior
   - Look in `{crate}/tests/` directories

5. **Report issues**:
   - Include error messages, logs, and steps to reproduce
   - Specify OS, Rust version, and hardware specs
   - Mention if issue occurs in debug or release build
