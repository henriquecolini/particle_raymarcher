use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeUniform {
	s: f32,
}

impl TimeUniform {
	pub fn since(since: &std::time::Instant) -> Self {
		Self {
			s: since.elapsed().as_secs_f32(),
		}
	}
	pub fn bytes(&self) -> &[u8] {
		bytemuck::bytes_of(self)
	}
}

pub fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
	device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Time Buffer"),
		contents: bytemuck::bytes_of(&TimeUniform { s: 0.0 }),
		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	})
}
