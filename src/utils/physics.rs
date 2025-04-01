use super::*;
use crate::vec2f;

pub fn screen_to_world_2d(coords: &Vec2, win_size: &Vec2) -> Vec2 {
  Vec2 {
    x: coords.x - (win_size.x / 2.0),
    y: coords.y - (win_size.y / 2.0),
  }
}

pub fn point_in_rect(point: &Vec2, rect_origin: &Vec2, rect_size: &Vec2) -> bool {
  let x_min = rect_origin.x - rect_size.x / 2.0;
  let x_max = rect_origin.x + rect_size.x / 2.0;
  let y_min = rect_origin.y - rect_size.y / 2.0;
  let y_max = rect_origin.y + rect_size.y / 2.0;
  let x_in = point.x > x_min && point.x < x_max;
  let y_in = point.y > y_min && point.y < y_max;
  x_in && y_in
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