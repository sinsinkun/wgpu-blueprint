use crate::renderer::*;
use crate::math::*;
use crate::{vec2f, vec3f};

#[derive(Debug, Clone)]
pub struct UiButton {
  object_id: RObjectId,
  text_tx: RTextureId,
  pub position: Vec3,
  pub size: Vec2,
  pub color: RColor,
  pub hover_color: RColor,
  pub text: String,
  pub radius: f32,
}
impl UiButton {
  pub fn new_pipeline(renderer: &mut Renderer) -> RPipelineId {
    renderer.add_pipeline(RPipelineSetup {
      shader: RShader::Custom(include_str!("../embed_assets/button.wgsl")),
      ..Default::default()
    })
  }
  // initialize fns
  pub fn new(renderer: &mut Renderer, btn_pipe: &RPipelineId, size: Vec2) -> Self {
    let tx_id = renderer.add_texture(size.x as u32, size.y as u32, None, false);
    let rect_data = Primitives::rect_indexed(size.x, size.y, 0.0);
    let rect_id = renderer.add_object(RObjectSetup {
      pipeline_id: *btn_pipe,
      texture1_id: Some(tx_id),
      vertex_data: rect_data.0,
      indices: rect_data.1,
      ..Default::default()
    });
    Self {
      object_id: rect_id,
      text_tx: tx_id,
      position: vec3f!(0.0, 0.0, 0.0),
      size,
      color: RColor::GRAY,
      hover_color: RColor::WHITE,
      radius: size.y / 4.0,
      text: String::new(),
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
  pub fn with_text(mut self, renderer: &mut Renderer, text: String, font_size: f32, color: RColor) -> Self {
    self.text = text;
    // measure text size to find proper base point
    let srect = renderer.measure_str_size(1, &self.text, font_size);
    let bx = (self.size.x - srect.width - self.radius) / 2.0;
    let by = (self.size.y - srect.max_y - srect.min_y) / 2.0;
    renderer.redraw_texture_with_str(1, self.text_tx, &self.text, font_size, color, vec2f!(bx, by), 2.0);
    self
  }
  // update fns
  pub fn update(
    &mut self,
    renderer: &mut Renderer,
    camera: Option<&RCamera>,
    mouse_pos: Vec2,
    screen_size: Vec2,
    action_available: bool,
  ) -> bool {
    let mut aa = action_available;
    let mut active_color = self.color;
    if aa {
      // update logic
      let mouse_pos_world = screen_to_world_2d(&mouse_pos, &screen_size);
      let hovered = point_in_rect(&mouse_pos_world, &self.position.xy(), &self.size);
      if hovered {
        aa = false;
        active_color = self.hover_color;
      }
    }

    // update render object
    let mut update = RObjectUpdate::default()
      .with_position(self.position)
      .with_color(active_color)
      .with_round_border(self.size, self.radius);
    if let Some(cam) = camera {
      update = update.with_camera(cam);
    }
    renderer.update_object(self.object_id, update);

    // return if action was consumed
    aa
  }
}