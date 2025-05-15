pub fn create_texture(device: &wgpu::Device, width: u32, height: u32, depth: u32) -> wgpu::Texture {
	device.create_texture(&wgpu::TextureDescriptor {
		label: Some("SDF texture"),
		size: wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: depth,
		},
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D3,
		format: wgpu::TextureFormat::Rgba16Float,
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
		view_formats: &[wgpu::TextureFormat::Rgba16Float],
	})
}

pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
	device.create_sampler(&wgpu::wgt::SamplerDescriptor {
		label: Some("SDF sampler"),
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		..Default::default()
	})
}

pub fn create_view(sdf_texture: &wgpu::Texture) -> wgpu::TextureView {
	sdf_texture.create_view(&wgpu::wgt::TextureViewDescriptor {
		..Default::default()
	})
}
