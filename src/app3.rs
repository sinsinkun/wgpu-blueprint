use crate::*;
use renderer::*;

#[derive(Debug)]
struct Circle {
  obj_id: RObjectId,
  color: RColor,
  pos: Vec2,
  radius: f32,
  velocity: Vec2,
}

#[derive(Debug)]
pub struct App {
  pipe: RPipelineId,
  circles: Vec<Circle>,
}
impl Default for App {
  fn default() -> Self {
    Self {
      pipe: RPipelineId(0),
      circles: Vec::new(),
    }
  }
}
impl App {
  fn init_cir(&mut self, renderer: &mut Renderer, color: RColor, pos: Vec2, velocity: Vec2) {
    let model = Primitives::reg_polygon(50.0, 32, 0.0);
    let obj = renderer.add_object(RObjectSetup {
      pipeline_id: self.pipe,
      vertex_data: model,
      ..Default::default()
    });
    self.circles.push(Circle {
      obj_id: obj,
      color,
      pos,
      radius: 50.0,
      velocity,
    });
  }
}
impl AppBase for App {
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    renderer.set_clear_color(RColor::hsv(0.65, 0.4, 0.02));
    let pipe = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::FlatColor,
      ..Default::default()
    });
    self.pipe = pipe;
    self.init_cir(renderer, RColor::RED, vec2f!(-100.0, -100.0), vec2f!(40.0, 40.0));
    self.init_cir(renderer, RColor::BLUE, vec2f!(100.0, 100.0), vec2f!(0.0, 0.0));
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // update positions
    let (a, b) = self.circles.split_at_mut(1);
    let cir1 = &mut a[0];
    let cir2 = &mut b[0];
    let distance = cir1.pos - cir2.pos;
    // collision check
    if distance.magnitude() < cir1.radius + cir2.radius {
      let new_magnitude = cir1.velocity.magnitude() + cir2.velocity.magnitude();
      let new_dir = (cir1.velocity - cir2.velocity).normalize();
      cir1.velocity = new_dir * -1.0 * new_magnitude;
      cir2.velocity = new_dir * new_magnitude;
    }
    // mouse influence
    let origin = vec2f!(sys.win_size.x / 2.0, sys.win_size.y / 2.0);
    let mouse_position = sys.m_inputs.position - origin;
    cir2.velocity += (mouse_position - cir2.pos) * 0.001;
    // bounce on walls
    let screen_pos1 = cir1.pos + origin;
    if screen_pos1.x < 0.0 || screen_pos1.x > sys.win_size.x || screen_pos1.y < 0.0 || screen_pos1.y > sys.win_size.y {
      cir1.velocity = cir1.velocity * -0.8;
    }
    let screen_pos2 = cir2.pos + origin;
    if screen_pos2.x < 0.0 || screen_pos2.x > sys.win_size.x || screen_pos2.y < 0.0 || screen_pos2.y > sys.win_size.y {
      cir2.velocity = cir2.velocity * -0.8;
    }
    // cap velocity
    if cir1.velocity.magnitude() > 400.0 {
      cir1.velocity = cir1.velocity.normalize() * 400.0;
    }
    if cir2.velocity.magnitude() > 400.0 {
      cir2.velocity = cir2.velocity.normalize() * 400.0;
    }
    // finalize position
    cir1.pos += cir1.velocity * sys.frame_delta.as_secs_f32();
    cir2.pos += cir2.velocity * sys.frame_delta.as_secs_f32();

    // rendering
    for cir in &self.circles {
      renderer.update_object(cir.obj_id, RObjectUpdate::default()
        .with_color(cir.color)
        .with_position(vec3f!(cir.pos.x, cir.pos.y, 0.0))
      );
    }

    vec![self.pipe]
  }
}