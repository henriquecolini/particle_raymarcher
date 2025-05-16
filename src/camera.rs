use std::f32::{
	self,
	consts::{PI, TAU},
};

use glam::{Mat4, Vec2, Vec3};
use wgpu::util::DeviceExt;

#[derive(Debug, Default)]
pub struct Camera {
	pub aspect: f32,
	pub fov: f32,
	pub position: Vec3,
	pub yaw: f32,
	pub pitch: f32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	position: [f32; 3],
	aspect: f32,
	right: [f32; 3],
	fov: f32,
	up: [f32; 3],
	fov_scale: f32,
	forward: [f32; 4],
	proj: [f32; 16],
	view: [f32; 16],
	inv_proj: [f32; 16],
	inv_view: [f32; 16],
}

impl Camera {
	pub fn new() -> Self {
		Self {
			position: Vec3 {
				x: 0.0,
				y: 0.0,
				z: -5.0,
			},
			aspect: 1.0,
			fov: (60.0f32).to_radians(),
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
	pub fn view_matrix(&self) -> Mat4 {
		let forward = self.look_dir();
		let right = self.right_dir();
		let up = forward.cross(right);
		Mat4::look_to_lh(self.position, forward, up)
	}
	pub fn projection_matrix(&self) -> Mat4 {
		Mat4::perspective_lh(self.fov, self.aspect, 0.05, 20.0)
	}
	pub fn uniform(&self) -> CameraUniform {
		let forward = self.look_dir();
		let right = self.right_dir();
		let up = forward.cross(right);
		let proj = self.projection_matrix();
		let view = self.view_matrix();
		CameraUniform {
			aspect: self.aspect,
			fov: self.fov,
			fov_scale: (self.fov / 2.0).tan(),
			position: self.position.to_array(),
			right: right.to_array(),
			up: up.to_array(),
			forward: forward.extend(1.0).to_array(),
			proj: proj.to_cols_array(),
			view: view.to_cols_array(),
			inv_proj: proj.inverse().to_cols_array(),
			inv_view: view.inverse().to_cols_array(),
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
