use std::default::Default;
use std::convert::Into;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use ab_glyph::Rect;

use super::*;
use crate::{vec2f, vec3f};

// helper for defining camera/view matrix
#[derive(Debug, Clone)]
pub struct RCamera {
  pub cam_type: u8,
  pub position: Vec3,
  pub look_at: Vec3,
  pub up: Vec3,
  pub fov_y: f32,
  pub near: f32,
  pub far: f32,
  pub target_size: Option<Vec2>,
}
impl Default for RCamera {
  fn default() -> Self {
    Self {
      cam_type: RCamera::ORTHOGRAPHIC,
      position: vec3f!(0.0, 0.0, 100.0),
      look_at: vec3f!(0.0, 0.0, 0.0),
      up: vec3f!(0.0, 1.0, 0.0),
      fov_y: 0.0,
      near: 0.0,
      far: 1000.0,
      target_size: None,
    }
  }
}
impl RCamera {
  pub const ORTHOGRAPHIC: u8 = 1;
  pub const PERSPECTIVE: u8 = 2;
  pub fn new_ortho(near: f32, far: f32) -> Self {
    Self {
      cam_type: RCamera::ORTHOGRAPHIC,
      position: vec3f!(0.0, 0.0, 100.0),
      look_at: vec3f!(0.0, 0.0, 0.0),
      up: vec3f!(0.0, 1.0, 0.0),
      fov_y: 0.0,
      near,
      far,
      target_size: None,
    }
  }
  pub fn new_persp(fov_y: f32, near: f32, far: f32) -> Self {
    Self {
      cam_type: RCamera::PERSPECTIVE,
      position: vec3f!(0.0, 0.0, 1.0),
      look_at: vec3f!(0.0, 0.0, 0.0),
      up: vec3f!(0.0, 1.0, 0.0),
      fov_y,
      near,
      far,
      target_size: None,
    }
  }
}

// --- --- --- --- --- --- --- --- --- --- //
// --- --- --- Pipeline Helpers -- --- --- //
// --- --- --- --- --- --- --- --- --- --- //

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct RPipelineId (pub usize);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct RObjectId (pub usize);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct RTextureId {
  pub base: usize,
  pub msaa: usize,
  pub zbuffer: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RShader<'a> {
  #[default]
  Texture, Text, FlatColor, Custom(&'a str)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RCullMode {
  #[default]
  None, Front, Back
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RPolyMode {
  #[default]
  Fill, Line, Point
}

#[derive(Debug)]
pub struct RPipelineSetup<'a> {
  pub shader: RShader<'a>,
  pub cull_mode: RCullMode,
  pub poly_mode: RPolyMode,
  pub vertex_fn: &'a str,
  pub fragment_fn: &'a str,
  pub has_animations: bool,
}
impl Default for RPipelineSetup<'_> {
  fn default() -> Self {
    Self {
      shader: RShader::Texture,
      cull_mode: RCullMode::None,
      poly_mode: RPolyMode::Fill,
      vertex_fn: "vertexMain",
      fragment_fn: "fragmentMain",
      has_animations: false,
    }
  }
}

#[derive(Debug)]
pub struct RPipeline {
  pub pipe: wgpu::RenderPipeline,
  pub obj_indices: Vec<usize>,
  pub has_animations: bool,
}

// --- --- --- --- --- --- --- --- --- --- //
// --- ---  Render Object Helpers  --- --- //
// --- --- --- --- --- --- --- --- --- --- //

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct RVertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub normal: [f32; 3],
}
impl RVertex {
  pub fn add_joints(&self, joints: [u32; 4], weights: [f32; 4]) -> RVertexAnim {
    RVertexAnim {
      position: self.position,
      uv: self.uv,
      normal: self.normal,
      joint_ids: joints,
      joint_weights: weights
    }
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct RVertexAnim {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub normal: [f32; 3],
  pub joint_ids: [u32; 4],
  pub joint_weights: [f32; 4]
}

#[derive(Debug)]
pub struct RObjectSetup {
  pub pipeline_id: RPipelineId,
  pub vertex_data: Vec<RVertex>,
  pub instances: u32,
  pub indices: Vec<u32>,
  pub anim_vertex_data: Vec<RVertexAnim>,
  pub texture1_id: Option<RTextureId>,
  pub texture2_id: Option<RTextureId>,
  pub max_joints: usize,
}
impl Default for RObjectSetup {
  fn default() -> Self {
    Self {
      pipeline_id: RPipelineId(0),
      vertex_data: Vec::new(),
      indices: Vec::new(),
      instances: 1,
      anim_vertex_data: Vec::new(),
      texture1_id: None,
      texture2_id: None,
      max_joints: 0,
    }
  }
}

#[derive(Debug)]
pub struct RObject {
  pub visible: bool,
  pub pipe_id: RPipelineId,
  // vertex data
  pub v_buffer: wgpu::Buffer,
  pub v_count: usize,
  pub max_joints: usize,
  pub index_buffer: Option<wgpu::Buffer>,
  pub index_count: u32,
  pub instances: u32,
  // render data
  pub bind_group0: wgpu::BindGroup,
  pub buffers0: Vec<wgpu::Buffer>,
  pub texture1: Option<RTextureId>,
  pub texture2: Option<RTextureId>,
}

// helper for updating render object
#[derive(Debug, Clone)]
pub enum RRotation {
  AxisAngle(Vec3, f32),
  Euler(f32, f32, f32)
}

#[derive(Debug)]
pub struct RObjectUpdate<'a> {
  pub translate: Vec3,
  pub rotate: RRotation,
  pub scale: Vec3,
  pub visible: bool,
  pub camera: Option<&'a RCamera>,
  pub gen_buf: [f32; 64],
  pub uniforms: Vec<&'a [u8]>,
  pub anim_transforms: Vec<[f32; 16]>,
}
impl Default for RObjectUpdate<'_> {
  fn default() -> Self {
    RObjectUpdate {
      translate: vec3f!(0.0, 0.0, 0.0),
      rotate: RRotation::AxisAngle(vec3f!(0.0, 0.0, 1.0), 0.0),
      scale: vec3f!(1.0, 1.0, 1.0),
      visible: true,
      camera: None,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
      gen_buf: [0.0; 64],
    }
  }
}
impl<'a> RObjectUpdate<'a> {
  pub fn with_position(mut self, pos: Vec3) -> Self {
    self.translate = pos;
    self
  }
  pub fn with_rotation(mut self, axis: Vec3, angle_deg: f32) -> Self {
    self.rotate = RRotation::AxisAngle(axis, angle_deg);
    self
  }
  pub fn with_euler_rotation(mut self, roll: f32, pitch: f32, yaw: f32) -> Self {
    self.rotate = RRotation::Euler(roll, pitch, yaw);
    self
  }
  pub fn with_scale(mut self, scale: Vec3) -> Self {
    self.scale = scale;
    self
  }
  pub fn with_camera(mut self, camera: &'a RCamera) -> Self {
    self.camera = Some(camera);
    self
  }
  pub fn with_color(mut self, color: RColor) -> Self {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RColor {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}
impl Into<Vec<f32>> for RColor {
  fn into(self) -> Vec<f32> {
    vec![self.r, self.g, self.b, self.a]
  }
}
impl Into<[f32; 4]> for RColor {
  fn into(self) -> [f32; 4] {
    [self.r, self.g, self.b, self.a]
  }
}
impl RColor {
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

    let mut clr = RColor::WHITE;
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct SysData {
  pub screen: Vec2,
  pub mouse_pos: Vec2,
  pub obj_count: u32,
  pub merge_dist: f32,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum RSDFObjectType {
  #[default]
  None, Circle, Rectangle, Triangle, RectAngled, Pie,
}
impl From<RSDFObjectType> for u32 {
  fn from(value: RSDFObjectType) -> Self {
    match value {
      RSDFObjectType::Circle => 1,
      RSDFObjectType::Rectangle => 2,
      RSDFObjectType::Triangle => 3,
      RSDFObjectType::RectAngled => 4,
      _ => 0,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RSDFObject {
  pub obj_type: RSDFObjectType,
  pub center: Vec2,
  pub radius: f32,
  pub rect_size: Vec2,
  pub corner_radius: f32,
  pub rotation: f32,
  pub color: RColor,
  pub line_thickness: f32,
  pub tri_size: (Vec2, Vec2),
}
impl Default for RSDFObject {
  fn default() -> Self {
    Self {
      obj_type: RSDFObjectType::None,
      center: Vec2::zero(),
      radius: 10.0,
      rect_size: Vec2::zero(),
      corner_radius: 0.0,
      rotation: 0.0,
      color: RColor::WHITE,
      line_thickness: 0.0,
      tri_size: (Vec2::zero(), Vec2::zero())
    }
  }
}
impl RSDFObject {
  pub fn circle(pos: Vec2, r: f32) -> Self {
    Self {
      obj_type: RSDFObjectType::Circle,
      center: pos,
      radius: r,
      ..Default::default()
    }
  }
  pub fn rect(pos: Vec2, size: Vec2, angle: Option<f32>) -> Self {
    let mut obj_type = RSDFObjectType::Rectangle;
    let mut rotation = 0.0;
    if let Some(a) = angle {
      obj_type = RSDFObjectType::RectAngled;
      rotation = a.to_radians();
    }
    Self {
      obj_type,
      rotation,
      center: pos,
      rect_size: size,
      ..Default::default()
    }
  }
  pub fn triangle(pos: Vec2, rel_p1: Vec2, rel_p2: Vec2) -> Self {
    Self {
      obj_type: RSDFObjectType::Triangle,
      center: pos,
      tri_size: (rel_p1, rel_p2),
      ..Default::default()
    }
  }
  pub fn with_color(mut self, color: RColor) -> Self {
    self.color = color;
    self
  }
  pub fn with_corner(mut self, radius: f32) -> Self {
    self.corner_radius = radius;
    self
  }
  pub fn as_line(mut self, thickness: f32) -> Self {
    self.line_thickness = thickness;
    self
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct RSDFObjectC {
  // note: must be representable in sets of 4 32-bit values (vec4f)
  pub object_type: u32,
  pub radius: f32,
  pub center: [f32; 2],
  pub v2: [f32; 2],
  pub corner_radius: f32,
  pub rotation: f32,
  pub onion: f32,
  pub v3: [f32; 2],
  buffer: f32,
  pub color: [f32; 4],
}
impl RSDFObjectC {
  pub fn from(a: &RSDFObject) -> Self {
    let mut v2 = a.rect_size.as_array();
    let mut v3 = Vec2::zero().as_array();
    if a.obj_type == RSDFObjectType::Triangle {
      v2 = a.tri_size.0.as_array();
      v3 = a.tri_size.1.as_array();
    }
    Self {
      object_type: a.obj_type.into(),
      radius: a.radius,
      center: a.center.as_array(),
      v2,
      corner_radius: a.corner_radius,
      rotation: a.rotation,
      onion: a.line_thickness,
      v3,
      buffer: 0.0,
      color: a.color.into(),
    }
  }
}
