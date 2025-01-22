use std::time::Duration;

use crate::*;
use renderer::*;

#[derive(Debug)]
pub struct App {
  overlay: Option<(RPipelineId, RObjectId, RTextureId)>,
  sdf_pipe: RPipelineId,
  sdfs: Vec<RSDFObject>,
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
      if self.time_since_last_fps > Duration::from_millis(800) {
        self.time_since_last_fps = Duration::from_nanos(0);
        self.fps_txt = format!("FPS: {:.2}", 1.0 / sys.frame_delta.as_secs_f32());
      } else {
        self.time_since_last_fps += *sys.frame_delta;
      }
      renderer.queue_overlay_text(StringPlacement {
        string: self.fps_txt.clone(),
        size: 30.0,
        color: RColor::rgba(0x34, 0xff, 0x00, 0xff),
        base_point: vec2f!(5.0, 20.0),
        spacing: 2.0,
      });
      renderer.redraw_texture_with_queue(1, overlay.2);
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
  }  
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {

    renderer.queue_overlay_text(StringPlacement {
      string: "Follow me".to_owned(),
      base_point: sys.m_inputs.position,
      size: 30.0,
      ..Default::default()
    });
    renderer.queue_overlay_text(StringPlacement {
      string: "Hello world".to_owned(),
      base_point: vec2f!(5.0, sys.win_size.y - 10.0),
      color: RColor::RED,
      size: 30.0,
      ..Default::default()
    });

    // finalize render
    renderer.update_sdf_objects(self.sdf_pipe, sys.win_size, sys.m_inputs.position, 30.0, &self.sdfs);
    match self.overlay {
      Some((p,_,_)) => {
        self.update_overlay(sys, renderer);
        vec![self.sdf_pipe, p]
      }
      None => vec![self.sdf_pipe]
    }
  }
}