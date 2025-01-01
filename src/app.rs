use crate::*;

#[derive(Debug, Default)]
pub struct App {
  fps: f32,
}
impl AppBase for App {
  fn init(&mut self, _renderer: &mut Renderer) {
    // todo
  }
  fn update(&mut self, sys: SystemInfo) {
    self.fps = 1.0 / sys.frame_delta.as_secs_f32();
    println!("Inputs since last frame: {:?}", sys.inputs);
  }
}
