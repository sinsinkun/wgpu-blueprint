use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent, StartCause, Ime};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::{Window, WindowId};

const RENDER_FPS_LOCK: Duration = Duration::from_millis(8);

#[derive(Debug)]
struct App {
	// system diagnostics
  window: Option<Window>,
	lifetime: Duration,
	last_frame: Instant,
	frame_delta: Duration,
	last_render_frame: Instant,
	render_frame_delta: Duration,
	is_render_frame: bool,
}
impl Default for App {
	fn default() -> Self {
		Self {
			window: None,
			lifetime: Duration::from_millis(0),
			last_frame: Instant::now(),
			frame_delta: Duration::from_millis(0),
			last_render_frame: Instant::now(),
			render_frame_delta: Duration::from_millis(0),
			is_render_frame: true,
		}
	}
}
impl ApplicationHandler for App {
	// initialization
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_min_inner_size(PhysicalSize::new(400.0, 300.0))
			.with_inner_size(PhysicalSize::new(800.0, 560.0))
			.with_title("CXGui");
		match event_loop.create_window(window_attributes) {
			Ok(win) => {
				self.window = Some(win);
			}
			Err(e) => {
				println!("Failed to create window: {}", e);
				event_loop.exit();
			}
		};
		// initialize other things
	}
	// system updates
	fn new_events(&mut self, _evt_loop: &ActiveEventLoop, _cause: StartCause) {
		// calculate time data
		let now = Instant::now();
		self.frame_delta = now - self.last_frame;
		self.last_frame = now;
		self.lifetime += self.frame_delta;
		// restrict rendering pace
		self.render_frame_delta = now - self.last_render_frame;
		if self.render_frame_delta > RENDER_FPS_LOCK {
			self.is_render_frame = true;
			self.last_render_frame = now;
			self.window.as_ref().unwrap().request_redraw();
		} else {
			self.is_render_frame = false;
		}
	}
	// handle events
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
			WindowEvent::Ime(ime) => {
				match ime {
					Ime::Enabled => {
						println!("Enabled IME inputs");
					}
					Ime::Commit(chr) => {
						println!("Committing character {chr}");
					}
					_ => ()
				}
			}
			WindowEvent::RedrawRequested => {
				// update
				let fps_1 = 1.0 / self.frame_delta.as_secs_f32();
				let fps_2 = 1.0 / self.render_frame_delta.as_secs_f32();
				println!("FPS - Updates: {fps_1}, Renders: {fps_2}");
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
