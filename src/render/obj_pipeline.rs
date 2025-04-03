use wgpu::{
  vertex_attr_array, BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, 
  BufferAddress, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, 
  DepthStencilState, Device, Face, FragmentState, IndexFormat, MultisampleState, PipelineCompilationOptions, 
  PipelineLayoutDescriptor, PolygonMode, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, StencilState, 
  TextureFormat, VertexBufferLayout, VertexState, VertexStepMode
};

use super::{
  build_default_bind_group, build_default_bind_group_layout, build_default_shader_module,
  build_primitive_state, build_shader_module, create_mvp, RenderObject, RenderObjectSetup,
  RenderObjectUpdate, RenderVertex
};

#[derive(Debug)]
pub struct ObjPipeline {
  pub pipeline: RenderPipeline,
  pub objects: Vec<RenderObject>,
}
impl ObjPipeline {
  pub fn new(device: &Device, target_format: TextureFormat, use_depth: bool) -> Self {
    let shader_mod = build_default_shader_module(device);
    let bind_group0_layout = build_default_bind_group_layout(device);
    let bind_group_container: Vec<&BindGroupLayout> = vec![&bind_group0_layout];

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("pipeline-layout"),
      bind_group_layouts: bind_group_container.as_slice(),
      push_constant_ranges: &[]
    });
    // switch between static/dynamic vertex layouts
    let vertex_attr_static = vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
    let vertex_layout = VertexBufferLayout {
      array_stride: std::mem::size_of::<RenderVertex>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: &vertex_attr_static,
    };

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("render-pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &shader_mod,
        entry_point: Some("vertex_main"),
        buffers: &[vertex_layout],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(FragmentState{
        module: &shader_mod,
        entry_point: Some("fragment_main"),
        targets: &[Some(ColorTargetState{
          format: target_format,
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
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: true,
      },
      depth_stencil: if use_depth { 
        Some(DepthStencilState {
          format: TextureFormat::Depth24Plus,
          depth_write_enabled: true,
          depth_compare: CompareFunction::LessEqual,
          stencil: StencilState::default(),
          bias: DepthBiasState::default(),
        })
      } else { None },
      primitive: build_primitive_state(Some(Face::Back), PolygonMode::Fill),
      multiview: None,
      cache: None,
    });

    Self {
      pipeline,
      objects: Vec::new(),
    }
  }
  pub fn add_object(&mut self, device: &Device, queue: &Queue, setup: RenderObjectSetup) -> usize {
    // create vertex buffer
    let vlen = setup.vertex_data.len();
    let v_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("vertex-buffer"),
      size: (std::mem::size_of::<RenderVertex>() * vlen) as u64,
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
      mapped_at_creation: false
    });
    queue.write_buffer(&v_buffer, 0, bytemuck::cast_slice(&setup.vertex_data));

    // create index buffer
    let mut index_buffer: Option<Buffer> = None;
    let ilen: usize = setup.indices.len();
    if ilen > 0 {
      let i_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("index-buffer"),
        size: (std::mem::size_of::<u32>() * ilen) as u64,
        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        mapped_at_creation: false
      });
      queue.write_buffer(&i_buffer, 0, bytemuck::cast_slice(&setup.indices));
      index_buffer = Some(i_buffer);
    }

    // create bind group 0
    let (bind_group0, buffers0) = build_default_bind_group(device, &self.pipeline, &setup.texture1, &setup.texture2);

    // save to cache
    let obj = RenderObject {
      visible: true,
      v_buffer,
      v_count: vlen,
      index_buffer,
      index_count: ilen as u32,
      instances: 1,
      bind_group0,
      buffers0,
      texture1: setup.texture1,
      texture2: setup.texture2,
      max_joints: setup.max_joints,
    };
    self.objects.push(obj);
    let idx = self.objects.len() - 1;
    self.update_object(idx, queue, RenderObjectUpdate {
      camera: setup.camera,
      ..Default::default()
    });
    idx
  }
  pub fn update_object(&mut self, idx: usize, queue: &Queue, update: RenderObjectUpdate) {
    let mvp = create_mvp(&update);
    let buf = update.gen_buf;
    let obj = &mut self.objects[idx];
    obj.visible = update.visible;

    // let stride = self.limits.min_uniform_buffer_offset_alignment;
    queue.write_buffer(&obj.buffers0[0], 0, bytemuck::cast_slice(&mvp));
    queue.write_buffer(&obj.buffers0[1], 0, bytemuck::cast_slice(&buf.as_slice()));

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
      queue.write_buffer(&obj.buffers0[1], 0, bytemuck::cast_slice(&anim_buffer));
    }
  }
  pub fn render(&self, pass: &mut RenderPass) {
    pass.set_pipeline(&self.pipeline);
    for i in 0..self.objects.len() {
      let obj = &self.objects[i];
      if !obj.visible { continue; }
      pass.set_vertex_buffer(0, obj.v_buffer.slice(..));
      pass.set_bind_group(0, &obj.bind_group0, &[]);
      if let Some(i_buffer) = &obj.index_buffer {
        pass.set_index_buffer(i_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..obj.index_count, 0, 0..obj.instances);
      } else {
        pass.draw(0..(obj.v_count as u32), 0..obj.instances);
      }
    }
  }
  pub fn destroy(&mut self) {
    for i in 0..self.objects.len() {
      self.objects[i].v_buffer.destroy();
      if let Some(b) = &self.objects[i].index_buffer { b.destroy(); }
      if let Some(tx) = &self.objects[i].texture1 { tx.destroy(); }
      if let Some(tx) = &self.objects[i].texture2 { tx.destroy(); }
      for b in &self.objects[i].buffers0 { b.destroy(); }
    }
  }
}