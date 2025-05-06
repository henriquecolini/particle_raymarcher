use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeUniform {
	s: f32,
}

pub struct TimeBindingData {
	pub time_buffer: wgpu::Buffer,
	pub time_bind_group_layout: wgpu::BindGroupLayout,
	pub time_bind_group: wgpu::BindGroup,
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

impl TimeBindingData {
	pub fn new(device: &wgpu::Device) -> Self {
		let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Time Buffer"),
			contents: bytemuck::bytes_of(&TimeUniform { s: 0.0 }),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let time_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Time Bind Group Layout"),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});

		let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Time Bind Group"),
			layout: &time_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: time_buffer.as_entire_binding(),
			}],
		});

		Self {
			time_buffer,
			time_bind_group_layout,
			time_bind_group,
		}
	}
}
