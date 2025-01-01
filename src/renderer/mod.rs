#![allow(dead_code, unused_imports)]

use bytemuck::{Pod, Zeroable};

mod root;
pub use root::*;

mod lin_alg;
pub use lin_alg::*;

mod primitives;
pub use primitives::*;

// --- --- --- --- --- --- --- --- //
// --- -- Render Components -- --- //
// --- --- --- --- --- --- --- --- //
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RId {
  pub pipeline: usize,
  pub texture: usize,
  pub object: usize
}
impl RId {
  pub fn pipeline(id: usize) -> Self {
    Self { pipeline:id, texture:0, object:0 }
  }
  pub fn texture(id: usize) -> Self {
    Self { pipeline:0, texture:id, object:0 }
  }
  pub fn object(pipe_id: usize, id: usize) -> Self {
    Self { pipeline:pipe_id, texture:0, object:id }
  }
}

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
pub struct RObject {
  pub visible: bool,
  v_buffer: wgpu::Buffer,
  v_count: usize,
  pipe_index: usize,
  index_buffer: Option<wgpu::Buffer>,
  index_count: u32,
  instances: u32,
}

#[derive(Debug)]
pub struct RBindGroup {
  base: wgpu::BindGroup,
  entries: Vec<wgpu::Buffer>,
}

#[derive(Debug)]
pub struct RPipeline {
  pipe: wgpu::RenderPipeline,
  objects: Vec<RObject>,
  max_obj_count: usize,
  vertex_type: u8,
  max_joints_count: u32,
  bind_group0: RBindGroup,
  bind_group1: Option<RBindGroup>,
  // bind_group2: Option<RBindGroup>,
  // bind_group3: Option<RBindGroup>,
}

// helper for defining camera/view matrix
#[derive(Debug)]
pub struct RCamera {
  pub cam_type: u8,
  pub position: [f32; 3],
  pub look_at: [f32; 3],
  pub up: [f32; 3],
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
      position: [0.0, 0.0, 100.0],
      look_at: [0.0, 0.0, 0.0],
      up: [0.0, 1.0, 0.0],
      fov_y: 0.0,
      near,
      far,
    }
  }
  pub fn new_persp(fov_y: f32, near: f32, far: f32) -> Self {
    Self {
      cam_type: RCamera::PERSPECTIVE,
      position: [0.0, 0.0, 1.0],
      look_at: [0.0, 0.0, 0.0],
      up: [0.0, 1.0, 0.0],
      fov_y,
      near,
      far,
    }
  }
}

// --- --- --- --- --- --- --- --- //
// --- ---  Pipeline Setup --- --- //
// --- --- --- --- --- --- --- --- //
#[derive(Debug)]
pub struct RUniformSetup {
  pub bind_slot: u32,
  pub visibility: u8,
  pub size_in_bytes: u32,
}
impl RUniformSetup {
  pub const VISIBILITY_VERTEX: u8 = 1;
  pub const VISIBILITY_FRAGMENT: u8 = 2;
  pub const VISIBILITY_BOTH: u8 = 0;
}
#[derive(Debug)]
pub struct RPipelineSetup<'a> {
  pub shader: &'a str,
  pub max_obj_count: usize,
  pub texture1_id: Option<u32>,
  pub texture2_id: Option<u32>,
  pub cull_mode: u8,
  pub poly_mode: u8,
  pub vertex_fn: Option<&'a str>,
  pub fragment_fn: Option<&'a str>,
  pub uniforms: Vec<RUniformSetup>,
  pub vertex_type: u8,
  pub max_joints_count: u32,
}
impl Default for RPipelineSetup<'_> {
  fn default() -> Self {
    Self {
      shader: include_str!("../embed_assets/base.wgsl"),
      max_obj_count: 10,
      texture1_id: None,
      texture2_id: None,
      cull_mode: RPipelineSetup::CULL_MODE_NONE,
      poly_mode: RPipelineSetup::POLY_MODE_TRI,
      vertex_fn: Some("vertexMain"),
      fragment_fn: Some("fragmentMain"),
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
  pub const VERTEX_TYPE_STATIC: u8 = 3;
  pub const VERTEX_TYPE_ANIM: u8 = 4;
  // polygon mode constants
  pub const POLY_MODE_TRI: u8 = 5;
  pub const POLY_MODE_LINE: u8 = 6;
  pub const POLY_MODE_POINT: u8 = 7;
}

// --- --- --- --- --- --- --- --- //
// --- -- -- Object Setup -- -- -- //
// --- --- --- --- --- --- --- --- //
#[derive(Debug)]
pub struct RObjectSetup {
  pub pipeline_id: usize,
  pub vertex_data: Vec<RVertex>,
  pub instances: u32,
  pub indices: Vec<u32>,
  pub vertex_type: u8,
  pub anim_vertex_data: Vec<RVertexAnim>,
}
impl Default for RObjectSetup {
  fn default() -> Self {
    RObjectSetup  {
      pipeline_id: 0,
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
#[derive(Debug)]
pub struct RObjectUpdate<'a> {
  pub id: RId,
  pub translate: &'a [f32; 3],
  pub rotate_axis: &'a [f32; 3],
  pub rotate_deg: f32,
  pub scale: &'a [f32; 3],
  pub visible: bool,
  pub camera: Option<&'a RCamera>,
  pub uniforms: Vec<&'a [u8]>,
  pub anim_transforms: Vec<[f32; 16]>,
}
impl Default for RObjectUpdate<'_> {
  fn default() -> Self {
    RObjectUpdate {
      id: RId::object(0, 0),
      translate: &[0.0, 0.0, 0.0],
      rotate_axis: &[0.0, 0.0, 1.0],
      rotate_deg: 0.0,
      scale: &[1.0, 1.0, 1.0],
      visible: true,
      camera: None,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
    }
  }
}
impl<'a> RObjectUpdate<'a> {
  pub fn from_shape(shape: &'a Shape) -> Self {
    RObjectUpdate {
      id: shape.id,
      translate: &shape.position,
      rotate_axis: &shape.rotate_axis,
      rotate_deg: shape.rotate_deg,
      scale: &shape.scale,
      visible: shape.visible,
      camera: None,
      uniforms: Vec::new(),
      anim_transforms: Vec::new(),
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
