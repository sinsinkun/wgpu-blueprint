use std::time::Duration;

use renderer::*;

use crate::*;
use crate::ui::*;

#[derive(Debug)]
pub struct App {
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  objects: Vec<RObjectId>,
  buttons: Vec<UiButton>,
  camera_3d: RCamera,
  camera_overlay: RCamera,
  time_since_last_fps: Duration,
}
impl Default for App {
  fn default() -> Self {
    Self {
      pipelines: Vec::new(),
      textures: Vec::new(),
      objects:  Vec::new(),
      buttons: Vec::new(),
      camera_3d: RCamera::default(),
      camera_overlay: RCamera::default(),
      time_since_last_fps: Duration::from_secs(1),
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

    let btn = UiButton::new(renderer, vec2f!(100.0, 50.0))
      .at(vec3f!(100.0, 100.0, 0.0))
      .with_color(RColor::rgb(0xdd, 0xaf, 0x4f))
      .with_radius(10.0)
      .with_text(renderer, "Click me".to_string(), 24.0, RColor::MAGENTA);
    self.buttons.push(btn);
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

    // update circle
    renderer.update_object(RObjectUpdate::obj(self.objects[1])
      .with_position(vec3f!(ax, ay, 0.0))
      .with_camera(&self.camera_overlay)
      .with_color(RColor::rgba_pct(1.0 - mx, 1.0, 1.0 - my, 1.0)));
    // update rect
    renderer.update_object(RObjectUpdate::obj(self.objects[2])
      .with_position(vec3f!(200.0, 100.0, -1.0))
      .with_camera(&self.camera_overlay)
      .with_color(RColor::rgba_pct(0.2, my, mx, 1.0))
      .with_round_border(vec2f!(200.0, 100.0), 20.0));
    self.buttons[0].update(renderer, Some(&self.camera_overlay));
    renderer.render_on_texture(&vec![
      self.pipelines[1],
      self.buttons[0].get_pipeline(),
      self.pipelines[2],
    ], self.textures[0], Some([0.02, 0.02, 0.06, 1.0]));

    // update inner screen
    renderer.update_object(RObjectUpdate::obj(self.objects[0])
      .with_position(vec3f!(0.0, 0.0, -6.5))
      .with_euler_rotation(ry, rx, 0.0)
      .with_camera(&self.camera_3d));
    // render fps text to inner screen
    if self.time_since_last_fps > Duration::from_millis(800) {
      self.time_since_last_fps = Duration::from_nanos(0);
      let fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
      renderer.redraw_texture_with_str(
        self.textures[1], &fps_txt, 40.0, RColor::rgba(0x34, 0xff, 0x00, 0x22), vec2f!(10.0, 30.0), 2.0
      );
    } else {
      self.time_since_last_fps += *sys.frame_delta;
    }

    // output pipelines to render to screen
    vec!(self.pipelines[0])
  }
}
impl App {
  fn init_overlay(&mut self, renderer: &mut Renderer) {
    renderer.load_font("./src/embed_assets/NotoSansCB.ttf");
    let tx = renderer.add_texture(1000, 750, None, true);
    let txt_tx = renderer.add_texture(1000, 750, None, false);
    let pipe = renderer.add_pipeline(RPipelineSetup{
      shader: RShader::Custom(include_str!("../assets/base_radial_shadow.wgsl")),
      texture1_id: Some(tx),
      texture2_id: Some(txt_tx),
      ..Default::default()
    });
    let rect_data = Primitives::rect_indexed(20.0, 15.0, 0.0);
    let rect = renderer.add_object(RObjectSetup {
      pipeline_id: pipe,
      vertex_data: rect_data.0,
      indices: rect_data.1,
      ..Default::default()
    });

    self.pipelines.push(pipe);
    self.textures.push(tx);
    self.textures.push(txt_tx);
    self.objects.push(rect);
  }
  fn init_circle(&mut self, renderer: &mut Renderer) {
    let pipe2 = renderer.add_pipeline(RPipelineSetup{
      shader: RShader::FlatColor,
      ..Default::default()
    });
    let cir_data = Primitives::reg_polygon(10.0, 16, 0.0);
    let cir = renderer.add_object(RObjectSetup {
      pipeline_id: pipe2,
      vertex_data: cir_data,
      ..Default::default()
    });

    self.pipelines.push(pipe2);
    self.objects.push(cir);
  }
  fn init_rounded_rect(&mut self, renderer: &mut Renderer) {
    let pipe3 = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::Custom("embed_assets/rounded_rect.wgsl"),
      ..Default::default()
    });
    let rect_data = Primitives::rect_indexed(200.0, 100.0, 0.0);
    let rect = renderer.add_object(RObjectSetup {
      pipeline_id: pipe3,
      vertex_data: rect_data.0,
      indices: rect_data.1,
      ..Default::default()
    });

    self.pipelines.push(pipe3);
    self.objects.push(rect);
  }
}
