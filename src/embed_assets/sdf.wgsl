@group(0) @binding(0) var<uniform> sys_data: SysData;
@group(0) @binding(1) var<uniform> obj_data: array<ObjData, 100>;

struct SysData {
  screen: vec2f,
  mp: vec2f,
  oc: u32,
  md: f32,
}

struct ObjData {
  obj_type: u32,
  r: f32,
  pos: vec2f,
  rsize: vec2f,
  cr: f32,
  rot: f32,
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
  // define vars
  var merge_sd = 0.0;
  let merge_dist = sys_data.md;
  let bg = vec4f(0.004, 0.005, 0.008, 1.0);
  var fg = vec4f(0.0, 0.0, 0.0, 1.0);
  // calculate
  for (var i: u32 = 0; i < sys_data.oc; i++) {
    let obj: ObjData = obj_data[i];
    var d = 1000.0;
    if (obj.obj_type == 1) { // circle
      d = sdCircle(p, obj.pos, obj.r);
    } else if (obj.obj_type == 2) { // box
      d = sdBox(p, obj.pos, obj.rsize);
    } else if (obj.obj_type == 4) { // angledbox
      d = sdBoxAngled(p, obj.pos, obj.rsize, obj.rot);
    }
    if (obj.cr > 0.0) {
      d = opRound(d, obj.cr);
    }
    let sq = min(d - merge_dist, 0.0) * min(d - merge_dist, 0.0);
    fg = mix(fg, obj.color, smoothstep(merge_dist, 0.0, d) * 0.8);
    merge_sd = merge_sd + sq;
  }
  let merge_fsd = sqrt(merge_sd) - merge_dist;

  // output
  return mix(bg, fg, smoothstep(0.0, 1.0, merge_fsd));
}