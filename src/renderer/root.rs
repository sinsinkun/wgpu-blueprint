use std::sync::Arc;
use std::num::NonZeroU64;
use std::path::Path;

use image::{ImageReader, DynamicImage, GenericImageView};
use winit::window::Window;

use wgpu::*;
use super::*;

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
  pub pipelines: Vec<RPipeline>,
  pub textures: Vec<wgpu::Texture>,
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
      pipelines: Vec::new(),
      textures: Vec::new(),
    };
  }
  // update surface/surface textures on resize
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
  // exposed render method
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
  // core render actions to feed into encoder
  fn render_impl(&mut self, encoder: &mut CommandEncoder, target: &TextureView) {
    let view = self.msaa.create_view(&TextureViewDescriptor::default());
    let zbuffer_view = self.zbuffer.create_view(&TextureViewDescriptor::default());
    let _ = encoder.begin_render_pass(&RenderPassDescriptor {
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
  // create new texture and add it to the textures collection
  pub fn add_texture(
    &mut self,
    width: u32,
    height: u32,
    texture_path: Option<&Path>,
    use_device_format: bool
  ) -> u32 {
    let id = self.textures.len() as u32;
    let mut texture_size = Extent3d { width, height, depth_or_array_layers: 1 };
    let mut texture_data: Option<DynamicImage> = None;

    // modify texture size/data based on file data
    if let Some(str) = texture_path {
      match ImageReader::open(str) {
        Ok(img_file) => match img_file.decode() {
          Ok(img_data) => {
            texture_size.width = img_data.dimensions().0;
            texture_size.height = img_data.dimensions().1;
            texture_data = Some(img_data);
          }
          Err(..) => {
            eprintln!("Err: Could not decode image file");
          }
        }
        Err(..) => {
          eprintln!("Err: Could not open image file");
        }
      };
    }

    // create texture
    let tex_format = if use_device_format { self.surface_format } 
    else { TextureFormat::Rgba8Unorm };
    let texture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: tex_format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[]
    });
    if let Some(img) = texture_data {
      // copy image into texture
      self.queue.write_texture(
        ImageCopyTexture {
          texture: &texture,
          mip_level: 0,
          origin: Origin3d::ZERO,
          aspect: TextureAspect::All,
        }, 
        &img.to_rgba8(),
        ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(4 * texture_size.width),
          rows_per_image: Some(texture_size.height),
        },
        texture_size
      );
    }
    // add to cache
    self.textures.push(texture);
    id
  }
  // update image on texture
  pub fn update_texture(&mut self, texture_id: u32, texture_path: &Path) {
    let texture = &mut self.textures[texture_id as usize];
    match ImageReader::open(texture_path) {
      Ok(img_file) => match img_file.decode() {
        Ok(img_data) => {
          // get data from image file
          let rgba8 = img_data.to_rgba8();
          let dimensions = img_data.dimensions();
          let texture_size = Extent3d { 
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
          };
          // write to texture
          self.queue.write_texture(
            ImageCopyTexture {
              texture: &texture,
              mip_level: 0,
              origin: Origin3d::ZERO,
              aspect: TextureAspect::All,
            },
            &rgba8,
            ImageDataLayout {
              offset: 0,
              bytes_per_row: Some(4 * dimensions.0),
              rows_per_image: Some(dimensions.1),
            },
            texture_size
          );
        }
        Err(..) => {
          eprintln!("Err: Could not decode image file");
        }
      }
      Err(..) => {
        eprintln!("Err: Could not open image file");
      }
    }
  }
  // re-create texture with new size
  pub fn resize_texture(&mut self, texture_id: u32, pipeline_id: Option<u32>, width: u32, height: u32) {
    let old_texture = &mut self.textures[texture_id as usize];

    // make new texture
    let texture_size = Extent3d { width, height, depth_or_array_layers: 1 };
    let new_texture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: old_texture.format(),
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[]
    });
    old_texture.destroy();
    self.textures[texture_id as usize] = new_texture;

    // update bind group
    if let Some(p_id) = pipeline_id {
      let new_bind_id = {
        let pipeline = &self.pipelines[p_id as usize];
        let pipe = &pipeline.pipe;
        self.build_bind_group0(pipe, pipeline.max_obj_count, Some(texture_id), None, pipeline.vertex_type, pipeline.max_joints_count) // TODO: handle resizing second texture
      };
      let pipeline = &mut self.pipelines[p_id as usize];
      pipeline.bind_group0 = new_bind_id;
    }
  }
  // create new render pipeline and add it to pipeline collection
  pub fn add_pipeline(&mut self, setup: RPipelineSetup) -> u32 {
    let id = self.pipelines.len() as u32;

    // translate cullmode
    let cull_mode: Option<Face> = match setup.cull_mode {
      1 => Some(Face::Back),
      2 => Some(Face::Front),
      _ => None
    };
    // translate polygon mode
    let (polygon_mode, topology): (PolygonMode, PrimitiveTopology) = match setup.poly_mode {
      6 => (PolygonMode::Line, PrimitiveTopology::LineList),
      7 => (PolygonMode::Point, PrimitiveTopology::PointList),
      _ => (PolygonMode::Fill, PrimitiveTopology::TriangleList),
    };

    // build render pipeline
    let shader_mod = self.device.create_shader_module(ShaderModuleDescriptor {
      label: Some("shader-module"),
      source: ShaderSource::Wgsl(setup.shader.into()),
    });
    // switch between static/dynamic vertex bind group entries
    let mut bind_group0_entries: Vec<BindGroupLayoutEntry> = vec![
      // mvp matrix
      BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: true,
          min_binding_size: None,
        },
        count: None,
      },
      // texture sampler
      BindGroupLayoutEntry {
        binding: 1,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
      },
      // texture 1
      BindGroupLayoutEntry {
        binding: 2,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
          sample_type: TextureSampleType::Float { filterable: true },
          view_dimension: TextureViewDimension::D2,
          multisampled: false,
        },
        count: None,
      },
      // texture 2
      BindGroupLayoutEntry {
        binding: 3,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
          sample_type: TextureSampleType::Float { filterable: true },
          view_dimension: TextureViewDimension::D2,
          multisampled: false,
        },
        count: None,
      },
    ];
    if setup.vertex_type == RPipelineSetup::VERTEX_TYPE_ANIM {
      bind_group0_entries.push(BindGroupLayoutEntry {
        binding: 4,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      });
    }
    let bind_group0_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("bind-group0-layout"),
      entries: &bind_group0_entries
    });
    let mut bind_group_container: Vec<&BindGroupLayout> = vec![&bind_group0_layout];
    // build custom bind group layout
    let bind_group1_layout: BindGroupLayout;
    if setup.uniforms.len() > 0 {
      let mut entries: Vec<BindGroupLayoutEntry> = Vec::new();
      // add bind group entries to layout
      for u in &setup.uniforms {
        let visibility = match u.visibility {
          1 => ShaderStages::VERTEX,
          2 => ShaderStages::FRAGMENT,
          _ => ShaderStages::VERTEX_FRAGMENT,
        };
        entries.push(BindGroupLayoutEntry {
          binding: u.bind_slot,
          visibility,
          ty: BindingType::Buffer { 
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: None,
          },
          count: None
        });
      }
      bind_group1_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind-group0-layout"),
        entries: &entries.as_slice()
      });
      bind_group_container.push(&bind_group1_layout);
    }
    let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("pipeline-layout"),
      bind_group_layouts: bind_group_container.as_slice(),
      push_constant_ranges: &[]
    });
    // switch between static/dynamic vertex layouts
    let vertex_attr_static = vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
    let vertex_attr_anim = vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Uint32x4, 4 => Float32x4];
    let vertex_layout = match setup.vertex_type {
      RPipelineSetup::VERTEX_TYPE_ANIM => VertexBufferLayout {
        array_stride: std::mem::size_of::<RVertexAnim>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_anim,
      },
      _ => VertexBufferLayout {
        array_stride: std::mem::size_of::<RVertex>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_static,
      }
    };
    let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("render-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader_mod,
        entry_point: setup.vertex_fn,
        buffers: &[vertex_layout],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: setup.fragment_fn,
        targets: &[Some(ColorTargetState{
          format: self.surface_format,
          blend: Some(BlendState { 
            color: BlendComponent {
              operation: BlendOperation::Add,
              src_factor: BlendFactor::SrcAlpha,
              dst_factor: BlendFactor::OneMinusSrcAlpha
            },
            alpha: BlendComponent {
              operation: BlendOperation::Add,
              src_factor: BlendFactor::SrcAlpha,
              dst_factor: BlendFactor::OneMinusSrcAlpha
            }
          }),
          write_mask: ColorWrites::ALL
        })],
        compilation_options: PipelineCompilationOptions::default(),
      }),
      multisample: MultisampleState {
        count: 4,
        mask: !0,
        alpha_to_coverage_enabled: true,
      },
      depth_stencil: Some(DepthStencilState {
        format: TextureFormat::Depth24Plus,
        depth_write_enabled: true,
        depth_compare: CompareFunction::LessEqual,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
      }),
      primitive: PrimitiveState {
        cull_mode,
        polygon_mode,
        topology,
        ..PrimitiveState::default()
      },
      multiview: None,
      cache: None,
    });
    // build bind groups
    let bind_group0: RBindGroup = self.build_bind_group0(&pipeline, setup.max_obj_count, setup.texture1_id, setup.texture2_id, setup.vertex_type, setup.max_joints_count);
    let mut bind_group1: Option<RBindGroup> = None;
    if setup.uniforms.len() > 0 {
      bind_group1 = Some(self.build_bind_group1(&pipeline, setup.max_obj_count, setup.uniforms));
    }
    // add to cache
    let pipe = RPipeline {
      pipe: pipeline,
      objects: Vec::new(),
      max_obj_count: setup.max_obj_count,
      vertex_type: setup.vertex_type,
      max_joints_count: setup.max_joints_count,
      bind_group0,
      bind_group1,
    };
    self.pipelines.push(pipe);
    id
  }
  // part of pipeline build process
  fn build_bind_group0(
    &self, pipeline: &RenderPipeline,
    max_obj_count: usize,
    texture1: Option<u32>,
    texture2: Option<u32>,
    vertex_type: u8,
    max_joints: u32,
  ) -> RBindGroup {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    // create mvp buffer
    let mvp_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("mvp-uniform-buffer"),
      size: min_stride as u64 * max_obj_count as u64,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    // create texture
    let texture1_view: TextureView;
    let texture2_view: TextureView;
    let texture_size = Extent3d {
      width: 10,
      height: 10,
      depth_or_array_layers: 1,
    };
    let ftexture = self.device.create_texture(&TextureDescriptor {
      label: Some("input-texture"),
      size: texture_size,
      sample_count: 1,
      mip_level_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8Unorm,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[]
    });
    if let Some(tx_id) = texture1 {
      texture1_view = self.textures[tx_id as usize].create_view(&TextureViewDescriptor::default());
    } else {
      texture1_view = ftexture.create_view(&TextureViewDescriptor::default());
    }
    if let Some(tx_id) = texture2 {
      texture2_view = self.textures[tx_id as usize].create_view(&TextureViewDescriptor::default());
    } else {
      texture2_view = ftexture.create_view(&TextureViewDescriptor::default());
    }
    // create sampler
    let sampler = self.device.create_sampler(&SamplerDescriptor {
      label: Some("texture-sampler"),
      address_mode_u: AddressMode::ClampToEdge,
      address_mode_v: AddressMode::ClampToEdge,
      address_mode_w: AddressMode::ClampToEdge,
      mag_filter: FilterMode::Linear,
      min_filter: FilterMode::Nearest,
      mipmap_filter: FilterMode::Nearest,
      ..Default::default()
    });
    // create bind entries
    let mvp_size = NonZeroU64::new(192); // 4 bytes * 4 rows * 4 columns * 3 matrices
    let mut bind_entries: Vec<BindGroupEntry> = vec![
      BindGroupEntry {
        binding: 0,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &mvp_buffer, offset: 0, size: mvp_size
        })
      },
      BindGroupEntry {
        binding: 1,
        resource: BindingResource::Sampler(&sampler)
      },
      BindGroupEntry {
        binding: 2,
        resource: BindingResource::TextureView(&texture1_view)
      },
      BindGroupEntry {
        binding: 3,
        resource: BindingResource::TextureView(&texture2_view)
      },
    ];
    // create joints matrix buffer
    let joints_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("joint-transforms-buffer"),
      size: (max_joints * 4 * 4 * 4).into(), // 4x4 matrix of f32 values
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false
    });
    if vertex_type == RPipelineSetup::VERTEX_TYPE_ANIM {
      bind_entries.push(BindGroupEntry {
        binding: 4,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &joints_buffer, offset: 0, size: None
        })
      });
    }

    // create bind group
    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      label: Some("bind-group-0"),
      layout: &pipeline.get_bind_group_layout(0),
      entries: &bind_entries
    });

    // create output
    let mut output_entries = vec![mvp_buffer];
    if vertex_type == RPipelineSetup::VERTEX_TYPE_ANIM {
      output_entries.push(joints_buffer);
    }
    RBindGroup {
      base: bind_group,
      entries: output_entries
    }
  }
  // part of pipeline build process
  fn build_bind_group1(
    &self,
    pipeline: &RenderPipeline,
    max_obj_count: usize,
    uniforms: Vec<RUniformSetup>,
  ) -> RBindGroup {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    let mut bind_entries: Vec<Buffer> = Vec::new();
    let mut bind_desc: Vec<BindGroupEntry> = Vec::new();
    // build bind entries for uniforms
    for i in 0..uniforms.len() {
      let size = min_stride * max_obj_count as u32;
      let label = "custom-uniform".to_owned() + &i.to_string();
      let entry = self.device.create_buffer(&BufferDescriptor { 
        label: Some(&label),
        size: size as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false 
      });
      bind_entries.push(entry);
    }
    // build bind_descriptors for uniforms
    for (i, u) in uniforms.iter().enumerate() {
      let desc = BindGroupEntry {
        binding: i as u32,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &bind_entries[i], offset: 0, size: NonZeroU64::new(u.size_in_bytes as u64)
        })
      };
      bind_desc.push(desc);
    }
    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      label: Some("bind-group-1"),
      layout: &pipeline.get_bind_group_layout(1),
      entries: &bind_desc
    });

    return RBindGroup {
      base: bind_group,
      entries: bind_entries
    }
  }
  // destroy all resources
  pub fn destroy(&mut self, destroy_renderer: bool) {
    // destroy textures
    for tx in &mut self.textures {
      tx.destroy();
    }
    self.textures.clear();
    // destroy pipeline buffers
    for pipe in &mut self.pipelines {
      for obj in &mut pipe.objects {
        obj.v_buffer.destroy();
        if let Some(ibf) = &mut obj.index_buffer {
          ibf.destroy();
        }
      }
      for bf in &mut pipe.bind_group0.entries {
        bf.destroy();
      }
      if let Some(bg1) = &mut pipe.bind_group1 {
        for bf in &mut bg1.entries {
          bf.destroy();
        }
      }
    }
    self.pipelines.clear();
    // destroy device
    if destroy_renderer {
      self.msaa.destroy();
      self.zbuffer.destroy();
      self.device.destroy();
    }
  }
}