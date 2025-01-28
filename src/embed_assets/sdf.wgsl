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

fn mod289v3(x: vec3f) -> vec3f {
  return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289(x: vec2f) -> vec2f {
  return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec3f) -> vec3f {
  return mod289v3(((x*34.0)+10.0)*x);
}

fn snoise(v: vec2f) -> f32 {
  let nc = vec4f(0.211324865405187,  // (3.0-sqrt(3.0))/6.0
                0.366025403784439,  // 0.5*(sqrt(3.0)-1.0)
               -0.577350269189626,  // -1.0 + 2.0 * C.x
                0.024390243902439); // 1.0 / 41.0
  // First corner
  var i: vec2f = floor(v + dot(v, nc.yy) );
  let x0 = v - i + dot(i, nc.xx);

  // Other corners
  var i1: vec2f;
  //i1.x = step( x0.y, x0.x ); // x0.x > x0.y ? 1.0 : 0.0
  //i1.y = 1.0 - i1.x;
  if (x0.x > x0.y) {
    i1 = vec2f(1.0, 0.0);
  } else {
    i1 = vec2f(0.0, 1.0);
  }
  // x0 = x0 - 0.0 + 0.0 * nc.xx ;
  // x1 = x0 - i1 + 1.0 * nc.xx ;
  // x2 = x0 - 1.0 + 2.0 * nc.xx ;
  var x12: vec4f = x0.xyxy + nc.xxzz;
  x12= vec4f(x12.x - i1.x, x12.y - i1.y, x12.zw);

  // Permutations
  i = mod289(i); // Avoid truncation effects in permutation
  let p = permute( permute( i.y + vec3f(0.0, i1.y, 1.0 )) + i.x + vec3f(0.0, i1.x, 1.0 ));
  var m = max(0.5 - vec3f(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), vec3f(0.0));
  m = m*m;
  m = m*m;

  // Gradients: 41 points uniformly over a line, mapped onto a diamond.
  // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
  let x = 2.0 * fract(p * nc.www) - 1.0;
  let h = abs(x) - 0.5;
  let ox = floor(x + 0.5);
  let a0 = x - ox;

  // Normalise gradients implicitly by scaling m
  // Approximation of: m *= inversesqrt( a0*a0 + h*h );
  m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );

  // Compute final noise value at P
  let g = vec3f(
    a0.x  * x0.x  + h.x  * x0.y,
    a0.yz * x12.xz + h.yz * x12.yw
  );
  return 130.0 * dot(m, g);
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

fn ray_march(origin: vec2f, dir: vec2f, max_dist: f32) -> f32 {
  let ndir = normalize(-dir);
  var p = origin;
  var sdf = calculate_sdf(p, max_dist);
  var ray_dist = sdf.sdf;
  var iter = 0;
  while ray_dist < max_dist && sdf.sdf > 0.999 && iter < 99999 {
    iter += 1;
    p = p + ndir * sdf.sdf;
    sdf = calculate_sdf(p, max_dist);
    ray_dist += sdf.sdf;
  }
  return min(ray_dist, max_dist);
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
  // calculate all object SDFs - also interpolates colors
  let sdf = calculate_sdf(p, 1000.0);
  let d = distance(p, sys_data.light_pos);
  let rm = ray_march(p, p - sys_data.light_pos, d);
  // output
  var out = sdf.clr;
  if (sys_data.light_dist > 0.0 && abs(d - rm) < 1.0) {
    out += sys_data.light_color * smoothstep(sys_data.light_dist, 0.0, d);
  }
  return out;
}