use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{ Device, Queue, Surface, SurfaceConfiguration, TextureFormat };
use winit::{
  application::ApplicationHandler,
  dpi::{PhysicalSize, PhysicalPosition},
  event::{Ime, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	keyboard::{PhysicalKey, KeyCode},
  platform::windows::IconExtWindows,
  window::{Icon, Window, WindowAttributes, WindowId}
};

use crate::utils::Vec2;

// --- --- --- --- --- --- --- --- --- //
// --- --- ---- APP SETUP ---- --- --- //
// --- --- --- --- --- --- --- --- --- //

#[derive(Debug)]
pub struct GpuAccess<'a> {
	pub device: Device,
	pub queue: Queue,
	pub screen_surface: Surface<'a>,
	pub screen_config: SurfaceConfiguration,
	pub screen_format: TextureFormat,
}
#[allow(unused)]
impl GpuAccess<'_> {
	pub fn begin_render(&mut self) -> Result<(wgpu::CommandEncoder, wgpu::SurfaceTexture), wgpu::SurfaceError> {
		let output = self.screen_surface.get_current_texture()?;
		let encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-encoder") }
    );
		Ok((encoder, output))
	}
	pub fn clear(&self, encoder: &mut wgpu::CommandEncoder, surface: &wgpu::SurfaceTexture, color: Option<wgpu::Color>) {
		let clear_color = color.unwrap_or(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0});
    let target = surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("clear-render"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &target,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(clear_color),
					store: wgpu::StoreOp::Store
				}
			})],
			..Default::default()
		});
	}
	pub fn end_render(&self, encoder: wgpu::CommandEncoder, surface: wgpu::SurfaceTexture) {
		self.queue.submit(std::iter::once(encoder.finish()));
		surface.present();
	}
	pub fn resize_screen(&mut self, width: u32, height: u32) {
		self.screen_config.width = width;
		self.screen_config.height = height;
		self.screen_surface.configure(&self.device, &self.screen_config);
	}
}

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
	cursor_in: bool,
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
			cursor_in: true,
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
pub struct SystemAccess {
	input_cache: HashMap<KeyCode, MKBState>,
	mouse_cache: MouseState,
  frame_delta: Duration,
	last_frame: Instant,
  window_size: (u32, u32),
	pub debug: bool,
	exit: bool,
}
#[allow(dead_code)]
impl SystemAccess {
	pub fn kb_inputs(&self) -> &HashMap<KeyCode, MKBState> {
		&self.input_cache
	}
	pub fn m_inputs(&self) -> &MouseState {
		&self.mouse_cache
	}
	pub fn time_delta(&self) -> f32 {
		self.frame_delta.as_secs_f32()
	}
	pub fn fps(&self) -> f32 {
		1.0 / self.frame_delta.as_secs_f32()
	}
	pub fn win_size(&self) -> Vec2 {
		Vec2::from_u32_tuple(self.window_size)
	}
	pub fn win_center(&self) -> Vec2 {
		let x = self.window_size.0 as f32 / 2.0;
		let y = self.window_size.1 as f32 / 2.0;
		Vec2::new(x, y)
	}
	pub fn request_exit(&mut self) {
		self.exit = true;
	}
}

#[allow(unused)]
pub trait AppBase {
	/// create initial app state (without winit or wgpu assets)
	fn new() -> Self where Self: Sized;
	/// actions to take on initialization (after window creation + gpu is successful)
	fn init(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess);
	/// actions to take when screen resizes (asynchronous with update call)
	fn resize(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess, width: u32, height: u32) {}
	/// actions to take per frame
	fn update(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess);
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
	pub resizable: bool,
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
			resizable: true,
		}
	}
}

#[derive(Debug)]
struct WinitApp<'a, T> {
	wait_duration: Duration,
	window_attributes: WindowAttributes,
	gpu: Option<GpuAccess<'a>>,
	windows: HashMap<WindowId, Arc<Window>>,
	// custom app definition
	sys: SystemAccess,
	app: T,
}
impl<'a, T: AppBase> WinitApp<'a, T> {
  fn new(config: WinitConfig, app: T) -> Self {
		// convert fps to wait duration
		let mms = if let Some(n) = config.max_fps { 1000000 / n } else { 0 };
		// create window attributes
		let icon = match &config.icon {
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
			.with_min_inner_size(PhysicalSize::new(config.min_size.0, config.min_size.1))
			.with_inner_size(PhysicalSize::new(config.size.0, config.size.1))
			.with_resizable(config.resizable)
			.with_window_icon(icon)
			.with_title(config.title.as_str());
		// create shared data between winit and user app
		let sys = SystemAccess {
			input_cache: HashMap::new(),
			mouse_cache: MouseState::new(),
			frame_delta: Duration::from_micros(0),
			last_frame: Instant::now(),
			window_size: config.size,
			debug: config.debug,
			exit: false,
		};
    Self {
			window_attributes,
			wait_duration: Duration::from_micros(mms.into()),
			gpu: None,
			windows: HashMap::new(),
			sys,
			app,
    }
  }
	fn cur_window(&self, id: &WindowId) -> Option<&Arc<Window>> {
		self.windows.get(id)
	}
	async fn wgpu_init(&mut self, win: Arc<Window>) {
		let size = win.inner_size();

		// The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			..Default::default()
		});
    let surface = instance.create_surface(win).unwrap();

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
        required_limits: wgpu::Limits::default(),
        label: None,
        memory_hints: Default::default(),
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
				.find(|f| f.is_srgb())
				.copied()
				.unwrap_or(surface_caps.formats[0])
		};

		if self.sys.debug {
			println!("Surface format: {:?}", surface_format);
		}

		let config = SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::AutoNoVsync,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };
		// invoked via resize call
		// surface.configure(&device, &config);

		if self.sys.debug {
			println!("Sucessfully linked gpu: {:?}", adapter.get_info());
		}
		self.gpu = Some(GpuAccess {
			device,
			queue,
			screen_surface: surface,
			screen_config: config,
			screen_format: surface_format,
		});
	}
}
impl<'a, T: AppBase> ApplicationHandler for WinitApp<'a, T> {
  // initialization
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.gpu.is_some() {
			if self.sys.debug {
				println!("Resuming event loop");
			}
			return;
		}
		println!("Starting event loop");
		match event_loop.create_window(self.window_attributes.clone()) {
			Ok(win) => {
				win.set_ime_allowed(true);
				let window_handle = Arc::new(win);
				pollster::block_on(self.wgpu_init(window_handle.clone()));
				if self.sys.debug {
					println!("Successfully launched window {:?}", window_handle.id());
				}
				self.windows.insert(window_handle.id(), window_handle);
				self.app.init(&mut self.sys, self.gpu.as_mut().unwrap());
			}
			Err(e) => {
				println!("Failed to create window: {}", e);
				event_loop.exit();
			}
		};
	}
  // system updates
  fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
    // calculate time data
		let now = Instant::now();
		self.sys.frame_delta = now - self.sys.last_frame;
		if self.sys.frame_delta > self.wait_duration {
			self.sys.last_frame = now;
			for win in &self.windows {
				win.1.request_redraw();
			}
		}
  }
  // handle events
	fn window_event(&mut self, event_loop: &ActiveEventLoop, win_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				// close if window is closed externally
				event_loop.exit();
			}
			WindowEvent::Resized( phys_size, .. ) => {
				self.sys.window_size = phys_size.into();
				if self.sys.debug {
					println!("Resized window {:?} - ({}, {})", win_id, phys_size.width, phys_size.height);
				}
				if let Some(r) = &mut self.gpu {
					self.app.resize(&mut self.sys, r, phys_size.width, phys_size.height);
				}
			}
			WindowEvent::KeyboardInput { event: KeyEvent { physical_key: key, state, repeat, .. }, .. } => {
				// add key to input cache
				if let PhysicalKey::Code(x) = key {
					if state.is_pressed() && !repeat {
						self.sys.input_cache.insert(x, MKBState::Pressed);
					}
					else if !state.is_pressed() {
						self.sys.input_cache.insert(x, MKBState::Released);
					}
				}
			}
			WindowEvent::MouseInput { state, button, .. } => {
        if button == MouseButton::Left {
          if state.is_pressed() {
            self.sys.mouse_cache.left = MKBState::Pressed;
          }
          else if !state.is_pressed() {
            self.sys.mouse_cache.left = MKBState::Released;
          }
        }
        if button == MouseButton::Right {
          if state.is_pressed() {
            self.sys.mouse_cache.right = MKBState::Pressed;
          }
          else if !state.is_pressed() {
            self.sys.mouse_cache.right = MKBState::Released;
          }
        }
      }
			WindowEvent::MouseWheel { delta, .. } => {
				match delta {
					MouseScrollDelta::LineDelta(_x, y) => {
						self.sys.mouse_cache.scroll += y;
					}
					MouseScrollDelta::PixelDelta(_ps) => ()
				}
			}
			WindowEvent::CursorMoved { position, .. } => {
        self.sys.mouse_cache.instp.x = position.x as f32;
				self.sys.mouse_cache.instp.y = position.y as f32;
      }
      WindowEvent::CursorLeft { .. } => {
				self.sys.mouse_cache.cursor_in = false;
			}
			WindowEvent::CursorEntered { .. } => {
				self.sys.mouse_cache.cursor_in = true;
			}
			WindowEvent::Ime(ime) => {
				match ime {
					Ime::Enabled => {
						println!("Enabled IME inputs");
						let pos: PhysicalPosition<f32> = self.sys.mouse_cache.position.as_array().into();
						let size = PhysicalSize::new(100, 100);
						match self.cur_window(&win_id) {
							Some(w) => w.set_ime_cursor_area(pos, size),
							None => println!("ERR: Could not find window for IME")
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
				if let Some(r) = &mut self.gpu {
					self.sys.mouse_cache.frame_sync();
					self.app.update(&mut self.sys, r);
				}
				// respond to app requests
				if self.sys.exit { event_loop.exit(); }

				// clean up input cache
				let mut rm_k: Vec<KeyCode> = Vec::new();
				for k in &mut self.sys.input_cache.iter_mut() {
					if *k.1 == MKBState::Pressed { *k.1 = MKBState::Down; }
					else if *k.1 == MKBState::Released { rm_k.push(*k.0); }
				}
				for k in rm_k {
					self.sys.input_cache.remove(&k);
				}

				// clean up mouse cache
				self.sys.mouse_cache.scroll = 0.0;
				if self.sys.mouse_cache.left == MKBState::Pressed {
					self.sys.mouse_cache.left = MKBState::Down;
				} else if self.sys.mouse_cache.left == MKBState::Released {
					self.sys.mouse_cache.left = MKBState::None;
				}
				if self.sys.mouse_cache.right == MKBState::Pressed {
					self.sys.mouse_cache.right = MKBState::Down;
				} else if self.sys.mouse_cache.right == MKBState::Released {
					self.sys.mouse_cache.right = MKBState::None;
				}

				// wait until
				if self.wait_duration > Duration::from_micros(0) {
					event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.wait_duration));
				}
			}
			_ => (),
		}
  }
	// note: not all devices support suspend events
	fn suspended(&mut self, _evt_loop: &ActiveEventLoop) {
		if self.sys.debug {
			println!("Suspending event loop");
		}
	}
	// clean up (if necessary)
	fn exiting(&mut self, _evt_loop: &ActiveEventLoop) {
		self.app.cleanup();
		if let Some(r) = &self.gpu {
			r.device.destroy();
		}
		if self.sys.debug {
			println!("Exiting event loop");
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