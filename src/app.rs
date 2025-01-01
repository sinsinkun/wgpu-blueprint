use crate::*;

#[derive(Debug, Default)]
pub struct App {

}
impl AppBase for App {
  fn init(&mut self, _renderer: &mut Renderer) {
    // todo
  }
  fn update(&mut self, inputs: &HashMap<KeyCode, KBState>, _frame_delta: &Duration) {
    println!("Inputs since last frame: {:?}", inputs);
  }
  fn render(&mut self, _renderer: &mut Renderer) {
    // todo
  }
  fn cleanup(&mut self) {
    // todo
  }
}
