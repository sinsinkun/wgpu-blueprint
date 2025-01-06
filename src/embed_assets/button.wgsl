@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(1) var<uniform> buf: Buf;
@group(0) @binding(2) var tx_sampler: sampler;
@group(0) @binding(3) var texture1: texture_2d<f32>;

struct MVP {
  model: mat4x4<f32>,
  view: mat4x4<f32>,
  proj: mat4x4<f32>,
}

struct Buf {
  albedo: vec4f,
  rect_size: vec2f,
  radius: f32,
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
fn vertexMain(input: VertIn) -> VertOut {
  var out: VertOut;
  let mvp_mat = mvp.proj * mvp.view * mvp.model;
  out.pos = mvp_mat * vec4f(input.pos, 1.0);
  out.uv = vec2f(input.uv.x, 1.0 - input.uv.y);
  out.normal = (mvp.model * vec4f(input.normal, 0.0)).xyz;
  return out;
}

@fragment
fn fragmentMain(input: VertOut) -> @location(0) vec4f {
  let r = buf.radius;
  let x = input.uv.x * buf.rect_size.x;
  let y = input.uv.y * buf.rect_size.y;

  // top left
  var rx = x - r;
  var ry = y - r;
  if x < r && y < r && (rx * rx + ry * ry > r * r) {
    discard;
  }
  // top right
  rx = x - (buf.rect_size.x - r);
  ry = y - r;
  if x > (buf.rect_size.x - r) && y < r && (rx * rx + ry * ry > r * r) {
    discard;
  }
  // bottom left
  rx = x - r;
  ry = y - (buf.rect_size.y - r);
  if x < r && y > (buf.rect_size.y - r) && (rx * rx + ry * ry > r * r) {
    discard;
  }
  // bottom right
  rx = x - (buf.rect_size.x - r);
  ry = y - (buf.rect_size.y - r);
  if x > (buf.rect_size.x - r) && y > (buf.rect_size.y - r) && (rx * rx + ry * ry > r * r) {
    discard;
  }

  var out = buf.albedo;
  // add text
  let tx1 = textureSample(texture1, tx_sampler, input.uv);
  out =  mix(out, vec4f(tx1.rgb, 1.0), tx1.a);

  // drop shadow
  let ir = r * 0.5;
  if x > buf.rect_size.x - ir || y > buf.rect_size.y - ir {
    out = mix(out, vec4f(0.0, 0.0, 0.0, 1.0), 0.5);
  } else if x > (buf.rect_size.x - r) && y > buf.rect_size.y - r && (rx * rx + ry * ry > ir * ir) {
    out = mix(out, vec4f(0.0, 0.0, 0.0, 1.0), 0.5);
  }

  return out;
}