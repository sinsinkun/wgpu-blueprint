use wgpu::SurfaceError;
use winit::keyboard::KeyCode;

mod utils;
use utils::{Vec2, Vec3};
mod wrapper;
use wrapper::{launch, AppBase, GpuAccess, SystemAccess, WinitConfig};
mod render;
use render::{
  ObjPipeline, Primitives, RenderCamera, RenderColor, RenderObjectSetup,
  RenderObjectUpdate, ShaderType, TextEngine
};

#[derive(Debug)]
pub struct App {
  overlay: Option<ObjPipeline>,
  camera: RenderCamera,
  text_engine: TextEngine,
  refresh_timeout: f32,
}
impl App {
  fn update_fps(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    // update fps text
    self.refresh_timeout += sys.time_delta_sec();
    if self.refresh_timeout > 1.0 {
      self.refresh_timeout = 0.0;
      if let Some(objp) = &mut self.overlay {
        let txt = format!("FPS: {:.2}", sys.fps());
        let word_tx = self.text_engine.create_texture(
          &gpu.device, &gpu.queue, &txt,
          26.0, RenderColor::rgb(40, 200, 0).into(), Some(150.0), Some(30.0)
        );
        objp.replace_texture(&gpu.device, 0, 1, word_tx);
      }
    }

    // update fps position
    if let Some(p) = &mut self.overlay {
      p.update_object(0, &gpu.queue, RenderObjectUpdate::default()
        .with_position(vec3f!(76.0 - sys.win_center().x, sys.win_center().y - 16.0, 0.0))
        .with_camera(&self.camera)
      );
    }

  }
}
impl AppBase for App {
  fn new() -> Self {
    Self {
      overlay: None,
      camera: RenderCamera::default(),
      text_engine: TextEngine::new(),
      refresh_timeout: 2.0,
    }
  }
  fn init(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    println!("Hello world");
    self.camera = RenderCamera::new_ortho(1.0, 1000.0, sys.win_size());
    let mut objp = ObjPipeline::new(&gpu.device, gpu.screen_format, ShaderType::Overlay, false);
    let (verts1, index1) = Primitives::rect_indexed(150.0, 30.0, 0.0);
    objp.add_object(&gpu.device, &gpu.queue, RenderObjectSetup {
      vertex_data: verts1,
      indices: index1,
      camera: Some(&self.camera),
      ..Default::default()
    });
    self.overlay = Some(objp);
  }
  fn resize(&mut self, _sys: &mut SystemAccess, gpu: &mut GpuAccess, width: u32, height: u32) {
    gpu.resize_screen(width, height);
    self.camera.target_size = vec2f!(width as f32, height as f32);
  }
  fn update(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    if sys.kb_inputs().contains_key(&KeyCode::Escape) {
      sys.request_exit();
    }

    // update scene
    self.update_fps(sys, gpu);

    // render
    match gpu.begin_render() {
      Ok((mut encoder, surface)) => {
        let target = surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
          let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear-render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
              view: &target,
              resolve_target: None,
              ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(RenderColor::rgb(1, 2, 5).into()),
                store: wgpu::StoreOp::Store
              }
            })],
            ..Default::default()
          });
          if let Some(p) = &self.overlay { p.render(&mut pass); }
        }
        gpu.end_render(encoder, surface);
      }
      Err(SurfaceError::Lost | SurfaceError::Outdated) => {
        println!("Err: surface was lost or outdated. Attempting to re-connect");
        gpu.resize_screen(sys.win_size().x as u32, sys.win_size().y as u32);
      }
      Err(SurfaceError::OutOfMemory) => {
        println!("Err: Out of memory. Exiting");
        sys.request_exit();
      }
      Err(e) => {
        println!("Err: {:?}", e);
      }
    }
  }
  fn cleanup(&mut self) {
    if let Some(p) = &mut self.overlay {
      p.destroy();
      self.overlay = None;
    }
    println!("Goodbye");
  }
}

fn main() {
  launch(WinitConfig {
    size: (800, 600),
    max_fps: Some(120),
    title: "Re:Blueprint".to_owned(),
    icon: Some("icon.ico".to_owned()),
    ..Default::default()
  }, App::new());
}