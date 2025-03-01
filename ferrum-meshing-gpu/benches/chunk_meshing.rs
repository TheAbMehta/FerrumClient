use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferrum_meshing_gpu::{checkerboard_chunk, terrain_chunk, uniform_chunk, GpuChunkMesher};

fn bench_uniform_air(c: &mut Criterion) {
    let mesher = GpuChunkMesher::new().expect("Failed to create GPU mesher");
    let chunk = uniform_chunk(0);
    c.bench_function("mesh_uniform_air", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_uniform_stone(c: &mut Criterion) {
    let mesher = GpuChunkMesher::new().expect("Failed to create GPU mesher");
    let chunk = uniform_chunk(1);
    c.bench_function("mesh_uniform_stone", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_checkerboard(c: &mut Criterion) {
    let mesher = GpuChunkMesher::new().expect("Failed to create GPU mesher");
    let chunk = checkerboard_chunk(1);
    c.bench_function("mesh_checkerboard", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_terrain(c: &mut Criterion) {
    let mesher = GpuChunkMesher::new().expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    c.bench_function("mesh_terrain", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

criterion_group!(
    benches,
    bench_uniform_air,
    bench_uniform_stone,
    bench_checkerboard,
    bench_terrain
);
criterion_main!(benches);
