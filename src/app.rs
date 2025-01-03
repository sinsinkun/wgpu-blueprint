use renderer::*;

use crate::*;

#[derive(Debug)]
pub struct App {
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  shapes: Vec<Shape>,
  camera_3d: RCamera,
}
impl Default for App {
  fn default() -> Self {
    Self {
      pipelines: Vec::new(),
      textures: Vec::new(),
      shapes:  Vec::new(),
      camera_3d: RCamera::new_ortho(0.0, 1000.0),
    }
  }
}
impl AppBase for App {
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    self.camera_3d = RCamera::new_persp(90.0, 0.1, 1000.0);
    self.init_overlay(renderer);
    self.init_circle(renderer);
    self.init_rounded_rect(renderer);
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // process inputs
    if !sys.kb_inputs.is_empty() {
      println!("Inputs: {:?}", sys.kb_inputs);
    }
    if sys.m_inputs.left == MKBState::Down {
      println!("Mouse State: {:?} -> {:?}", sys.m_inputs.pos_delta, sys.m_inputs.position);
    }

    let fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
    // change color based on mouse position
    let mx = sys.m_inputs.position.x / sys.win_size.x;
    let my = sys.m_inputs.position.y / sys.win_size.y;
    // follow mouse
    let ax = sys.m_inputs.position.x - (sys.win_size.x / 2.0);
    let ay = sys.m_inputs.position.y - (sys.win_size.y / 2.0);

    // update inner screen
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[0].id,
      translate: &[0.0, 0.0, -6.5],
      rotate: RRotation::Euler(my * 2.0 - 1.0, mx * 2.0 - 1.0, 0.0),
      camera: Some(&self.camera_3d),
      color: &[1.0 - my, 0.2, mx, 1.0],
      ..Default::default()
    });

    // update render objects for overlay
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[1].id,
      translate: &[ax, ay, 0.0],
      color: &[mx, 0.5, my, 1.0],
      ..Default::default()
    });
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[2].id,
      translate: &[200.0, 100.0, -1.0],
      color: &[0.2, my, mx, 1.0],
      rect_size: Some([200.0, 100.0]),
      rect_radius: 20.0,
      ..Default::default()
    });
    renderer.render_on_texture(&self.pipelines[1..3], self.textures[0], Some([0.02, 0.02, 0.06, 1.0]));

    // render fps text to overlay
    renderer.render_str_on_blank_texture(
      self.textures[1], &fps_txt, 60.0, [0x34, 0xff, 0x34, 0xff], [10.0, 40.0], 2.0
    );
    vec!(self.pipelines[0])
  }
}
impl App {
  fn init_overlay(&mut self, renderer: &mut Renderer) {
    renderer.load_font("./src/embed_assets/NotoSansCB.ttf");
    let tx = renderer.add_texture(2000, 1500, None, true);
    let txt_tx = renderer.add_texture(2000, 1500, None, true);
    let pipe = renderer.add_pipeline(RPipelineSetup{
      texture1_id: Some(tx),
      texture2_id: Some(txt_tx),
      ..Default::default()
    });
    let rect_data = Primitives::rect_indexed(20.0, 15.0, 0.0);
    let rect = Shape::new(renderer, pipe, rect_data.0, Some(rect_data.1));

    self.pipelines.push(pipe);
    self.textures.push(tx);
    self.textures.push(txt_tx);
    self.shapes.push(rect);
  }
  fn init_circle(&mut self, renderer: &mut Renderer) {
    let pipe2 = renderer.add_pipeline(RPipelineSetup{
      shader: RShader::FlatColor,
      ..Default::default()
    });
    let cir_data = Primitives::reg_polygon(40.0, 32, 0.0);
    let cir = Shape::new(renderer, pipe2, cir_data, None);

    self.pipelines.push(pipe2);
    self.shapes.push(cir);
  }
  fn init_rounded_rect(&mut self, renderer: &mut Renderer) {
    let pipe3 = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::RoundedRect,
      ..Default::default()
    });
    let rect_data = Primitives::rect_indexed(200.0, 100.0, 0.0);
    let rect = Shape::new(renderer, pipe3, rect_data.0, Some(rect_data.1));

    self.pipelines.push(pipe3);
    self.shapes.push(rect);
  }
}
