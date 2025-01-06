#![allow(dead_code)]

use std::ops::{Add, AddAssign, Sub, SubAssign};

pub const PI: f32 = 3.14159265;

/**
 * Note: These matrices are in column major order, as per wgpu requirements. 
 * If you need to use them to perform regular matrix transformations,
 * please transpose the result.
 */
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Mat4 {
  a00: f32, a01: f32, a02: f32, a03: f32,
  a10: f32, a11: f32, a12: f32, a13: f32,
  a20: f32, a21: f32, a22: f32, a23: f32,
  a30: f32, a31: f32, a32: f32, a33: f32,
}
impl Mat4 {
  // util fns
  pub fn from_row_major(arr: [f32; 16]) -> Self {
    Self {
      a00: arr[0], a01: arr[1], a02: arr[2], a03: arr[3],
      a10: arr[4], a11: arr[5], a12: arr[6], a13: arr[7],
      a20: arr[8], a21: arr[9], a22: arr[10], a23: arr[11],
      a30: arr[12], a31: arr[13], a32: arr[14], a33: arr[15],
    }
  }
  pub fn from_col_major(arr: [f32; 16]) -> Self {
    Self {
      a00: arr[0], a01: arr[4], a02: arr[8], a03: arr[12],
      a10: arr[1], a11: arr[5], a12: arr[9], a13: arr[13],
      a20: arr[2], a21: arr[6], a22: arr[10], a23: arr[14],
      a30: arr[3], a31: arr[7], a32: arr[11], a33: arr[15],
    }
  }
  pub fn identity() -> Self {
    Self {
      a00: 1.0, a01: 0.0, a02: 0.0, a03: 0.0,
      a10: 0.0, a11: 1.0, a12: 0.0, a13: 0.0,
      a20: 0.0, a21: 0.0, a22: 1.0, a23: 0.0,
      a30: 0.0, a31: 0.0, a32: 0.0, a33: 1.0,
    }
  }
  pub fn as_row_major_array(&self) -> [f32; 16] {
    [
      self.a00, self.a01, self.a02, self.a03,
      self.a10, self.a11, self.a12, self.a13,
      self.a20, self.a21, self.a22, self.a23,
      self.a30, self.a31, self.a32, self.a33
    ]
  }
  pub fn as_col_major_array(&self) -> [f32; 16] {
    [
      self.a00, self.a10, self.a20, self.a30,
      self.a01, self.a11, self.a21, self.a31,
      self.a02, self.a12, self.a22, self.a32,
      self.a03, self.a13, self.a23, self.a33
    ]
  }
  pub fn row(&self, n: usize) -> [f32; 4] {
    if n > 3 {
      panic!("Rows are indexed 0 to 3")
    }
    match n {
      0 => [self.a00, self.a01, self.a02, self.a03],
      1 => [self.a10, self.a11, self.a12, self.a13],
      2 => [self.a20, self.a21, self.a22, self.a23],
      3 => [self.a30, self.a31, self.a32, self.a33],
      _ => [0.0, 0.0, 0.0, 0.0]
    }
  }
  pub fn update_row(&mut self, row: usize, values: [f32; 4]) {
    match row {
      0 => {
        self.a00 = values[0];
        self.a01 = values[1];
        self.a02 = values[2];
        self.a03 = values[3];
      }
      1 => {
        self.a10 = values[0];
        self.a11 = values[1];
        self.a12 = values[2];
        self.a13 = values[3];
      }
      2 => {
        self.a20 = values[0];
        self.a21 = values[1];
        self.a22 = values[2];
        self.a23 = values[3];
      }
      3 => {
        self.a30 = values[0];
        self.a31 = values[1];
        self.a32 = values[2];
        self.a33 = values[3];
      }
      _ => ()
    }
  }
  pub fn col(&self, n: usize) -> [f32; 4] {
    if n > 3 {
      panic!("Cols are indexed 0 to 3")
    }
    match n {
      0 => [self.a00, self.a10, self.a20, self.a30],
      1 => [self.a01, self.a11, self.a21, self.a31],
      2 => [self.a02, self.a12, self.a22, self.a32],
      3 => [self.a03, self.a13, self.a23, self.a33],
      _ => [0.0, 0.0, 0.0, 0.0]
    }
  }
  pub fn update_col(&mut self, col: usize, values: [f32; 4]) {
    match col {
      0 => {
        self.a00 = values[0];
        self.a10 = values[1];
        self.a20 = values[2];
        self.a30 = values[3];
      }
      1 => {
        self.a01 = values[0];
        self.a11 = values[1];
        self.a21 = values[2];
        self.a31 = values[3];
      }
      2 => {
        self.a02 = values[0];
        self.a12 = values[1];
        self.a22 = values[2];
        self.a32 = values[3];
      }
      3 => {
        self.a03 = values[0];
        self.a13 = values[1];
        self.a23 = values[2];
        self.a33 = values[3];
      }
      _ => ()
    }
  }
  pub fn cell(&self, row: usize, col: usize) -> f32 {
    if row > 3 || col > 3 {
      panic!("Row/Col cannot be greater than 3");
    }
    let r = self.row(row);
    r[col]
  }
  pub fn update_cell(&mut self, row: usize, col: usize, value: f32) {
    match row {
      0 => {
        match col {
          0 => self.a00 = value,
          1 => self.a01 = value,
          2 => self.a02 = value,
          3 => self.a03 = value,
          _ => ()
        }
      }
      1 => {
        match col {
          0 => self.a10 = value,
          1 => self.a11 = value,
          2 => self.a12 = value,
          3 => self.a13 = value,
          _ => ()
        }
      }
      2 => {
        match col {
          0 => self.a20 = value,
          1 => self.a21 = value,
          2 => self.a22 = value,
          3 => self.a23 = value,
          _ => ()
        }
      }
      3 => {
        match col {
          0 => self.a30 = value,
          1 => self.a31 = value,
          2 => self.a32 = value,
          3 => self.a33 = value,
          _ => ()
        }
      }
      _ => ()
    }
  }
  pub fn multiply_f32(&self, value: f32) -> Mat4 {
    Mat4 {
      a00: self.a00 * value, a01: self.a01 * value, a02: self.a02 * value, a03: self.a03 * value,
      a10: self.a10 * value, a11: self.a11 * value, a12: self.a12 * value, a13: self.a13 * value,
      a20: self.a20 * value, a21: self.a21 * value, a22: self.a22 * value, a23: self.a23 * value,
      a30: self.a30 * value, a31: self.a31 * value, a32: self.a32 * value, a33: self.a33 * value,
    }
  }
  pub fn multiply_vec4(&self, vec4: &Vec4) -> Vec4 {
    let vec = vec4.as_array();
    let mut out: [f32; 4] = [0.0; 4];
    for r in 0..4 {
      for (i, v) in self.row(r).iter().enumerate() {
        out[r] += v * vec[i];
      }
    }
    Vec4::from_array(out)
  }
  pub fn multiply_mat4(&self, rhs: &Mat4) -> Mat4 {
    Mat4 {
      a00: self.a00 * rhs.a00 + self.a01 * rhs.a10 + self.a02 * rhs.a20 + self.a03 * rhs.a30,
      a01: self.a00 * rhs.a01 + self.a01 * rhs.a11 + self.a02 * rhs.a21 + self.a03 * rhs.a31,
      a02: self.a00 * rhs.a02 + self.a01 * rhs.a12 + self.a02 * rhs.a22 + self.a03 * rhs.a32,
      a03: self.a00 * rhs.a03 + self.a01 * rhs.a13 + self.a02 * rhs.a23 + self.a03 * rhs.a33,

      a10: self.a10 * rhs.a00 + self.a11 * rhs.a10 + self.a12 * rhs.a20 + self.a13 * rhs.a30,
      a11: self.a10 * rhs.a01 + self.a11 * rhs.a11 + self.a12 * rhs.a21 + self.a13 * rhs.a31,
      a12: self.a10 * rhs.a02 + self.a11 * rhs.a12 + self.a12 * rhs.a22 + self.a13 * rhs.a32,
      a13: self.a10 * rhs.a03 + self.a11 * rhs.a13 + self.a12 * rhs.a23 + self.a13 * rhs.a33,

      a20: self.a20 * rhs.a00 + self.a21 * rhs.a10 + self.a22 * rhs.a20 + self.a23 * rhs.a30,
      a21: self.a20 * rhs.a01 + self.a21 * rhs.a11 + self.a22 * rhs.a21 + self.a23 * rhs.a31,
      a22: self.a20 * rhs.a02 + self.a21 * rhs.a12 + self.a22 * rhs.a22 + self.a23 * rhs.a32,
      a23: self.a20 * rhs.a03 + self.a21 * rhs.a13 + self.a22 * rhs.a23 + self.a23 * rhs.a33,

      a30: self.a30 * rhs.a00 + self.a31 * rhs.a10 + self.a32 * rhs.a20 + self.a33 * rhs.a30,
      a31: self.a30 * rhs.a01 + self.a31 * rhs.a11 + self.a32 * rhs.a21 + self.a33 * rhs.a31,
      a32: self.a30 * rhs.a02 + self.a31 * rhs.a12 + self.a32 * rhs.a22 + self.a33 * rhs.a32,
      a33: self.a30 * rhs.a03 + self.a31 * rhs.a13 + self.a32 * rhs.a23 + self.a33 * rhs.a33,
    }
  }
  pub fn to_string(&self) -> String {
    let mut str = "Mat4: { ".to_owned();
    for r in 0..4 {
      str += "[";
      for v in self.row(r) {
        str = str + &format!("{:.4}", v) + ", ";
      }
      str.pop();
      str.pop();
      str += "], "
    }
    str.pop();
    str.pop();
    str += " }";
    str
  }
  // matrix transforms
  pub fn perspective(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> [f32; 16] {
    let f = f32::tan(PI * 0.5 - 0.5 * fov_y * PI / 180.0);
    let range = 1.0 / (near - far);
    let a = f / aspect_ratio;
    let c = far * range;
    let d = near * far * range;
    [
      a, 0.0, 0.0, 0.0,
      0.0, f, 0.0, 0.0,
      0.0, 0.0, c, -1.0,
      0.0, 0.0, d, 0.0
    ]
  }
  pub fn ortho(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> [f32; 16] {
    let a = 2.0 / (right - left);
    let b = 2.0 / (top - bottom);
    let c = 1.0 / (near - far);
    let d = (right + left) / (left - right);
    let e = (top + bottom) / (bottom - top);
    let f = near / (near - far);
    [
      a, 0.0, 0.0, 0.0,
      0.0, b, 0.0, 0.0,
      0.0, 0.0, c, 0.0,
      d, e, f, 1.0
    ]
  }
  pub fn translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      x, y, z, 1.0
    ]
  }
  pub fn translate_inverse(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      -x, -y, -z, 1.0
    ]
  }
  pub fn rotate(axis: &Vec3, deg: f32) -> [f32; 16] {
    // normalize axis
    let n = axis.normalize();
    // helpers
    let xx = n.x * n.x;
    let yy = n.y * n.y;
    let zz = n.z * n.z;
    let c = f32::cos(deg * PI / 180.0);
    let s = f32::sin(deg * PI / 180.0);
    let o = 1.0 - c;
    [
      xx + (1.0 - xx) * c,
      n.x * n.y * o + n.z * s,
      n.x * n.z * o - n.y * s,
      0.0,

      n.x * n.y * o - n.z * s,
      yy + (1.0 - yy) * c,
      n.y * n.z * o + n.x * s,
      0.0,

      n.x * n.z * o + n.y * s,
      n.y * n.z * o - n.x * s,
      zz + (1.0 - zz) * c,
      0.0,

      0.0,
      0.0,
      0.0,
      1.0
    ]
  }
  // note: quaternion rotation preferred over euler rotation
  pub fn rotate_euler(roll: f32, pitch: f32, yaw: f32) -> [f32; 16] {
    let a = roll * PI / 180.0;
    let cosa = f32::cos(a);
    let sina = f32::sin(a);
    let b = pitch * PI / 180.0;
    let cosb = f32::cos(b);
    let sinb = f32::sin(b);
    let c = yaw * PI / 180.0;
    let cosc = f32::cos(c);
    let sinc = f32::sin(c);
    [
      cosb * cosc,
      cosb * sinc,
      -sinb,
      0.0,

      sina * sinb * cosc - cosa * sinc,
      sina * sinb * sinc + cosa * cosc,
      sina * cosb,
      0.0,

      cosa * sinb * cosc + sina * sinc,
      cosa * sinb * sinc - sina * cosc,
      cosa * cosb,
      0.0,

      0.0,
      0.0,
      0.0,
      1.0,
    ]
  }
  pub fn scale(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
      x, 0.0, 0.0, 0.0,
      0.0, y, 0.0, 0.0,
      0.0, 0.0, z, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]
  }
  pub fn multiply(a: &[f32;16], b: &[f32;16]) -> [f32; 16] {
    let mut dst = [0.0; 16];
    let a00 = a[0];
    let a01 = a[1];
    let a02 = a[2];
    let a03 = a[3];
    let a10 = a[ 4 + 0];
    let a11 = a[ 4 + 1];
    let a12 = a[ 4 + 2];
    let a13 = a[ 4 + 3];
    let a20 = a[ 8 + 0];
    let a21 = a[ 8 + 1];
    let a22 = a[ 8 + 2];
    let a23 = a[ 8 + 3];
    let a30 = a[12 + 0];
    let a31 = a[12 + 1];
    let a32 = a[12 + 2];
    let a33 = a[12 + 3];
    let b00 = b[0];
    let b01 = b[1];
    let b02 = b[2];
    let b03 = b[3];
    let b10 = b[ 4 + 0];
    let b11 = b[ 4 + 1];
    let b12 = b[ 4 + 2];
    let b13 = b[ 4 + 3];
    let b20 = b[ 8 + 0];
    let b21 = b[ 8 + 1];
    let b22 = b[ 8 + 2];
    let b23 = b[ 8 + 3];
    let b30 = b[12 + 0];
    let b31 = b[12 + 1];
    let b32 = b[12 + 2];
    let b33 = b[12 + 3];

    dst[ 0] = a00 * b00 + a10 * b01 + a20 * b02 + a30 * b03;
    dst[ 1] = a01 * b00 + a11 * b01 + a21 * b02 + a31 * b03;
    dst[ 2] = a02 * b00 + a12 * b01 + a22 * b02 + a32 * b03;
    dst[ 3] = a03 * b00 + a13 * b01 + a23 * b02 + a33 * b03;
    dst[ 4] = a00 * b10 + a10 * b11 + a20 * b12 + a30 * b13;
    dst[ 5] = a01 * b10 + a11 * b11 + a21 * b12 + a31 * b13;
    dst[ 6] = a02 * b10 + a12 * b11 + a22 * b12 + a32 * b13;
    dst[ 7] = a03 * b10 + a13 * b11 + a23 * b12 + a33 * b13;
    dst[ 8] = a00 * b20 + a10 * b21 + a20 * b22 + a30 * b23;
    dst[ 9] = a01 * b20 + a11 * b21 + a21 * b22 + a31 * b23;
    dst[10] = a02 * b20 + a12 * b21 + a22 * b22 + a32 * b23;
    dst[11] = a03 * b20 + a13 * b21 + a23 * b22 + a33 * b23;
    dst[12] = a00 * b30 + a10 * b31 + a20 * b32 + a30 * b33;
    dst[13] = a01 * b30 + a11 * b31 + a21 * b32 + a31 * b33;
    dst[14] = a02 * b30 + a12 * b31 + a22 * b32 + a32 * b33;
    dst[15] = a03 * b30 + a13 * b31 + a23 * b32 + a33 * b33;
    dst
  }
  pub fn transpose(src: &[f32;16]) -> [f32;16] {
    let mut dst = [0.0; 16];
    for i in 0..4 {
      for j in 0..4 {
        dst[i*4 + j] = src[j*4 + i];
      }
    }
    dst
  }
  // helpers for inverting matrix
  fn determinant_3x3(m: &[f32; 9]) -> f32 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) -
    m[1] * (m[3] * m[8] - m[5] * m[6]) +
    m[2] * (m[3] * m[7] - m[4] * m[6])
  }
  fn cofactor_4x4(m: &[f32; 16], row: usize, col: usize) -> f32 {
    let mut submatrix = [0.0; 9];
    let mut sub_index = 0;
    for i in 0..4 {
      if i == row { continue; }
      for j in 0..4 {
        if j == col { continue; }
        submatrix[sub_index] = m[i * 4 + j];
        sub_index += 1;
      }
    }
    Self::determinant_3x3(&submatrix) * if (row + col) % 2 == 0 { 1.0 } else { -1.0 }
  }
  fn determinant_4x4(m: &[f32; 16]) -> f32 {
    let mut det = 0.0;
    for i in 0..4 {
      det += m[i] * Self::cofactor_4x4(m, 0, i);
    }
    det
  }
  fn adjugate_4x4(m: &[f32; 16]) -> [f32; 16] {
    let mut adjugate = [0.0; 16];
    for i in 0..4 {
      for j in 0..4 {
        adjugate[j * 4 + i] = Self::cofactor_4x4(m, i, j);
      }
    }
    adjugate
  }
  pub fn inverse(src: &[f32;16]) -> [f32; 16] {
    let det = Self::determinant_4x4(src);
    if det == 0.0 {
      println!("ERR: cannot inverse matrix with determinant of 0 - returning identity");
      let idm = Mat4::identity();
      return idm.as_col_major_array();
    }

    let adj = Self::adjugate_4x4(src);
    let mut dst = [0.0; 16];
    for i in 0..16 {
      dst[i] = adj[i] / det;
    }

    dst
  }
  pub fn view_rot(cam: &Vec3, target: &Vec3, up: &Vec3) ->  [f32; 16] {
    let fwd = (*cam - *target).normalize();
    let right = up.cross(fwd).normalize();
    let n_up = fwd.cross(right).normalize();
    [
      right.x, n_up.x, fwd.x, 0.0,
      right.y, n_up.y, fwd.y, 0.0,
      right.z, n_up.z, fwd.z, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]
  }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Vec4 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}
impl Vec4 {
  pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
    Self { x, y, z, w }
  }
  pub fn from_array(arr: [f32; 4]) -> Self {
    Self { x: arr[0], y: arr[1], z: arr[2], w: arr[3] }
  }
  pub fn as_array(&self) -> [f32; 4] {
    [self.x, self.y, self.z, self.w]
  }
  pub fn normalize(&self) -> Vec4 {
    let n = self.magnitude();
    if n < 0.00001 { return Vec4::new(0.0, 0.0, 0.0, 0.0) };
    Vec4::new(self.x / n, self.y / n, self.z / n, self.w / n)
  }
  pub fn magnitude(&self) -> f32 {
    f32::sqrt(
      self.x * self.x + self.y * self.y + 
      self.z * self.z + self.w * self.w
    )
  }
  pub fn quat_from_axis_angle(axis: Vec3, a_rad: f32) -> Self {
    Self {
      x: axis.x * f32::sin(a_rad / 2.0),
      y: axis.y * f32::sin(a_rad / 2.0),
      z: axis.z * f32::sin(a_rad / 2.0),
      w: f32::cos(a_rad / 2.0),
    }
  }
}
impl Add for Vec4 {
  type Output = Vec4;
  fn add(self, rhs: Self) -> Self::Output {
    Vec4::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
  }
}
impl AddAssign for Vec4 {
  fn add_assign(&mut self, rhs: Self) {
    self.x += rhs.x;
    self.y += rhs.y;
    self.z += rhs.z;
    self.w += rhs.w;
  }
}
impl Sub for Vec4 {
  type Output = Vec4;
  fn sub(self, rhs: Self) -> Self::Output {
    Vec4::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w)
  }
}
impl SubAssign for Vec4 {
  fn sub_assign(&mut self, rhs: Self) {
    self.x -= rhs.x;
    self.y -= rhs.y;
    self.z -= rhs.z;
    self.w -= rhs.w;
  }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Vec3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}
impl Vec3 {
  pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
    Self { x, y, z }
  }
  pub fn from_array(arr: [f32; 3]) -> Self {
    Self { x: arr[0], y: arr[1], z: arr[2] }
  }
  pub fn as_array(&self) -> [f32; 3] {
    [self.x, self.y, self.z]
  }
  pub fn dot(&self, rhs: Vec3) -> f32 {
    let mut out = self.x * rhs.x;
    out = out + self.y * rhs.y;
    out = out + self.z * rhs.z;
    out
  }
  pub fn cross(&self, rhs: Vec3) -> Vec3 {
    Vec3::new(
      self.y * rhs.z - self.z * rhs.y,
      self.z * rhs.x - self.x * rhs.z,
      self.x * rhs.y - self.y * rhs.x
    )
  }
  pub fn normalize(&self) -> Vec3 {
    let n = self.magnitude();
    if n < 0.00001 { return Vec3::new(0.0, 0.0, 0.0) };
    Vec3::new(self.x / n, self.y / n, self.z / n)
  }
  pub fn magnitude(&self) -> f32 {
    f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
  }
  pub fn xy(&self) -> Vec2 {
    Vec2{ x: self.x, y: self.y }
  }
}
impl Add for Vec3 {
  type Output = Vec3;
  fn add(self, rhs: Self) -> Self::Output {
    Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
  }
}
impl AddAssign for Vec3 {
  fn add_assign(&mut self, rhs: Self) {
    self.x += rhs.x;
    self.y += rhs.y;
    self.z += rhs.z;
  }
}
impl Sub for Vec3 {
  type Output = Vec3;
  fn sub(self, rhs: Self) -> Self::Output {
    Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
  }
}
impl SubAssign for Vec3 {
  fn sub_assign(&mut self, rhs: Self) {
    self.x -= rhs.x;
    self.y -= rhs.y;
    self.z -= rhs.z;
  }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Vec2 { pub x: f32, pub y: f32 }
impl Vec2 {
  pub fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }
  pub fn from_tuple(t: (f32, f32)) -> Self {
    Vec2 {
      x: t.0,
      y: t.1
    }
  }
  pub fn from_u32_tuple(t: (u32, u32)) -> Self {
    Vec2 {
      x: t.0 as f32,
      y: t.1 as f32,
    }
  }
  pub fn as_array(&self) -> [f32; 2] {
    [self.x, self.y]
  }
  pub fn magnitude(&self) -> f32 {
    f32::sqrt(self.x * self.x + self.y * self.y)
  }
  pub fn dot(&self, rhs: Vec2) -> f32 {
    self.x * rhs.x + self.y * rhs.y
  }
}
impl Add for Vec2 {
  type Output = Vec2;
  fn add(self, rhs: Self) -> Self::Output {
    Vec2::new(self.x + rhs.x, self.y + rhs.y)
  }
}
impl AddAssign for Vec2 {
  fn add_assign(&mut self, rhs: Self) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}
impl Sub for Vec2 {
  type Output = Vec2;
  fn sub(self, rhs: Self) -> Self::Output {
    Vec2::new(self.x - rhs.x, self.y - rhs.y)
  }
}
impl SubAssign for Vec2 {
  fn sub_assign(&mut self, rhs: Self) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}
impl Into<[f32; 2]> for Vec2 {
  fn into(self) -> [f32; 2] {
    [self.x, self.y]
  }
}

#[cfg(test)]
mod lin_alg_tests {
  use super::*;
  #[test]
  fn mat4_ortho() {
    let o = Mat4::ortho(0.0, 200.0, 0.0, 100.0, 0.0, 1000.0);
    assert_eq!(o, [
      0.01, 0.0, 0.0, 0.0,
      0.0, -0.02, 0.0, 0.0,
      0.0, 0.0, -0.001, 0.0,
      -1.0, 1.0, 0.0, 1.0, 
    ]);
  }
  #[test]
  fn mat4_persp() {
    let o = Mat4::perspective(80.0, 1.5, 1.0, 1000.0);
    assert_eq!(o, [
      0.79450244, 0.0, 0.0, 0.0,
      0.0, 1.1917536, 0.0, 0.0,
      0.0, 0.0, -1.001001, -1.0,
      0.0, 0.0, -1.001001, 0.0, 
    ]);
  }
  #[test]
  fn mat4_rotate1() {
    let axis = Vec3::new(0.0, 0.0, 1.0);
    let a = Mat4::rotate(&axis, 30.0);
    let b = Mat4::rotate_euler(0.0, 0.0, 30.0);
    assert_eq!(a, b);
  }
  #[test]
  fn mat4_rotate2() {
    let a = Mat4::rotate(&Vec3::new(0.0, 1.0, 0.0), 45.0);
    let b = Mat4::rotate_euler(0.0, 45.0, 0.0);
    assert_eq!(a, b);
  }
  #[test]
  fn mat4_rotate3() {
    let a = Mat4::rotate(&Vec3::new(1.0, 0.0, 0.0), 60.0);
    let b = Mat4::rotate_euler(60.0, 0.0, 0.0);
    assert_eq!(a, b);
  }
  #[test]
  fn mat4_transpose() {
    let o = Mat4::transpose(&[
      1.0, 2.0, 3.0, 4.0,
      5.0, 6.0, 7.0, 8.0,
      9.0, 3.0, 2.0, 4.0,
      0.0, 1.0, 2.0, 5.0
    ]);
    let ans: [f32; 16] = [
      1.0, 5.0, 9.0, 0.0,
      2.0, 6.0, 3.0, 1.0,
      3.0, 7.0, 2.0, 2.0,
      4.0, 8.0, 4.0, 5.0
    ];
    assert_eq!(o, ans);
  }
  #[test]
  fn mat4_inverse() {
    let o = Mat4::inverse(&[
      1.0, 2.0, 3.0, 4.0,
      5.0, 6.0, 7.0, 8.0,
      9.0, 3.0, 2.0, 4.0,
      0.0, 1.0, 2.0, 5.0
    ]);
    let ans: [f32; 16] = [
      0.825, -0.325, 0.2, -0.3,
      -4.025, 1.525, -0.4, 1.1,
      3.575, -1.075, 0.2, -1.3,
      -0.625, 0.125, 0.0, 0.5
    ];
    assert_eq!(o, ans);
  }
  #[test]
  fn mvp_test() {
    // model
    let model_r = Mat4::rotate(&Vec3::new(0.0, 1.0, 0.0), 0.0);
    let model_t = Mat4::translate(0.0, 0.0, 400.0);
    let model = Mat4::multiply(&model_r, &model_t);
    // view
    let view_t = Mat4::translate(-0.0, -0.0, -200.0);
    let view_r = Mat4::view_rot(
      &Vec3::new(0.0, 0.0, 200.0), &Vec3::new(0.0, 0.0, 0.0), &Vec3::new(0.0, 1.0, 0.0)
    );
    let view = Mat4::multiply(&view_r, &view_t);
    // proj
    let proj = Mat4::perspective(60.0, 600.0/800.0, 1.0, 1000.0);
    // mvp
    let mvp_temp = Mat4::multiply(&model, &view);
    let mvp = Mat4::multiply(&proj, &mvp_temp);
    let mvp_mat = Mat4::from_col_major(mvp);
    let p = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let clip_p = Mat4::multiply_vec4(&mvp_mat, &p);

    println!("mvp: {} x p: {p:?} = clip_p: {clip_p:.4?}\n", mvp_mat.to_string());
    assert!(true); // use cargo test mvp_test -- --nocapture
  }
}