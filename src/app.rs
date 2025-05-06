use std::sync::Arc;

use crate::state::State;
use winit::{
	application::ApplicationHandler,
	event::{MouseButton, WindowEvent},
	event_loop::ActiveEventLoop,
	window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
	state: Option<State>,
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window = Arc::new(
			event_loop
				.create_window(Window::default_attributes())
				.unwrap(),
		);

		let state = pollster::block_on(State::new(window.clone()));
		self.state = Some(state);

		window.request_redraw();
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: winit::event::DeviceId,
		event: winit::event::DeviceEvent,
	) {
		let state = self.state.as_mut().unwrap();
		match event {
			winit::event::DeviceEvent::MouseMotion { delta } => state.mouse(delta.into()),
			winit::event::DeviceEvent::Key(raw_key_event) => state.keyboard(raw_key_event),
			_ => {}
		}
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
		let appstate = self.state.as_mut().unwrap();
		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::RedrawRequested => {
				appstate.render();
				appstate.window().request_redraw();
			}
			WindowEvent::Resized(size) => {
				appstate.resize(size);
			}
			WindowEvent::MouseInput { state, button, .. } => {
				if state.is_pressed() && button == MouseButton::Left {
					appstate.lock();
				}
			}
			WindowEvent::KeyboardInput { event, .. } => {
				match (event.physical_key, event.state.is_pressed()) {
					(
						winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
						true,
					) => {
						appstate.unlock();
					}
					_ => {}
				}
			}
			_ => (),
		}
	}
}
