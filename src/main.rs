use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
  window: Option<Window>,
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_min_inner_size(PhysicalSize::new(400.0, 300.0))
			.with_inner_size(PhysicalSize::new(1024.0, 720.0))
			.with_title("Wgpu App");
		match event_loop.create_window(window_attributes) {
			Ok(win) => {
				self.window = Some(win);
			}
			Err(e) => {
				println!("Failed to create window: {}", e);
			}
		};
		// initialize other things
	}
	fn window_event(&mut self, event_loop: &ActiveEventLoop, _win_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				// close if window is closed externally
				event_loop.exit();
			}
			WindowEvent::KeyboardInput { event: KeyEvent { physical_key: key, state, repeat, .. }, .. } => {
				match key {
					PhysicalKey::Code(KeyCode::Escape) => {
						if state.is_pressed() && !repeat {
							event_loop.exit();
						}
					}
					PhysicalKey::Code(KeyCode::F1) => {
						if state.is_pressed() && !repeat {
							println!("Hello world");
						}
					}
					_ => ()
				}
			}
			WindowEvent::RedrawRequested => {
				// update
				// render
			}
			_ => (),
		}
	}
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);
	let mut app = App::default();
	let _ = event_loop.run_app(&mut app);
}
