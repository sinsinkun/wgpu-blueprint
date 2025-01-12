#![allow(unused)]

use crate::math::*;

mod button;
pub use button::*;

#[derive(Debug)]
pub enum UiComponent {
  Button(UiButton)
}