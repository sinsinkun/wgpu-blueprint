use wgpu::SurfaceError;
use winit::keyboard::KeyCode;

use crate::{
  vec2f, vec3f,
  utils::{Vec2, Vec3},
  wrapper::{GpuAccess, MKBState, SceneBase, SystemAccess, WindowContainer},
  render::{
    ObjPipeline, Primitives, RenderCamera, RenderColor, RenderObjectSetup,
    RenderObjectUpdate, ShaderType, TextEngine
  }
};

#[derive(Debug)]
pub struct Scene2 {
  overlay: Option<ObjPipeline>,
  camera: RenderCamera,
  text_engine: TextEngine,
  refresh_timeout: f32,
}
impl Scene2 {
  fn update_fps(&mut self, sys: &mut SystemAccess, gpu: &GpuAccess, win_center: Vec2) {
    // update fps text
    self.refresh_timeout += sys.time_delta_sec();
    if self.refresh_timeout > 1.0 {
      self.refresh_timeout = 0.0;
      if let Some(objp) = &mut self.overlay {
        let txt = format!("FPS: {:.2}", sys.fps());
        let word_tx = self.text_engine.create_texture(
          &gpu.device, &gpu.queue, &txt,
          26.0, RenderColor::rgb(231, 217, 16).into(), Some(150.0), Some(30.0)
        );
        objp.replace_texture(&gpu.device, 0, 1, word_tx);
      }
    }

    // update fps position
    if let Some(p) = &mut self.overlay {
      p.update_object(0, &gpu.queue, RenderObjectUpdate::default()
        .with_position(vec3f!(76.0 - win_center.x, win_center.y - 16.0, 0.0))
        .with_camera(&self.camera)
      );
    }

  }
}
impl SceneBase for Scene2 {
  fn new() -> Self {
    Self {
      overlay: None,
      camera: RenderCamera::default(),
      text_engine: TextEngine::new(),
      refresh_timeout: 2.0,
    }
  }
  fn init(&mut self, _sys: &mut SystemAccess, gpu: &GpuAccess, window: &WindowContainer) {
    println!("Hello world 2");
    self.camera = RenderCamera::new_ortho(1.0, 1000.0, window.win_size_vec2());
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
  fn resize(&mut self, _sys: &mut SystemAccess, gpu: &GpuAccess, window: &WindowContainer, width: u32, height: u32) {
    self.resize_screen(gpu, window.gpu_surface(), width, height);
    self.camera.target_size = vec2f!(width as f32, height as f32);
  }
  fn update(&mut self, sys: &mut SystemAccess, gpu: &GpuAccess, window: &WindowContainer) {
    if sys.kb_inputs().contains_key(&KeyCode::Escape) {
      sys.request_exit();
    }
    if sys.kb_inputs().get(&KeyCode::F1) == Some(&MKBState::Released) {
      sys.request_new_window(0);
    }

    // update scene
    let win_center = window.win_size_vec2() * 0.5;
    self.update_fps(sys, gpu, win_center);

    // render
    match self.begin_render(&gpu.device, window.gpu_surface()) {
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
        self.end_render(&gpu.queue, encoder, surface);
      }
      Err(SurfaceError::Lost | SurfaceError::Outdated) => {
        println!("Err: surface was lost or outdated. Attempting to re-connect");
        self.resize_screen(gpu, window.gpu_surface(), window.win_size().0, window.win_size().1);
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
    println!("Goodbye 2");
  }
}