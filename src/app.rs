use renderer::*;

use crate::*;

#[derive(Debug, Default)]
pub struct App {
  fps: f32,
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  shapes: Vec<(Shape, [f32; 4])>,
}
impl AppBase for App {
  fn init(&mut self, renderer: &mut Renderer) {
    renderer.load_font("./src/embed_assets/NotoSansCB.ttf");
    let (tx1, pipe1) = renderer.add_overlay_pipeline();
    let pipe2 = renderer.add_pipeline(RPipelineSetup{
      max_obj_count: 100,
      shader: include_str!("embed_assets/flat_color.wgsl"),
      uniforms: vec![
        RUniformSetup{
          bind_slot:0,
          visibility: RUniformVisibility::Fragment,
          size_in_bytes: 16
        },
      ],
      ..Default::default()
    });
    let cir_data = Primitives::reg_polygon(40.0, 32, 1.0);
    let mut cir = Shape::new(renderer, pipe2, cir_data, None);
    cir.position = [200.0, 200.0, 1.0];
    let cir_clr: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    renderer.update_object(RObjectUpdate{
      object_id: cir.id,
      uniforms: vec![
        bytemuck::cast_slice(&cir_clr),
      ],
      ..Default::default()
    });

    // upload objects
    self.pipelines.push(pipe1);
    self.pipelines.push(pipe2);
    self.textures.push(tx1);
    self.shapes.push((cir, [0.0, 0.0, 0.0, 1.0]));
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    // resize overlay
    renderer.update_texture_size(self.textures[0], Some(self.pipelines[0]), width, height);
  }
  fn update(&mut self, sys: SystemInfo) {
    self.fps = 1.0 / sys.frame_delta.as_secs_f32();
    let mx = sys.m_inputs.position.x as f32 / sys.win_size.0 as f32;
    let my = sys.m_inputs.position.y as f32 / sys.win_size.1 as f32;
    let cir_clr = [mx, 0.0, my, 1.0];
    self.shapes[0].1 = cir_clr;
    if !sys.kb_inputs.is_empty() {
      println!("Inputs: {:?}", sys.kb_inputs);
    }
    if sys.m_inputs.left == MKBState::Down {
      println!("Mouse State: {:?} -> {:?}", sys.m_inputs.pos_delta, sys.m_inputs.position);
    }
  }
  fn render(&mut self, _sys: SystemInfo, renderer: &mut Renderer) -> &Vec<RPipelineId> {
    let fps_txt = format!("FPS: {:.2}", self.fps);
    renderer.update_object(RObjectUpdate{
      object_id: self.shapes[0].0.id,
      uniforms: vec![
        bytemuck::cast_slice(&self.shapes[0].1),
      ],
      ..Default::default()
    });
    renderer.render_on_texture(&self.pipelines[1..2], self.textures[0], None);
    renderer.render_str_on_texture(
      self.textures[0], &fps_txt, 24.0, [0x34, 0xff, 0x34, 0xff], [10.0, 24.0], 2.0
    );
    &self.pipelines
  }
}
