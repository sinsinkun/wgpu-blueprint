use crate::renderer::*;
use crate::{vec2f, vec3f};

#[derive(Debug, Clone)]
pub struct UiButton {
  pipeline_id: RPipelineId,
  object_id: RObjectId,
  text_tx: RTextureId,
  pub position: Vec3,
  pub size: Vec2,
  pub color: RColor,
  pub text: String,
  pub radius: f32,
}
impl UiButton {
  // initialize fns
  pub fn new(renderer: &mut Renderer, size: Vec2) -> Self {
    let tx_id = renderer.add_texture(size.x as u32, size.y as u32, None, false);
    let pipeline_id = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::Custom(include_str!("../embed_assets/button.wgsl")),
      max_obj_count: 1,
      texture1_id: Some(tx_id),
      ..Default::default()
    });

    let rect_data = Primitives::rect_indexed(size.x, size.y, 0.0);
    let rect_id = renderer.add_object(RObjectSetup {
      pipeline_id,
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
      radius: size.y / 4.0,
      text: String::new(),
    }
  }
  pub fn at(mut self, pos: Vec3) -> Self {
    self.position = pos;
    self
  }
  pub fn with_color(mut self, color: RColor) -> Self {
    self.color = color;
    self
  }
  pub fn with_radius(mut self, radius: f32) -> Self {
    self.radius = radius;
    self
  }
  pub fn with_text(mut self, renderer: &mut Renderer, text: String, font_size: f32, color: RColor) -> Self {
    self.text = text;
    // todo: measure text size to find proper base point
    renderer.redraw_texture_with_str(self.text_tx, &self.text, font_size, color, vec2f!(5.0, 20.0), 2.0);
    self
  }
  // update fns
  pub fn update(&mut self, renderer: &mut Renderer, camera: Option<&RCamera>) {
    // update logic

    // update render object
    let mut update = RObjectUpdate::obj(self.object_id)
      .with_position(vec3f!(self.size.x, self.size.y, self.position.z))
      .with_color(self.color)
      .with_round_border(self.size, self.radius);
    if let Some(cam) = camera {
      update = update.with_camera(cam);
    }
    renderer.update_object(update);
  }
  pub fn get_pipeline(&self) -> RPipelineId {
    self.pipeline_id
  }
}