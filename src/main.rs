use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{Ime, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::{Window, WindowId};

use wgpu::SurfaceError;

// custom components
mod renderer;
use renderer::{RPipelineId, Renderer, Vec2};

mod app;
use app::App;

const RENDER_FPS_LOCK: Duration = Duration::from_millis(10);
const DEFAULT_SIZE: (u32, u32) = (800, 600);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MKBState { None, Pressed, Down, Released }

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MouseState {
  left: MKBState,
  right: MKBState,
  instp: Vec2,
  position: Vec2,
  pos_delta: Vec2,
}
impl MouseState {
  fn new() -> Self {
    Self {
      left: MKBState::None,
      right: MKBState::None,
      instp: Vec2::new(0.0, 0.0),
      position: Vec2::new(0.0, 0.0),
      pos_delta: Vec2::new(0.0, 0.0),
    }
  }
  fn frame_sync(&mut self) {
    let dx = self.instp.x - self.position.x;
    let dy = self.instp.y - self.position.y;
    self.pos_delta = Vec2::new(dx, dy);
    self.position = self.instp;
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SystemInfo<'a> {
  kb_inputs: &'a HashMap<KeyCode, MKBState>,
  m_inputs: &'a MouseState,
  frame_delta: &'a Duration,
  win_size: Vec2,
}

#[allow(unused_variables)]
pub trait AppBase {
	/// actions to take on initialization
	/// - prepare render pipelines
	/// - instantialize data objects
	fn init(&mut self, renderer: &mut Renderer);
	/// actions to take on window resize
	/// - called before updates
	fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {}
	/// actions to take per frame
	/// - respond to inputs
	/// - state changes
	/// - update render object variables
	/// - render to textures
	/// output pipeline ids to render to screen
	fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId>;
  /// actions to take after exiting event loop
	/// - destroy dangling resources
	fn cleanup(&mut self) {}
}
impl std::fmt::Debug for dyn AppBase {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "AppBase{{Unknown}}")
	}
}

#[derive(Debug)]
struct WinitApp<'a, T> {
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
	resize_state: u8,
	// input handling
	input_cache: HashMap<KeyCode, MKBState>,
  mouse_cache: MouseState,
	// app state separation
	app: T,
}
impl<'a, T: AppBase> ApplicationHandler for WinitApp<'a, T> {
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
				self.resize_state = 1;
				self.window_size = phys_size.into();
			}
			WindowEvent::KeyboardInput { event: KeyEvent { physical_key: key, state, repeat, .. }, .. } => {
				// add key to input cache
				if let PhysicalKey::Code(x) = key {
					if state.is_pressed() && !repeat {
						self.input_cache.insert(x, MKBState::Pressed);
					}
					else if !state.is_pressed() {
						self.input_cache.insert(x, MKBState::Released);
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
			WindowEvent::MouseInput { state, button, .. } => {
        if button == MouseButton::Left {
          if state.is_pressed() {
            self.mouse_cache.left = MKBState::Pressed;
          }
          else if !state.is_pressed() {
            self.mouse_cache.left = MKBState::Released;
          }
        }
        if button == MouseButton::Right {
          if state.is_pressed() {
            self.mouse_cache.right = MKBState::Pressed;
          }
          else if !state.is_pressed() {
            self.mouse_cache.right = MKBState::Released;
          }
        }
      }
      WindowEvent::CursorMoved { position, .. } => {
        self.mouse_cache.instp.x = position.x as f32;
				self.mouse_cache.instp.y = position.y as f32;
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
				// update system
				if self.resize_state == 1 {
					// skip frame if window is being resized
					self.resize_state = 2;
					return;
				} else if self.resize_state == 2 {
					// call resize updates
					if let Some(r) = &mut self.renderer {
						r.resize(self.window_size.0, self.window_size.1);
						self.app.resize(r, self.window_size.0, self.window_size.1);
					}
					self.resize_state = 0;
				}
        self.mouse_cache.frame_sync();
				let sys = SystemInfo {
					kb_inputs: &self.input_cache,
          m_inputs: &self.mouse_cache,
          frame_delta: &self.frame_delta,
          win_size: Vec2::from_u32_tuple(self.window_size),
				};
				if let Some(r) = &mut self.renderer {
					// run internal app updates
					let pipes = self.app.update(sys, r);
					// run render engine actions
					match r.render_to_screen(&pipes) {
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
					if *k.1 == MKBState::Pressed { *k.1 = MKBState::Down; }
					else if *k.1 == MKBState::Released { rm_k.push(*k.0); }
				}
				for k in rm_k {
					self.input_cache.remove(&k);
				}
        // clean up mouse cache
        if self.mouse_cache.left == MKBState::Pressed {
          self.mouse_cache.left = MKBState::Down;
        } else if self.mouse_cache.left == MKBState::Released {
          self.mouse_cache.left = MKBState::None;
        }
        if self.mouse_cache.right == MKBState::Pressed {
          self.mouse_cache.right = MKBState::Down;
        } else if self.mouse_cache.right == MKBState::Released {
          self.mouse_cache.right = MKBState::None;
        }
				// wait until
				let wait_until = Instant::now() + (RENDER_FPS_LOCK.mul_f32(0.5));
				event_loop.set_control_flow(ControlFlow::WaitUntil(wait_until));
			}
			_ => (),
		}
	}
}
impl<T: AppBase> WinitApp<'_, T> {
  fn new(ext_app: T) -> Self {
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
			resize_state: 0,
			input_cache: HashMap::new(),
      mouse_cache: MouseState::new(),
			app: ext_app,
		}
  }
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
	let mut winit_app = WinitApp::new(App::default());
	let _ = event_loop.run_app(&mut winit_app);
	winit_app.cleanup();
}
