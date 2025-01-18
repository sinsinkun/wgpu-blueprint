use std::time::Duration;

use crate::*;
use renderer::*;

#[derive(Debug)]
pub struct App {
  overlay: Option<(RPipelineId, RObjectId, RTextureId)>,
  sdf_pipe: RPipelineId,
  sdfs: Vec<RSDFObject>,
  time_since_last_fps: Duration,
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
}
impl AppBase for App {
  fn new() -> Self {
    Self {
      overlay: None,
      sdf_pipe: RPipelineId(0),
      sdfs: Vec::new(),
      time_since_last_fps: Duration::from_secs(1),
    }
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    if let Some(overlay) = self.overlay {
      renderer.resize_texture(overlay.2, overlay.1, width, height);
    }
  }
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    self.init_overlay(renderer);
    let p = renderer.add_sdf_pipeline();
    self.sdf_pipe = p;
    
    let cir = RSDFObject::circle(vec2f!(400.0, 300.0), 40.0).with_color(RColor::RED);
    let rect = RSDFObject::rect(vec2f!(200.0, 400.0), vec2f!(50.0, 80.0), None)
      .with_corner(20.0).with_color(RColor::BLUE);
    let rect2 = RSDFObject::rect(vec2f!(350.0, 200.0), vec2f!(60.0, 140.0), Some(45.0))
      .with_corner(20.0).with_color(RColor::GREEN);
    self.sdfs.push(rect);
    self.sdfs.push(rect2);
    self.sdfs.push(cir);
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // update objects
    self.sdfs[2].center = sys.m_inputs.position;

    // finalize render
    renderer.update_sdf_objects(self.sdf_pipe, sys.win_size, sys.m_inputs.position, &self.sdfs);
    match self.overlay {
      Some((p,_,_)) => {
        self.update_overlay(sys, renderer);
        vec![self.sdf_pipe, p]
      }
      None => vec![self.sdf_pipe]
    }
  }
}