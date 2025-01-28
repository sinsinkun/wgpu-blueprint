use std::time::Duration;

use crate::*;
use renderer::*;
use math::{calculate_sdf, ray_march_dist};

#[derive(Debug)]
pub struct App {
  overlay: Option<(RPipelineId, RObjectId, RTextureId)>,
  sdf_pipe: RPipelineId,
  indicator_pipe: RPipelineId,
  sdfs: Vec<RSDFObject>,
  indicator_sdf: RSDFObject,
  time_since_last_fps: Duration,
  fps_txt: String,
}
impl App {
  fn init_overlay(&mut self, renderer: &mut Renderer) {
    renderer.set_clear_color(RColor::hsv(0.65, 0.4, 0.02));
    let ol = renderer.add_overlay_pipe();
    self.overlay = Some(ol)
  }
  fn update_overlay(&mut self, sys: SystemInfo, renderer: &mut Renderer) {
    if let Some(overlay) = self.overlay {
      renderer.update_object(overlay.1, RObjectUpdate::default());
      if self.time_since_last_fps > Duration::from_millis(10) {
        self.time_since_last_fps = Duration::from_nanos(0);
        self.fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
        renderer.queue_overlay_text(StringPlacement {
          string: self.fps_txt.clone(),
          size: 30.0,
          color: RColor::rgba(0x34, 0xff, 0x00, 0xff),
          base_point: vec2f!(5.0, 20.0),
          spacing: 2.0,
        });
        renderer.redraw_texture_with_queue(1, overlay.2);
      } else {
        self.time_since_last_fps += *sys.frame_delta;
        renderer.clear_overlay_queue();
      }
    }
  }
}
impl AppBase for App {
  fn new() -> Self {
    Self {
      overlay: None,
      sdf_pipe: RPipelineId(0),
      indicator_pipe: RPipelineId(0),
      sdfs: Vec::new(),
      indicator_sdf: RSDFObject::circle(vec2f!(0.0, 0.0), 10.0).as_line(1.0),
      time_since_last_fps: Duration::from_secs(1),
      fps_txt: String::new(),
    }
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    if let Some(overlay) = self.overlay {
      renderer.resize_texture(overlay.2, overlay.1, width, height);
    }
  }
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    self.init_overlay(renderer);
    self.sdf_pipe = renderer.add_sdf_pipeline();
    self.indicator_pipe = renderer.add_sdf_pipeline();

    self.sdfs.push(RSDFObject::circle(vec2f!(380.0, 100.0), 60.0)
      .with_color(RColor::RED));
    self.sdfs.push(RSDFObject::rect(vec2f!(200.0, 400.0), vec2f!(80.0, 60.0), None)
      .with_corner(5.0).with_color(RColor::PURPLE));
    self.sdfs.push(RSDFObject::rect(vec2f!(300.0, 180.0), vec2f!(100.0, 60.0), None)
      .as_line(10.0).with_color(RColor::BLUE));
    // self.sdfs.push(RSDFObject::triangle(vec2f!(400.0, 400.0), vec2f!(80.0, 0.0), vec2f!(80.0, 80.0))
    //   .with_color(RColor::GREEN));
    self.sdfs.push(RSDFObject::line(vec2f!(400.0, 400.0), vec2f!(410.0, 480.0), 5.0)
      .with_color(RColor::GREEN));
    self.sdfs.push(RSDFObject::rect(vec2f!(700.0, 320.0), vec2f!(30.0, 120.0), Some(-30.0))
      .with_color(RColor::BLUE));
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // calculate sdf
    let origin = sys.win_center();
    let origin_to_mouse = (sys.m_inputs.position - origin).magnitude();
    let sdf_m = calculate_sdf(sys.m_inputs.position, 1000.0, &self.sdfs);
    let dir = origin - sys.m_inputs.position;
    let raym_m = ray_march_dist(sys.m_inputs.position, dir, origin_to_mouse, &self.sdfs);
    self.indicator_sdf.center = sys.m_inputs.position;
    if sdf_m >= 0.0 {
      self.indicator_sdf.radius = sdf_m;
      self.indicator_sdf.color = RColor::WHITE;
    } else {
      self.indicator_sdf.radius = -sdf_m;
      self.indicator_sdf.color = RColor::BLACK;
    }

    // create debug ray circles
    let mut ray_cirs: Vec<RSDFObject> = Vec::new();
    ray_cirs.push(self.indicator_sdf);
    ray_cirs.push(RSDFObject::circle(origin, 2.0).with_color(RColor::YELLOW));
    ray_march_debug(&mut ray_cirs, sys.m_inputs.position, dir, origin_to_mouse, &self.sdfs);

    // update debug text
    let txt = format!(
      "P: ({:.2}, {:.2}), SDF: {:.2}, D: {:.2}, RM: {:.2}", 
      sys.m_inputs.position.x,
      sys.m_inputs.position.y,
      sdf_m,
      origin_to_mouse,
      raym_m
    );
    renderer.queue_overlay_text(StringPlacement {
      string: txt.clone(),
      base_point: sys.m_inputs.position,
      size: 30.0,
      ..Default::default()
    });
    renderer.queue_overlay_text(StringPlacement {
      string: txt.clone(),
      base_point: vec2f!(5.0, sys.win_size.y - 10.0),
      color: RColor::RED,
      size: 30.0,
      ..Default::default()
    });

    // finalize render
    renderer.update_sdf_objects(self.sdf_pipe, sys.win_size, sys.m_inputs.position, 0.0, &self.sdfs);
    renderer.update_sdf_objects(
      self.indicator_pipe, sys.win_size, sys.m_inputs.position, 0.0, &ray_cirs
    );
    match self.overlay {
      Some((p,_,_)) => {
        self.update_overlay(sys, renderer);
        vec![self.sdf_pipe, self.indicator_pipe, p]
      }
      None => vec![self.sdf_pipe]
    }
  }
}

fn ray_march_debug(col: &mut Vec<RSDFObject>, origin: Vec2, dir: Vec2, max_dist: f32, objs: &Vec<RSDFObject>) {
  let ndir = dir.normalize();
  let mut p = origin;
  let mut sdf = calculate_sdf(p, max_dist, objs);
  let mut ray_dist = sdf;
  let mut iter = 0;
  while ray_dist < max_dist && sdf > 0.999 && iter < 99999 {
    iter += 1;
    p = p + ndir * sdf;
    sdf = calculate_sdf(p, max_dist, objs);
    col.push(RSDFObject::circle(p, sdf).as_line(1.0).with_color(RColor::rgb(34, 34, 34)));
    ray_dist += sdf;
  }
  if ray_dist > max_dist { ray_dist = max_dist; }
  col.push(RSDFObject::line(origin, origin + ray_dist * ndir, 1.0).with_color(RColor::rgba(200, 200, 0, 180)));
}