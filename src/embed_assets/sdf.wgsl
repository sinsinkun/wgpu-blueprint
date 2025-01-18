@group(0) @binding(0) var<uniform> sys_data: SysData;
@group(0) @binding(1) var<uniform> obj_data: array<ObjData, 100>;

struct SysData {
  screen: vec2f,
  m_pos: vec2f,
  obj_count: f32,
}

struct ObjData {
  obj_type: f32,
  pos: vec2f,
  radius: f32,
  rect_size: vec2f,
  corner_radius: f32,
  rotation: f32,
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
fn sdCircle(p: vec2f, c: vec2f, r: f32) -> f32 {
  return length(p - c) - r;
}

fn sdBox(p: vec2f, c: vec2f, b: vec2f) -> f32 {
  let d: vec2f = abs(p - c) - b;
  return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sdBoxAngled(p: vec2f, c: vec2f, b: vec2f, a: f32) -> f32 {
  var np = p;
  np.x = (p.x - c.x) * cos(-a) - (p.y - c.y) * sin(-a) + c.x;
  np.y = (p.y - c.y) * cos(-a) + (p.x - c.x) * sin(-a) + c.y;
  return sdBox(np, c, b);
}

// round corners
fn opRound(sd: f32, r: f32) -> f32 {
  return sd - r;
}

// hollow center
fn opOnion(sd: f32, r: f32) -> f32 {
  return abs(sd) - r;
}

fn round_merge(sd1: f32, sd2: f32, r: f32) -> f32 {
  let intsp = min(vec2f(sd1 - r, sd2 - r), vec2f(0.0));
  return length(intsp) - r;
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
  // define shapes
  let cir_o = sys_data.m_pos;
  let cir_r = 50.0;
  let box_o = vec2f(400.0 + 50.0 * sin(0.005 * sys_data.m_pos.x), 200.0);
  let box_s = vec2f(100.0, 40.0);
  let box2_o = vec2f(400.0, 300.0);
  let box2_s = vec2f(50.0, 70.0);
  // sdf
  let d = round_merge(
    opOnion(sdCircle(p, cir_o, cir_r), 10.0),
    opRound(sdBox(p, box_o, box_s), 20.0),
    30.0
  );
  let d2 = round_merge(
    opOnion(sdCircle(p, cir_o, cir_r), 10.0),
    opRound(sdBoxAngled(p, box2_o, box2_s, 0.01 * sys_data.m_pos.y), 20.0),
    30.0
  );
  let d3 = max(d, d2);

  var r = smoothstep(-2.0, 2.0, d3) * 0.5;
  var g = 0.1 + 0.1 * sin(d3);
  var b = 0.4;
  // output
  return vec4f(r, g, b, 1.0);
}