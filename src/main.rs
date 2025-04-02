use wgpu::SurfaceError;
use winit::keyboard::KeyCode;

mod utils;
mod wrapper;
use wrapper::{launch, AppBase, SystemInfo, WinitConfig};

#[derive(Debug)]
pub struct App {
  fps: f32,
  exiting: bool,
}
impl AppBase for App {
  fn new() -> Self {
    Self {
      fps: 0.0,
      exiting: false,
    }
  }
  fn init(&mut self, _sys: SystemInfo) {
    println!("Hello world");
  }
  fn update(&mut self, sys: SystemInfo) {
    self.fps = 1.0 / sys.frame_delta.as_secs_f32();
    println!("FPS: {}", self.fps);

    if sys.kb_inputs.contains_key(&KeyCode::F1) {
      self.exiting = true;
    }

    // render
    match sys.gpu.begin_render() {
      Ok((mut encoder, surface)) => {
        sys.gpu.clear(&mut encoder, &surface, Some(wgpu::Color { r: 0.01, g: 0.0, b: 0.02, a: 1.0 }));
        sys.gpu.end_render(encoder, surface);
      }
      Err(SurfaceError::Lost | SurfaceError::Outdated) => {
        println!("Err: surface was lost or outdated. Attempting to re-connect");
        sys.gpu.resize_screen(sys.win_size.x as u32, sys.win_size.y as u32);
      }
      Err(SurfaceError::OutOfMemory) => {
        println!("Err: Out of memory. Exiting");
        self.exiting = true;
      }
      Err(e) => {
        println!("Err: {:?}", e);
      }
    }
  }
  fn request_exit(&self) -> bool {
    self.exiting
  }
  fn cleanup(&mut self) {
    println!("Goodbye");
  }
}

fn main() {
  launch(WinitConfig {
    size: (800, 600),
    max_fps: Some(120),
    title: "Re:Blueprint".to_owned(),
    icon: Some("icon.ico".to_owned()),
    ..Default::default()
  }, App::new());
}