@group(0) @binding(0) var<uniform> sys_data: SysData;
@group(0) @binding(1) var<uniform> obj_data: array<ObjData, 100>;

struct SysData {
  screen: vec2f,
  mouse_pos: vec2f,
  obj_count: u32,
  light_dist: f32,
  light_pos: vec2f,
  light_color: vec4f,
}

struct ObjData {
  obj_type: u32,
  r: f32,
  pos: vec2f, // first quad
  v2: vec2f,
  cr: f32,
  rot: f32, // second quad
  onion: f32,
  v3: vec2f, // third quad
  color: vec4f, // 4th quad
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
// ------------ NOISE GENERATOR ------------ //
// ----------------------------------------- //

const noise_granularity: f32 = 0.5 / 255.0;

fn random(c: vec2f) -> f32 {
  return fract(sin(dot(c, vec2f(12.9898,78.233))) * 43758.5453);
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

fn sdLine(p: vec2f, a: vec2f, b: vec2f) -> f32 {
  let pa = p - a;
  let ba = b - a;
  let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
  return length(pa - ba * h);
}

fn dot2(v: vec2f) -> f32 {
  return dot(v, v);
}

fn sdTriangle(p: vec2f, c: vec2f, p0: vec2f, p1: vec2f, p2: vec2f) -> f32 {
  let np = p - c;

  let e0 = p1 - p0;
  let v0 = np - p0;
  let d0: f32 = dot2(v0-e0*clamp(dot(v0,e0)/dot(e0,e0),0.0,1.0));

  let e1 = p2 - p1;
  let v1 = np - p1;
  let d1: f32 = dot2(v1-e1*clamp(dot(v1,e1)/dot(e1,e1),0.0,1.0));

  let e2 = p0 - p2;
  let v2 = np - p2;
  let d2: f32 = dot2(v2-e2*clamp(dot(v2,e2)/dot(e2,e2),0.0,1.0));

  let o: f32 = e0.x * e2.y - e0.y * e2.x;
  let d: vec2f = min(min(vec2(d0,o*(v0.x*e0.y-v0.y*e0.x)),
                         vec2(d1,o*(v1.x*e1.y-v1.y*e1.x))),
                         vec2(d2,o*(v2.x*e2.y-v2.y*e2.x)));
	return -sqrt(d.x) * sign(d.y);
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

struct SdfOut {
  sdf: f32,
  clr: vec4f,
}

// calculates sdf + interpolates color between all objects
fn calculate_sdf(p: vec2f, max_dist: f32) -> SdfOut {
  var sdf = max_dist;
  var clr = vec4f(0.0);
  for (var i: u32 = 0; i < sys_data.obj_count; i++) {
    let obj: ObjData = obj_data[i];
    var d = 1000.0;
    if (obj.obj_type == 1) { // circle
      d = sdCircle(p, obj.pos, obj.r);
    } else if (obj.obj_type == 2) { // box
      d = sdBox(p, obj.pos, obj.v2);
    } else if (obj.obj_type == 4) { // angledbox
      let a = radians(obj.rot);
      d = sdBoxAngled(p, obj.pos, obj.v2, a);
    } else if (obj.obj_type == 3) { // triangle
      let p1 = vec2f(0.0, 0.0);
      d = sdTriangle(p, obj.pos, p1, obj.v2, obj.v3);
    } else if (obj.obj_type == 5) { // line
      d = sdLine(p, obj.pos, obj.v2);
    }
    if (obj.cr > 0.0) {
      d = opRound(d, obj.cr);
    }
    if (obj.onion > 0.0) {
      d = opOnion(d, obj.onion);
    }
    sdf = min(d, sdf);
    let intensity = smoothstep(1.0, -1.0, d);
    clr = mix(clr, obj.color, intensity * obj.color.a);
  }
  var sdf_out: SdfOut;
  sdf_out.sdf = sdf;
  sdf_out.clr = clr;
  return sdf_out;
}

struct RayMarchOut {
  d: f32,
  min_sdf: f32,
}

fn ray_march(origin: vec2f, dir: vec2f, max_dist: f32) -> RayMarchOut {
  let ndir = normalize(-dir);
  var p = origin;
  var sdf = calculate_sdf(p, max_dist);
  var ray_dist = sdf.sdf;
  var min_sdf = sdf.sdf;
  var iter = 0;
  while ray_dist < max_dist && sdf.sdf > 0.999 && iter < 99999 {
    iter += 1;
    p = p + ndir * sdf.sdf;
    sdf = calculate_sdf(p, max_dist);
    ray_dist += sdf.sdf;
    if (sdf.sdf < min_sdf) {
      min_sdf = sdf.sdf;
    }
  }
  var out: RayMarchOut;
  out.d = min(ray_dist, max_dist);
  out.min_sdf = min_sdf;
  return out;
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
  let shadow_smoothing = 4.0;
  // calculate all object SDFs - also interpolates colors
  let sdf = calculate_sdf(p, 10000.0);
  let d = distance(p, sys_data.light_pos);
  let shadow_offset = step(0.1, sys_data.light_dist) * normalize(p - sys_data.light_pos) * shadow_smoothing;
  let rm = ray_march(p - shadow_offset, p - sys_data.light_pos, d);
  // output
  var out = sdf.clr;
  // calculate lighting
  if (sys_data.light_dist > 0.0 && abs(d - rm.d) < 0.1) {
    var lc = sys_data.light_color;
    // soft shadow smoothing
    if (d > shadow_smoothing) {
      lc = lc * smoothstep(0.0, shadow_smoothing, rm.min_sdf);
    }
    // light distance attenuation
    let dlc = lc * smoothstep(sys_data.light_dist, 0.0, d);
    out += dlc;
  }
  return out;
}