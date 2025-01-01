use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent, StartCause, Ime};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::{Window, WindowId};

use wgpu::SurfaceError;

// custom components
mod renderer;
use renderer::Renderer;

mod app;
use app::App;

const RENDER_FPS_LOCK: Duration = Duration::from_millis(100);
const DEFAULT_SIZE: (u32, u32) = (800, 600);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KBState { Pressed, Down, Released }

pub trait AppBase {
	fn init(&mut self, renderer: &mut Renderer);
	fn update(&mut self, inputs: &HashMap<KeyCode, KBState>, frame_delta: &Duration);
	fn render(&mut self, renderer: &mut Renderer);
	fn cleanup(&mut self);
}
impl std::fmt::Debug for dyn AppBase {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "AppBase{{Unknown}}")
	}
}

#[derive(Debug)]
struct WinitApp<'a> {
	// system diagnostics
  window: Option<Arc<Window>>,
	window_size: (u32, u32),
	lifetime: Duration,
	last_event_frame: Instant,
	event_frame_delta: Duration,
	// render handling
	last_frame: Instant,
	frame_delta: Duration,
	is_render_frame: bool,
	renderer: Option<Renderer<'a>>,
	// input handling
	input_cache: HashMap<KeyCode, KBState>,
	// app state separation
	app: Box<dyn AppBase>,
}
impl Default for WinitApp<'_> {
	fn default() -> Self {
		Self {
			window: None,
			window_size: DEFAULT_SIZE,
			lifetime: Duration::from_millis(0),
			last_event_frame: Instant::now(),
			event_frame_delta: Duration::from_millis(0),
			last_frame: Instant::now(),
			frame_delta: Duration::from_millis(0),
			is_render_frame: true,
			renderer: None,
			input_cache: HashMap::new(),
			app: Box::new(App::default()),
		}
	}
}
impl<'a> ApplicationHandler for WinitApp<'a> {
	// initialization
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_min_inner_size(PhysicalSize::new(400.0, 300.0))
			.with_inner_size(PhysicalSize::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1))
			.with_title("CXGui");
		match event_loop.create_window(window_attributes) {
			Ok(win) => {
				let window_handle = Arc::new(win);
				self.window = Some(window_handle.clone());
				env_logger::init();
				let mut wgpu = pollster::block_on(Renderer::new(window_handle.clone()));
				self.app.init(&mut wgpu);
				self.renderer = Some(wgpu);
			}
			Err(e) => {
				println!("Failed to create window: {}", e);
				event_loop.exit();
			}
		};
	}
	// system updates
	fn new_events(&mut self, _evt_loop: &ActiveEventLoop, _cause: StartCause) {
		// calculate time data
		let now = Instant::now();
		self.event_frame_delta = now - self.last_event_frame;
		self.last_event_frame = now;
		self.lifetime += self.event_frame_delta;
		// restrict rendering pace
		self.frame_delta = now - self.last_frame;
		if self.frame_delta > RENDER_FPS_LOCK {
			self.is_render_frame = true;
			self.last_frame = now;
			self.window.as_ref().unwrap().request_redraw();
			// fps debug
			// let fps_1 = 1.0 / self.event_frame_delta.as_secs_f32();
			// let fps_2 = 1.0 / self.frame_delta.as_secs_f32();
			// println!("FPS - Updates: {fps_1}, Renders: {fps_2}");
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
			WindowEvent::Resized(phys_size) => {
				if let Some(r) = &mut self.renderer {
					r.resize(phys_size.width, phys_size.height);
				}
				self.window_size = phys_size.into();
			}
			WindowEvent::KeyboardInput { event: KeyEvent { physical_key: key, state, repeat, .. }, .. } => {
				// add key to input cache
				if let PhysicalKey::Code(x) = key {
					if state.is_pressed() && !repeat {
						self.input_cache.insert(x, KBState::Pressed);
					}
					else if !state.is_pressed() {
						self.input_cache.insert(x, KBState::Released);
					}
				}
				match key {
					PhysicalKey::Code(KeyCode::Escape) => {
						if state.is_pressed() && !repeat {
							event_loop.exit();
						}
					}
					PhysicalKey::Code(KeyCode::F1) => {
						if state.is_pressed() && !repeat {
							if let Some(r) = &mut self.renderer {
								r.clear_color = wgpu::Color::BLUE;
							}
						}
					}
					PhysicalKey::Code(KeyCode::F2) => {
						if state.is_pressed() && !repeat {
							if let Some(r) = &mut self.renderer {
								r.clear_color = wgpu::Color::GREEN;
							}
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
				// run internal app updates
				self.app.update(&self.input_cache, &self.frame_delta);
				if let Some(r) = &mut self.renderer {
					// run internal render updates
					self.app.render(r);
					// run render engine actions
					match r.render() {
						Ok(_) => (),
						Err(SurfaceError::Lost | SurfaceError::Outdated) => {
							println!("Err: surface was lost or outdated. Attempting to re-connect");
							r.resize(self.window_size.0, self.window_size.1);
						}
						Err(SurfaceError::OutOfMemory) => {
							println!("Err: Out of memory. Exiting");
							event_loop.exit();
						}
						Err(SurfaceError::Timeout) => {
							println!("Err: render frame timed out");
						}
					};
				}
				// clean up input cache
				let mut rm_k: Vec<KeyCode> = Vec::new();
				for k in &mut self.input_cache.iter_mut() {
					if *k.1 == KBState::Pressed { *k.1 = KBState::Down; }
					else if *k.1 == KBState::Released { rm_k.push(*k.0); }
				}
				for k in rm_k {
					self.input_cache.remove(&k);
				}
				// wait until
				let wait_until = Instant::now() + RENDER_FPS_LOCK;
				event_loop.set_control_flow(ControlFlow::WaitUntil(wait_until));
			}
			_ => (),
		}
	}
}
impl WinitApp<'_> {
	fn cleanup(&mut self) {
		self.app.cleanup();
		if let Some(r) = &mut self.renderer {
			r.destroy(true);
		}
	}
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now()));
	let mut app = WinitApp::default();
	let _ = event_loop.run_app(&mut app);
	app.cleanup();
}
