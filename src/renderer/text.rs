#![allow(dead_code)]

use ab_glyph::{Font, FontRef, Glyph, Rect};
use image::{Rgba, RgbaImage};
use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Texture, TextureAspect, TextureFormat};

use crate::vec2f;

use super::{RColor, Vec2};

#[derive(Debug, PartialEq)]
pub enum TextError {
  FileNotFound,
  FileLoadError,
  GlyphOutlineError,
  ExceedsBounds,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StringRect {
  pub width: f32,
  pub min_y: f32,
  pub max_y: f32,
}

// create image of glyph to append onto texture
pub fn load_new_glyph(c: char, color: [u8; 3]) -> Result<(RgbaImage, f32), TextError> {
  // open font
  let font = FontRef::try_from_slice(include_bytes!("../embed_assets/NotoSerifCHB.ttf"))
    .map_err(|_| TextError::FileLoadError)?;

  // declare glyph
  let glyph: Glyph = font.glyph_id(c).with_scale(20.0);

  if let Some(ch) = font.outline_glyph(glyph) {
    // define image bounds
    let bounds: Rect = ch.px_bounds();
    let w = bounds.max.x - bounds.min.x;
    let h = bounds.max.y - bounds.min.y;
    // define image buffer
    let mut img = RgbaImage::new(w as u32, h as u32);

    // write pixels to image
    ch.draw(|x, y, c| {
      let r = color[0];
      let g = color[1];
      let b = color[2];
      let a: u8 = f32::floor(c * 255.0) as u8;
      img.put_pixel(x, y, Rgba([r,g,b,a]));
    });

    Ok((img, bounds.min.y.abs()))
  } else {
    Err(TextError::GlyphOutlineError)
  }
}

// same as load_new_glyph but with cached font data
pub fn load_cached_glyph(font_raw: &Vec<u8>, c: char, size: f32, color: [u8; 3]) -> Result<(RgbaImage, f32), TextError> {
  let font = FontRef::try_from_slice(font_raw).map_err(|_| TextError::FileLoadError)?;
  let glyph: Glyph = font.glyph_id(c).with_scale(size);

  if let Some(ch) = font.outline_glyph(glyph) {
    // define image bounds
    let bounds: Rect = ch.px_bounds();
    let w = bounds.max.x - bounds.min.x;
    let h = bounds.max.y - bounds.min.y;
    // define image buffer
    let mut img = RgbaImage::new(w as u32, h as u32);

    // write pixels to image
    ch.draw(|x, y, c| {
      let r = color[0];
      let g = color[1];
      let b = color[2];
      let a: u8 = f32::floor(c * 255.0) as u8;
      if a < 10 {
        img.put_pixel(x, y, Rgba([0,0,0,0]));
      } else {
        img.put_pixel(x, y, Rgba([r,g,b,a]));
      }
    });

    Ok((img, bounds.min.y.abs()))
  } else {
    Err(TextError::GlyphOutlineError)
  }
}

/// measures string size
/// - min_y will generally output as a negative number
/// - max_y is overflow underneath origin.y
pub fn measure_str_size(font_raw: &Vec<u8>, str: &str, size: f32) -> Result<StringRect, TextError> {
  let font = FontRef::try_from_slice(font_raw).map_err(|_| TextError::FileLoadError)?;
  let mut rect = StringRect{ width: 0.0, min_y: 0.0, max_y: 0.0 };
  for c in str.chars() {
    let glyph: Glyph = font.glyph_id(c).with_scale(size);
    if let Some(ch) = font.outline_glyph(glyph.clone()) {
      let bounds: Rect = ch.px_bounds();
      rect.width += bounds.max.x;
      if bounds.min.y < rect.min_y {
        rect.min_y = bounds.min.y;
      }
      if bounds.max.y > rect.max_y {
        rect.max_y = bounds.max.y;
      }
    } else {
      let w = font.glyph_bounds(&glyph).width();
      rect.width += w;
    }
  }
  Ok(rect)
}

/// draws string onto a texture
/// - creates empty image and writes pixels directly onto image
/// - image is copied onto wgpu texture
pub fn draw_str_on_texture(
  queue: &Queue,
  texture: &mut Texture,
  font_data: &Vec<u8>,
  string: &str,
  size: f32,
  color: RColor,
  base_point: Vec2,
  spacing: f32,
) -> Result<(), TextError> {
  // define font
  let font = FontRef::try_from_slice(font_data).map_err(|_| TextError::FileLoadError)?;
  // define image buffer
  let mut img = RgbaImage::new(texture.width(), texture.height());

  let mut c_pos: [f32; 2] = base_point.into();
  for c in string.chars() {
    let glyph = font.glyph_id(c).with_scale(size);
    if let Some(ch) = font.outline_glyph(glyph.clone()) {
      let bounds = ch.px_bounds();
      let mut x_offset = 0.0;
      let y_offset = bounds.min.y;
      // write pixels to image
      ch.draw(|x, y, c| {
        if x as f32 > x_offset { x_offset = x as f32; }
        let absx = c_pos[0] + x as f32;
        let absy = c_pos[1] + y_offset + y as f32;
        // skip offscreen chars
        if absx < 1.0 || absx >= img.width() as f32 { return; }
        else if absy < 1.0 || absy >= img.height() as f32 { return; }
        // draw pixel
        let r = c * color.r * 255.0;
        let g = c * color.g * 255.0;
        let b = c * color.b * 255.0;
        let a = if c > color.a { color.a * 255.0 } else { c * 255.0 };
        let clr = [r as u8, g as u8, b as u8, a as u8];
        img.put_pixel(absx as u32, absy as u32, Rgba(clr));
      });
      // update position to draw glyph
      c_pos[0] += x_offset + spacing;
    } else {
      let w = font.glyph_bounds(&glyph).width();
      // handling blank space
      c_pos[0] += w + spacing;
    }
  }

  // write img to texture
  let dimensions = img.dimensions();
  let img_size = Extent3d { 
    width: dimensions.0,
    height: dimensions.1,
    depth_or_array_layers: 1
  };
  queue.write_texture(
    ImageCopyTexture {
      texture,
      mip_level: 0,
      origin: Origin3d { x:0, y:0, z:0 },
      aspect: TextureAspect::All,
    },
    &img.as_raw(),
    ImageDataLayout {
      offset: 0,
      bytes_per_row: Some(4 * dimensions.0),
      rows_per_image: Some(dimensions.1),
    },
    img_size
  );

  Ok(())
}

#[derive(Debug, Clone)]
pub struct StringPlacement {
  pub string: String,
  pub size: f32,
  pub color: RColor,
  pub base_point: Vec2,
  pub spacing: f32,
}
impl Default for StringPlacement {
  fn default() -> Self {
    Self {
      string: String::new(),
      size: 18.0,
      color: RColor::WHITE,
      base_point: vec2f!(0.0, 0.0),
      spacing: 2.0,
    }
  }
}

fn place_text_on_img(img: &mut RgbaImage, font: &FontRef, sp: &StringPlacement) {
  let mut c_pos: [f32; 2] = sp.base_point.into();
  for c in sp.string.chars() {
    let glyph = font.glyph_id(c).with_scale(sp.size);
    if let Some(ch) = font.outline_glyph(glyph.clone()) {
      let bounds = ch.px_bounds();
      let mut x_offset = 0.0;
      let y_offset = bounds.min.y;
      // write pixels to image
      ch.draw(|x, y, c| {
        if x as f32 > x_offset { x_offset = x as f32; }
        let absx = c_pos[0] + x as f32;
        let absy = c_pos[1] + y_offset + y as f32;
        // skip offscreen chars
        if absx < 1.0 || absx >= img.width() as f32 { return; }
        else if absy < 1.0 || absy >= img.height() as f32 { return; }
        // draw pixel
        let r = c * sp.color.r * 255.0;
        let g = c * sp.color.g * 255.0;
        let b = c * sp.color.b * 255.0;
        let a = if c > sp.color.a { sp.color.a * 255.0 } else { c * 255.0 };
        let clr = [r as u8, g as u8, b as u8, a as u8];
        img.put_pixel(absx as u32, absy as u32, Rgba(clr));
      });
      // update position to draw glyph
      c_pos[0] += x_offset + sp.spacing;
    } else {
      let w = font.glyph_bounds(&glyph).width();
      // handling blank space
      c_pos[0] += w + sp.spacing;
    }
  }
}

pub(crate) fn draw_full_text_texture(
  queue: &Queue,
  texture: &mut Texture,
  font_data: &Vec<u8>,
  placements: &Vec<StringPlacement>,
) -> Result<(), TextError> {
  // define font
  let font = FontRef::try_from_slice(font_data).map_err(|_| TextError::FileLoadError)?;
  // define image buffer
  let mut img = RgbaImage::new(texture.width(), texture.height());

  // draw text to img per placement
  for p in placements {
    place_text_on_img(&mut img, &font, &p);
  }

  // write img to texture
  let dimensions = img.dimensions();
  let img_size = Extent3d { 
    width: dimensions.0,
    height: dimensions.1,
    depth_or_array_layers: 1
  };
  queue.write_texture(
    ImageCopyTexture {
      texture,
      mip_level: 0,
      origin: Origin3d { x:0, y:0, z:0 },
      aspect: TextureAspect::All,
    },
    &img.as_raw(),
    ImageDataLayout {
      offset: 0,
      bytes_per_row: Some(4 * dimensions.0),
      rows_per_image: Some(dimensions.1),
    },
    img_size
  );
  Ok(())
}

#[cfg(test)]
mod glyph_brush_test {
  use super::*;
  #[test]
  fn glyph_test() {
    let a = load_new_glyph('B', [100, 10, 100]);
    let b = load_new_glyph('o', [100, 10, 100]);
    let c = load_new_glyph('d', [100, 10, 100]);
    let d = load_new_glyph('y', [100, 10, 100]);
    assert!(a.is_ok());
    assert!(b.is_ok());
    assert!(c.is_ok());
    assert!(d.is_ok());
  }
  #[test]
  fn glyph_cached_test() {
    let font = include_bytes!("../embed_assets/NotoSerifCHB.ttf");
    let a = load_cached_glyph(&font.to_vec(), 'B', 18.0, [100, 10, 100]);
    let b = load_cached_glyph(&font.to_vec(), 'o', 18.0, [100, 10, 100]);
    let c = load_cached_glyph(&font.to_vec(), 'd', 18.0, [100, 10, 100]);
    let d = load_cached_glyph(&font.to_vec(), 'y', 18.0, [100, 10, 100]);
    assert!(a.is_ok());
    assert!(b.is_ok());
    assert!(c.is_ok());
    assert!(d.is_ok());
  }
  #[test]
  fn measure_str_test() {
    let font = include_bytes!("../embed_assets/NotoSansCB.ttf");
    let r = measure_str_size(&font.to_vec(), "Hot Dog", 16.0);
    assert!(r.is_ok())
  }
}