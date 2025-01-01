use std::sync::Arc;
use winit::window::Window;

use wgpu::*;

#[derive(Debug)]
pub struct Renderer<'a> {
  win_surface: Surface<'a>,
  surface_format: TextureFormat,
  device: Device,
  queue: Queue,
  pub config: SurfaceConfiguration,
  msaa: Texture,
  zbuffer: Texture,
  limits: Limits,
  // configurable
  pub clear_color: Color,
}
impl<'a> Renderer<'a> {
  // Creating some of the wgpu types requires async code
  pub async fn new(window: Arc<Window>) -> Self {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    let instance = Instance::new(InstanceDescriptor {
      backends: Backends::PRIMARY,
      ..Default::default()
    });
    // instantiate surface for window
    let win_surface = instance.create_surface(window).unwrap();

    // handle for graphics card
    let adapter = instance.request_adapter(&RequestAdapterOptions {
      power_preference: PowerPreference::default(),
      compatible_surface: Some(&win_surface),
      force_fallback_adapter: false,
    }).await.unwrap();

    // grab device & queue from adapter
    let (device, queue) = adapter.request_device(
      &DeviceDescriptor {
        required_features: Features::POLYGON_MODE_LINE | Features::POLYGON_MODE_POINT,
        required_limits: { Limits::default() },
        label: None,
        memory_hints: MemoryHints::Performance,
      },
      None, // Trace path
    ).await.unwrap();

    let surface_caps = win_surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
      .find(|f| f.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);
    let config = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::AutoNoVsync,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    let texture_size = wgpu::Extent3d {
      width: config.width,
      height: config.height,
      depth_or_array_layers: 1,
    };

    // create msaa texture
    let msaa = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("msaa-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: surface_format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });

    // create zbuffer texture
    let zbuffer = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("zbuffer-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Depth24Plus,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });

    return Self {
      win_surface,
      surface_format,
      device,
      queue,
      config,
      msaa,
      zbuffer,
      limits: Limits::default(),
      clear_color: Color::BLACK,
      // pipelines: Vec::new(),
      // textures: Vec::new(),
    };
  }
  pub fn resize(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      self.config.width = width;
      self.config.height = height;
      self.win_surface.configure(&self.device, &self.config);

      let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      };

      // remake msaa texture
      let msaa = self.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("msaa-texture"),
        size: texture_size,
        sample_count: 4,
        mip_level_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: self.surface_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[]
      });
      self.msaa.destroy();
      self.msaa = msaa;

      // remake zbuffer texture
      let zbuffer = self.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zbuffer-texture"),
        size: texture_size,
        sample_count: 4,
        mip_level_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[]
      });
      self.zbuffer.destroy();
      self.zbuffer = zbuffer;
    }
  }
  pub fn render(&mut self) -> Result<(), SurfaceError> {
    let output = self.win_surface.get_current_texture()?;
    let target = output.texture.create_view(&TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });
    // separate encoding to handle borrow checking
    self.render_impl(&mut encoder, &target);
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
  }
  fn render_impl(&mut self, encoder: &mut CommandEncoder, target: &TextureView) {
    let view = self.msaa.create_view(&TextureViewDescriptor::default());
    let zbuffer_view = self.zbuffer.create_view(&TextureViewDescriptor::default());
    let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
      label: Some("Primary Render Pass"),
      color_attachments: &[Some(RenderPassColorAttachment {
        view: &view, // render onto MSAA texture
        resolve_target: Some(target), // copy MSAA output onto target
        ops: Operations {
          load: LoadOp::Clear(self.clear_color),
          store: StoreOp::Store,
        }
      })],
      depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
        view: &zbuffer_view,
        depth_ops: Some(Operations {
          load: LoadOp::Clear(1.0),
          store: StoreOp::Store
        }),
        stencil_ops: None,
      }),
      ..Default::default()
    });
  }
}