use renderer::RTextureId;

use crate::*;

#[derive(Debug, Default)]
pub struct App {
  fps: f32,
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
}
impl AppBase for App {
  fn init(&mut self, renderer: &mut Renderer) {
    renderer.load_font("./src/embed_assets/NotoSansCB.ttf");
    let p1 = renderer.add_overlay_pipeline();
    self.pipelines.push(p1.1);
    self.textures.push(p1.0);
  }
  fn update(&mut self, sys: SystemInfo) {
    self.fps = 1.0 / sys.frame_delta.as_secs_f32();
    if !sys.kb_inputs.is_empty() {
      println!("Inputs: {:?}", sys.kb_inputs);
    }
    if sys.m_inputs.left == MKBState::Down {
      println!("Mouse State: {:?} -> {:?}", sys.m_inputs.pos_delta, sys.m_inputs.position);
    }
  }
  fn pre_render(&mut self, _sys: SystemInfo, renderer: &mut Renderer) -> &Vec<RPipelineId> {
    let fps_txt = format!("FPS: {:.3}", self.fps);
    renderer.clear_texture(self.textures[0], None);
    renderer.render_str_on_texture(
      self.textures[0], &fps_txt, 24.0, [0x34, 0xff, 0x34],
      [10, 20], 2);
    &self.pipelines
  }
}
