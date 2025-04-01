use super::*;
use crate::vec2f;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum SDFObjectType {
  #[default]
  None, Circle, Rectangle, Triangle, RectAngled, Line, Pie,
}
impl From<SDFObjectType> for u32 {
  fn from(value: SDFObjectType) -> Self {
    match value {
      SDFObjectType::Circle => 1,
      SDFObjectType::Rectangle => 2,
      SDFObjectType::Triangle => 3,
      SDFObjectType::RectAngled => 4,
      SDFObjectType::Line => 5,
      _ => 0,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SDFObject {
  pub obj_type: SDFObjectType,
  pub center: Vec2,
  pub radius: f32,
  pub rect_size: Vec2,
  pub corner_radius: f32,
  pub rotation: f32,
  pub line_thickness: f32,
  pub tri_size: (Vec2, Vec2),
}
impl Default for SDFObject {
  fn default() -> Self {
    Self {
      obj_type: SDFObjectType::None,
      center: Vec2::zero(),
      radius: 10.0,
      rect_size: Vec2::zero(),
      corner_radius: 0.0,
      rotation: 0.0,
      line_thickness: 0.0,
      tri_size: (Vec2::zero(), Vec2::zero())
    }
  }
}
impl SDFObject {
  pub fn circle(pos: Vec2, r: f32) -> Self {
    Self {
      obj_type: SDFObjectType::Circle,
      center: pos,
      radius: r,
      ..Default::default()
    }
  }
  pub fn rect(pos: Vec2, size: Vec2, angle: Option<f32>) -> Self {
    let mut obj_type = SDFObjectType::Rectangle;
    let mut rotation = 0.0;
    if let Some(a) = angle {
      obj_type = SDFObjectType::RectAngled;
      rotation = a;
    }
    Self {
      obj_type,
      rotation,
      center: pos,
      rect_size: size,
      ..Default::default()
    }
  }
  pub fn triangle(pos: Vec2, rel_p1: Vec2, rel_p2: Vec2) -> Self {
    Self {
      obj_type: SDFObjectType::Triangle,
      center: pos,
      tri_size: (rel_p1, rel_p2),
      ..Default::default()
    }
  }
  pub fn line(p1: Vec2, p2: Vec2, thickness: f32) -> Self {
    Self {
      obj_type: SDFObjectType::Line,
      center: p1,
      rect_size: p2,
      line_thickness: thickness,
      ..Default::default()
    }
  }
  pub fn with_corner(mut self, radius: f32) -> Self {
    self.corner_radius = radius;
    self
  }
  pub fn as_line(mut self, thickness: f32) -> Self {
    self.line_thickness = thickness;
    self
  }
  pub fn update_line(&mut self, p1: Vec2, p2: Vec2) {
    self.center = p1;
    self.rect_size = p2;
  }
}

pub fn signed_dist_to_cir(point: Vec2, cir_center: Vec2, cir_radius: f32) -> f32 {
  let vector = cir_center - point;
  // note: negative distance if point is within the circle
  vector.magnitude() - cir_radius
}

pub fn signed_dist_to_rect(
  point: Vec2, rect_center: Vec2, rect_size: Vec2, rect_rotation: Option<f32>
) -> f32 {
  let rot_p = if let Some(r) = rect_rotation {
    let rad = r.to_radians();
    let x = (point.x - rect_center.x) * f32::cos(-rad) - (point.y - rect_center.y) * f32::sin(-rad) + rect_center.x;
    let y = (point.y - rect_center.y) * f32::cos(-rad) + (point.x - rect_center.x) * f32::sin(-rad) + rect_center.y;
    vec2f!(x, y)
  } else { point };
  let mut abs_p = rot_p - rect_center;
  if abs_p.x < 0.0 { abs_p.x = -abs_p.x };
  if abs_p.y < 0.0 { abs_p.y = -abs_p.y };
  let d0 = abs_p - rect_size;
  let mut d = d0;
  if d.x < 0.0 { d.x = 0.0 };
  if d.y < 0.0 { d.y = 0.0 };
  let outer = d.magnitude();
  let inner = f32::min(f32::max(d0.x, d0.y), 0.0);
  outer + inner
}

// note: p0/p1/p2 is relative to center
pub fn signed_dist_to_triangle(
  point: Vec2, center: Vec2, p0: Vec2, p1: Vec2, p2: Vec2
) -> f32 {
  let np = point - center;

  let e0 = p1 - p0;
  let v0 = np - p0;
  let d0 = v0 - e0 * f32::clamp(v0.dot(e0)/e0.dot(e0), 0.0, 1.0);
  let d0d = d0.dot(d0);

  let e1 = p2 - p1;
  let v1 = np - p1;
  let d1 = v1 - e1 * f32::clamp(v1.dot(e1)/e1.dot(e1), 0.0, 1.0);
  let d1d = d1.dot(d1);

  let e2 = p0 - p2;
  let v2 = np - p2;
  let d2 = v2 - e2 * f32::clamp(v2.dot(e2)/e2.dot(e2), 0.0, 1.0);
  let d2d = d2.dot(d2);

  let o: f32 = e0.x * e2.y - e0.y * e2.x;
  let y0 = o*(v0.x*e0.y - v0.y*e0.x);
  let y1 = o*(v1.x*e1.y - v1.y*e1.x);
  let y2 = o*(v2.x*e2.y - v2.y*e2.x);
  let mut min_d = d0d;
  if d1d < min_d { min_d = d1d; }
  if d2d < min_d { min_d = d2d; }
  let mut min_y = y0;
  if y1 < min_y { min_y = y1; }
  if y2 < min_y { min_y = y2; }
  let sign = if min_y > 0.0 { -1.0 } else { 1.0 };

  f32::sqrt(min_d) * sign
}

pub fn signed_dist_to_line(point: Vec2, p0: Vec2, p1: Vec2) -> f32 {
  let pa = point - p0;
  let ba = p1 - p0;
  let h = f32::clamp(pa.dot(ba) / ba.dot(ba), 0.0, 1.0);
  (pa - ba * h).magnitude()
}

pub fn signed_dist_with_corner(sd: f32, radius: f32) -> f32 {
  sd - radius
}

pub fn signed_dist_as_border(sd: f32, thickness: f32) -> f32 {
  f32::abs(sd) - thickness
}

pub fn calculate_sdf(p: Vec2, max_dist: f32, objs: &Vec<SDFObject>) -> f32 {
  let mut sdf = max_dist;
  for obj in objs {
    let mut d = max_dist;
    match obj.obj_type {
      SDFObjectType::Circle => {
        d = signed_dist_to_cir(p, obj.center, obj.radius);
      }
      SDFObjectType::Rectangle => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, None);
      }
      SDFObjectType::RectAngled => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, Some(obj.rotation));
      }
      SDFObjectType::Triangle => {
        // assumes p0 is the center
        d = signed_dist_to_triangle(p, obj.center, vec2f!(0.0, 0.0), obj.tri_size.0, obj.tri_size.1);
      }
      SDFObjectType::Line => {
        d = signed_dist_to_line(p, obj.center, obj.rect_size);
      }
      _ => ()
    }
    if obj.corner_radius > 0.0 {
      d = signed_dist_with_corner(d, obj.corner_radius);
    }
    if obj.line_thickness > 0.0 {
      d = signed_dist_as_border(d, obj.line_thickness);
    }
    if d < sdf { sdf = d; }
  }
  sdf
}

pub fn calculate_sdf_alt(p: Vec2, max_dist: f32, merge_dist: f32, objs: &Vec<SDFObject>) -> f32 {
  let mut sdf = 0.0;
  for obj in objs {
    let mut d = max_dist;
    match obj.obj_type {
      SDFObjectType::Circle => {
        d = signed_dist_to_cir(p, obj.center, obj.radius);
      }
      SDFObjectType::Rectangle => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, None);
      }
      SDFObjectType::RectAngled => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, Some(obj.rotation));
      }
      _ => ()
    }
    if obj.corner_radius > 0.0 {
      d = signed_dist_with_corner(d, obj.corner_radius);
    }
    if obj.line_thickness > 0.0 {
      d = signed_dist_as_border(d, obj.line_thickness);
    }
    let sq = f32::min(d - merge_dist, 0.0) * f32::min(d - merge_dist, 0.0);
    sdf = sdf + sq;
  }
  f32::sqrt(sdf) - merge_dist
}

pub fn ray_march_dist(origin: Vec2, dir: Vec2, max_dist: f32, objs: &Vec<SDFObject>) -> f32 {
  let ndir = dir.normalize();
  let mut p = origin;
  let mut sdf = calculate_sdf(p, max_dist, objs);
  let mut ray_dist = sdf;
  let mut iter = 0;
  while ray_dist < max_dist && sdf > 0.999 && iter < 99999 {
    iter += 1;
    p = p + ndir * sdf;
    sdf = calculate_sdf(p, max_dist, objs);
    ray_dist += sdf;
  }
  if ray_dist > max_dist { max_dist }
  else { ray_dist }
}
