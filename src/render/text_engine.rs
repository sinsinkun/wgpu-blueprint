use cosmic_text::{Attrs, Buffer, Color, Edit, Editor, Font, FontSystem, Metrics, Shaping, SwashCache};
use wgpu::{
  Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, 
  Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages
};

use super::RenderColor;

#[derive(Debug)]
pub struct TextEngine {
  font_system: FontSystem,
  swash_cache: SwashCache,
}
impl TextEngine {
  pub fn new() -> Self {
    let font_system = FontSystem::new();
    let swash_cache = SwashCache::new();

    Self {
      font_system,
      swash_cache,
    }
  }
  pub fn create_texture(
    &mut self, device: &Device, queue: &Queue,
    text: &str, text_size: f32, text_color: [u8; 4],
    fixed_width: Option<f32>, fixed_height: Option<f32>
  ) -> Texture {
    // create text buffer for cosmic-text
    let mut text_buffer = Buffer::new(&mut self.font_system, Metrics::new(text_size, f32::ceil(text_size * 1.05)));
    text_buffer.set_size(&mut self.font_system, fixed_width, fixed_height);
    text_buffer.set_text(&mut self.font_system, text, &Attrs::new(), Shaping::Advanced);
    // texture sizing
    let mut texture_size = Extent3d {
      width: 100,
      height: 100,
      depth_or_array_layers: 1,
    };
    let mut auto_width: f32 = 0.0;
    let mut auto_height: f32 = 0.0;
    for run in text_buffer.layout_runs() {
      if run.line_w > auto_width { auto_width = run.line_w; }
      if run.line_height > 0.0 { auto_height += run.line_height; }
    }
    texture_size.width = match fixed_width {
      Some(w) => f32::ceil(w) as u32,
      None => f32::ceil(auto_width) as u32
    };
    texture_size.height = match fixed_height {
      Some(h) => f32::ceil(h) as u32,
      None => f32::ceil(auto_height) as u32
    };
    // create wgpu texture + bytedata buffer
    let texture = device.create_texture(&TextureDescriptor {
      size: texture_size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8Unorm,
      usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
      label: Some("pixel_texture"),
      view_formats: &[]
    });
    let mut pixel_buffer: Vec<u8> = vec![0; (texture_size.width * texture_size.height * 4) as usize];
    text_buffer.draw(
      &mut self.font_system, 
      &mut self.swash_cache,
      Color::rgba(text_color[0], text_color[1], text_color[2], text_color[3]),
      |x, y, _w, _h, color| {
        let idx = (y * texture_size.width as i32 + x) * 4;
        if idx < 0 { return; }
        let idx = idx as usize;
        if idx > pixel_buffer.len() { return; }
        // draw pixel into buffer
        pixel_buffer[idx] = color.r();
        pixel_buffer[idx + 1] = color.g();
        pixel_buffer[idx + 2] = color.b();
        pixel_buffer[idx + 3] = color.a();
      }
    );

    queue.write_texture(
      TexelCopyTextureInfo {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      &pixel_buffer,
      TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(4 * texture_size.width),
        rows_per_image: Some(texture_size.height),
      },
      texture_size,
    );

    texture
  }
}