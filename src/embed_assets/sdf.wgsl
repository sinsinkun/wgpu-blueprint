@group(0) @binding(0) var<uniform> sys_data: SysData;
@group(0) @binding(1) var<uniform> obj_data: array<ObjData, 100>;

struct SysData {
  screen: vec2f,
  m_pos: vec2f,
}

struct ObjData {
  obj_type: f32,
  pos: vec2f,
  radius: f32,
  rect_size: vec2f,
  corner_radius: f32,
  color: vec4f,
}

struct VertIn {
  @location(0) pos: vec3f,
  @location(1) uv: vec2f,
}

struct VertOut {
  @builtin(position) pos: vec4f,
  @location(0) uv: vec2f,
}

// ----------------------------------------- //
// ------------- SDF FUNCTIONS ------------- //
// ----------------------------------------- //
fn sdCircle(p: vec2f, cp: vec2f, r: f32) -> f32 {
  return length(p - cp) - r;
}

fn sdBox(p: vec2f, cr: vec2f, b: vec2f) -> f32 {
  let d: vec2f = abs(p - cr) - b;
  return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

// round corners
fn opRound(sd: f32, r: f32) -> f32 {
  return sd - r;
}

// hollow center
fn opOnion(sd: f32, r: f32) -> f32 {
  return abs(sd) - r;
}

// ----------------------------------------- //
// ----------- SHADER DEFINITION ----------- //
// ----------------------------------------- //
@vertex
fn vertexMain(input: VertIn) -> VertOut {
  var out: VertOut;
  out.pos = vec4f(input.pos, 1.0);
  out.uv = vec2f(input.uv.x, 1.0 - input.uv.y);
  return out;
}

@fragment
fn fragmentMain(input: VertOut) -> @location(0) vec4f {
  // work in screen space
  let p = input.pos.xy;
  // define circle
  let cir_o = sys_data.m_pos;
  let cir_r = 50.0;
  let box_o = vec2f(400.0, 200.0);
  let box_s = vec2f(100.0, 40.0);
  // sdf
  let d = min(sdCircle(p, cir_o, cir_r), opRound(sdBox(p, box_o, box_s), 20.0));

  let r = smoothstep(2.0, -2.0, d);
  // output
  return vec4f(r, 0.1, 0.1, 1.0);
}