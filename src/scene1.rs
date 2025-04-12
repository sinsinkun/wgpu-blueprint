use wgpu::SurfaceError;
use winit::keyboard::KeyCode;

use crate::{
  render::{
    ObjPipeline, Primitives, RenderCamera, RenderColor, RenderObjectSetup,
    RenderObjectUpdate, ShaderType, TextEngine
  }, utils::Vec3, vec3f, wrapper::{GpuAccess, MKBState, SceneBase, SystemAccess}
};

#[derive(Debug)]
pub struct Scene1 {
  overlay: Option<ObjPipeline>,
  overlay_camera: RenderCamera,
  obj_pipe: Option<ObjPipeline>,
  obj_camera: RenderCamera,
  text_engine: TextEngine,
  refresh_timeout: f32,
  lifetime: f32,
}
impl Scene1 {
  fn update_fps(&mut self, sys: &SystemAccess, gpu: &GpuAccess) {
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
        .with_camera(&self.overlay_camera)
      );
    }

  }
}
impl SceneBase for Scene1 {
  fn new() -> Self {
    Self {
      overlay: None,
      overlay_camera: RenderCamera::default(),
      obj_pipe: None,
      obj_camera: RenderCamera::default(),
      text_engine: TextEngine::new(),
      refresh_timeout: 2.0,
      lifetime: 0.0,
    }
  }
  fn init(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    println!("Init scene 1");
    self.overlay_camera = RenderCamera::new_ortho(1.0, 1000.0, sys.win_size());
    let mut overlayp = ObjPipeline::new(&gpu.device, gpu.screen_format, ShaderType::Overlay, false);
    let (verts1, index1) = Primitives::rect_indexed(150.0, 30.0, 0.0);
    overlayp.add_object(&gpu.device, &gpu.queue, RenderObjectSetup {
      vertex_data: verts1,
      indices: index1,
      camera: Some(&self.overlay_camera),
      ..Default::default()
    });
    self.overlay = Some(overlayp);

    self.obj_camera = RenderCamera::new_persp(45.0, 1.0, 1000.0, sys.win_size());
    let mut objp = ObjPipeline::new(&gpu.device, gpu.screen_format, ShaderType::Default, false);
    let (verts2, index2) = Primitives::cylinder(8.0, 12.0, 24);
    objp.add_object(&gpu.device, &gpu.queue, RenderObjectSetup {
      vertex_data: verts2,
      indices: index2,
      camera: Some(&self.obj_camera),
      ..Default::default()
    });
    self.obj_pipe = Some(objp);
  }
  fn resize(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess, width: u32, height: u32) {
    gpu.resize_screen(width, height);
    self.overlay_camera.target_size = sys.win_size();
    self.obj_camera.target_size = sys.win_size();
  }
  fn update(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    self.lifetime += sys.time_delta_sec();
    if sys.kb_inputs().contains_key(&KeyCode::Escape) {
      sys.request_exit();
    }

    if sys.kb_inputs().get(&KeyCode::Digit1) == Some(&MKBState::Released) {
      sys.next_scene = 0;
    }
    if sys.kb_inputs().get(&KeyCode::Digit2) == Some(&MKBState::Released) {
      sys.next_scene = 1;
    }

    // update scene
    self.update_fps(sys, gpu);
    if let Some(p) = &mut self.obj_pipe {
      p.update_object(0, &gpu.queue, RenderObjectUpdate::default()
        .with_camera(&self.obj_camera)
        .with_color(RenderColor::GREEN)
        .with_position(vec3f!(0.0, 0.0, -50.0))
        .with_rotation(vec3f!(1.0, 0.8, 0.2), self.lifetime * 10.0)
      );
    }

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
          if let Some(p) = &self.obj_pipe { p.render(&mut pass); }
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
  }
}
