use crate::{
	camera::{self, Camera},
	particle, screen, sdf, time,
};
use glam::{Vec2, Vec3};
use std::{collections::HashSet, num::NonZero, sync::Arc, time::Instant};
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
	compute_clear_pipeline: wgpu::ComputePipeline,
	compute_calc_pipeline: wgpu::ComputePipeline,
	render_pipeline: wgpu::RenderPipeline,
	compute_write_tmp_group: wgpu::BindGroup,
	compute_write_main_group: wgpu::BindGroup,
	render_group: wgpu::BindGroup,
	particles_buffer: wgpu::Buffer,
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
			.request_device(&wgpu::DeviceDescriptor {
				required_features: wgpu::Features::TIMESTAMP_QUERY,
				..Default::default()
			})
			.await
			.unwrap();

		let size = window.inner_size();
		let surface = instance.create_surface(window.clone()).unwrap();
		let cap = surface.get_capabilities(&adapter);
		let surface_format = cap.formats[0];

		let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Raymarch Compute Shader"),
			source: wgpu::ShaderSource::Wgsl(
				std::fs::read_to_string("src/compute.wgsl").unwrap().into(),
			),
		});

		let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Raymarch Render Shader"),
			source: wgpu::ShaderSource::Wgsl(
				std::fs::read_to_string("src/shader.wgsl").unwrap().into(),
			),
		});

		let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Compute Layout Group"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: true,
						min_binding_size: None,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::StorageTexture {
						access: wgpu::StorageTextureAccess::WriteOnly,
						format: wgpu::TextureFormat::Rgba16Float,
						view_dimension: wgpu::TextureViewDimension::D3,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: false },
						view_dimension: wgpu::TextureViewDimension::D3,
						multisampled: false,
					},
					count: None,
				},
			],
		});

		let render_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Render Layout Group"),
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
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D3,
						multisampled: false,
					},
					count: None,
				},
			],
		});

		let compute_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Raymarch Compute Pipeline Layout"),
				bind_group_layouts: &[&compute_layout],
				push_constant_ranges: &[],
			});

		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Raymarch Render Pipeline Layout"),
				bind_group_layouts: &[&render_layout],
				push_constant_ranges: &[],
			});

		let compute_clear_pipeline =
			device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("Compute Pipeline (Clear)"),
				layout: Some(&compute_pipeline_layout),
				module: &compute_shader,
				entry_point: Some("cs_clear"),
				compilation_options: Default::default(),
				cache: Default::default(),
			});

		let compute_calc_pipeline =
			device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("Compute Pipeline (Calc SDF)"),
				layout: Some(&compute_pipeline_layout),
				module: &compute_shader,
				entry_point: Some("cs_sdf"),
				compilation_options: Default::default(),
				cache: Default::default(),
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &render_shader,
				entry_point: Some("vs_main"),
				compilation_options: Default::default(),
				buffers: &[],
			},
			fragment: Some(wgpu::FragmentState {
				module: &render_shader,
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

		let tex_width = 64;
		let tex_height = 64;
		let tex_depth = 64;

		let screen_buffer = screen::create_buffer(&device);
		let camera_buffer = camera::create_buffer(&device);
		let time_buffer = time::create_buffer(&device);
		let particles_buffer = particle::create_buffer(&device);

		let sdf_tmp_texture = sdf::create_texture(&device, tex_width, tex_height, tex_depth);
		let sdf_tmp_view = sdf::create_view(&sdf_tmp_texture);
		let sdf_texture = sdf::create_texture(&device, tex_width, tex_height, tex_depth);
		let sdf_view = sdf::create_view(&sdf_texture);
		let sdf_sampler = sdf::create_sampler(&device);

		let compute_write_tmp_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Compute Group (Write to Temp)"),
			layout: &compute_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &particles_buffer,
						offset: 0,
						size: Some(NonZero::new(256).unwrap()),
					}),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(&sdf_tmp_view),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(&sdf_view),
				},
			],
		});

		let compute_write_main_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Compute Group (Write to Main)"),
			layout: &compute_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &particles_buffer,
						offset: 0,
						size: Some(NonZero::new(256).unwrap()),
					}),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(&sdf_view),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(&sdf_tmp_view),
				},
			],
		});

		let render_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Render Group"),
			layout: &render_layout,
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
					resource: wgpu::BindingResource::TextureView(&sdf_view),
				},
			],
		});

		let camera = Camera::new();

		let state = State {
			window,
			device,
			queue,
			size,
			surface,
			surface_format,
			compute_clear_pipeline,
			compute_calc_pipeline,
			render_pipeline,
			compute_write_tmp_group,
			compute_write_main_group,
			render_group,
			particles_buffer,
			screen_buffer,
			time_buffer,
			camera_buffer,
			start_time: Instant::now(),
			last_time: Instant::now(),
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
			present_mode: wgpu::PresentMode::Immediate,
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

	fn update(&mut self) {
		let now_time = Instant::now();
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

	pub fn render(&mut self) {
		self.update();

		let frame_start = Instant::now();

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

		let u_screen = screen::ScreenUniform::new(self.window.inner_size());
		let u_time = time::TimeUniform::since(&self.start_time);
		let u_camera = self.camera.uniform();

		let u_screen = u_screen.bytes();
		let u_time = u_time.bytes();
		let u_camera = u_camera.bytes();

		self.queue.write_buffer(&self.screen_buffer, 0, u_screen);
		self.queue.write_buffer(&self.time_buffer, 0, u_time);
		self.queue.write_buffer(&self.camera_buffer, 0, u_camera);

		let mut encoder = self.device.create_command_encoder(&Default::default());

		// Compute Pass
		{
			let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
				label: Some("Compute Pass"),
				timestamp_writes: None,
			});

			let (wg_x, wg_y, wg_z) = (8, 4, 2);
			let dispatch_x = (64 + wg_x - 1) / wg_x;
			let dispatch_y = (64 + wg_y - 1) / wg_y;
			let dispatch_z = (64 + wg_z - 1) / wg_z;

			pass.set_pipeline(&self.compute_clear_pipeline);
			pass.set_bind_group(0, &self.compute_write_main_group, &[0]);
			pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);

			pass.set_pipeline(&self.compute_calc_pipeline);

			let mut offset = 0;
			let mut mode = true;
			while offset < self.particles_buffer.size() as u32 {
				if mode {
					pass.set_bind_group(0, &self.compute_write_tmp_group, &[offset]);
					pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);
				} else {
					pass.set_bind_group(0, &self.compute_write_main_group, &[offset]);
					pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);
				}
				mode = !mode;
				offset += 256;
			}
		}

		// Render Pass
		{
			let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &texture_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				timestamp_writes: None,
				depth_stencil_attachment: None,
				occlusion_query_set: None,
			});
			pass.set_bind_group(0, &self.render_group, &[]);
			pass.set_pipeline(&self.render_pipeline);
			pass.draw(0..6, 0..1);
		}

		self.queue.submit([encoder.finish()]);
		self.window.pre_present_notify();
		surface_texture.present();

		println!("Frame time: {}ms", frame_start.elapsed().as_millis())
	}
}
