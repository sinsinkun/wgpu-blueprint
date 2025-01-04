use std::default;
use std::convert::Into;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use ab_glyph::Rect;

use super::*;
use crate::vec3f;

// helper for defining object transform data
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Shape {
  pub id: RObjectId,
  pub position: Vec3,
  pub rotate_axis: Vec3,
  pub rotate_deg: f32,
  pub scale: Vec3,
  pub visible: bool,
  pub v_index: Option<Vec<f32>>,
  pub anim_transforms: Vec<[f32; 16]>,
}
impl Shape {
  pub fn new(renderer: &mut Renderer, pipeline_id: RPipelineId, vertex_data: Vec<RVertex>, index_data: Option<Vec<u32>>) -> Self {
    let mut setup = RObjectSetup {
      pipeline_id,
      vertex_data,
      ..Default::default()
    };
    if let Some(indices) = index_data {
      setup.indices = indices;
    }
    let id = renderer.add_object(setup);
    Self {
      id,
      position: vec3f!(0.0, 0.0, 0.0),
      rotate_axis: vec3f!(0.0, 0.0, 1.0),
      rotate_deg: 0.0,
      scale: vec3f!(1.0, 1.0, 1.0),
      visible: true,
      v_index: None,
      anim_transforms: Vec::new(),
    }
  }
  pub fn new_anim(renderer: &mut Renderer, pipeline_id: RPipelineId, vertex_data: Vec<RVertexAnim>, index_data: Option<Vec<u32>>) -> Self {
    let mut setup = RObjectSetup {
      pipeline_id,
      anim_vertex_data: vertex_data,
      vertex_type: RObjectSetup::VERTEX_TYPE_ANIM,
      ..Default::default()
    };
    if let Some(indices) = index_data {
      setup.indices = indices;
    }
    let id = renderer.add_object(setup);
    Self {
      id,
      position: vec3f!(0.0, 0.0, 0.0),
      rotate_axis: vec3f!(0.0, 0.0, 1.0),
      rotate_deg: 0.0,
      scale: vec3f!(1.0, 1.0, 1.0),
      visible: true,
      v_index: None,
      anim_transforms: Vec::new(),
    }
  }
}

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
    }
  }
}

// helper for building new pipeline
#[derive(Debug, Default, Clone)]
pub enum RShader<'a> {
  #[default]
  Texture, Text, FlatColor, RoundedRect, Custom(&'a str)
}
#[derive(Debug, Default)]
pub enum RUniformVisibility {
  #[default]
  Vertex, Fragment, Both
}
#[derive(Debug)]
pub struct RUniformSetup {
  pub bind_slot: u32,
  pub visibility: RUniformVisibility,
  pub size_in_bytes: u32,
}
#[derive(Debug)]
pub struct RPipelineSetup<'a> {
  pub shader: RShader<'a>,
  pub max_obj_count: usize,
  pub texture1_id: Option<RTextureId>,
  pub texture2_id: Option<RTextureId>,
  pub cull_mode: u8,
  pub poly_mode: u8,
  pub vertex_fn: &'a str,
  pub fragment_fn: &'a str,
  pub uniforms: Vec<RUniformSetup>,
  pub vertex_type: u8,
  pub max_joints_count: u32,
}
impl Default for RPipelineSetup<'_> {
  fn default() -> Self {
      RPipelineSetup {
        shader: RShader::Texture,
        max_obj_count: 10,
        texture1_id: None,
        texture2_id: None,
        cull_mode: RPipelineSetup::CULL_MODE_NONE,
        poly_mode: RPipelineSetup::POLY_MODE_TRI,
        vertex_fn: "vertexMain",
        fragment_fn: "fragmentMain",
        uniforms: Vec::new(),
        vertex_type: RPipelineSetup::VERTEX_TYPE_STATIC,
        max_joints_count: 0,
      }
  }
}
impl RPipelineSetup<'_> {
  // cull mode constants
  pub const CULL_MODE_NONE: u8 = 0;
  pub const CULL_MODE_BACK: u8 = 1;
  pub const CULL_MODE_FRONT: u8 = 2;
  // vertex type constants
  pub const VERTEX_TYPE_STATIC: u8 = 0;
  pub const VERTEX_TYPE_ANIM: u8 = 1;
  // polygon mode constants
  pub const POLY_MODE_TRI: u8 = 0;
  pub const POLY_MODE_LINE: u8 = 1;
  pub const POLY_MODE_POINT: u8 = 2;
}

// helper for building new render object
#[derive(Debug)]
pub struct RObjectSetup {
  pub pipeline_id: RPipelineId,
  pub vertex_data: Vec<RVertex>,
  pub instances: u32,
  pub indices: Vec<u32>,
  pub vertex_type: u8,
  pub anim_vertex_data: Vec<RVertexAnim>,
}
impl Default for RObjectSetup {
  fn default() -> Self {
    RObjectSetup  {
      pipeline_id: RPipelineId(0),
      vertex_data: Vec::new(),
      indices: Vec::new(),
      instances: 1,
      anim_vertex_data: Vec::new(),
      vertex_type: RObjectSetup::VERTEX_TYPE_STATIC,
    }
  }
}
impl RObjectSetup {
  pub const VERTEX_TYPE_STATIC: u8 = 0;
  pub const VERTEX_TYPE_ANIM: u8 = 1;
}

// helper for updating render object
#[derive(Debug, Clone)]
pub enum RRotation {
  AxisAngle(Vec3, f32),
  Euler(f32, f32, f32)
}

#[derive(Debug)]
pub struct RObjectUpdate<'a> {
  pub object_id: RObjectId,
  pub translate: Vec3,
  pub rotate: RRotation,
  pub scale: Vec3,
  pub visible: bool,
  pub camera: Option<&'a RCamera>,
  pub color: RColor,
  pub uniforms: Vec<&'a [u8]>,
  pub anim_transforms: Vec<[f32; 16]>,
  pub rect_size: Option<[f32; 2]>,
  pub rect_radius: f32,
}
impl Default for RObjectUpdate<'_> {
  fn default() -> Self {
    RObjectUpdate {
      object_id: RObjectId(0, 0),
      translate: vec3f!(0.0, 0.0, 0.0),
      rotate: RRotation::AxisAngle(vec3f!(0.0, 0.0, 1.0), 0.0),
      scale: vec3f!(1.0, 1.0, 1.0),
      visible: true,
      camera: None,
      color: RColor::WHITE,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
      rect_size: None,
      rect_radius: 0.0,
    }
  }
}
impl<'a> RObjectUpdate<'a> {
  pub fn from_shape(shape: &'a Shape) -> Self {
    RObjectUpdate {
      object_id: shape.id,
      translate: shape.position,
      rotate: RRotation::AxisAngle(vec3f!(0.0, 0.0, 1.0), 0.0),
      scale: shape.scale,
      visible: shape.visible,
      camera: None,
      color: RColor::WHITE,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
      rect_size: None,
      rect_radius: 0.0,
    }
  }
  pub fn with_camera(mut self, camera: &'a RCamera) -> Self {
    self.camera = Some(camera);
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
