use crate::vec2f;

use super::*;

pub struct CollisionResponse {
  p1: Vec2,
  v1: Vec2,
  p2: Vec2,
  v2: Vec2,
}

pub fn cir_2d_collision(r1: f32, r2: f32, p1: Vec2, p2: Vec2, v1: Vec2, v2: Vec2, dt: f32) -> CollisionResponse {
  let distance = p1 - p2;
  let mut out = CollisionResponse {
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