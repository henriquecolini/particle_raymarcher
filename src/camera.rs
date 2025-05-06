use std::f32::consts::{PI, TAU};

use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

#[derive(Debug, Default)]
pub struct Camera {
	pub position: Vec3,
	pub yaw: f32,
	pub pitch: f32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	position: [f32; 3],
	_pad: f32,
	direction: [f32; 3],
	_pad2: f32,
}

impl Camera {
	pub fn new() -> Self {
		Self {
			position: Vec3 {
				x: 0.0,
				y: 0.0,
				z: -3.0,
			},
			..Default::default()
		}
	}
	pub fn forward_dir(&self) -> Vec3 {
		let (x, z) = self.yaw.sin_cos();
		Vec3 { x, y: 0.0, z }
	}
	pub fn right_dir(&self) -> Vec3 {
		Vec3::Y.cross(self.forward_dir())
	}
	pub fn look_dir(&self) -> Vec3 {
		Vec3 {
			x: self.yaw.sin() * self.pitch.cos(),
			y: self.pitch.sin(),
			z: self.yaw.cos() * self.pitch.cos(),
		}
		.normalize_or_zero()
	}
	pub fn uniform(&self) -> CameraUniform {
		CameraUniform {
			position: self.position.to_array(),
			direction: self.look_dir().to_array(),
			..Default::default()
		}
	}

	pub fn update(&mut self, input_dir: Vec3, mouse_delta: Vec2, time_delta: f32) {
		let mov_dir = input_dir.x * self.right_dir()
			+ input_dir.y * self.forward_dir()
			+ input_dir.z * Vec3::Y;
		self.yaw += (mouse_delta.x) * time_delta * 0.2;
		self.pitch += (-mouse_delta.y) * time_delta * 0.2;
		self.yaw = self.yaw.rem_euclid(TAU);
		self.pitch = self
			.pitch
			.clamp((-PI / 2.0) + f32::EPSILON, (PI / 2.0) - f32::EPSILON);
		self.position += mov_dir * time_delta;
	}
}

impl CameraUniform {
	pub fn bytes(&self) -> &[u8] {
		bytemuck::bytes_of(self)
	}
}

pub fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
	device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Camera Buffer"),
		contents: bytemuck::bytes_of(&CameraUniform::default()),
		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	})
}
