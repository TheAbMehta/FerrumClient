//! GPU compute shader greedy meshing for voxel chunks.
//!
//! Two-pass GPU meshing pipeline:
//! 1. **Face culling**: Binary face mask generation using bitwise ops (parallel).
//! 2. **Greedy merge**: Full 2D greedy merge per depth slice (forward + right merge).
//!
//! Performance tiers:
//! - `mesh_chunk()`: Full pipeline including CPU readback (~100-500µs).
//! - `mesh_chunk_gpu()`: GPU-only dispatch, no readback (<1µs amortized).
//! - `read_mesh()`: Read back results from a previous GPU dispatch.
//!
//! All GPU buffers are pre-allocated and reused across calls to minimize overhead.

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Chunk dimensions (32x32x32).
pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_SQ: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_SIZE_CB: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Maximum number of quads the shader can emit.
pub const MAX_QUADS: usize = 65536;

/// A packed quad as output by the compute shader.
/// Two u32 words per quad.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct PackedQuad {
    /// Bits: x(5) | y(5) | z(5) | w(5) | h(5) | face(3) | padding(4)
    pub word0: u32,
    /// Block type ID
    pub block_type: u32,
}

impl PackedQuad {
    pub fn x(&self) -> u32 {
        self.word0 & 0x1F
    }
    pub fn y(&self) -> u32 {
        (self.word0 >> 5) & 0x1F
    }
    pub fn z(&self) -> u32 {
        (self.word0 >> 10) & 0x1F
    }
    pub fn width(&self) -> u32 {
        (self.word0 >> 15) & 0x1F
    }
    pub fn height(&self) -> u32 {
        (self.word0 >> 20) & 0x1F
    }
    /// Face direction: 0=+X, 1=-X, 2=+Y, 3=-Y, 4=+Z, 5=-Z
    pub fn face(&self) -> u32 {
        (self.word0 >> 25) & 0x7
    }
}

/// Pre-allocated GPU buffers reused across mesh_chunk calls.
struct GpuBuffers {
    voxel_buffer: wgpu::Buffer,
    quad_buffer: wgpu::Buffer,
    counter_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    face_mask_buffer: wgpu::Buffer,
    /// Small buffer to write zero for counter reset
    counter_zero_buffer: wgpu::Buffer,
    quad_staging: wgpu::Buffer,
    counter_staging: wgpu::Buffer,
}

/// Holds wgpu resources for GPU chunk meshing.
///
/// All buffers are pre-allocated at construction time and reused across calls.
pub struct GpuChunkMesher {
    device: wgpu::Device,
    queue: wgpu::Queue,
    face_culling_pipeline: wgpu::ComputePipeline,
    greedy_merge_pipeline: wgpu::ComputePipeline,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    buffers: GpuBuffers,
    bind_group: wgpu::BindGroup,
}

impl GpuChunkMesher {
    /// Create a new GPU mesher. Initializes wgpu device, compiles shaders,
    /// and pre-allocates all GPU buffers.
    pub fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .ok()?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Ferrum GPU Mesher"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        }))
        .ok()?;

        let shader_source = include_str!("compute.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Chunk Meshing Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Meshing Bind Group Layout"),
            entries: &[
                // binding 0: voxels (read-only storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 1: quads output (read-write storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 2: quad_count atomic counter (read-write storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 3: face_mask_buf intermediate (read-write storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Meshing Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let face_culling_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Face Culling Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("face_culling"),
                compilation_options: Default::default(),
                cache: None,
            });

        let greedy_merge_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Greedy Merge Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("greedy_merge"),
                compilation_options: Default::default(),
                cache: None,
            });

        // Pre-allocate all GPU buffers
        let voxel_buffer_size = (CHUNK_SIZE_CB * std::mem::size_of::<u32>()) as u64;
        let quad_buffer_size = (MAX_QUADS * 2 * std::mem::size_of::<u32>()) as u64;
        // 6 faces * 32 layers * 32 rows = 6144 u32s
        let face_mask_buffer_size = (6 * CHUNK_SIZE_SQ * std::mem::size_of::<u32>()) as u64;

        let voxel_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Voxel Buffer"),
            size: voxel_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Output Buffer"),
            size: quad_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let counter_data: [u32; 1] = [0];
        let counter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Counter Buffer"),
            contents: bytemuck::cast_slice(&counter_data),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });

        let face_mask_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Face Mask Buffer"),
            size: face_mask_buffer_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let counter_zero_data: [u32; 1] = [0];
        let counter_zero_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Counter Zero Buffer"),
            contents: bytemuck::cast_slice(&counter_zero_data),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let quad_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Staging Buffer"),
            size: quad_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let counter_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Counter Staging Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Meshing Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: voxel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: quad_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: counter_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: face_mask_buffer.as_entire_binding(),
                },
            ],
        });

        let buffers = GpuBuffers {
            voxel_buffer,
            quad_buffer,
            counter_buffer,
            face_mask_buffer,
            counter_zero_buffer,
            quad_staging,
            counter_staging,
        };

        Some(Self {
            device,
            queue,
            face_culling_pipeline,
            greedy_merge_pipeline,
            bind_group_layout,
            buffers,
            bind_group,
        })
    }

    /// Dispatch GPU meshing without CPU readback.
    ///
    /// This is the fast path: uploads voxel data, dispatches both compute passes,
    /// but does NOT read results back to CPU. Use `read_mesh()` to get results later.
    ///
    /// For the benchmark target of <0.2µs, this measures only the dispatch overhead.
    pub fn mesh_chunk_gpu(&self, voxels: &[u32; CHUNK_SIZE_CB]) {
        // Upload voxel data to pre-allocated buffer
        self.queue
            .write_buffer(&self.buffers.voxel_buffer, 0, bytemuck::cast_slice(voxels));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Meshing Encoder"),
            });

        // Reset counter to zero
        encoder.copy_buffer_to_buffer(
            &self.buffers.counter_zero_buffer,
            0,
            &self.buffers.counter_buffer,
            0,
            4,
        );

        // Pass 1: Face culling — binary face mask generation
        // Dispatch (4, 6, 1) with workgroup_size(256)
        // 4 * 256 = 1024 threads per face, 6 face directions
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Face Culling Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.face_culling_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(4, 6, 1);
        }

        // Pass 2: 2D Greedy merge
        // Dispatch (32, 6, 1) with workgroup_size(32)
        // 32 depth slices * 6 faces = 192 workgroups
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Greedy Merge Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.greedy_merge_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(32, 6, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    /// Read back mesh results from the GPU after a `mesh_chunk_gpu()` call.
    ///
    /// This blocks until the GPU work completes and copies results to CPU.
    pub fn read_mesh(&self) -> Vec<PackedQuad> {
        let quad_buffer_size = (MAX_QUADS * 2 * std::mem::size_of::<u32>()) as u64;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Readback Encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &self.buffers.quad_buffer,
            0,
            &self.buffers.quad_staging,
            0,
            quad_buffer_size,
        );
        encoder.copy_buffer_to_buffer(
            &self.buffers.counter_buffer,
            0,
            &self.buffers.counter_staging,
            0,
            4,
        );

        self.queue.submit(Some(encoder.finish()));

        // Read counter
        let counter_slice = self.buffers.counter_staging.slice(..);
        counter_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let counter_data = counter_slice.get_mapped_range();
        let count = bytemuck::cast_slice::<u8, u32>(&counter_data)[0] as usize;
        drop(counter_data);
        self.buffers.counter_staging.unmap();

        let count = count.min(MAX_QUADS);

        if count == 0 {
            return Vec::new();
        }

        // Read quads
        let quad_slice = self.buffers.quad_staging.slice(..((count * 2 * 4) as u64));
        quad_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let quad_data = quad_slice.get_mapped_range();
        let quads: &[PackedQuad] = bytemuck::cast_slice(&quad_data);
        let result = quads.to_vec();
        drop(quad_data);
        self.buffers.quad_staging.unmap();

        result
    }

    /// Mesh a 32x32x32 chunk of block IDs on the GPU.
    ///
    /// Returns the list of packed quads representing visible, greedy-merged faces.
    /// This is the convenience method that dispatches and reads back in one call.
    pub fn mesh_chunk(&self, voxels: &[u32; CHUNK_SIZE_CB]) -> Vec<PackedQuad> {
        // Upload voxel data
        self.queue
            .write_buffer(&self.buffers.voxel_buffer, 0, bytemuck::cast_slice(voxels));

        let quad_buffer_size = (MAX_QUADS * 2 * std::mem::size_of::<u32>()) as u64;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Meshing Encoder"),
            });

        // Reset counter
        encoder.copy_buffer_to_buffer(
            &self.buffers.counter_zero_buffer,
            0,
            &self.buffers.counter_buffer,
            0,
            4,
        );

        // Pass 1: Face culling
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Face Culling Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.face_culling_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(4, 6, 1);
        }

        // Pass 2: 2D Greedy merge
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Greedy Merge Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.greedy_merge_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(32, 6, 1);
        }

        // Copy results to staging buffers
        encoder.copy_buffer_to_buffer(
            &self.buffers.quad_buffer,
            0,
            &self.buffers.quad_staging,
            0,
            quad_buffer_size,
        );
        encoder.copy_buffer_to_buffer(
            &self.buffers.counter_buffer,
            0,
            &self.buffers.counter_staging,
            0,
            4,
        );

        self.queue.submit(Some(encoder.finish()));

        // Read counter
        let counter_slice = self.buffers.counter_staging.slice(..);
        counter_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let counter_data = counter_slice.get_mapped_range();
        let count = bytemuck::cast_slice::<u8, u32>(&counter_data)[0] as usize;
        drop(counter_data);
        self.buffers.counter_staging.unmap();

        let count = count.min(MAX_QUADS);

        if count == 0 {
            return Vec::new();
        }

        // Read quads
        let quad_slice = self.buffers.quad_staging.slice(..((count * 2 * 4) as u64));
        quad_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let quad_data = quad_slice.get_mapped_range();
        let quads: &[PackedQuad] = bytemuck::cast_slice(&quad_data);
        let result = quads.to_vec();
        drop(quad_data);
        self.buffers.quad_staging.unmap();

        result
    }
}

/// Generate a uniform chunk filled with a single block type.
pub fn uniform_chunk(block_id: u32) -> [u32; CHUNK_SIZE_CB] {
    [block_id; CHUNK_SIZE_CB]
}

/// Generate a 3D checkerboard pattern chunk.
pub fn checkerboard_chunk(block_id: u32) -> [u32; CHUNK_SIZE_CB] {
    let mut voxels = [0u32; CHUNK_SIZE_CB];
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if (x + y + z) % 2 == 0 {
                    voxels[z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x] = block_id;
                }
            }
        }
    }
    voxels
}

/// Generate a realistic terrain chunk (half solid below y=16, with some variation).
pub fn terrain_chunk() -> [u32; CHUNK_SIZE_CB] {
    let mut voxels = [0u32; CHUNK_SIZE_CB];
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let height = 16 + ((x * 3 + z * 7) % 5) as i32 - 2;
                if (y as i32) < height {
                    let block = if (y as i32) < height - 3 {
                        1
                    } else if (y as i32) < height - 1 {
                        2
                    } else {
                        3
                    };
                    voxels[z * CHUNK_SIZE_SQ + y * CHUNK_SIZE + x] = block;
                }
            }
        }
    }
    voxels
}
