use bytemuck::{Pod, Zeroable};
use wgpu::{
  AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
  BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
  BufferDescriptor, BufferUsages, Device, Extent3d, Face, FilterMode, Limits, PolygonMode, PrimitiveState,
  PrimitiveTopology, RenderPipeline, SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor,
  ShaderSource, ShaderStages, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
  TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension
};

use crate::{vec2f, vec3f};
use crate::utils::{ Vec2, Vec3, Mat4 };

// --- --- --- --- --- --- --- --- --- //
// --- --- - HELPER STRUCTS -- --- --- //
// --- --- --- --- --- --- --- --- --- //

// helper for updating render object
#[derive(Debug, Clone)]
pub enum RenderRotation {
  AxisAngle(Vec3, f32),
  Euler(f32, f32, f32)
}

// helper for defining camera/view matrix
#[derive(Debug, Clone)]
pub struct RenderCamera {
  pub cam_type: u8,
  pub position: Vec3,
  pub look_at: Vec3,
  pub up: Vec3,
  pub fov_y: f32,
  pub near: f32,
  pub far: f32,
  pub target_size: Vec2,
}
impl Default for RenderCamera {
  fn default() -> Self {
    Self {
      cam_type: RenderCamera::ORTHOGRAPHIC,
      position: vec3f!(0.0, 0.0, 100.0),
      look_at: vec3f!(0.0, 0.0, 0.0),
      up: vec3f!(0.0, 1.0, 0.0),
      fov_y: 0.0,
      near: 0.0,
      far: 1000.0,
      target_size: vec2f!(100.0, 100.0),
    }
  }
}
impl RenderCamera {
  const ORTHOGRAPHIC: u8 = 1;
  const PERSPECTIVE: u8 = 2;
  pub fn new_ortho(near: f32, far: f32, target_size: Vec2) -> Self {
    Self {
      cam_type: RenderCamera::ORTHOGRAPHIC,
      position: vec3f!(0.0, 0.0, 100.0),
      look_at: vec3f!(0.0, 0.0, 0.0),
      up: vec3f!(0.0, 1.0, 0.0),
      fov_y: 0.0,
      near,
      far,
      target_size,
    }
  }
  pub fn new_persp(fov_y: f32, near: f32, far: f32, target_size: Vec2) -> Self {
    Self {
      cam_type: RenderCamera::PERSPECTIVE,
      position: vec3f!(0.0, 0.0, 1.0),
      look_at: vec3f!(0.0, 0.0, 0.0),
      up: vec3f!(0.0, 1.0, 0.0),
      fov_y,
      near,
      far,
      target_size,
    }
  }
}

// color helper (for passing into uniform)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RenderColor {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}
impl Into<Vec<f32>> for RenderColor {
  fn into(self) -> Vec<f32> {
    vec![self.r, self.g, self.b, self.a]
  }
}
impl Into<[f32; 4]> for RenderColor {
  fn into(self) -> [f32; 4] {
    [self.r, self.g, self.b, self.a]
  }
}
impl Into<[u8; 4]> for RenderColor {
  fn into(self) -> [u8; 4] {
    let r = f32::round(self.r * 255.0);
    let g = f32::round(self.g * 255.0);
    let b = f32::round(self.b * 255.0);
    let a = f32::round(self.a * 255.0);
    [r as u8, g as u8, b as u8, a as u8]
  }
}
impl Into<wgpu::Color> for RenderColor {
  fn into(self) -> wgpu::Color {
    wgpu::Color {
      r: self.r as f64,
      g: self.g as f64,
      b: self.b as f64,
      a: self.a as f64
    }
  }
}
impl RenderColor {
  pub fn rgba_pct(r: f32, g: f32, b: f32, a: f32) -> Self {
    Self { r, g, b, a }
  }
  pub fn rgb(r: u8, g: u8, b: u8) -> Self {
    Self {
      r: r as f32 / 255.0,
      g: g as f32 / 255.0,
      b: b as f32 / 255.0,
      a: 1.0 
    }
  }
  pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
    Self {
      r: r as f32 / 255.0,
      g: g as f32 / 255.0,
      b: b as f32 / 255.0,
      a: a as f32 / 255.0,
    }
  }
  pub fn hsv(h: f32, s: f32, v: f32) -> Self {
    let i = f32::floor(h * 6.0);
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let mut clr = RenderColor::WHITE;
    match i % 6.0 {
      0.0 => { clr.r = v; clr.g = t; clr.b = p; }
      1.0 => { clr.r = q; clr.g = v; clr.b = p; }
      2.0 => { clr.r = p; clr.g = v; clr.b = t; }
      3.0 => { clr.r = p; clr.g = q; clr.b = v; }
      4.0 => { clr.r = t; clr.g = p; clr.b = v; }
      5.0 => { clr.r = v; clr.g = p; clr.b = q; }
      _ => ()
    }
    clr
  }
  // pre-defined colors
  pub const TRANSPARENT: Self = Self {
    r: 0.0, g: 0.0, b: 0.0, a: 0.0,
  };
  pub const BLACK: Self = Self {
    r: 0.0, g: 0.0, b: 0.0, a: 1.0,
  };
  pub const GRAY: Self = Self {
    r: 0.5, g: 0.5, b: 0.5, a: 1.0,
  };
  pub const WHITE: Self = Self {
    r: 1.0, g: 1.0, b: 1.0, a: 1.0,
  };
  pub const RED: Self = Self {
    r: 1.0, g: 0.0, b: 0.0, a: 1.0,
  };
  pub const GREEN: Self = Self {
    r: 0.0, g: 1.0, b: 0.0, a: 1.0,
  };
  pub const BLUE: Self = Self {
    r: 0.0, g: 0.0, b: 1.0, a: 1.0,
  };
  pub const YELLOW: Self = Self {
    r: 1.0, g: 1.0, b: 0.0, a: 1.0,
  };
  pub const CYAN: Self = Self {
    r: 0.0, g: 1.0, b: 1.0, a: 1.0,
  };
  pub const MAGENTA: Self = Self {
    r: 1.0, g: 0.0, b: 1.0, a: 1.0,
  };
  pub const ORANGE: Self = Self {
    r: 1.0, g: 0.5, b: 0.0, a: 1.0,
  };
  pub const PURPLE: Self = Self {
    r: 0.5, g: 0.0, b: 1.0, a: 1.0,
  };
}

// helper for defining object updates
#[derive(Debug)]
pub struct RenderObjectUpdate<'a> {
  pub translate: Vec3,
  pub rotate: RenderRotation,
  pub scale: Vec3,
  pub visible: bool,
  pub camera: Option<&'a RenderCamera>,
  pub gen_buf: [f32; 64],
  pub uniforms: Vec<&'a [u8]>,
  pub anim_transforms: Vec<[f32; 16]>,
}
impl Default for RenderObjectUpdate<'_> {
  fn default() -> Self {
    RenderObjectUpdate {
      translate: vec3f!(0.0, 0.0, 0.0),
      rotate: RenderRotation::AxisAngle(vec3f!(0.0, 0.0, 1.0), 0.0),
      scale: vec3f!(1.0, 1.0, 1.0),
      visible: true,
      camera: None,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
      gen_buf: [0.0; 64],
    }
  }
}
impl<'a> RenderObjectUpdate<'a> {
  pub fn with_position(mut self, pos: Vec3) -> Self {
    self.translate = pos;
    self
  }
  pub fn with_rotation(mut self, axis: Vec3, angle_deg: f32) -> Self {
    self.rotate = RenderRotation::AxisAngle(axis, angle_deg);
    self
  }
  pub fn with_euler_rotation(mut self, roll: f32, pitch: f32, yaw: f32) -> Self {
    self.rotate = RenderRotation::Euler(roll, pitch, yaw);
    self
  }
  pub fn with_scale(mut self, scale: Vec3) -> Self {
    self.scale = scale;
    self
  }
  pub fn with_camera(mut self, camera: &'a RenderCamera) -> Self {
    self.camera = Some(camera);
    self
  }
  pub fn with_color(mut self, color: RenderColor) -> Self {
    self.gen_buf[0] = color.r;
    self.gen_buf[1] = color.g;
    self.gen_buf[2] = color.b;
    self.gen_buf[3] = color.a;
    self
  }
  pub fn with_round_border(mut self, rect_size: Vec2, radius: f32) -> Self {
    self.gen_buf[4] = rect_size.x;
    self.gen_buf[5] = rect_size.y;
    self.gen_buf[6] = radius;
    self
  }
  pub fn with_uniforms(mut self, uniforms: Vec<&'a [u8]>) -> Self {
    self.uniforms = uniforms;
    self
  }
  pub fn with_anim(mut self, transforms: Vec<[f32; 16]>) -> Self {
    self.anim_transforms = transforms;
    self
  }
}

#[derive(Debug)]
pub struct RenderObject {
  pub visible: bool,
  // vertex data
  pub v_buffer: Buffer,
  pub v_count: usize,
  pub max_joints: usize,
  pub index_buffer: Option<Buffer>,
  pub index_count: u32,
  pub instances: u32,
  // render data
  pub bind_group0: wgpu::BindGroup,
  pub buffers0: Vec<wgpu::Buffer>,
  pub texture1: Option<Texture>,
  pub texture2: Option<Texture>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct RenderVertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub normal: [f32; 3],
}

#[derive(Debug)]
pub struct RenderObjectSetup<'a> {
  pub vertex_data: Vec<RenderVertex>,
  pub instances: u32,
  pub indices: Vec<u32>,
  pub texture1: Option<Texture>,
  pub texture2: Option<Texture>,
  pub max_joints: usize,
  pub camera: Option<&'a RenderCamera>,
}
impl Default for RenderObjectSetup<'_> {
  fn default() -> Self {
    Self {
      vertex_data: Vec::new(),
      indices: Vec::new(),
      instances: 1,
      texture1: None,
      texture2: None,
      max_joints: 0,
      camera: None,
    }
  }
}

// --- --- --- --- --- --- --- --- --- //
// --- --- - PIPELINE HELPER - --- --- //
// --- --- --- --- --- --- --- --- --- //

#[derive(Debug, Clone, Default)]
pub enum ShaderType<'a> {
  #[default]
  Default,
  FlatColor,
  Overlay,
  Custom(&'a str)
}

pub fn build_shader_module(device: &Device, shader_type: ShaderType) -> ShaderModule {
  // translate shader
  let shader = match shader_type {
    ShaderType::FlatColor => include_str!("shaders/flat_color.wgsl"),
    ShaderType::Overlay => include_str!("shaders/overlay.wgsl"),
    ShaderType::Custom(s) => s,
    _ => include_str!("shaders/default.wgsl")
  };
  // build render pipeline
  device.create_shader_module(ShaderModuleDescriptor {
    label: Some("shader-module"),
    source: ShaderSource::Wgsl(shader.into()),
  })
}

pub fn build_default_bind_group_layout(device: &Device) -> BindGroupLayout {
  let bind_group_entries: Vec<BindGroupLayoutEntry> = vec![
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
  device.create_bind_group_layout(&BindGroupLayoutDescriptor {
    label: Some("bind-group0-layout"),
    entries: &bind_group_entries,
  })
}

pub fn build_default_bind_group(
  device: &Device,
  pipeline: &RenderPipeline,
  texture1: &Option<Texture>,
  texture2: &Option<Texture>
) -> (BindGroup, Vec<Buffer>) {
  let limits = Limits::default();
  let min_stride = limits.min_uniform_buffer_offset_alignment;
  // create mvp buffer
  let mvp_buffer = device.create_buffer(&BufferDescriptor {
    label: Some("mvp-uniform-buffer"),
    size: min_stride as u64,
    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });
  // create general f32 buffer
  let gen_buffer = device.create_buffer(&BufferDescriptor {
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
  let ftexture = device.create_texture(&TextureDescriptor {
    label: Some("input-texture"),
    size: texture_size,
    sample_count: 1,
    mip_level_count: 1,
    dimension: TextureDimension::D2,
    format: TextureFormat::Rgba8Unorm,
    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    view_formats: &[]
  });
  if let Some(tx) = texture1 {
    texture1_view = tx.create_view(&TextureViewDescriptor::default());
  } else {
    texture1_view = ftexture.create_view(&TextureViewDescriptor::default());
  }
  if let Some(tx) = texture2 {
    texture2_view = tx.create_view(&TextureViewDescriptor::default());
  } else {
    texture2_view = ftexture.create_view(&TextureViewDescriptor::default());
  }

  // create sampler
  let sampler = device.create_sampler(&SamplerDescriptor {
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
  let bind_entries: Vec<BindGroupEntry> = vec![
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

  // create bind group
  let bind_group = device.create_bind_group(&BindGroupDescriptor {
    label: Some("default-bind-group"),
    layout: &pipeline.get_bind_group_layout(0),
    entries: &bind_entries
  });

  // create output
  (bind_group, vec![mvp_buffer, gen_buffer])
}

pub fn build_primitive_state(cull_mode: Option<Face>, polygon_mode: PolygonMode) -> wgpu::PrimitiveState {
  // translate polygon mode
  let topology: PrimitiveTopology = match polygon_mode {
    PolygonMode::Line => PrimitiveTopology::LineList,
    PolygonMode::Point => PrimitiveTopology::PointList,
    _ => PrimitiveTopology::TriangleList,
  };
  PrimitiveState {
    cull_mode,
    polygon_mode,
    topology,
    ..PrimitiveState::default()
  }
}

/// creates MVP matrix
pub fn create_mvp(update: &RenderObjectUpdate) -> [f32; 48] {
  let cam = match update.camera {
    Some(c) => c,
    None => &RenderCamera::default()
  };
  // model matrix
  let model_t = Mat4::translate(update.translate.x, update.translate.y, update.translate.z);
  let model_r = match update.rotate {
    RenderRotation::AxisAngle(axis, angle) => { Mat4::rotate(&axis, angle) }
    RenderRotation::Euler(x, y, z) => { Mat4::rotate_euler(x, y, z) }
  };
  let model_s = Mat4::scale(update.scale.x, update.scale.y, update.scale.z);
  let model = Mat4::multiply(&model_t, &Mat4::multiply(&model_s, &model_r));
  // view matrix
  let view_t = Mat4::translate(-cam.position.x, -cam.position.y, -cam.position.z);
  let view_r = Mat4::view_rot(&cam.position, &cam.look_at, &cam.up);
  let view = Mat4::multiply(&view_r, &view_t);
  // projection matrix
  let w2 = cam.target_size.x / 2.0;
  let h2 = cam.target_size.y / 2.0;
  let proj = match cam.cam_type {
    1 => Mat4::ortho(-w2, w2, h2, -h2, cam.near, cam.far),
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
