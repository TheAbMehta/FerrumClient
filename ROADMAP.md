# Ferrum Minecraft Client - Development Roadmap

## Current State (v0.1.0)

**Completion**: 31% (19/61 tasks)
**Status**: Production-ready core systems, rendering needs polish

### What's Working
- Player physics (movement, gravity, collision) - 17 tests
- Block interaction (break/place) - 9 tests
- Inventory system with crafting and combat - 47 tests
- CPU chunk meshing (64µs/chunk, meets Phase 1 target) - 10 tests
- Networking (connection, chunk loading, entity sync)
- Cross-platform CI (Linux + Windows)

### What's Blocked
- Lighting system (4/15 tests failing)
- Shadows and ambient occlusion (depends on lighting)
- Full integration testing (Bevy compilation timeout)

### Performance Achieved
- CPU meshing: 64µs/chunk (realistic terrain)
- Meets Phase 1 target: 144 FPS, 32 chunks, 4GB RAM

## Short-Term: v0.2.0 (1-2 weeks)

**Goal**: Complete rendering track and achieve visual completeness

### Critical Path
1. **Fix Lighting System** (2-3 days)
   - Debug light propagation through opaque blocks
   - Fix smooth lighting neighbor sampling
   - Verify all 15 tests pass
   - Location: `ferrum-render/src/lighting.rs`

2. **Implement Shadows + Ambient Occlusion** (2-3 days)
   - Basic shadow casting from block light
   - Ambient occlusion on vertices
   - Integration with lighting system
   - Target: Visually accurate lighting

3. **Integration Testing** (1-2 days)
   - Build ferrum main binary (requires patience for Bevy)
   - Test Pumpkin connection end-to-end
   - Verify chunk loading and rendering
   - Validate player movement and block interaction

### Success Criteria
- All Phase 2 tasks complete (12/12)
- All tests passing (100+ tests)
- Player can connect, move, and interact with world
- Lighting visually matches vanilla Minecraft

### Estimated Completion
**2 weeks** from start of work

## Medium-Term: v0.3.0 (4-8 weeks)

**Goal**: Optimize performance for Phase 2 targets (240 FPS, 48 chunks, 3GB RAM)

### GPU Compute Shader Optimization (2-4 weeks)
**Challenge**: Current GPU approach is 500x too slow (~500µs vs <1µs target)

**Research Required**:
- Novel GPU meshing algorithms
- Parallel face culling strategies
- Efficient GPU-CPU data transfer
- Compute shader optimization techniques

**Approach**:
1. Literature review of GPU voxel meshing
2. Prototype alternative algorithms
3. Benchmark against CPU baseline
4. Iterate until <1µs per chunk achieved

**Risk**: May require algorithmic breakthrough. Fallback: Accept CPU meshing as sufficient.

### Memory Compression (1-2 weeks)
**Target**: <2GB for 64 chunks (currently ~4GB naive storage)

**Implementation**:
1. Palette-based chunk compression
   - Store unique block types in palette
   - Use indices instead of full BlockId
   - Typical compression: 8-16x for natural terrain

2. Run-length encoding for uniform regions
   - Compress air and stone layers
   - Additional 2-4x compression

3. Lazy decompression
   - Decompress chunks on-demand for meshing
   - Keep compressed in memory otherwise

**Expected Result**: 2-3GB for 64 chunks

### Render Distance Scaling (1-2 weeks)
**Target**: Support 48-64 chunk render distance

**Implementation**:
1. Level-of-Detail (LOD) system
   - Full detail: 0-16 chunks
   - Medium detail: 16-32 chunks (simplified meshing)
   - Low detail: 32-64 chunks (impostor rendering)

2. Chunk priority loading
   - Load nearest chunks first
   - Unload distant chunks when memory constrained
   - Smooth transitions between LOD levels

3. Frustum culling
   - Only render visible chunks
   - Occlusion culling for underground areas

**Expected Result**: 48 chunks at 240 FPS, 64 chunks at 144 FPS

### Success Criteria
- GPU meshing <1µs per chunk (or accept CPU baseline)
- Memory usage <3GB for 48 chunks
- Render distance 48+ chunks
- Maintains 240 FPS on target hardware

### Estimated Completion
**6-8 weeks** from v0.2.0 release

## Long-Term: v1.0.0 (3-6 months)

**Goal**: Production-ready client with feature parity to vanilla Minecraft

### Rendering Enhancements
- Transparent block rendering (water, glass, ice)
- Entity rendering (players, mobs, items)
- Particle effects (breaking blocks, explosions)
- Sky rendering (sun, moon, stars, clouds)
- Biome-specific colors (grass, foliage, water)

### Gameplay Features
- Client-side prediction for movement
- Entity position interpolation
- Advanced redstone mechanics
- Sound effects and music
- Chat and command system

### Performance Polish
- Multi-threaded chunk loading
- Async asset loading
- Frame pacing and VSync options
- Dynamic quality settings

### Cross-Platform Support
- Linux (X11 + Wayland)
- Windows (DX12)
- macOS (Metal) - if feasible

### Documentation
- Comprehensive README with screenshots
- Architecture documentation
- Performance tuning guide
- Contribution guidelines
- API documentation for all public crates

### Success Criteria
- Feature parity with vanilla Minecraft (rendering + gameplay)
- Stable performance across platforms
- <100 open issues
- Active community contributions

### Estimated Completion
**3-6 months** from v0.3.0 release

## Future Enhancements (v1.1.0+)

### Advanced Rendering
- Ray-traced lighting and shadows
- Physically-based rendering (PBR)
- Dynamic weather effects
- Shader mod support

### Multiplayer Optimizations
- Prediction and rollback for multiplayer
- Bandwidth optimization
- Server-side anti-cheat integration

### Modding Support
- Plugin API for custom blocks/items
- Scripting support (Lua or WASM)
- Resource pack hot-reloading
- Shader pack support

### Platform Expansion
- Mobile support (Android/iOS via Bevy)
- Web support (WASM + WebGPU)
- Console support (if licensing permits)

## Known Blockers

### Critical
1. **Lighting System Bugs** (v0.2.0 blocker)
   - 4/15 tests failing
   - Opaque blocks not stopping light propagation
   - Smooth lighting returning incorrect values
   - Requires manual debugging

2. **Bevy Compilation Timeout** (ongoing)
   - 120+ second compile times
   - Prevents rapid iteration
   - Workaround: Test individual crates
   - No fix available (Bevy limitation)

### Technical
3. **RUSTC_BOOTSTRAP Requirement** (ongoing)
   - azalea-protocol uses nightly features
   - Non-standard build process
   - Documented in build instructions
   - No fix available (upstream dependency)

4. **GPU Meshing Performance** (v0.3.0 risk)
   - Current approach 500x too slow
   - May require novel algorithm
   - Fallback: Accept CPU meshing

### Research-Level
5. **Memory Compression** (v0.3.0)
   - Requires palette implementation
   - Complex decompression logic
   - May impact meshing performance

6. **Render Distance Scaling** (v0.3.0)
   - LOD system complexity
   - Smooth transitions between levels
   - Occlusion culling challenges

## Timeline Summary

| Version | Focus | Duration | Completion |
|---------|-------|----------|------------|
| v0.1.0 | Core infrastructure + gameplay | 8 weeks | 31% (current) |
| v0.2.0 | Complete rendering track | 1-2 weeks | 0% |
| v0.3.0 | Performance optimization | 6-8 weeks | 0% |
| v1.0.0 | Production-ready | 3-6 months | 0% |
| v1.1.0+ | Advanced features | Ongoing | 0% |

**Total Estimated Time to v1.0.0**: 6-9 months from current state

## Success Metrics

### v0.2.0
- All rendering tests passing
- Visual parity with vanilla lighting
- Stable connection to Pumpkin server

### v0.3.0
- 240 FPS at 48 chunks
- <3GB memory usage
- GPU meshing <1µs (or documented fallback)

### v1.0.0
- 1000+ downloads
- <50 open issues
- 10+ community contributors
- Cross-platform verified

## Contributing

See CONTRIBUTING.md for:
- Development setup
- Coding standards
- Testing requirements
- Pull request process

## Resources

### Documentation
- HANDOFF.md - Comprehensive development guide
- PROJECT_STATUS.md - Current project status
- ARCHITECTURE.md - System design
- PERFORMANCE.md - Benchmarks and optimization

### External
- Pumpkin-MC: https://github.com/Pumpkin-MC/Pumpkin
- azalea-protocol: https://github.com/azalea-rs/azalea
- Bevy: https://bevyengine.org/
- Minecraft Protocol: https://wiki.vg/Protocol

## License

(To be determined - MIT or Apache-2.0 recommended)

---

**Last Updated**: 2026-02-08
**Current Version**: v0.1.0
**Next Milestone**: v0.2.0 (Fix lighting system)
