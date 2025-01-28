use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::num::NonZeroU64;

use winit::window::Window;
use image::{ImageReader, DynamicImage, GenericImageView};
use wgpu::*;

use super::*;

// --- --- --- --- --- --- --- --- --- --- //
// -- -- Primary Renderer Interface -- --- //
// --- --- --- --- --- --- --- --- --- --- //
#[derive(Debug)]
pub struct Renderer<'a> {
  // wgpu related config
  device: wgpu::Device,
  queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  pub limits: wgpu::Limits,
  // screen render components
  screen: wgpu::Surface<'a>,
  screen_format: wgpu::TextureFormat,
  screen_msaa: wgpu::Texture,
  screen_zbuffer: wgpu::Texture,
  // custom setup
  default_cam: RCamera,
  pub clear_color: wgpu::Color,
  pub pipelines: Vec<RPipeline>,
  pub textures: Vec<wgpu::Texture>,
  pub objects: Vec<RObject>,
  font_cache: Vec<Vec<u8>>,
  str_placements: Vec<StringPlacement>,
}
impl<'a> Renderer<'a> {
  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- --- -- -- Setup --- --- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

  /// Create wgpu renderer instance attached to window
  /// - Creating some of the wgpu types requires async code
  pub async fn new(window: Arc<Window>) -> Self {
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

    let font1 = include_bytes!("../embed_assets/NotoSerifCHB.ttf");
    let font2 = include_bytes!("../embed_assets/NotoSansCB.ttf");
    let font_cache = vec!(font1.to_vec(), font2.to_vec());

    Self {
      device,
      queue,
      config,
      limits: Limits::default(),
      screen: surface,
      screen_format: surface_format,
      screen_msaa: msaa,
      screen_zbuffer: zbuffer,
      default_cam: RCamera::default(),
      pipelines: Vec::new(),
      textures: Vec::new(),
      objects: Vec::new(),
      clear_color: Color { r: 0.002, g: 0.002, b: 0.008, a: 1.0 },
      font_cache,
      str_placements: Vec::new(),
    }
  }
  /// Destroys surface screen texture and remakes it
  /// - Also destroys and remakes MSAA and z-buffer textures
  pub fn resize_screen(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      self.config.width = width;
      self.config.height = height;
      self.screen.configure(&self.device, &self.config);

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
        format: self.screen_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[]
      });
      self.screen_msaa.destroy();
      self.screen_msaa = msaa;

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
      self.screen_zbuffer.destroy();
      self.screen_zbuffer = zbuffer;
    }
  }
  /// update default clear_color
  pub fn set_clear_color(&mut self, color: RColor) {
    self.clear_color.r = color.r as f64;
    self.clear_color.g = color.g as f64;
    self.clear_color.b = color.b as f64;
    self.clear_color.a = color.a as f64;
  }
  /// manually free memory used by renderer
  pub fn destroy(&mut self, destroy_renderer: bool) {
    // destroy textures
    for tx in &mut self.textures {
      tx.destroy();
    }
    self.textures.clear();
    // destroy objects
    for obj in &mut self.objects {
      obj.v_buffer.destroy();
      if let Some(ibf) = &mut obj.index_buffer {
        ibf.destroy();
      }
      for buf in &mut obj.buffers0 {
        buf.destroy();
      }
    }
    self.objects.clear();
    // destroy pipeline buffers
    self.pipelines.clear();
    // destroy device
    if destroy_renderer {
      self.screen_msaa.destroy();
      self.screen_zbuffer.destroy();
      self.device.destroy();
    }
  }

  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- -- Texture Functions -- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

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
    let tex_format = if use_device_format { self.screen_format } 
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
      format: self.screen_format,
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
    RTextureId { base: id, msaa: id + 1, zbuffer: id + 2 }
  }
  /// copies image (from path) onto texture
  pub fn update_texture(&mut self, texture_id: RTextureId, texture_path: &Path) {
    let texture = &mut self.textures[texture_id.base];
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
  /// destroy existing texture and replace it with a new texture with a new size
  pub fn resize_texture(&mut self, texture_id: RTextureId, obj_id: RObjectId, width: u32, height: u32) {
    let old_texture = &mut self.textures[texture_id.base];

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
    self.textures[texture_id.base] = new_texture;

    // create new msaa texture
    let new_msaa = self.device.create_texture(&TextureDescriptor {
      label: Some("msaa-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: self.screen_format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });
    self.textures[texture_id.msaa].destroy();
    self.textures[texture_id.msaa] = new_msaa;

    // create new z-buffer texture
    let new_zbuffer = self.device.create_texture(&TextureDescriptor {
      label: Some("zbuffer-texture"),
      size: texture_size,
      sample_count: 4,
      mip_level_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Depth24Plus,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[]
    });
    self.textures[texture_id.zbuffer].destroy();
    self.textures[texture_id.zbuffer] = new_zbuffer;

    // update bind group
    let pipe_id = self.objects[obj_id.0].pipe_id;
    let tx1_id = self.objects[obj_id.0].texture1;
    let tx2_id = self.objects[obj_id.0].texture2;
    let max_joints = self.objects[obj_id.0].max_joints;
    let pipe = &self.pipelines[obj_id.0];
    let (bind_group0, buffers0) = self.add_bind_group0(pipe_id, tx1_id, tx2_id, pipe.has_animations, max_joints);
    self.objects[obj_id.0].bind_group0 = bind_group0;
    self.objects[obj_id.0].buffers0 = buffers0;
  }

  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- --- Render Pipeline --- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

  // selects shader to use with pipeline
  fn build_shader_module(&self, shader_op: &RShader) -> wgpu::ShaderModule {
    // translate shader
    let shader = match shader_op {
      RShader::Texture => { include_str!("../embed_assets/base.wgsl") }
      RShader::Text => { include_str!("../embed_assets/text.wgsl") }
      RShader::FlatColor => { include_str!("../embed_assets/flat_color.wgsl") }
      RShader::Custom(s) => { s }
    };
    // build render pipeline
    self.device.create_shader_module(ShaderModuleDescriptor {
      label: Some("shader-module"),
      source: ShaderSource::Wgsl(shader.into()),
    })
  }
  // defines pre-defined bind_group(0) layout
  fn build_bind_group0_layout(&self) -> wgpu::BindGroupLayout {
    let bind_group0_entries: Vec<BindGroupLayoutEntry> = vec![
      // mvp matrix
      BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: false,
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
          has_dynamic_offset: false,
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
    self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("bind-group0-layout"),
      entries: &bind_group0_entries
    })
  }
  // defines custom bind_group(1) layout
  fn build_bind_group1_layout(&self) -> wgpu::BindGroupLayout {
    todo!()
  }
  // defines how render objects are configured
  fn build_primitive_state(&self, cull_mode_op: &RCullMode, poly_mode: &RPolyMode) -> wgpu::PrimitiveState {
    // translate cullmode
    let cull_mode: Option<Face> = match cull_mode_op {
      RCullMode::Back => Some(Face::Back),
      RCullMode::Front => Some(Face::Front),
      _ => None
    };
    // translate polygon mode
    let (polygon_mode, topology): (PolygonMode, PrimitiveTopology) = match poly_mode {
      RPolyMode::Line => (PolygonMode::Line, PrimitiveTopology::LineList),
      RPolyMode::Point => (PolygonMode::Point, PrimitiveTopology::PointList),
      _ => (PolygonMode::Fill, PrimitiveTopology::TriangleList),
    };
    PrimitiveState {
      cull_mode,
      polygon_mode,
      topology,
      ..PrimitiveState::default()
    }
  }
  /// create render pipeline
  /// - creates rendering process that object passes through
  /// - defines shaders + uniforms
  pub fn add_pipeline(&mut self, setup: RPipelineSetup) -> RPipelineId {
    let id: usize = self.pipelines.len();

    // define pipeline config
    let shader_mod = self.build_shader_module(&setup.shader);
    let bind_group0_layout = self.build_bind_group0_layout();
    // let bind_group1_layout = self.build_custom_bind_group_layout();
    let mut bind_group_container: Vec<&BindGroupLayout> = vec![];
    bind_group_container.push(&bind_group0_layout);

    let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("pipeline-layout"),
      bind_group_layouts: bind_group_container.as_slice(),
      push_constant_ranges: &[]
    });
    // switch between static/dynamic vertex layouts
    let vertex_attr_static = vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
    let vertex_attr_anim = vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Uint32x4, 4 => Float32x4];
    let vertex_layout = if setup.has_animations {
      VertexBufferLayout {
        array_stride: std::mem::size_of::<RVertexAnim>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_anim,
      }
    } else {
      VertexBufferLayout {
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
          format: self.screen_format,
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
      primitive: self.build_primitive_state(&setup.cull_mode, &setup.poly_mode),
      multiview: None,
      cache: None,
    });

    // add to cache
    let pipe = RPipeline {
      pipe: pipeline,
      obj_indices: Vec::new(),
      has_animations: setup.has_animations,
    };
    self.pipelines.push(pipe);
    RPipelineId(id)
  }

  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- -- -- Render Object --- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

  // create bind_group that matches bind_group(0) layout
  fn add_bind_group0(
    &self,
    pipeline: RPipelineId,
    texture1: Option<RTextureId>,
    texture2: Option<RTextureId>,
    has_animations: bool,
    max_joints: usize,
  ) -> (wgpu::BindGroup, Vec<wgpu::Buffer>) {
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    // create mvp buffer
    let mvp_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("mvp-uniform-buffer"),
      size: min_stride as u64,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    // create general f32 buffer
    let gen_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("albedo-uniform-buffer"),
      size: min_stride as u64,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    // import textures
    let texture1_view: TextureView;
    let texture2_view: TextureView;
    // create placeholder texture
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
      texture1_view = self.textures[tx_id.base].create_view(&TextureViewDescriptor::default());
    } else {
      texture1_view = ftexture.create_view(&TextureViewDescriptor::default());
    }
    if let Some(tx_id) = texture2 {
      texture2_view = self.textures[tx_id.base].create_view(&TextureViewDescriptor::default());
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
    let mut bind_entries: Vec<BindGroupEntry> = vec![
      BindGroupEntry {
        binding: 0,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &mvp_buffer, offset: 0, size: None
        })
      },
      BindGroupEntry {
        binding: 1,
        resource: BindingResource::Buffer(BufferBinding {
          buffer: &gen_buffer, offset: 0, size: None
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
    if has_animations {
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
      layout: &self.pipelines[pipeline.0].pipe.get_bind_group_layout(0),
      entries: &bind_entries
    });

    // create output
    let mut output_entries = vec![mvp_buffer, gen_buffer];
    if has_animations {
      output_entries.push(joints_buffer);
    }
    (bind_group, output_entries)
  }
  // create bind_group that matches bind_group(1) layout
  fn add_bind_group1(&self) -> (wgpu::BindGroup, Vec<wgpu::Buffer>) {
    todo!()
  }
  /// add new render object to a pipeline
  pub fn add_object(&mut self, setup: RObjectSetup) -> RObjectId {
    let pipe = &self.pipelines[setup.pipeline_id.0];
    let id = self.objects.len();

    // create vertex buffer
    let vlen: usize;
    let v_buffer: Buffer;
    if pipe.has_animations {
      vlen = setup.anim_vertex_data.len();
      v_buffer = self.device.create_buffer(&BufferDescriptor {
        label: Some("anim-vertex-buffer"),
        size: (std::mem::size_of::<RVertexAnim>() * vlen) as u64,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false
      });
      self.queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&setup.anim_vertex_data));
    } else {
      vlen = setup.vertex_data.len();
      v_buffer = self.device.create_buffer(&BufferDescriptor {
        label: Some("vertex-buffer"),
        size: (std::mem::size_of::<RVertex>() * vlen) as u64,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false
      });
      self.queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&setup.vertex_data));
    }

    // create index buffer
    let mut index_buffer: Option<Buffer> = None;
    let ilen: usize = setup.indices.len();
    if ilen > 0 {
      let i_buffer = self.device.create_buffer(&BufferDescriptor {
        label: Some("index-buffer"),
        size: (std::mem::size_of::<u32>() * ilen) as u64,
        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        mapped_at_creation: false
      });
      self.queue.write_buffer(&i_buffer, 0, bytemuck::cast_slice(&setup.indices));
      index_buffer = Some(i_buffer);
    }

    // create bind group 0
    let (bind_group0, buffers0) = self.add_bind_group0(setup.pipeline_id, setup.texture1_id, setup.texture2_id, pipe.has_animations, setup.max_joints);

    // save to cache
    let obj = RObject {
      visible: true,
      pipe_id: setup.pipeline_id,
      v_buffer,
      v_count: vlen,
      index_buffer,
      index_count: ilen as u32,
      instances: 1,
      bind_group0,
      buffers0,
      texture1: setup.texture1_id,
      texture2: setup.texture2_id,
      max_joints: setup.max_joints,
    };
    self.objects.push(obj);
    self.pipelines[setup.pipeline_id.0].obj_indices.push(id);
    let object_id = RObjectId(id);
    self.update_object(object_id, RObjectUpdate{ ..Default::default() });
    object_id
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
  /// update existing render object attached to a pipeline
  pub fn update_object(&mut self, obj: RObjectId, update: RObjectUpdate) {
    let mvp = self.create_mvp(&update);
    let buf = update.gen_buf;
    let obj = &mut self.objects[obj.0];
    obj.visible = update.visible;

    // let stride = self.limits.min_uniform_buffer_offset_alignment;
    self.queue.write_buffer(&obj.buffers0[0], 0, bytemuck::cast_slice(&mvp));
    self.queue.write_buffer(&obj.buffers0[1], 0, bytemuck::cast_slice(&buf.as_slice()));

    // merge animation matrices into single buffer
    if obj.max_joints > 0 && update.anim_transforms.len() > 0 {
      let mut anim_buffer: Vec<f32> = Vec::new();
      for i in 0..obj.max_joints {
        if i >= update.anim_transforms.len() {
          break;
        }
        // merge [f32; 16] arrays into single anim_buffer
        let a = update.anim_transforms[i];
        anim_buffer.extend_from_slice(&a);
      }
      self.queue.write_buffer(&obj.buffers0[1], 0, bytemuck::cast_slice(&anim_buffer));
    }
    // update custom uniforms
    // if update.uniforms.len() > 0 {
    //   if let Some(bind_group1) = &pipe.bind_group1 {
    //     for (i, uniform) in update.uniforms.iter().enumerate() {
    //       self.queue.write_buffer(
    //         &bind_group1.entries[i],
    //         (stride * obj.pipe_index as u32) as u64,
    //         *uniform
    //       );
    //     }
    //   }
    // }
  }

  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- -- Update Operations -- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

  /// separate out pipeline implementation to resolve ownership issues
  /// - shared by render_on_texture and render_to_screen
  fn render_impl(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    target: wgpu::TextureView,
    msaa_view: wgpu::TextureView,
    zbuffer_view: wgpu::TextureView,
    pipeline_ids: &[RPipelineId],
    clear_color: Option<[f64; 4]>,
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
      for i in 0..pipeline.obj_indices.len() {
        let obj_id = pipeline.obj_indices[i];
        let obj = &self.objects[obj_id];
        if !obj.visible { continue; }
        pass.set_pipeline(&pipeline.pipe);
        pass.set_vertex_buffer(0, obj.v_buffer.slice(..));
        pass.set_bind_group(0, &obj.bind_group0, &[]);
        // if let Some(bind_group1) = &obj.bind_group1 {
        //   pass.set_bind_group(1, &bind_group1.base, &[stride]);
        // }
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
    let tx = &self.textures[target_id.base];
    let tx_msaa = &self.textures[target_id.msaa];
    let tx_zbuffer = &self.textures[target_id.zbuffer];
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
  /// runs rendering pipeline(s) on window surface
  /// - finalizes queue and submits it
  /// - draws to window surface
  pub fn render_to_screen(&mut self, pipeline_ids: &[RPipelineId]) -> Result<(), wgpu::SurfaceError> {
    let output = self.screen.get_current_texture()?;
    let tvd = TextureViewDescriptor::default();
    let target = output.texture.create_view(&tvd);
    let msaa_view = self.screen_msaa.create_view(&tvd);
    let zbuffer_view = self.screen_zbuffer.create_view(&tvd);
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor { label: Some("render-encoder") }
    );
    self.render_impl(&mut encoder, target, msaa_view, zbuffer_view, pipeline_ids, None);
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
  }

  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- --- Text Rendering  --- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

  /// shortcut to make pipeline that creates an overlay on the screen
  /// and draws text to it
  pub fn add_overlay_pipe(&mut self) -> (RPipelineId, RObjectId, RTextureId) {
    // build full screen texture
    let texture_id = self.add_texture(self.config.width, self.config.height, None, false);
    // build render pipeline
    let pipeline_id = self.add_pipeline(RPipelineSetup {
      shader: RShader::Text,
      ..Default::default()
    });
    // build object
    let (rect_data, rect_i) = Primitives::rect_indexed(2.0, 2.0, 0.0);
    let rect = self.add_object(RObjectSetup {
      pipeline_id,
      vertex_data: rect_data,
      indices: rect_i,
      texture1_id: Some(texture_id),
      ..Default::default()
    });
    (pipeline_id, rect, texture_id)
  }
  /// load new font data into font_cache
  pub fn load_font(&mut self, font_path: &str) -> Result<usize, std::io::Error> {
    match fs::read(font_path) {
      Ok(f) => {
        let idx = self.font_cache.len();
        self.font_cache.push(f);
        Ok(idx)
      }
      Err(e) => {
        println!("Err: Could not open font file");
        Err(e)
      }
    }
  }
  /// get string bounding box, origin set where text would begin
  pub fn measure_str_size(&self, font_idx: usize, text: &str, size: f32) -> StringRect {
    let empty = StringRect {
      width: 0.0,
      max_y: 0.0,
      min_y: 0.0,
    };
    if self.font_cache.len() <= font_idx {
      return empty;
    }
    match measure_str_size(&self.font_cache[font_idx], text, size) {
      Ok(x) => x,
      Err(_) => empty
    }
  }
  /// overwrite target texture with string image
  /// - non-text will be overwritten with transparent pixels
  /// - to overlay on a different texture, it must be fed into texture 2
  pub fn redraw_texture_with_str(
    &mut self,
    mut font_idx: usize,
    texture_id: RTextureId,
    input: &str,
    size:f32,
    color: RColor,
    base_point: Vec2,
    spacing: f32
  ) {
    let texture = &mut self.textures[texture_id.base];
    // fetch font data
    if self.font_cache.len() <= font_idx { 
      font_idx = 0;
    }
    // draw string onto existing texture
    match draw_str_on_texture(
      &self.queue,
      texture,
      &self.font_cache[font_idx],
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
  /// add text to overlay
  pub fn queue_overlay_text(&mut self, placement: StringPlacement) {
    self.str_placements.push(placement);
  }
  /// clear overlay text queue
  pub fn clear_overlay_queue(&mut self) {
    self.str_placements.clear();
  }
  /// overwrite target texture with string image
  /// - non-text will be overwritten with transparent pixels
  pub fn redraw_texture_with_queue(&mut self, mut font_idx: usize, texture_id: RTextureId) {
    let texture = &mut self.textures[texture_id.base];
    // fetch font data
    if self.font_cache.len() <= font_idx { 
      font_idx = 0;
    }
    // draw queue onto existing textures
    match draw_full_text_texture(
      &self.queue, texture, &self.font_cache[font_idx], &self.str_placements
    ) {
      Ok(()) => (),
      Err(e) => {
        println!("Error while drawing str queue - {:?}", e);
      }
    }
    // clear queue
    self.str_placements.clear();
  }

  // --- --- --- --- --- --- --- --- --- --- //
  // --- --- --- -- -- SDF -- -- --- --- --- //
  // --- --- --- --- --- --- --- --- --- --- //

  /// add rectangle to render to for SDF
  fn add_sdf_render_obj(&mut self, pipe_id: RPipelineId) -> RObjectId {
    let pipe = &self.pipelines[pipe_id.0];
    let id = self.objects.len();
    // create rect
    let (rect_data, rect_i) = Primitives::rect_indexed(2.0, 2.0, 0.0);
    // create vertex buffer
    let vlen: usize;
    let v_buffer: Buffer;
    vlen = rect_data.len();
    v_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("vertex-buffer"),
      size: (std::mem::size_of::<RVertex>() * vlen) as u64,
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
      mapped_at_creation: false
    });
    self.queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&rect_data));

    // create index buffer
    let ilen: usize = rect_i.len();
    let i_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("index-buffer"),
      size: (std::mem::size_of::<u32>() * ilen) as u64,
      usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
      mapped_at_creation: false
    });
    self.queue.write_buffer(&i_buffer, 0, bytemuck::cast_slice(&rect_i));
    let index_buffer = Some(i_buffer);

    // build bind group + buffer
    let min_stride = self.limits.min_uniform_buffer_offset_alignment;
    // create system data buffer
    let sys_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("sys-uniform-buffer"),
      size: min_stride as u64,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    // create object data buffer
    let obj_buffer = self.device.create_buffer(&BufferDescriptor {
      label: Some("obj-uniform-buffer"),
      size: min_stride as u64 * 100,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
      label: Some("bind-group-0"),
      layout: &pipe.pipe.get_bind_group_layout(0),
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &sys_buffer, offset: 0, size: None
          })
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &obj_buffer, offset: 0, size: None
          })
        },
      ]
    });

    // save to cache
    let obj = RObject {
      visible: true,
      pipe_id,
      v_buffer,
      v_count: vlen,
      index_buffer,
      index_count: ilen as u32,
      instances: 1,
      bind_group0: bind_group,
      buffers0: vec![sys_buffer, obj_buffer],
      texture1: None,
      texture2: None,
      max_joints: 0,
    };
    self.objects.push(obj);
    self.pipelines[pipe_id.0].obj_indices.push(id);
    RObjectId(id)
  }
  /// generate special pipeline for rendering SDF objects
  pub fn add_sdf_pipeline(&mut self) -> RPipelineId {
    let id: usize = self.pipelines.len();
    // build shader module
    let shader_mod = self.device.create_shader_module(ShaderModuleDescriptor {
      label: Some("shader-module"),
      source: ShaderSource::Wgsl(include_str!("../embed_assets/sdf.wgsl").into()),
    });
    // bind_group -> static singular layout
    let bind_group_layout_0 = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("bind-group0-layout"),
      entries: &[
        // system data
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        // object data
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ]
    });
    let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("pipeline-layout"),
      bind_group_layouts: &[&bind_group_layout_0],
      push_constant_ranges: &[]
    });
    let vertex_layout = VertexBufferLayout {
      array_stride: std::mem::size_of::<RVertex>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
    };
    // build pipeline
    let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("render-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader_mod,
        entry_point: Some("vertexMain"),
        buffers: &[vertex_layout],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: Some("fragmentMain"),
        targets: &[Some(ColorTargetState{
          format: self.screen_format,
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
      primitive: self.build_primitive_state(&RCullMode::None, &RPolyMode::Fill),
      multiview: None,
      cache: None,
    });
  
    let pipe = RPipeline {
      pipe: pipeline,
      obj_indices: Vec::new(),
      has_animations: false,
    };
    self.pipelines.push(pipe);
    let pipeline_id = RPipelineId(id);

    // create screen object
    let _rect = self.add_sdf_render_obj(pipeline_id);
    pipeline_id
  }
  /// push the finalized position of all SDF objects
  pub fn update_sdf_objects(
    &mut self,
    pipeline_id: RPipelineId,
    screen_size: Vec2,
    m_pos: Vec2,
    shadow_intensity: f32,
    light_pos: Vec2,
    objects: &Vec<RSDFObject>
  ) {
    let pipe = &self.pipelines[pipeline_id.0];
    let robj = &self.objects[pipe.obj_indices[0]];
    
    let obj_count = if objects.len() < 100 { objects.len() as u32 } else { 100 };
    let sys = SysData {
      screen: screen_size,
      mouse_pos: m_pos,
      obj_count,
      shadow_intensity,
      light_pos,
    };
    let mut objs: Vec<RSDFObjectC> = Vec::new();
    for o in objects {
      objs.push(RSDFObjectC::from(o));
    }
    // let stride = self.limits.min_uniform_buffer_offset_alignment;
    self.queue.write_buffer(&robj.buffers0[0], 0, bytemuck::cast_slice(&[sys]));
    self.queue.write_buffer(&robj.buffers0[1], 0, bytemuck::cast_slice(&objs.as_slice()));
  }
}