@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(1) var<uniform> buf: Buf;

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
  out.uv = input.uv;
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

  return buf.albedo;
}