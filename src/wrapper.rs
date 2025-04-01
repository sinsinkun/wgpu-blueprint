use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{ Device, Queue, TextureFormat, SurfaceConfiguration };
use winit::{
  application::ApplicationHandler,
  dpi::{PhysicalSize, PhysicalPosition},
  event::{Ime, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	keyboard::{PhysicalKey, KeyCode},
  platform::windows::IconExtWindows,
  window::{Icon, Window, WindowId}
};

use crate::utils::Vec2;

// --- --- --- --- --- --- --- --- --- //
// --- --- ---- APP SETUP ---- --- --- //
// --- --- --- --- --- --- --- --- --- //
#[allow(unused)]
#[derive(Debug)]
pub struct GpuAccess {
	pub window: Arc<Window>,
	pub device: Device,
	pub queue: Queue,
	pub screen_config: SurfaceConfiguration,
}

#[allow(unused)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MKBState { None, Pressed, Down, Released }

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MouseState {
  left: MKBState,
  right: MKBState,
  instp: Vec2,
  position: Vec2,
  pos_delta: Vec2,
	scroll: f32,
}
impl MouseState {
  fn new() -> Self {
    Self {
      left: MKBState::None,
      right: MKBState::None,
      instp: Vec2::new(400.0, 300.0),
      position: Vec2::new(400.0, 300.0),
      pos_delta: Vec2::new(0.0, 0.0),
			scroll: 0.0,
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
#[derive(Debug)]
pub struct SystemInfo<'a> {
	pub gpu: &'a mut GpuAccess,
  pub kb_inputs: &'a HashMap<KeyCode, MKBState>,
  pub m_inputs: &'a MouseState,
  pub frame_delta: &'a Duration,
  pub win_size: Vec2,
}
#[allow(dead_code)]
impl SystemInfo<'_> {
	fn time_delta(&self) -> f32 {
		self.frame_delta.as_secs_f32()
	}
	fn win_center(&self) -> Vec2 {
		let x = self.win_size.x / 2.0;
		let y = self.win_size.y / 2.0;
		Vec2::new(x, y)
	}
}

#[allow(unused)]
pub trait AppBase {
	/// create initial app state (without winit or wgpu assets)
	fn new() -> Self where Self: Sized;
	/// actions to take on initialization (after window creation + gpu is successful)
	fn init(&mut self, sys: SystemInfo);
	/// actions to take per frame
	fn update(&mut self, sys: SystemInfo);
  /// actions to take after exiting event loop
	fn cleanup(&mut self) {}
}
impl std::fmt::Debug for dyn AppBase {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "AppBase{{Unknown}}")
	}
}

// --- --- --- --- --- --- --- --- --- //
// --- --- WINIT + WGPU SETUP ---- --- //
// --- --- --- --- --- --- --- --- --- //

#[derive(Debug, Clone)]
pub struct WinitConfig {
	pub size: (u32, u32),
	pub min_size: (u32, u32),
	pub max_fps: Option<u32>,
	pub title: String,
	pub icon: Option<String>,
	pub debug: bool,
}
impl Default for WinitConfig {
	fn default() -> Self {
		Self {
			size: (800, 600),
			min_size: (400, 300),
			max_fps: None,
			title: "Blueprint".to_owned(),
			icon: None,
			debug: false,
		}
	}
}

#[derive(Debug)]
struct WinitApp<T> {
	setup: WinitConfig,
	wait_duration: Duration,
	gpu: Option<GpuAccess>,
	// input handling
	window_size: (u32, u32),
	input_cache: HashMap<KeyCode, MKBState>,
  mouse_cache: MouseState,
	frame_delta: Duration,
	last_frame: Instant,
	// custom app definition
	app: T,
}
impl<T: AppBase> WinitApp<T> {
  fn new(config: WinitConfig, app: T) -> Self {
		// convert fps to wait duration
		let mut mms = 1000;
		if let Some(n) = config.max_fps {
			mms = 1000000 / n;
		}
    Self {
			wait_duration: Duration::from_micros(mms.into()),
			gpu: None,
			app,
			input_cache: HashMap::new(),
			mouse_cache: MouseState::new(),
			frame_delta: Duration::from_micros(0),
			last_frame: Instant::now(),
			window_size: config.size,
			setup: config,
    }
  }
	async fn wgpu_init(&mut self, win: Window) {
		let window_handle = Arc::new(win);

		// The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
      backends: wgpu::Backends::PRIMARY,
      ..Default::default()
    });
    let surface = instance.create_surface(window_handle.clone()).unwrap();

    // handle for graphics card
    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
      },
    ).await.unwrap();

		// grab device & queue from adapter
    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        required_features: wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::POLYGON_MODE_POINT,
        required_limits: { wgpu::Limits::default() },
        label: None,
        memory_hints: wgpu::MemoryHints::Performance,
      },
      None, // Trace path
    ).await.unwrap();

		// define surface format for window
		let surface_caps = surface.get_capabilities(&adapter);
		let surface_format = if surface_caps.formats.contains(&TextureFormat::Rgba8UnormSrgb) {
			TextureFormat::Rgba8UnormSrgb
		} else if surface_caps.formats.contains(&TextureFormat::Rgba8Unorm) {
			TextureFormat::Rgba8Unorm
		} else {
			surface_caps.formats.iter()
				.copied()
				.filter(|f| f.is_srgb())
				.next()
				.unwrap_or(surface_caps.formats[0])
		};

		let config = SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: self.window_size.0,
      height: self.window_size.1,
      present_mode: wgpu::PresentMode::AutoNoVsync,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

		if self.setup.debug {
			println!("Sucessfully linked gpu: {:?}", adapter.get_info());
		}
		self.gpu = Some(GpuAccess {
			window: window_handle,
			device,
			queue,
			screen_config: config,
		});
	}
}
impl<T: AppBase> ApplicationHandler for WinitApp<T> {
  // initialization
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.gpu.is_some() {
			if self.setup.debug {
				println!("Resuming wrapper");
			}
			return;
		}
		let icon = match &self.setup.icon {
			Some(str) => {
				match Icon::from_path(str, None) {
					Ok(ico) => Some(ico),
					Err(e) => {
						println!("Failed to open icon: {:?}", e);
						None
					}
				}
			},
			None => None
		};
		let window_attributes = Window::default_attributes()
			.with_min_inner_size(PhysicalSize::new(self.setup.min_size.0, self.setup.min_size.1))
			.with_inner_size(PhysicalSize::new(self.setup.size.0, self.setup.size.1))
			.with_window_icon(icon)
			.with_title(self.setup.title.as_str());
		match event_loop.create_window(window_attributes) {
			Ok(win) => {
				win.set_ime_allowed(true);
				pollster::block_on(self.wgpu_init(win));
				self.app.init(SystemInfo { 
					gpu: self.gpu.as_mut().unwrap(),
					kb_inputs: &HashMap::new(),
					m_inputs: &MouseState::new(),
					frame_delta: &Duration::from_micros(0),
					win_size: Vec2::new(self.setup.size.0 as f32, self.setup.size.1 as f32),
				});
				if self.setup.debug {
					println!("Sucessfully launched wrapper");
				}
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
		self.frame_delta = now - self.last_frame;
		if self.frame_delta > self.wait_duration {
			self.last_frame = now;
			self.gpu.as_ref().unwrap().window.request_redraw();
		}
  }
  // handle events
	fn window_event(&mut self, event_loop: &ActiveEventLoop, _win_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				// close if window is closed externally
				event_loop.exit();
			}
			WindowEvent::Resized( phys_size, .. ) => {
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
			WindowEvent::MouseWheel { delta, .. } => {
				match delta {
					MouseScrollDelta::LineDelta(_x, y) => {
						self.mouse_cache.scroll += y;
					}
					MouseScrollDelta::PixelDelta(_ps) => ()
				}
			}
			WindowEvent::CursorMoved { position, .. } => {
        self.mouse_cache.instp.x = position.x as f32;
				self.mouse_cache.instp.y = position.y as f32;
      }
      // WindowEvent::CursorLeft { .. } => {}
			// WindowEvent::CursorEntered { .. } => {}
			WindowEvent::Ime(ime) => {
				match ime {
					Ime::Enabled => {
						println!("Enabled IME inputs");
						if let Some(gp) = &self.gpu {
							let pos: PhysicalPosition<f32> = self.mouse_cache.position.as_array().into();
							let size = PhysicalSize::new(100, 100);
							gp.window.set_ime_cursor_area(pos, size);
						}
					}
					Ime::Commit(chr) => {
						println!("Committing character {chr}");
					}
					_ => ()
				}
			}
			WindowEvent::RedrawRequested => {
				// app  update actions
				if let Some(gr) = &mut self.gpu {
					self.mouse_cache.frame_sync();
					self.app.update(SystemInfo {
						gpu: gr,
						kb_inputs: &self.input_cache,
						m_inputs: &self.mouse_cache,
						frame_delta: &self.frame_delta,
						win_size: Vec2::from_u32_tuple(self.window_size),
					});
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
				self.mouse_cache.scroll = 0.0;
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

				// wait until (doesn't work?)
				if self.setup.max_fps.is_some() {
					event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.wait_duration));
				}
			}
			_ => (),
		}
  }
	// note: not all devices support suspend events
	fn suspended(&mut self, _evt_loop: &ActiveEventLoop) {
		if self.setup.debug {
			println!("Suspending wrapper");
		}
	}
	// clean up (if necessary)
	fn exiting(&mut self, _evt_loop: &ActiveEventLoop) {
		self.app.cleanup();
		if self.setup.debug {
			println!("Exiting wrapper");
		}
	}
}

pub fn launch<T: AppBase>(config: WinitConfig, app: T) {
	let event_loop = EventLoop::new().unwrap();
	match config.max_fps {
		Some(_) => event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now())),
		None => event_loop.set_control_flow(ControlFlow::Poll)
	};
  let mut winit_app = WinitApp::new(config, app);
  match event_loop.run_app(&mut winit_app) {
		Ok(_) => (),
		Err(e) => println!("Winit closed unexpectedly - {}", e.to_string()),
	};
}