use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferrum_meshing_gpu::{terrain_chunk, uniform_chunk, GpuChunkMesher};

fn bench_single_terrain(c: &mut Criterion) {
    let mesher = GpuChunkMesher::new().expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    c.bench_function("single_terrain", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_single_uniform_stone(c: &mut Criterion) {
    let mesher = GpuChunkMesher::new().expect("Failed to create GPU mesher");
    let chunk = uniform_chunk(1);
    c.bench_function("single_uniform_stone", |b| {
        b.iter(|| mesher.mesh_chunk(black_box(&chunk)))
    });
}

fn bench_batch_64_terrain(c: &mut Criterion) {
    let mesher = GpuChunkMesher::with_batch_size(64).expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    let chunks: Vec<&[u32; 32768]> = (0..64).map(|_| &chunk).collect();
    c.bench_function("batch_64_terrain_total", |b| {
        b.iter(|| mesher.mesh_chunks_batch(black_box(&chunks)))
    });
}

fn bench_batch_64_terrain_per_chunk(c: &mut Criterion) {
    let mesher = GpuChunkMesher::with_batch_size(64).expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    let chunks: Vec<&[u32; 32768]> = (0..64).map(|_| &chunk).collect();

    let mut group = c.benchmark_group("amortized_per_chunk");
    group.bench_function("batch_64_terrain", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = mesher.mesh_chunks_batch(black_box(&chunks));
            }
            start.elapsed() / 64
        });
    });
    group.finish();
}

fn bench_batch_64_gpu_only(c: &mut Criterion) {
    let mesher = GpuChunkMesher::with_batch_size(64).expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    let chunks: Vec<&[u32; 32768]> = (0..64).map(|_| &chunk).collect();

    let mut group = c.benchmark_group("gpu_dispatch_amortized");
    group.bench_function("batch_64_terrain", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                mesher.mesh_chunks_batch_gpu(black_box(&chunks));
            }
            start.elapsed() / 64
        });
    });
    group.finish();
}

fn bench_batch_128_gpu_only(c: &mut Criterion) {
    let mesher = GpuChunkMesher::with_batch_size(128).expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    let chunks: Vec<&[u32; 32768]> = (0..128).map(|_| &chunk).collect();

    let mut group = c.benchmark_group("gpu_dispatch_amortized");
    group.bench_function("batch_128_terrain", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                mesher.mesh_chunks_batch_gpu(black_box(&chunks));
            }
            start.elapsed() / 128
        });
    });
    group.finish();
}

fn bench_dispatch_only_no_upload(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_only_no_upload");

    for &batch_size in &[1, 8, 32, 64, 128, 256] {
        let mesher =
            GpuChunkMesher::with_batch_size(batch_size).expect("Failed to create GPU mesher");
        let chunk = terrain_chunk();
        let chunks: Vec<&[u32; 32768]> = (0..batch_size).map(|_| &chunk).collect();
        mesher.mesh_chunks_batch_gpu(&chunks);

        group.bench_function(format!("batch_{batch_size}_terrain"), |b| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                for _ in 0..iters {
                    mesher.dispatch_only(batch_size);
                }
                start.elapsed() / (batch_size as u32)
            });
        });
    }

    group.finish();
}

fn bench_pass_isolation(c: &mut Criterion) {
    let batch = 64;
    let mesher = GpuChunkMesher::with_batch_size(batch).expect("Failed to create GPU mesher");
    let chunk = terrain_chunk();
    let chunks: Vec<&[u32; 32768]> = (0..batch).map(|_| &chunk).collect();
    mesher.mesh_chunks_batch_gpu(&chunks);

    let mut group = c.benchmark_group("pass_isolation");

    group.bench_function("face_culling_64", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                mesher.dispatch_face_culling_only(batch);
            }
            start.elapsed() / (batch as u32)
        });
    });

    group.bench_function("greedy_merge_64", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                mesher.dispatch_greedy_merge_only(batch);
            }
            start.elapsed() / (batch as u32)
        });
    });

    group.bench_function("both_passes_64", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                mesher.dispatch_only(batch);
            }
            start.elapsed() / (batch as u32)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_terrain,
    bench_single_uniform_stone,
    bench_batch_64_terrain,
    bench_batch_64_terrain_per_chunk,
    bench_batch_64_gpu_only,
    bench_batch_128_gpu_only,
    bench_dispatch_only_no_upload,
    bench_pass_isolation,
);
criterion_main!(benches);
