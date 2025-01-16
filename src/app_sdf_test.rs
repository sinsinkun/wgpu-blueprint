use std::time::Duration;

use crate::*;
use renderer::*;

#[derive(Debug)]
pub struct App {
  overlay: Option<(RPipelineId, RObjectId, RTextureId)>,
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
  fn new() -> Self{
    Self {
      overlay: None,
      time_since_last_fps: Duration::from_secs(0),
    }
  }
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    self.init_overlay(renderer);
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    if let Some(overlay) = self.overlay {
      renderer.resize_texture(overlay.2, overlay.1, width, height);
    }
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    match self.overlay {
      Some((p,_,_)) => {
        self.update_overlay(sys, renderer);
        vec![p]
      }
      None => vec![]
    }
  }
}