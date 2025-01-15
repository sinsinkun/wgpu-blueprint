#![allow(dead_code, unused_imports)]

use std::ops::{Add, AddAssign, Sub, SubAssign};

mod lin_alg;
pub use lin_alg::*;
mod physics;
pub use physics::*;
mod utils;
pub use utils::*;

#[macro_export]
macro_rules! vec2f {
  ($x:expr, $y:expr) => {
    Vec2::new($x, $y)
  };
}

#[macro_export]
macro_rules! vec3f {
  ($x:expr, $y:expr, $z:expr) => {
    Vec3::new($x, $y, $z)
  };
}

#[macro_export]
macro_rules! vec4f {
  ($x:expr, $y:expr, $z:expr, $w:expr) => {
    Vec4::new($x, $y, $z, $w)
  };
}
