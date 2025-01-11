use std::time::Duration;

use crate::*;
use renderer::*;

#[derive(Debug)]
pub struct App {
  pipelines: Vec<RPipelineId>,
  textures: Vec<RTextureId>,
  objects: Vec<RObjectId>,
  time_since_last_fps: Duration,
}
impl Default for App {
  fn default() -> Self {
    Self {
      pipelines: Vec::new(),
      textures: Vec::new(),
      objects:  Vec::new(),
      time_since_last_fps: Duration::from_secs(1),
    }
  }
}
impl App {
  fn init_circle(&mut self, renderer: &mut Renderer) {
    let pipe2 = renderer.add_pipeline(RPipelineSetup {
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
}
impl AppBase for App {
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    renderer.set_clear_color(RColor::hsv(0.577, 0.5, 0.2));
    let (pid, oid, tid) = renderer.add_overlay_pipe();
    self.pipelines.push(pid);
    self.objects.push(oid);
    self.textures.push(tid);
    self.init_circle(renderer);
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    renderer.resize_texture(self.textures[0], self.objects[0], width, height);
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // follow mouse
    let ax = sys.m_inputs.position.x - (sys.win_size.x / 2.0);
    let ay = sys.m_inputs.position.y - (sys.win_size.y / 2.0);

    // update circle
    renderer.update_object(self.objects[1], RObjectUpdate::default()
      .with_position(vec3f!(ax, ay, 10.0))
      .with_color(RColor::BLUE));

    // render fps text to inner screen
    if self.time_since_last_fps > Duration::from_millis(800) {
      self.time_since_last_fps = Duration::from_nanos(0);
      let fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
      renderer.redraw_texture_with_str(
        0, self.textures[0], &fps_txt, 40.0, RColor::rgba(0x34, 0xff, 0x00, 0xff), vec2f!(10.0, 30.0), 2.0
      );
    } else {
      self.time_since_last_fps += *sys.frame_delta;
    };

    self.pipelines.clone()
  }
}