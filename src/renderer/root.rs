use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::num::NonZeroU64;

use winit::window::Window;
use image::{ImageReader, DynamicImage, GenericImageView};
use wgpu::*;

use super::*;

// -- PRIMARY RENDERER INTERFACE --
#[derive(Debug)]
pub struct Renderer<'a> {
  surface: wgpu::Surface<'a>,
  surface_format: wgpu::TextureFormat,
  device: wgpu::Device,
  queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  msaa: wgpu::Texture,
  zbuffer: wgpu::Texture,
  limits: wgpu::Limits,
  pub default_cam: RCamera,
  pub clear_color: wgpu::Color,
  pub pipelines: Vec<RPipeline>,
  pub textures: Vec<wgpu::Texture>,
  font_cache: Option<Vec<u8>>,
}

impl<'a> Renderer<'a> {
  /// Create wgpu renderer instance attached to window
  /// - Creating some of the wgpu types requires async code
  pub async fn new(window: Arc<Window>) -> Renderer<'a> {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::PRIMARY,
      ..Default::default()
    });

    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    // handle for graphics card
    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::default(),
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
      },
    ).await.unwrap();

    // grab device & queue from adapter
    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        required_features: wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::POLYGON_MODE_POINT,
        required_limits: { wgpu::Limits::default() },
        label: None,
        memory_hints: MemoryHints::Performance,
      },
      None, // Trace path
    ).await.unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a different
    // one will result in all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps.formats.iter()
      .copied()
      .filter(|f| f.is_srgb())
      .next()
      .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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

    // create default camera setup
    let default_cam = RCamera::new_ortho(0.0, 1000.0);

    return Self {
      surface,
      surface_format,
      device,
      queue,
      config,
      pipelines: Vec::new(),
      textures: Vec::new(),
      msaa,
      zbuffer,
      limits: Limits::default(),
      clear_color: Color { r: 0.002, g: 0.002, b: 0.008, a: 1.0 },
      default_cam,
      font_cache: None,
    };
  }
  /// Destroys surface screen texture and remakes it
  /// - Also destroys and remakes MSAA and z-buffer textures
  pub fn resize(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      self.config.width = width;
      self.config.height = height;
      self.surface.configure(&self.device, &self.config);

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
  /// update default clear_color
  pub fn set_clear_color(&mut self, r: f64, g: f64, b:f64, a:f64) {
    self.clear_color.r = r;
    self.clear_color.g = g;
    self.clear_color.b = b;
    self.clear_color.a = a;
  }
  /// load new font data into font_cache
  pub fn load_font(&mut self, font_path: &str) {
    match fs::read(font_path) {
      Ok(f) => {
        self.font_cache = Some(f);
      }
      Err(_) => {
        println!("Err: Could not open font file");
      }
    };
  }
  /// create new texture
  pub fn add_texture(&mut self, width: u32, height: u32, texture_path: Option<&Path>, use_device_format: bool) -> RTextureId {
    let id = self.textures.len();
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
    
    // create msaa texture
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

    // create z-buffer texture
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

    // add to cache
    self.textures.push(texture);
    self.textures.push(msaa);
    self.textures.push(zbuffer);
    RTextureId(id, id+1, id+2)
  }
  /// copies image (from path) onto texture
  pub fn update_texture(&mut self, texture_id: RTextureId, texture_path: &Path) {
    let texture = &mut self.textures[texture_id.0];
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
  /// destroy existing texture and replace it 
  /// with a new texture with a new size
  pub fn update_texture_size(&mut self, texture_id: RTextureId, pipeline_id: Option<RPipelineId>, width: u32, height: u32) {
    let old_texture = &mut self.textures[texture_id.0];

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
    self.textures[texture_id.0] = new_texture;

    // create new msaa texture
    let new_msaa = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("msaa-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: self.surface_format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });
    self.textures[texture_id.1].destroy();
    self.textures[texture_id.1] = new_msaa;

    // create new z-buffer texture
    let new_zbuffer = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("zbuffer-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Depth24Plus,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });
    self.textures[texture_id.2].destroy();
    self.textures[texture_id.2] = new_zbuffer;

    // update bind group
    if let Some(p_id) = pipeline_id {
      let new_bind_id = {
        let pipeline = &self.pipelines[p_id.0];
        let pipe = &pipeline.pipe;
        self.add_bind_group0(
          pipe,
          pipeline.max_obj_count,
          Some(texture_id),
          None,
          pipeline.vertex_type,
          pipeline.max_joints_count
        ) // TODO: handle resizing second texture
      };
      let pipeline = &mut self.pipelines[p_id.0];
      pipeline.bind_group0 = new_bind_id;
    }
  }
  /// create render pipeline
  /// - creates rendering process that object passes through
  /// - defines shaders + uniforms
  pub fn add_pipeline(&mut self, setup: RPipelineSetup) -> RPipelineId {
    let id: usize = self.pipelines.len();
    // define pipeline config
    let shader_mod = self.build_shader_module(&setup);
    let mut bind_group_container: Vec<&BindGroupLayout> = vec![];
    let bind_group0_layout = self.build_bind_group0_layout(&setup);
    bind_group_container.push(&bind_group0_layout);
    // build custom bind group layout
    let bind_group1_layout: BindGroupLayout;
    if setup.uniforms.len() > 0 {
      bind_group1_layout = self.build_bind_group1_layout(&setup);
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
    // build render pipeline
    let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("render-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader_mod,
        entry_point: Some(setup.vertex_fn),
        buffers: &[vertex_layout],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: Some(setup.fragment_fn),
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
      primitive: self.build_primitive_state(&setup),
      multiview: None,
      cache: None,
    });

    // build bind groups
    let bind_group0: RBindGroup = self.add_bind_group0(&pipeline, setup.max_obj_count, setup.texture1_id, setup.texture2_id, setup.vertex_type, setup.max_joints_count);
    let mut bind_group1: Option<RBindGroup> = None;
    if setup.uniforms.len() > 0 {
      bind_group1 = Some(self.add_bind_group1(&pipeline, setup.max_obj_count, setup.uniforms));
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
    RPipelineId(id)
  }
  /// part of add_pipeline system
  /// - selects shader to use with pipeline
  fn build_shader_module(&self, setup: &RPipelineSetup) -> ShaderModule {
    // translate shader
    let shader = match setup.shader {
      RShader::Texture => { include_str!("../embed_assets/base.wgsl") }
      RShader::Text => { include_str!("../embed_assets/text.wgsl") }
      RShader::FlatColor => { include_str!("../embed_assets/flat_color.wgsl") }
      RShader::RoundedRect => { include_str!("../embed_assets/rounded_rect.wgsl") }
      RShader::Custom(s) => { s }
    };
    // build render pipeline
    self.device.create_shader_module(ShaderModuleDescriptor {
      label: Some("shader-module"),
      source: ShaderSource::Wgsl(shader.into()),
    })
  }
  /// part of add_pipeline system
  /// - defines pre-defined bind_group(0) layout
  fn build_bind_group0_layout(&self, setup: &RPipelineSetup) -> BindGroupLayout {
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
      // gen f32 buffer
      BindGroupLayoutEntry {
        binding: 1,
        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: true,
          min_binding_size: None,
        },
        count: None,
      },
      // texture sampler
      BindGroupLayoutEntry {
        binding: 2,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
      },
      // texture 1
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
      // texture 2
      BindGroupLayoutEntry {
        binding: 4,
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
        binding: 5,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      });
    }
    self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("bind-group0-layout"),
      entries: &bind_group0_entries
    })
  }
  /// part of add_pipeline system
  /// - defines custom bind_group(1) layout
  fn build_bind_group1_layout(&self, setup: &RPipelineSetup) -> BindGroupLayout {
    let mut entries: Vec<BindGroupLayoutEntry> = Vec::new();
    // add bind group entries to layout
    for u in &setup.uniforms {
      let visibility = match u.visibility {
        RUniformVisibility::Vertex => ShaderStages::VERTEX,
        RUniformVisibility::Fragment => ShaderStages::FRAGMENT,
        RUniformVisibility::Both => ShaderStages::all(),
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
    self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("bind-group0-layout"),
      entries: &entries.as_slice()
    })
  }
  /// part of add_pipeline system
  /// - defines how render objects are configured
  fn build_primitive_state(&self, setup: &RPipelineSetup) -> PrimitiveState {
    // translate cullmode
    let cull_mode: Option<Face> = match setup.cull_mode {
      1 => Some(Face::Back),
      2 => Some(Face::Front),
      _ => None
    };
    // translate polygon mode
    let (polygon_mode, topology): (PolygonMode, PrimitiveTopology) = match setup.poly_mode {
      1 => (PolygonMode::Line, PrimitiveTopology::LineList),
      2 => (PolygonMode::Point, PrimitiveTopology::PointList),
      _ => (PolygonMode::Fill, PrimitiveTopology::TriangleList),
    };
    PrimitiveState {
      cull_mode,
      polygon_mode,
      topology,
      ..PrimitiveState::default()
    }
  }
  /// part of add_pipeline system
  /// - creates bind_group(0) for pre-defined uniforms
  fn add_bind_group0(
    &self, pipeline: &RenderPipeline,
    max_obj_count: usize,
    texture1: Option<RTextureId>,
    texture2: Option<RTextureId>,
    vertex_type: u8,
    max_joints: usize,
  ) -> RBindGroup {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    // create mvp buffer
    let mvp_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("mvp-uniform-buffer"),
      size: min_stride as u64 * max_obj_count as u64,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    // create general f32 buffer
    let gen_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("albedo-uniform-buffer"),
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
      texture1_view = self.textures[tx_id.0].create_view(&TextureViewDescriptor::default());
    } else {
      texture1_view = ftexture.create_view(&TextureViewDescriptor::default());
    }
    if let Some(tx_id) = texture2 {
      texture2_view = self.textures[tx_id.0].create_view(&TextureViewDescriptor::default());
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
    let gen_size = NonZeroU64::new(256); // 64 f32 values
    let mut bind_entries: Vec<BindGroupEntry> = vec![
      BindGroupEntry {
        binding: 0,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &mvp_buffer, offset: 0, size: mvp_size
        })
      },
      BindGroupEntry {
        binding: 1,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &gen_buffer, offset: 0, size: gen_size
        })
      },
      BindGroupEntry {
        binding: 2,
        resource: BindingResource::Sampler(&sampler)
      },
      BindGroupEntry {
        binding: 3,
        resource: BindingResource::TextureView(&texture1_view)
      },
      BindGroupEntry {
        binding: 4,
        resource: BindingResource::TextureView(&texture2_view)
      },
    ];
    // create joints matrix buffer
    let joints_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("joint-transforms-buffer"),
      size: (max_joints * 4 * 4 * 4) as u64, // 4x4 matrix of f32 values
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false
    });
    if vertex_type == RPipelineSetup::VERTEX_TYPE_ANIM {
      bind_entries.push(BindGroupEntry {
        binding: 5,
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
    let mut output_entries = vec![mvp_buffer, gen_buffer];
    if vertex_type == RPipelineSetup::VERTEX_TYPE_ANIM {
      output_entries.push(joints_buffer);
    }
    RBindGroup {
      base: bind_group,
      entries: output_entries
    }
  }
  /// part of add_pipeline system
  /// - creates bind_group(1) for custom uniforms
  fn add_bind_group1(
    &self,
    pipeline: &RenderPipeline,
    max_obj_count: usize,
    uniforms: Vec<RUniformSetup>,
  ) -> RBindGroup {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    let mut bind_entries: Vec<Buffer> = Vec::new();
    let mut bind_desc: Vec<BindGroupEntry> = Vec::new();
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
  /// shorthand for creating a texture of the same size as the window
  /// - also creates a default pipeline for overlaying text
  pub fn add_overlay_pipeline(&mut self) -> (RTextureId, RPipelineId) {
    // build full screen texture
    let texture_id = self.add_texture(self.config.width, self.config.height, None, true);
    // build render pipeline
    let pipeline_id = self.add_pipeline(RPipelineSetup {
      shader: RShader::Text,
      texture1_id: Some(texture_id),
      ..Default::default()
    });
    // build object
    let (rect_data, rect_i) = Primitives::rect_indexed(2.0, 2.0, 0.0);
    let _rect = Shape::new(self, pipeline_id, rect_data, Some(rect_i));
    // output fields
    (texture_id, pipeline_id)
  }
  /// shorthand for wiping a texture
  /// - shortcut for render_on_texture(...)
  pub fn clear_texture(&mut self, texture_id: RTextureId, clear_color: Option<[f64;4]>) {
    self.render_on_texture(&[], texture_id, clear_color);
  }
  /// add new render object to a pipeline
  pub fn add_object(&mut self, obj_data: RObjectSetup) -> RObjectId {
    let pipe = &mut self.pipelines[obj_data.pipeline_id.0];
    let id = pipe.objects.len();

    // create vertex buffer
    let vlen: usize;
    let v_buffer: Buffer;
    match obj_data.vertex_type {
      RObjectSetup::VERTEX_TYPE_ANIM => {
        vlen = obj_data.anim_vertex_data.len();
        v_buffer = self.device.create_buffer(&BufferDescriptor {
          label: Some("anim-vertex-buffer"),
          size: (std::mem::size_of::<RVertexAnim>() * vlen) as u64,
          usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
          mapped_at_creation: false
        });
        self.queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&obj_data.anim_vertex_data));
      }
      _ => {
        vlen = obj_data.vertex_data.len();
        v_buffer = self.device.create_buffer(&BufferDescriptor {
          label: Some("vertex-buffer"),
          size: (std::mem::size_of::<RVertex>() * vlen) as u64,
          usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
          mapped_at_creation: false
        });
        self.queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&obj_data.vertex_data));
      }
    }

    // create index buffer
    let mut index_buffer: Option<Buffer> = None;
    let ilen: usize = obj_data.indices.len();
    if ilen > 0 {
      let i_buffer = self.device.create_buffer(&BufferDescriptor {
        label: Some("index-buffer"),
        size: (std::mem::size_of::<u32>() * ilen) as u64,
        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        mapped_at_creation: false
      });
      self.queue.write_buffer(&i_buffer, 0, bytemuck::cast_slice(&obj_data.indices));
      index_buffer = Some(i_buffer);
    }

    // save to cache
    let obj = RObject {
      visible: true,
      v_buffer,
      v_count: vlen,
      pipe_index: id,
      index_buffer,
      index_count: ilen as u32,
      instances: 1,
    };
    pipe.objects.push(obj);
    let object_id = RObjectId(obj_data.pipeline_id.0, id);
    self.update_object(RObjectUpdate{ object_id, ..Default::default() });
    object_id
  }
  /// update existing render object attached to a pipeline
  pub fn update_object(&mut self, update: RObjectUpdate) {
    let mvp = self.create_mvp(&update);
    let buf = self.create_buf(&update);
    let pipe = &mut self.pipelines[update.object_id.0];
    let obj = &mut pipe.objects[update.object_id.1];
    obj.visible = update.visible;

    let stride = self.limits.min_uniform_buffer_offset_alignment;
    self.queue.write_buffer(
      &pipe.bind_group0.entries[0], 
      (stride * obj.pipe_index as u32) as u64, 
      bytemuck::cast_slice(&mvp)
    );
    self.queue.write_buffer(
      &pipe.bind_group0.entries[1], 
      (stride * obj.pipe_index as u32) as u64, 
      bytemuck::cast_slice(&buf.as_slice())
    );

    // merge animation matrices into single buffer
    if pipe.max_joints_count > 0 && update.anim_transforms.len() > 0 {
      let mut anim_buffer: Vec<f32> = Vec::new();
      for i in 0..pipe.max_joints_count {
        if i >= update.anim_transforms.len() {
          break;
        }
        // merge [f32; 16] arrays into single anim_buffer
        let a = update.anim_transforms[i];
        anim_buffer.extend_from_slice(&a);
      }
      self.queue.write_buffer(&pipe.bind_group0.entries[1], 0, bytemuck::cast_slice(&anim_buffer));
    }
    // update custom uniforms
    if update.uniforms.len() > 0 {
      if let Some(bind_group1) = &pipe.bind_group1 {
        for (i, uniform) in update.uniforms.iter().enumerate() {
          self.queue.write_buffer(
            &bind_group1.entries[i],
            (stride * obj.pipe_index as u32) as u64,
            *uniform
          );
        }
      }
    }
  }
  /// part of update_object process
  /// - creates MVP matrix
  fn create_mvp(&self, update: &RObjectUpdate) -> [f32; 48] {
    let cam = match update.camera {
      Some(c) => c,
      None => &self.default_cam
    };
    // model matrix
    let model_t = Mat4::translate(update.translate.x, update.translate.y, update.translate.z);
    let model_r = match update.rotate {
      RRotation::AxisAngle(axis, angle) => { Mat4::rotate(&axis, angle) }
      RRotation::Euler(x, y, z) => { Mat4::rotate_euler(x, y, z) }
    };
    let model_s = Mat4::scale(update.scale.x, update.scale.y, update.scale.z);
    let model = Mat4::multiply(&model_t, &Mat4::multiply(&model_s, &model_r));
    // view matrix
    let view_t = Mat4::translate(-cam.position.x, -cam.position.y, -cam.position.z);
    let view_r = Mat4::view_rot(&cam.position, &cam.look_at, &cam.up);
    let view = Mat4::multiply(&view_r, &view_t);
    // projection matrix
    let (w2, h2) = match cam.target_size {
      Some(s) => {
        let w = s.x / 2.0;
        let h = s.y / 2.0;
        (w, h)
      }
      None => {
        let w = (self.config.width / 2) as f32;
        let h = (self.config.height / 2) as f32;
        (w, h)
      }
    };
    let proj = match cam.cam_type {
      1 => Mat4::ortho(-w2, w2, -h2, h2, cam.near, cam.far),
      2 => Mat4::perspective(cam.fov_y, w2/h2, cam.near, cam.far),
      _ => Mat4::identity().as_col_major_array()
    };
    // merge together
    let mut mvp: [f32; 48] = [0.0; 48]; // 16 * 3 = 48
    for i in 0..48 {
      if i < 16 { mvp[i] = model[i]; }
      else if i < 32 { mvp[i] = view[i - 16]; }
      else { mvp[i] = proj[i - 32]; }
    }
    mvp
  }
  /// part of update_object process
  /// - creates general uniform buffer
  fn create_buf(&self, update: &RObjectUpdate) -> Vec<f32> {
    // note: max size is 64
    let mut buf: Vec<f32> = Vec::new();
    // albedo
    buf.append(&mut update.color.into());
    // rect size
    if let Some(rs) = update.rect_size {
      buf.append(&mut Vec::from(rs));
      buf.push(update.rect_radius);
    }
    buf
  }
  /// separate out pipeline implementation to resolve ownership issues
  /// - shared by render_on_texture and render_to_screen
  fn render_impl(
    &self,
    encoder: &mut CommandEncoder,
    target: TextureView,
    msaa_view: TextureView,
    zbuffer_view: TextureView,
    pipeline_ids: &[RPipelineId],
    clear_color: Option<[f64;4]>,
  ) {
    let mut clear_clr = self.clear_color;
    if let Some(c) = clear_color {
      clear_clr = Color { r:c[0], g:c[1], b:c[2], a:c[3] };
    }

    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
      label: Some("render-pass"),
      color_attachments: &[Some(RenderPassColorAttachment {
        view: &msaa_view,
        resolve_target: Some(&target),
        ops: Operations {
          load: LoadOp::Clear(clear_clr),
          store: StoreOp::Store,
        },
      })],
      depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
        view: &zbuffer_view,
        depth_ops: Some(Operations {
          load: LoadOp::Clear(1.0),
          store: StoreOp::Store
        }),
        stencil_ops: None,
      }),
      occlusion_query_set: None,
      timestamp_writes: None,
    });
    // add objects to render
    for p_id in pipeline_ids {
      let pipeline = &self.pipelines[p_id.0];
      for obj in &pipeline.objects {
        if !obj.visible { continue; }
        let stride = self.limits.min_uniform_buffer_offset_alignment * obj.pipe_index as u32;
        pass.set_pipeline(&pipeline.pipe);
        pass.set_vertex_buffer(0, obj.v_buffer.slice(..));
        pass.set_bind_group(0, &pipeline.bind_group0.base, &[stride, stride]);
        if let Some(bind_group1) = &pipeline.bind_group1 {
          pass.set_bind_group(1, &bind_group1.base, &[stride]);
        }
        if let Some(i_buffer) = &obj.index_buffer {
          pass.set_index_buffer(i_buffer.slice(..), IndexFormat::Uint32);
          pass.draw_indexed(0..obj.index_count, 0, 0..obj.instances);
        } else {
          pass.draw(0..(obj.v_count as u32), 0..obj.instances);
        }
      }
    }
  }
  /// runs rendering pipeline(s) on target texture
  pub fn render_on_texture(&mut self, pipeline_ids: &[RPipelineId], target_id: RTextureId, clear_color: Option<[f64;4]>) {
    let tx = &self.textures[target_id.0];
    let tx_msaa = &self.textures[target_id.1];
    let tx_zbuffer = &self.textures[target_id.2];
    let tvd = TextureViewDescriptor::default();
    let target = tx.create_view(&tvd);
    let view = tx_msaa.create_view(&tvd);
    let zbuffer_view = tx_zbuffer.create_view(&tvd);
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-texture-encoder") }
    );
    self.render_impl(&mut encoder, target, view, zbuffer_view, pipeline_ids, clear_color);
    self.queue.submit(std::iter::once(encoder.finish()));
  }
  /// overlays text string on target texture
  /// - non-text will be overwritten with transparent pixels
  /// - to overlay on a different texture, it must be fed into texture 2
  pub fn render_str_on_blank_texture(
    &mut self,
    texture_id: RTextureId,
    input: &str,
    size:f32,
    color: [u8; 4],
    base_point: [f32; 2],
    spacing: f32
  ) {
    let texture = &mut self.textures[texture_id.0];
    // fetch font data
    if self.font_cache.is_none() { 
      let font = include_bytes!("../embed_assets/NotoSerifCHB.ttf");
      self.font_cache = Some(font.to_vec());
    }
    let font_data = self.font_cache.as_ref().unwrap();
    // draw string onto existing texture
    match draw_str_on_texture(
      &self.queue,
      texture,
      font_data,
      input,
      size,
      color,
      base_point,
      spacing,
    ) {
      Ok(()) => (),
      Err(e) => {
        println!("Error while drawing str: \"{}\" - {:?}", input, e);
      }
    };
  }
  /// runs rendering pipeline(s) on window surface
  /// - finalizes queue and submits it
  /// - draws to window surface
  pub fn render_to_screen(&mut self, pipeline_ids: &Vec<RPipelineId>) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let tvd = TextureViewDescriptor::default();
    let target = output.texture.create_view(&tvd);
    let msaa_view = self.msaa.create_view(&tvd);
    let zbuffer_view = self.zbuffer.create_view(&tvd);
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-encoder") }
    );
    self.render_impl(&mut encoder, target, msaa_view, zbuffer_view, pipeline_ids, None);
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
  }
  /// manually free memory used by renderer
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