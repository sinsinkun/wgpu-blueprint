use crate::vec2f;

use super::*;

// --- --- --- --- --- --- --- //
// --- Collision Detection --- //
// --- --- --- --- --- --- --- //
pub fn signed_dist_to_cir(point: Vec2, cir_center: Vec2, cir_radius: f32) -> f32 {
  let vector = cir_center - point;
  // note: negative distance if point is within the circle
  vector.magnitude() - cir_radius
}

pub fn signed_dist_to_rect(
  point: Vec2, rect_center: Vec2, rect_size: Vec2, rect_rotation: Option<f32>
) -> f32 {
  let mut rel = point - rect_center;
  // rotate rel_vec by -r
  if let Some(r) = rect_rotation {
    rel.x = rel.x * f32::cos(-r) - rel.y * f32::sin(-r);
    rel.y = rel.y * f32::cos(-r) + rel.x * f32::sin(-r);
  }
  let dx = f32::max(rel.x - rect_size.x / 2.0, 0.0);
  let dy = f32::max(rel.y - rect_size.y / 2.0, 0.0);
  let outer_d = if dx > dy { dx } else { dy };
  let ix = f32::min(rel.x - rect_size.x / 2.0, 0.0);
  let iy = f32::min(rel.y - rect_size.y / 2.0, 0.0);
  let inner_d = if ix > iy { ix } else { iy };
  outer_d + inner_d
}

// --- --- --- --- --- --- --- //
// --- Collision Response  --- //
// --- --- --- --- --- --- --- //
pub struct CollisionResponse2D {
  p1: Vec2,
  v1: Vec2,
  p2: Vec2,
  v2: Vec2,
}

pub fn cir_to_cir_collision_2d(
  r1: f32, r2: f32, p1: Vec2, p2: Vec2, v1: Vec2, v2: Vec2, dt: f32
) -> CollisionResponse2D {
  let distance = p1 - p2;
  let mut out = CollisionResponse2D {
    p1: vec2f!(0.0, 0.0),
    v1: vec2f!(0.0, 0.0),
    p2: vec2f!(0.0, 0.0),
    v2: vec2f!(0.0, 0.0)
  };
  // collision check
  if distance.magnitude() < r1 + r2 {
    let new_magnitude = v1.magnitude() + v2.magnitude();
    let new_dir = (v1 - v2).normalize();
    out.v1 = new_dir * -1.0 * new_magnitude;
    out.v2 = new_dir * new_magnitude;
  };
  out.p1 = p1 + out.v1 * dt;
  out.p2 = p2 + out.v2 * dt;
  out
}

#[cfg(test)]
mod physics_tests {
  use super::*;
  // use `cargo test physics_tests -- --nocapture` for logging
  #[test]
  fn signed_dist_rect() {
    let c = vec2f!(0.0, 0.0);
    let size = vec2f!(4.0, 4.0);

    let p1 = vec2f!(5.0, 0.0);
    let d1 = signed_dist_to_rect(p1, c, size, None);
    assert_eq!(d1, 3.0);

    let p2 = vec2f!(0.0, 5.0);
    let d2 = signed_dist_to_rect(p2, c, size, None);
    assert_eq!(d2, 3.0);

    let p3 = vec2f!(1.0, 0.0);
    let d3 = signed_dist_to_rect(p3, c, size, None);
    assert_eq!(d3, -1.0);

    let p4 = vec2f!(2.0, 2.0);
    let d4 = signed_dist_to_rect(p4, c, size, None);
    assert_eq!(d4, 0.0);
  }
}