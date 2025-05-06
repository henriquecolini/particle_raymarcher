use glam::{vec3, Vec3};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	position: [f32; 3],
	radius: f32,
}

pub struct ParticleBindingData {
	pub particles_bind_group_layout: wgpu::BindGroupLayout,
	pub particles_bind_group: wgpu::BindGroup,
}

const fn uvec3(x: usize, y: usize, z: usize) -> Vec3 {
	vec3(x as f32, y as f32, z as f32)
}

pub fn grid(size_x: usize, size_y: usize, size_z: usize) -> Vec<Particle> {
	let mut particles = vec![];
	let middle = uvec3(size_x - 1, size_y - 1, size_z - 1) / 2.0;

	for x in 0..size_x {
		for y in 0..size_y {
			for z in 0..size_z {
				particles.push(Particle {
					position: (uvec3(x, y, z) - middle).to_array(),
					radius: 1.0,
				})
			}
		}
	}
	particles
}

impl ParticleBindingData {
	pub fn new(device: &wgpu::Device, data: &[Particle]) -> Self {
		let particles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Particles Buffer"),
			contents: bytemuck::cast_slice(data),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
		});

		let particles_len_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Particles Length Buffer"),
			contents: bytemuck::bytes_of(&(data.len() as u32)),
			usage: wgpu::BufferUsages::UNIFORM,
		});

		let particles_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Particles Bind Group Layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Storage { read_only: true },
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					},
				],
			});

		let particles_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Particles Bind Group"),
			layout: &particles_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: particles_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: particles_len_buffer.as_entire_binding(),
				},
			],
		});

		Self {
			particles_bind_group_layout,
			particles_bind_group,
		}
	}
}
