mod utils;
mod wrapper;
use wrapper::{launch, AppBase, SystemInfo, WinitConfig};

#[derive(Debug)]
pub struct App {
  fps: f32,
}
impl AppBase for App {
  fn new() -> Self {
    Self {
      fps: 0.0,
    }
  }
  fn init(&mut self, _sys: SystemInfo) {
    println!("Hello world");
  }
  fn update(&mut self, sys: SystemInfo) {
    self.fps = 1.0 / sys.frame_delta.as_secs_f32();
    println!("FPS: {}", self.fps);
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