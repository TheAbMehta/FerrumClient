//! GPU compute shader greedy meshing for voxel chunks.
//!
//! Takes a 32x32x32 chunk of block IDs and produces a buffer of greedy-merged
//! quads using a WGSL compute shader dispatched via wgpu.

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

/// Holds wgpu resources for GPU chunk meshing.
pub struct GpuChunkMesher {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuChunkMesher {
    /// Create a new GPU mesher. Initializes wgpu device and compiles the compute shader.
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
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Meshing Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Meshing Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("face_culling"),
            compilation_options: Default::default(),
            cache: None,
        });

        Some(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
        })
    }

    /// Mesh a 32x32x32 chunk of block IDs on the GPU.
    ///
    /// Returns the list of packed quads representing visible, greedy-merged faces.
    pub fn mesh_chunk(&self, voxels: &[u32; CHUNK_SIZE_CB]) -> Vec<PackedQuad> {
        let voxel_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Voxel Buffer"),
                contents: bytemuck::cast_slice(voxels),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let quad_buffer_size = (MAX_QUADS * 2 * std::mem::size_of::<u32>()) as u64;
        let quad_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Output Buffer"),
            size: quad_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let counter_data: [u32; 1] = [0];
        let counter_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Counter Buffer"),
                contents: bytemuck::cast_slice(&counter_data),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            });

        let quad_staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Staging Buffer"),
            size: quad_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let counter_staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Counter Staging Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Meshing Bind Group"),
            layout: &self.bind_group_layout,
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
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Meshing Encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Meshing Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(4, 6, 1);
        }

        encoder.copy_buffer_to_buffer(&quad_buffer, 0, &quad_staging, 0, quad_buffer_size);
        encoder.copy_buffer_to_buffer(&counter_buffer, 0, &counter_staging, 0, 4);

        self.queue.submit(Some(encoder.finish()));

        let counter_slice = counter_staging.slice(..);
        counter_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let counter_data = counter_slice.get_mapped_range();
        let count = bytemuck::cast_slice::<u8, u32>(&counter_data)[0] as usize;
        drop(counter_data);
        counter_staging.unmap();

        let count = count.min(MAX_QUADS);

        if count == 0 {
            return Vec::new();
        }

        let quad_slice = quad_staging.slice(..((count * 2 * 4) as u64));
        quad_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let quad_data = quad_slice.get_mapped_range();
        let quads: &[PackedQuad] = bytemuck::cast_slice(&quad_data);
        let result = quads.to_vec();
        drop(quad_data);
        quad_staging.unmap();

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
