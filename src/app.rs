use renderer::*;

use crate::*;

#[derive(Debug, Default)]
pub struct App {
  fps: f32,
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  shapes: Vec<Shape>,
}
impl AppBase for App {
  fn init(&mut self, renderer: &mut Renderer) {
    self.init_overlay(renderer);
    self.init_circle(renderer);
    self.init_rect(renderer);
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    // resize overlay
    renderer.update_texture_size(self.textures[0], Some(self.pipelines[0]), width, height);
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> &Vec<RPipelineId> {
    // process inputs
    if !sys.kb_inputs.is_empty() {
      println!("Inputs: {:?}", sys.kb_inputs);
    }
    if sys.m_inputs.left == MKBState::Down {
      println!("Mouse State: {:?} -> {:?}", sys.m_inputs.pos_delta, sys.m_inputs.position);
    }
    
    self.fps = 1.0 / sys.frame_delta.as_secs_f32();
    let fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
    // change color based on mouse position
    let mx = sys.m_inputs.position.x as f32 / sys.win_size.0 as f32;
    let my = sys.m_inputs.position.y as f32 / sys.win_size.1 as f32;
    // follow mouse
    let ax = sys.m_inputs.position.x as f32 - (sys.win_size.0 as f32 / 2.0);
    let ay = sys.m_inputs.position.y as f32 - (sys.win_size.1 as f32 / 2.0);

    // updater render objects for overlay
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[0].id,
      color: &[mx, 0.5, my, 1.0],
      translate: &[ax, ay, 0.0],
      ..Default::default()
    });
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[1].id,
      color: &[1.0 - my, 0.2, mx, 1.0],
      translate: &[0.0, 0.0, -15.0],
      rotate_axis: &[0.0, 1.0, 0.0],
      rotate_deg:  mx * 10.0 - 5.0,
      camera: Some(&RCamera::new_persp(90.0, 0.1, 1000.0)),
      ..Default::default()
    });
    renderer.render_on_texture(&self.pipelines[1..3], self.textures[0], None);

    // render fps text to overlay
    renderer.render_str_on_texture(
      self.textures[0], &fps_txt, 24.0, [0x34, 0xff, 0x34, 0xff], [10.0, 24.0], 2.0
    );
    &self.pipelines
  }
}
impl App {
  fn init_overlay(&mut self, renderer: &mut Renderer) {
    renderer.load_font("./src/embed_assets/NotoSansCB.ttf");
    let (tx1, pipe1) = renderer.add_overlay_pipeline();

    self.pipelines.push(pipe1);
    self.textures.push(tx1);
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
  fn init_rect(&mut self, renderer: &mut Renderer) {
    let pipe3 = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::Custom(include_str!("embed_assets/rounded_rect.wgsl")),
      ..Default::default()
    });
    let rect_data = Primitives::rect_indexed(20.0, 10.0, 0.0);
    let rect = Shape::new(renderer, pipe3, rect_data.0, Some(rect_data.1));

    self.pipelines.push(pipe3);
    self.shapes.push(rect);
  }
}
