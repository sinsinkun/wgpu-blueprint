use std::time::Duration;

use crate::*;
use renderer::*;

#[derive(Debug)]
struct Circle {
  obj_id: RObjectId,
  color: RColor,
  pos: Vec2,
  radius: f32,
}

#[derive(Debug)]
struct Rectangle {
  obj_id: RObjectId,
  color: RColor,
  pos: Vec2,
  size: Vec2,
}

#[derive(Debug)]
pub struct App {
  overlay: Option<(RPipelineId, RObjectId, RTextureId)>,
  obj_pipe: RPipelineId,
  p_pipe: RPipelineId,
  circles: Vec<Circle>,
  rects: Vec<Rectangle>,
  p_cir: Circle,
  time_since_last_fps: Duration,
}
impl App {
  fn init_overlay(&mut self, renderer: &mut Renderer) {
    renderer.set_clear_color(RColor::hsv(0.65, 0.4, 0.02));
    let ol = renderer.add_overlay_pipe();
    self.overlay = Some(ol)
  }
  fn init_cir(&mut self, renderer: &mut Renderer, radius: f32, color: RColor, pos: Vec2) {
    let model = Primitives::reg_polygon(radius, 32, 0.0);
    let obj = renderer.add_object(RObjectSetup {
      pipeline_id: self.obj_pipe,
      vertex_data: model,
      ..Default::default()
    });
    self.circles.push(Circle {
      obj_id: obj,
      color,
      pos,
      radius,
    });
  }
  fn init_rect(&mut self, renderer: &mut Renderer, size: Vec2, color: RColor, pos: Vec2) {
    let model = Primitives::rect(size.x, size.y, 0.0);
    let obj = renderer.add_object(RObjectSetup {
      pipeline_id: self.obj_pipe,
      vertex_data: model,
      ..Default::default()
    });
    self.rects.push(Rectangle {
      obj_id: obj,
      color,
      pos,
      size
    });
  }
  fn init_p_cir(&mut self, renderer: &mut Renderer) {
    let model = Primitives::reg_polygon(1.0, 64, 0.0);
    let obj = renderer.add_object(RObjectSetup {
      pipeline_id: self.p_pipe,
      vertex_data: model,
      ..Default::default()
    });
    self.p_cir = Circle {
      obj_id: obj,
      color: RColor::WHITE,
      pos: vec2f!(0.0, 0.0),
      radius: 1.0,
    };
  }
  fn update_overlay(&mut self, sys: SystemInfo, renderer: &mut Renderer) {
    if let Some(overlay) = self.overlay {
      renderer.update_object(overlay.1, RObjectUpdate::default());
      if self.time_since_last_fps > Duration::from_millis(800) {
        self.time_since_last_fps = Duration::from_nanos(0);
        let fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
        renderer.redraw_texture_with_str(
          1, overlay.2, &fps_txt, 30.0, RColor::rgba(0x34, 0xff, 0x00, 0xff), vec2f!(5.0, 20.0), 2.0
        );
      } else {
        self.time_since_last_fps += *sys.frame_delta;
      }
    }
  }
  fn render_cirs(&self, renderer: &mut Renderer) {
    for cir in &self.circles {
      renderer.update_object(cir.obj_id, RObjectUpdate::default()
        .with_color(cir.color)
        .with_position(vec3f!(cir.pos.x, cir.pos.y, 0.0))
      );
    }
  }
  fn render_rects(&self, renderer: &mut Renderer) {
    for rect in &self.rects {
      renderer.update_object(rect.obj_id, RObjectUpdate::default()
        .with_color(rect.color)
        .with_position(vec3f!(rect.pos.x, rect.pos.y, 0.0))
      );
    }
  }
  fn render_p_cir(&self, m_pos: Vec2, renderer: &mut Renderer) {
    let color = if self.p_cir.radius > 0.0 {
      RColor::GREEN
    } else {
      RColor::RED
    };
    renderer.update_object(self.p_cir.obj_id, RObjectUpdate::default()
      .with_color(color)
      .with_position(vec3f!(m_pos.x, m_pos.y, 0.0))
      .with_scale(vec3f!(self.p_cir.radius, self.p_cir.radius, 1.0))
    );
  }
}
impl AppBase for App {
  fn new() -> Self{
    Self {
      overlay: None,
      obj_pipe: RPipelineId(0),
      p_pipe: RPipelineId(0),
      circles: Vec::new(),
      rects: Vec::new(),
      p_cir: Circle {
        obj_id: RObjectId(0),
        color: RColor::WHITE,
        pos: vec2f!(0.0, 0.0),
        radius: 1.0
      },
      time_since_last_fps: Duration::from_secs(0),
    }
  }
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    self.init_overlay(renderer);
    self.obj_pipe = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::FlatColor,
      ..Default::default()
    });
    self.p_pipe = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::Custom(include_str!("../assets/radial_border.wgsl")),
      ..Default::default()
    });
    self.init_cir(renderer, 40.0, RColor::BLUE, vec2f!(-100.0, 200.0));
    self.init_cir(renderer, 40.0, RColor::BLUE, vec2f!(-200.0, 50.0));
    self.init_cir(renderer, 40.0, RColor::BLUE, vec2f!(200.0, -150.0));
    self.init_rect(renderer, vec2f!(60.0, 80.0), RColor::BLUE, vec2f!(10.0, -20.0));
    self.init_rect(renderer, vec2f!(40.0, 80.0), RColor::BLUE, vec2f!(-220.0, 150.0));
    self.init_rect(renderer, vec2f!(80.0, 50.0), RColor::BLUE, vec2f!(300.0, -180.0));
    self.init_p_cir(renderer);
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    if let Some(overlay) = self.overlay {
      renderer.resize_texture(overlay.2, overlay.1, width, height);
    }
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // logic updates
    let p = sys.m_pos_world_space_2d(None);
    let mut signed_dst = 600.0;
    for cir in &self.circles {
      let d = math::signed_dist_to_cir(p, cir.pos, cir.radius);
      if d < signed_dst { signed_dst = d }
    }
    for rect in &self.rects {
      let d = math::signed_dist_to_rect(p, rect.pos, rect.size, None);
      if d < signed_dst { signed_dst = d }
    }
    self.p_cir.radius = signed_dst;

    // render updates
    self.render_cirs(renderer);
    self.render_rects(renderer);
    self.render_p_cir(p, renderer);
    match self.overlay {
      Some((p,_,_)) => {
        self.update_overlay(sys, renderer);
        vec![self.obj_pipe, self.p_pipe, p]
      }
      None => vec![self.obj_pipe, self.p_pipe]
    }
  }
}