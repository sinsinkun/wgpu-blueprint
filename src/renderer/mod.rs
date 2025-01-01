#![allow(dead_code, unused_imports)]

use bytemuck::{Pod, Zeroable};

mod root;
pub use root::*;

mod text;
pub use text::*;

mod primitives;
pub use primitives::*;

mod lin_alg;
pub use lin_alg::*;

mod util;
pub use util::*;

mod model_loader;
pub use model_loader::*;


// -- HELPER STRUCTS --
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RObjectId (pub usize, pub usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RPipelineId (pub usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RTextureId (pub usize);