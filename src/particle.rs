use std::f32::consts::TAU;

use glam::{vec2, vec3, Vec2, Vec3};
use rand::Rng;
use wgpu::util::DeviceExt;

pub const BUNDLE_SIZE: u32 = 32;
pub const BUNDLE_SIZE_BYTES: u32 = std::mem::size_of::<Particle>() as u32 * BUNDLE_SIZE;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
	position: [f32; 3],
	radius: f32,
}

impl Particle {
	pub fn position(&self) -> Vec3 {
		vec3(self.position[0], self.position[1], self.position[2])
	}
}

const fn uvec3(x: usize, y: usize, z: usize) -> Vec3 {
	vec3(x as f32, y as f32, z as f32)
}

const fn uvec2(x: usize, y: usize) -> Vec2 {
	vec2(x as f32, y as f32)
}

pub fn as_bundled(particles: &[Particle]) -> &[Particle] {
	const N: usize = BUNDLE_SIZE as usize;
	let max_len = (particles.len() / N) * N;
	&particles[0..max_len]
}

pub fn into_bundled(mut particles: Vec<Particle>) -> Box<[Particle]> {
	const N: usize = BUNDLE_SIZE as usize;
	let max_len = (particles.len() / N) * N;
	while particles.len() > max_len {
		particles.remove(particles.len() - 1);
	}
	particles.into_boxed_slice()
}

#[allow(unused)]
pub fn grid(n_x: usize, n_y: usize, n_z: usize) -> Vec<Particle> {
	let mut particles = vec![];
	let mut rng = rand::rng();
	let size = uvec3(n_x, n_y, n_z);
	for x in 0..n_x {
		for y in 0..n_y {
			for z in 0..n_z {
				let mut position = uvec3(x, y, z);
				position += vec3(0.5, 0.5, 0.5);
				position /= size;
				position -= vec3(0.5, 0.5, 0.5);
				let position = position.to_array();
				particles.push(Particle {
					position,
					radius: rng.random_range(0.05..=0.1),
					..Default::default()
				})
			}
		}
	}
	particles
}

#[allow(unused)]
pub fn wave(
	n_x: usize,
	n_z: usize,
	width: f32,
	depth: f32,
	amp_x: f32,
	amp_z: f32,
	freq_x: f32,
	freq_z: f32,
) -> Vec<Particle> {
	let mut particles = vec![];
	let size = uvec2(n_x, n_z);
	for x in 0..n_x {
		for z in 0..n_z {
			let Vec2 { x, y } =
				(((uvec2(x, z) + vec2(0.5, 0.5)) / size) - vec2(0.5, 0.5)) * vec2(width, depth);
			let z = y;
			let height = (amp_x * (x * TAU * freq_x).sin()) + (amp_z * (z * TAU * freq_z).sin());
			let position = (vec3(x, height, z)).to_array();
			particles.push(Particle {
				position,
				radius: 0.1,
				..Default::default()
			})
		}
	}
	particles
}

#[allow(unused)]
pub fn random(n: usize) -> Vec<Particle> {
	let mut rng = rand::rng();
	let mut particles = Vec::with_capacity(n);

	for _ in 0..n {
		let position = [
			rng.random_range(0.2..=0.8),
			rng.random_range(0.2..=0.8),
			rng.random_range(0.2..=0.8),
		];
		let radius = rng.random_range(0.025..=0.05);
		particles.push(Particle {
			position,
			radius,
			..Default::default()
		});
	}

	particles
}

pub fn create_buffer(device: &wgpu::Device, particles: &[Particle]) -> wgpu::Buffer {
	device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Particle Buffer"),
		contents: bytemuck::cast_slice(as_bundled(particles)),
		usage: wgpu::BufferUsages::STORAGE,
	})
}

// impl ParticleBindingData {
// 	pub fn new(device: &wgpu::Device, data: &[Particle]) -> Self {
// 		let particles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// 			label: Some("Particles Buffer"),
// 			contents: bytemuck::cast_slice(data),
// 			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
// 		});

// 		let particles_len_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// 			label: Some("Particles Length Buffer"),
// 			contents: bytemuck::bytes_of(&(data.len() as u32)),
// 			usage: wgpu::BufferUsages::UNIFORM,
// 		});

// 		let particles_bind_group_layout =
// 			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
// 				label: Some("Particles Bind Group Layout"),
// 				entries: &[
// 					wgpu::BindGroupLayoutEntry {
// 						binding: 0,
// 						visibility: wgpu::ShaderStages::FRAGMENT,
// 						ty: wgpu::BindingType::Buffer {
// 							ty: wgpu::BufferBindingType::Storage { read_only: true },
// 							has_dynamic_offset: false,
// 							min_binding_size: None,
// 						},
// 						count: None,
// 					},
// 					wgpu::BindGroupLayoutEntry {
// 						binding: 1,
// 						visibility: wgpu::ShaderStages::FRAGMENT,
// 						ty: wgpu::BindingType::Buffer {
// 							ty: wgpu::BufferBindingType::Uniform,
// 							has_dynamic_offset: false,
// 							min_binding_size: None,
// 						},
// 						count: None,
// 					},
// 				],
// 			});

// 		let particles_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
// 			label: Some("Particles Bind Group"),
// 			layout: &particles_bind_group_layout,
// 			entries: &[
// 				wgpu::BindGroupEntry {
// 					binding: 0,
// 					resource: particles_buffer.as_entire_binding(),
// 				},
// 				wgpu::BindGroupEntry {
// 					binding: 1,
// 					resource: particles_len_buffer.as_entire_binding(),
// 				},
// 			],
// 		});

// 		Self {
// 			particles_bind_group_layout,
// 			particles_bind_group,
// 		}
// 	}
// }
