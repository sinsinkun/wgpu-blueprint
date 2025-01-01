use crate::*;

#[derive(Debug, Default)]
pub struct App {
  fps: f32,
}
impl AppBase for App {
  fn init(&mut self, _renderer: &mut Renderer) {
    // todo
  }
  fn update(&mut self, inputs: &HashMap<KeyCode, KBState>, frame_delta: &Duration) {
    self.fps = 1.0 / frame_delta.as_secs_f32();
    println!("Inputs since last frame: {:?}", inputs);
  }
  fn pre_render(&mut self, _renderer: &mut Renderer) {
    // todo
  }
  fn cleanup(&mut self) {
    // todo
  }
}
