@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(2) var tx_sampler: sampler;
@group(0) @binding(3) var texture1: texture_2d<f32>;
@group(0) @binding(4) var texture2: texture_2d<f32>;

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
  let n = (1.0 + input.normal) / 2.0;
  var tx1 = textureSample(texture1, tx_sampler, input.uv);
  var tx2 = textureSample(texture2, tx_sampler, input.uv);
  // draw normal instead of texture if alpha < 0.0001
  tx1 = mix(tx1, vec4f(n, 1.0), step(tx1.a, 0.0001));
  // mix tx1 and tx2, tx2 overwrites tx1
  let blend = mix(tx1, tx2, tx2.a);
  return vec4f(blend.rgb, tx1.a);
}