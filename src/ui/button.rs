use crate::renderer::*;
use crate::math::*;
use crate::{vec2f, vec3f};

#[derive(Debug, Clone)]
pub struct UiButton {
  pipeline_id: RPipelineIdV2,
  object_id: RObjectIdV2,
  text_tx: RTextureIdV2,
  pub position: Vec3,
  pub size: Vec2,
  pub color: RColor,
  pub hover_color: RColor,
  pub text: String,
  pub radius: f32,
  updated: bool,
}
impl UiButton {
  // initialize fns
  pub fn new(renderer: &mut RendererV2, size: Vec2) -> Self {
    let tx_id = renderer.add_texture(size.x as u32, size.y as u32, None, false);
    let pipeline_id = renderer.add_pipeline(RPipelineSetupV2 {
      shader: RShader::Custom(include_str!("../embed_assets/button.wgsl")),
      ..Default::default()
    });

    let rect_data = Primitives::rect_indexed(size.x, size.y, 0.0);
    let rect_id = renderer.add_object(RObjectSetupV2 {
      pipeline_id,
      texture1_id: Some(tx_id),
      vertex_data: rect_data.0,
      indices: rect_data.1,
      ..Default::default()
    });
    Self {
      pipeline_id,
      object_id: rect_id,
      text_tx: tx_id,
      position: vec3f!(0.0, 0.0, 0.0),
      size,
      color: RColor::GRAY,
      hover_color: RColor::WHITE,
      radius: size.y / 4.0,
      text: String::new(),
      updated: false,
    }
  }
  pub fn at(mut self, pos: Vec3) -> Self {
    self.position = pos;
    self
  }
  pub fn with_colors(mut self, color: RColor, hover_color: RColor) -> Self {
    self.color = color;
    self.hover_color = hover_color;
    self
  }
  pub fn with_radius(mut self, radius: f32) -> Self {
    self.radius = radius;
    self
  }
  pub fn with_text(mut self, renderer: &mut RendererV2, text: String, font_size: f32, color: RColor) -> Self {
    self.text = text;
    // measure text size to find proper base point
    let srect = renderer.measure_str_size(0, &self.text, font_size);
    let bx = (self.size.x - srect.width - self.radius) / 2.0;
    let by = (self.size.y - srect.max_y - srect.min_y) / 2.0;
    renderer.redraw_texture_with_str(0, self.text_tx, &self.text, font_size, color, vec2f!(bx, by), 2.0);
    self
  }
  // update fns
  pub fn update(
    &mut self,
    renderer: &mut RendererV2,
    camera: Option<&RCamera>,
    mouse_pos: Vec2,
    screen_size: Vec2,
  ) {
    // update logic
    let mouse_pos_world = screen_to_world_2d(&mouse_pos, &screen_size);
    let hovered = point_in_rect(&mouse_pos_world, &self.position.xy(), &self.size);
    let active_color = if hovered { self.hover_color } else { self.color };

    // update render object
    let mut update = RObjectUpdate::default()
      .with_position(self.position)
      .with_color(active_color)
      .with_round_border(self.size, self.radius);
    if let Some(cam) = camera {
      update = update.with_camera(cam);
    }
    renderer.update_object(self.object_id, update);
  }
  pub fn get_pipeline(&self) -> RPipelineIdV2 {
    self.pipeline_id
  }
}