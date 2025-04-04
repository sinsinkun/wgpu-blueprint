@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(1) var<uniform> albedo: vec4f;

struct MVP {
  model: mat4x4<f32>,
  view: mat4x4<f32>,
  proj: mat4x4<f32>,
}

struct VertIn {
  @location(0) pos: vec3f,
  @location(1) uv: vec2f,
  @location(2) normal: vec3f,
}

struct VertOut {
  @builtin(position) pos: vec4f,
  @location(0) uv: vec2f,
  @location(1) normal: vec3f,
}

@vertex
fn vertex_main(input: VertIn) -> VertOut {
  var out: VertOut;
  let mvp_mat = mvp.proj * mvp.view * mvp.model;
  out.pos = mvp_mat * vec4f(input.pos, 1.0);
  out.uv = vec2f(input.uv.x, input.uv.y);
  out.normal = (mvp.model * vec4f(input.normal, 0.0)).xyz;
  return out;
}

@fragment
fn fragment_main(input: VertOut) -> @location(0) vec4f {
  return albedo;
}