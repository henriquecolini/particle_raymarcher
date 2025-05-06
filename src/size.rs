use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SizeUniform {
	width: f32,
	height: f32,
}

pub struct SizeBindingData {
	pub size_buffer: wgpu::Buffer,
	pub size_bind_group_layout: wgpu::BindGroupLayout,
	pub size_bind_group: wgpu::BindGroup,
}

impl SizeUniform {
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

impl SizeBindingData {
	pub fn new(device: &wgpu::Device) -> Self {
		let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Size Buffer"),
			contents: bytemuck::bytes_of(&SizeUniform::default()),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let size_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Size Bind Group Layout"),
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

		let size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Size Bind Group"),
			layout: &size_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: size_buffer.as_entire_binding(),
			}],
		});

		Self {
			size_buffer,
			size_bind_group_layout,
			size_bind_group,
		}
	}
}
