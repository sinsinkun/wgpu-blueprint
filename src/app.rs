use renderer::*;

use crate::*;

#[derive(Debug)]
pub struct App {
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  shapes: Vec<Shape>,
  camera_3d: RCamera,
  camera_overlay: RCamera,
}
impl Default for App {
  fn default() -> Self {
    Self {
      pipelines: Vec::new(),
      textures: Vec::new(),
      shapes:  Vec::new(),
      camera_3d: RCamera::default(),
      camera_overlay: RCamera::default(),
    }
  }
}
impl AppBase for App {
  fn init(&mut self, sys: SystemInfo, renderer: &mut Renderer) {
    self.camera_3d = RCamera::new_persp(90.0, 0.1, 1000.0);
    self.camera_overlay = RCamera::new_ortho(0.0, 1000.0);
    self.camera_overlay.target_size = Some(sys.win_size);
    self.init_overlay(renderer);
    self.init_circle(renderer);
    self.init_rounded_rect(renderer);
  }
  fn resize(&mut self, _renderer: &mut Renderer, _width: u32, height: u32) {
    let h = height as f32;
    let w = h * 4.0 / 3.0;
    self.camera_overlay.target_size = Some(vec2f!(w, h));
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
    // set rotation
    let rx = if mx > 0.8 { (mx - 0.8) * 30.0 }
    else if mx < 0.2 { (mx - 0.2) * 30.0 }
    else { 0.0 };
    let ry = if my > 0.8 { (my - 0.8) * 30.0 }
    else if my < 0.2 { (my - 0.2) * 30.0 }
    else { 0.0 };

    // update inner screen
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[0].id,
      translate: vec3f!(0.0, 0.0, -6.5),
      rotate: RRotation::Euler(ry, rx, 0.0),
      camera: Some(&self.camera_3d),
      ..Default::default()
    });

    // update render objects for overlay
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[1].id,
      translate: vec3f!(ax, ay, 0.0),
      camera: Some(&self.camera_overlay),
      color: RColor::rgba_pct(1.0 - mx, 1.0, 1.0 - my, 1.0),
      ..Default::default()
    });
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[2].id,
      translate: vec3f!(200.0, 100.0, -1.0),
      camera: Some(&self.camera_overlay),
      color: RColor::rgba_pct(0.2, my, mx, 1.0),
      rect_size: Some([200.0, 100.0]),
      rect_radius: 20.0,
      ..Default::default()
    });
    renderer.render_on_texture(&self.pipelines[1..3], self.textures[0], Some([0.02, 0.02, 0.06, 1.0]));

    // render fps text to overlay
    renderer.render_str_on_blank_texture(
      self.textures[1], &fps_txt, 30.0, [0x34, 0xff, 0x34, 0xff], [10.0, 20.0], 2.0
    );
    vec!(self.pipelines[0])
  }
}
impl App {
  fn init_overlay(&mut self, renderer: &mut Renderer) {
    renderer.load_font("./src/embed_assets/NotoSansCB.ttf");
    let tx = renderer.add_texture(1000, 750, None, true);
    let txt_tx = renderer.add_texture(1000, 750, None, true);
    let pipe = renderer.add_pipeline(RPipelineSetup{
      shader: RShader::Custom(include_str!("../assets/base_radial_shadow.wgsl")),
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
    let cir_data = Primitives::reg_polygon(10.0, 16, 0.0);
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
