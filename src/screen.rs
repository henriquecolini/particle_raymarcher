use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniform {
	width: f32,
	height: f32,
}

impl ScreenUniform {
	pub fn new(size: PhysicalSize<u32>) -> Self {
		Self {
			width: size.width as f32,
			height: size.height as f32,
		}
	}
	pub fn bytes(&self) -> &[u8] {
		bytemuck::bytes_of(self)
	}
}

pub fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
	device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Size Buffer"),
		contents: bytemuck::bytes_of(&ScreenUniform::default()),
		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	})
}
