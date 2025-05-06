use crate::{
	camera::{self, Camera},
	screen, sdf, time,
};
use glam::{Vec2, Vec3};
use std::{collections::HashSet, sync::Arc};
use wgpu::{Extent3d, TexelCopyBufferLayout, TexelCopyTextureInfo};
use winit::{
	dpi::PhysicalPosition,
	event::{ElementState, RawKeyEvent},
	keyboard::KeyCode,
	window::{CursorGrabMode, Window},
};

pub struct State {
	window: Arc<Window>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	size: winit::dpi::PhysicalSize<u32>,
	surface: wgpu::Surface<'static>,
	surface_format: wgpu::TextureFormat,
	pipeline: wgpu::RenderPipeline,
	uniform_group: wgpu::BindGroup,
	screen_buffer: wgpu::Buffer,
	time_buffer: wgpu::Buffer,
	camera_buffer: wgpu::Buffer,
	start_time: std::time::Instant,
	last_time: std::time::Instant,
	camera: Camera,
	input: Input,
	locked: bool,
}

#[derive(Default)]
struct Input {
	keys: HashSet<KeyCode>,
	mouse_delta: Vec2,
}
impl Input {
	fn pressed<const N: usize>(&self, keys: [KeyCode; N]) -> [bool; N] {
		let mut out = [false; N];
		for (i, key) in keys.iter().enumerate() {
			out[i] = self.keys.contains(key);
		}
		out
	}
	fn dir(&self) -> Vec3 {
		Vec3 {
			x: match self.pressed([KeyCode::KeyA, KeyCode::KeyD]) {
				[true, true] => 0.0,
				[false, false] => 0.0,
				[true, false] => -1.0,
				[false, true] => 1.0,
			},
			y: match self.pressed([KeyCode::KeyS, KeyCode::KeyW]) {
				[true, true] => 0.0,
				[false, false] => 0.0,
				[true, false] => -1.0,
				[false, true] => 1.0,
			},
			z: match self.pressed([KeyCode::ShiftLeft, KeyCode::Space]) {
				[true, true] => 0.0,
				[false, false] => 0.0,
				[true, false] => -1.0,
				[false, true] => 1.0,
			},
		}
	}
}

impl State {
	pub async fn new(window: Arc<Window>) -> State {
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions::default())
			.await
			.unwrap();
		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor::default())
			.await
			.unwrap();

		let size = window.inner_size();
		let surface = instance.create_surface(window.clone()).unwrap();
		let cap = surface.get_capabilities(&adapter);
		let surface_format = cap.formats[0];

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Raymarch Shader"),
			source: wgpu::ShaderSource::Wgsl(
				std::fs::read_to_string("src/shader.wgsl").unwrap().into(),
			),
		});

		let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Uniform Layout Group"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
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
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 3,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 4,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: false },
						view_dimension: wgpu::TextureViewDimension::D3,
						multisampled: false,
					},
					count: None,
				},
			],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Raymarch Pipeline Layout"),
			bind_group_layouts: &[&layout],
			push_constant_ranges: &[],
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Raymarch Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vs_main"),
				compilation_options: Default::default(),
				buffers: &[],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fs_main"),
				compilation_options: Default::default(),
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: Default::default(),
			multisample: Default::default(),
			depth_stencil: Default::default(),
			multiview: Default::default(),
			cache: Default::default(),
		});

		let screen_buffer = screen::create_buffer(&device);
		let camera_buffer = camera::create_buffer(&device);
		let time_buffer = time::create_buffer(&device);
		let sdf_sampler = sdf::create_sampler(&device);
		let sdf_texture = sdf::create_texture(&device);

		let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Uniform Group"),
			layout: &layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: screen_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: camera_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: time_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::Sampler(&sdf_sampler),
				},
				wgpu::BindGroupEntry {
					binding: 4,
					resource: wgpu::BindingResource::TextureView(&sdf::create_view(&sdf_texture)),
				},
			],
		});

		queue.write_texture(
			TexelCopyTextureInfo {
				texture: &sdf_texture,
				mip_level: 0,
				origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
				aspect: wgpu::TextureAspect::All,
			},
			bytemuck::cast_slice(&vec![0.0f32; 128 * 128 * 128]),
			TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(4 * 128),
				rows_per_image: Some(128),
			},
			Extent3d {
				width: 128,
				height: 128,
				depth_or_array_layers: 128,
			},
		);

		let camera = Camera::new();

		let state = State {
			window,
			device,
			queue,
			size,
			surface,
			surface_format,
			pipeline,
			uniform_group,
			screen_buffer,
			time_buffer,
			camera_buffer,
			start_time: std::time::Instant::now(),
			last_time: std::time::Instant::now(),
			input: Default::default(),
			camera,
			locked: false,
		};

		// Configure surface for the first time
		state.configure_surface();

		state
	}

	pub fn window(&self) -> &Window {
		&self.window
	}

	fn configure_surface(&self) {
		let surface_config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: self.surface_format,
			// Request compatibility with the sRGB-format texture view we‘re going to create later.
			view_formats: vec![self.surface_format.add_srgb_suffix()],
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			width: self.size.width,
			height: self.size.height,
			desired_maximum_frame_latency: 3,
			present_mode: wgpu::PresentMode::Fifo,
		};
		self.surface.configure(&self.device, &surface_config);
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.size = new_size;

		// reconfigure the surface
		self.configure_surface();
	}

	pub fn keyboard(&mut self, ev: RawKeyEvent) {
		match ev.physical_key {
			winit::keyboard::PhysicalKey::Code(key_code) => {
				match ev.state {
					ElementState::Pressed => self.input.keys.insert(key_code),
					ElementState::Released => self.input.keys.remove(&key_code),
				};
			}
			_ => {}
		}
	}

	pub fn mouse(&mut self, (x, y): (f64, f64)) {
		self.input.mouse_delta += Vec2 {
			x: x as f32,
			y: y as f32,
		};
	}

	pub fn lock(&mut self) {
		let _ = self.window.set_cursor_grab(CursorGrabMode::Locked);
		self.window.set_cursor_visible(false);
		self.locked = true;
	}

	pub fn unlock(&mut self) {
		let _ = self.window.set_cursor_grab(CursorGrabMode::None);
		self.window.set_cursor_visible(true);
		self.locked = false;
	}

	pub fn render(&mut self) {
		let surface_texture = self
			.surface
			.get_current_texture()
			.expect("failed to acquire next swapchain texture");

		let texture_view = surface_texture
			.texture
			.create_view(&wgpu::TextureViewDescriptor {
				format: Some(self.surface_format.add_srgb_suffix()),
				..Default::default()
			});

		let mut encoder = self.device.create_command_encoder(&Default::default());

		{
			let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &texture_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});

			self.queue.write_buffer(
				&self.screen_buffer,
				0,
				screen::ScreenUniform::new(self.window.inner_size()).bytes(),
			);

			self.queue.write_buffer(
				&self.time_buffer,
				0,
				time::TimeUniform::since(&self.start_time).bytes(),
			);

			self.queue
				.write_buffer(&self.camera_buffer, 0, self.camera.uniform().bytes());

			pass.set_bind_group(0, &self.uniform_group, &[]);
			pass.set_pipeline(&self.pipeline);
			pass.draw(0..6, 0..1);
		}

		// Submit the command in the queue to execute
		self.queue.submit([encoder.finish()]);
		self.window.pre_present_notify();
		surface_texture.present();

		let now_time = std::time::Instant::now();
		let time_delta = now_time - self.last_time;
		let mouse_delta = if self.locked {
			let cursor_pos = self.window.inner_size();
			let cursor_pos = PhysicalPosition::new(cursor_pos.width / 2, cursor_pos.height / 2);
			let _ = self.window.set_cursor_position(cursor_pos);
			self.input.mouse_delta
		} else {
			Vec2::ZERO
		};
		self.camera
			.update(self.input.dir(), mouse_delta, time_delta.as_secs_f32());
		self.input.mouse_delta = Vec2::ZERO;
		self.last_time = now_time;
	}
}
