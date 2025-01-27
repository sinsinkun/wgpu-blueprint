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

fn sdTriangle(p: vec2f, c: vec2f, p0: vec2f, p1: vec2f, p2: vec2f) -> f32 {
  let np = p - c;
  let e0 = p1-p0;
  let e1 = p2-p1;
  let e2 = p0-p2;
  let v0 = np-p0;
  let v1 = np-p1;
  let v2 = np-p2;
  let pq0 = v0 - e0*clamp( dot(v0,e0)/dot(e0,e0), 0.0, 1.0 );
  let pq1 = v1 - e1*clamp( dot(v1,e1)/dot(e1,e1), 0.0, 1.0 );
  let pq2 = v2 - e2*clamp( dot(v2,e2)/dot(e2,e2), 0.0, 1.0 );
  let s: f32 = sign( e0.x*e2.y - e0.y*e2.x );
  let d: vec2f = min(min(vec2(dot(pq0,pq0), s*(v0.x*e0.y-v0.y*e0.x)),
                    vec2(dot(pq1,pq1), s*(v1.x*e1.y-v1.y*e1.x))),
                    vec2(dot(pq2,pq2), s*(v2.x*e2.y-v2.y*e2.x)));
  return -sqrt(d.x)*sign(d.y);
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
  for (var i: u32 = 0; i < sys_data.oc; i++) {
    let obj: ObjData = obj_data[i];
    var d = 1000.0;
    if (obj.obj_type == 1) { // circle
      d = sdCircle(p, obj.pos, obj.r);
    } else if (obj.obj_type == 2) { // box
      d = sdBox(p, obj.pos, obj.v2);
    } else if (obj.obj_type == 4) { // angledbox
      d = sdBoxAngled(p, obj.pos, obj.v2, obj.rot);
    } else if (obj.obj_type == 3) { // triangle
      let p0 = vec2f(0.0, 0.0);
      let p1 = obj.v2;
      let p2 = obj.v3;
      d = sdTriangle(p, obj.pos, p0, p1, p2);
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
  // calculate all object SDFs - contained in interpolate_colors
  let sdf = calculate_sdf(p, 1000.0);
  // output
  return sdf.clr;
}