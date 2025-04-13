use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferrum_meshing_cpu::*;

fn bench_cpu_uniform_air(c: &mut Criterion) {
    let mesher = CpuMesher::new();
    let chunk = uniform_chunk(0);
    c.bench_function("cpu_mesh_uniform_air", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_cpu_uniform_stone(c: &mut Criterion) {
    let mesher = CpuMesher::new();
    let chunk = uniform_chunk(1);
    c.bench_function("cpu_mesh_uniform_stone", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_cpu_checkerboard(c: &mut Criterion) {
    let mesher = CpuMesher::new();
    let chunk = checkerboard_chunk(1);
    c.bench_function("cpu_mesh_checkerboard", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_cpu_terrain(c: &mut Criterion) {
    let mesher = CpuMesher::new();
    let chunk = terrain_chunk();
    c.bench_function("cpu_mesh_terrain", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

criterion_group!(
    benches,
    bench_cpu_uniform_air,
    bench_cpu_uniform_stone,
    bench_cpu_checkerboard,
    bench_cpu_terrain
);
criterion_main!(benches);
