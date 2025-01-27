use super::*;
use crate::vec2f;
use crate::renderer::{RSDFObject, RSDFObjectType};

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
  let e1 = p2 - p1;
  let e2 = p0 - p2;
  let v0 = np - p0;
  let v1 = np - p1;
  let v2 = np - p2;
  let pq0 = v0 - e0 * f32::clamp( v0.dot(e0) / e0.dot(e0), 0.0, 1.0);
  let pq1 = v1 - e1 * f32::clamp( v1.dot(e1) / e1.dot(e1), 0.0, 1.0);
  let pq2 = v2 - e2 * f32::clamp( v2.dot(e2) / e2.dot(e2), 0.0, 1.0);
  let s = if e0.x * e2.y - e0.y * e2.x > 0.0 { 1.0 } else { -1.0 };
  let d1 = vec2f!(pq0.dot(pq0), s * (v0.x * e0.y - v0.y * e0.x));
  let d2 = vec2f!(pq1.dot(pq1), s * (v1.x * e1.y - v1.y * e1.x));
  let d3 = vec2f!(pq2.dot(pq2), s * (v2.x * e2.y - v2.y * e2.x));
  let mut min_dx = d1.x;
  if min_dx > d2.x { min_dx = d2.x; }
  if min_dx > d3.x { min_dx = d3.x; }
  let mut min_dy = d1.y;
  if min_dy > d2.y { min_dy = d2.y; }
  if min_dy > d3.y { min_dy = d3.y; }
  let sign = if min_dy > 0.0 { -1.0 } else { 1.0 };

  f32::sqrt(min_dx) * sign
}

pub fn signed_dist_with_corner(sd: f32, radius: f32) -> f32 {
  sd - radius
}

pub fn signed_dist_as_border(sd: f32, thickness: f32) -> f32 {
  f32::abs(sd) - thickness
}

pub fn calculate_sdf(p: Vec2, max_dist: f32, objs: &Vec<RSDFObject>) -> f32 {
  let mut sdf = max_dist;
  for obj in objs {
    let mut d = max_dist;
    match obj.obj_type {
      RSDFObjectType::Circle => {
        d = signed_dist_to_cir(p, obj.center, obj.radius);
      }
      RSDFObjectType::Rectangle => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, None);
      }
      RSDFObjectType::RectAngled => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, Some(obj.rotation));
      }
      RSDFObjectType::Triangle => {
        // asums p0 is the center
        d = signed_dist_to_triangle(p, obj.center, vec2f!(0.0, 0.0), obj.tri_size.0, obj.tri_size.1);
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

pub fn calculate_sdf_alt(p: Vec2, max_dist: f32, merge_dist: f32, objs: &Vec<RSDFObject>) -> f32 {
  let mut sdf = 0.0;
  for obj in objs {
    let mut d = max_dist;
    match obj.obj_type {
      RSDFObjectType::Circle => {
        d = signed_dist_to_cir(p, obj.center, obj.radius);
      }
      RSDFObjectType::Rectangle => {
        d = signed_dist_to_rect(p, obj.center, obj.rect_size, None);
      }
      RSDFObjectType::RectAngled => {
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

pub fn ray_march_dist(origin: Vec2, dir: Vec2, max_dist: f32, objs: &Vec<RSDFObject>) -> f32 {
  let ndir = dir.normalize();
  let mut p = origin;
  let mut sdf = calculate_sdf(p, max_dist, objs);
  let mut ray_dist = sdf;
  while ray_dist < max_dist && sdf > 0.999 {
    p = p + ndir * sdf;
    sdf = calculate_sdf(p, max_dist, objs);
    ray_dist += sdf;
  }
  ray_dist
}
