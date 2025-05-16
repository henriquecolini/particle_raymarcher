use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniform {
	size: [f32; 2],
}

impl ScreenUniform {
	pub fn new(size: PhysicalSize<u32>) -> Self {
		let width = size.width as f32;
		let height = size.height as f32;
		let size = [width, height];
		Self { size }
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
