#![allow(dead_code, unused_imports)]

use bytemuck::{Pod, Zeroable};

mod root;
pub use root::*;

mod lin_alg;
pub use lin_alg::*;

// --- --- --- --- --- --- --- --- //
// --- ---  Render Objects --- --- //
// --- --- --- --- --- --- --- --- //
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
