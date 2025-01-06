use super::*;

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

#[cfg(test)]
mod math_util_tests {
  use super::*;
  use crate::vec2f;

  #[test]
  fn stw_2d() {
    let o = screen_to_world_2d(&vec2f!(400.0, 300.0), &vec2f!(800.0, 600.0));
    assert_eq!(o, vec2f!(0.0, 0.0));
  }
}