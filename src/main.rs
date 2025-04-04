use wgpu::SurfaceError;
use winit::keyboard::KeyCode;

mod utils;
use utils::{Vec2, Vec3};
mod wrapper;
use wrapper::{launch, AppBase, GpuAccess, SystemAccess, WinitConfig};
mod render;
use render::{ObjPipeline, Primitives, RenderCamera, RenderColor, RenderObjectSetup, RenderObjectUpdate, TextEngine};

#[derive(Debug)]
pub struct App {
  obj_pipe: Option<ObjPipeline>,
  camera: RenderCamera,
  text_engine: TextEngine,
}
impl AppBase for App {
  fn new() -> Self {
    Self {
      obj_pipe: None,
      camera: RenderCamera::default(),
      text_engine: TextEngine::new(),
    }
  }
  fn init(&mut self, sys:  &mut SystemAccess, gpu: &mut GpuAccess) {
    println!("Hello world");
    self.camera = RenderCamera::new_persp(60.0, 1.0, 1000.0, sys.win_size());
    let word_tx = self.text_engine.create_texture(&gpu.device, &gpu.queue, "ネタバレ Please help me", (300, 200));

    let mut objp = ObjPipeline::new(&gpu.device, gpu.screen_format, false, false);
    let (verts1, index1) = Primitives::cube_indexed(15.0, 10.0, 20.0);
    objp.add_object(&gpu.device, &gpu.queue, RenderObjectSetup {
      vertex_data: verts1,
      indices: index1,
      camera: Some(&self.camera),
      texture2: Some(word_tx),
      ..Default::default()
    });
    self.obj_pipe = Some(objp);
  }
  fn resize(&mut self, _sys: &mut SystemAccess, gpu: &mut GpuAccess, width: u32, height: u32) {
    gpu.resize_screen(width, height);
    self.camera.target_size = vec2f!(width as f32, height as f32);
  }
  fn update(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    if sys.kb_inputs().contains_key(&KeyCode::Escape) {
      sys.request_exit();
    }

    // update objects
    if let Some(p) = &mut self.obj_pipe {
      p.update_object(0, &gpu.queue, RenderObjectUpdate {
        translate: vec3f!(5.0, -10.0, -25.0),
        camera: Some(&self.camera),
        ..Default::default()
      }.with_color(RenderColor::GRAY));
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
    if let Some(p) = &mut self.obj_pipe {
      p.destroy();
      self.obj_pipe = None;
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