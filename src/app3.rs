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
  fn init_cir(&mut self, renderer: &mut Renderer, radius: f32, color: RColor, pos: Vec2, velocity: Vec2) {
    let model = Primitives::reg_polygon(radius, 32, 0.0);
    let obj = renderer.add_object(RObjectSetup {
      pipeline_id: self.pipe,
      vertex_data: model,
      ..Default::default()
    });
    self.circles.push(Circle {
      obj_id: obj,
      color,
      pos,
      radius,
      velocity,
    });
  }
  fn spawn_new_cir(&mut self, sys: &SystemInfo, renderer: &mut Renderer) {
    // spawn new cir in mouse dir
    let origin = vec2f!(sys.win_size.x / 2.0, sys.win_size.y / 2.0);
    let mouse_position = sys.m_inputs.position - origin;
    let mut pos = self.circles[0].pos;
    pos += (mouse_position - pos).normalize() * 60.0;
    let vel = vec2f!(0.0, 0.0);
    self.init_cir(renderer, 60.0, RColor::RED, pos, vel);
  }
  fn render_cir(&self, renderer: &mut Renderer) {
    for cir in &self.circles {
      renderer.update_object(cir.obj_id, RObjectUpdate::default()
        .with_color(cir.color)
        .with_position(vec3f!(cir.pos.x, cir.pos.y, 0.0))
      );
    }
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
    self.init_cir(renderer, 60.0, RColor::RED, vec2f!(-100.0, -100.0), vec2f!(40.0, 40.0));
    self.init_cir(renderer, 50.0, RColor::BLUE, vec2f!(100.0, 100.0), vec2f!(0.0, 0.0));
    self.init_cir(renderer, 50.0, RColor::BLUE, vec2f!(-100.0, 100.0), vec2f!(0.0, 0.0));
    self.init_cir(renderer, 50.0, RColor::BLUE, vec2f!(100.0, -100.0), vec2f!(0.0, 0.0));
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // spawn new cirs
    for input in sys.kb_inputs {
      if input.0 == &KeyCode::Space && input.1 == &MKBState::Pressed {
        self.spawn_new_cir(&sys, renderer);
      }
    }
    // mouse influence
    for cir in &mut self.circles {
      if cir.color == RColor::RED {
        let origin = vec2f!(sys.win_size.x / 2.0, sys.win_size.y / 2.0);
        let mouse_position = sys.m_inputs.position - origin;
        cir.velocity += (mouse_position - cir.pos).normalize();
      }
    }
    // collisions
    let l = self.circles.len();
    for i in 0..l {
      let (a, b) = self.circles.split_at_mut(i);
      if let Some(cir1) = a.last_mut() {
        for cir2 in b {
          update_cir(cir1, cir2, &sys);
        }
      };
    }
    // rendering
    self.render_cir(renderer);
    vec![self.pipe]
  }
}

fn update_cir(cir1: &mut Circle, cir2: &mut Circle, sys: &SystemInfo) {
  let distance = cir1.pos - cir2.pos;
  // collision check
  if distance.magnitude() < cir1.radius + cir2.radius {
    let new_magnitude = cir1.velocity.magnitude() + cir2.velocity.magnitude();
    let new_dir = (cir2.pos - cir1.pos).normalize();
    if cir1.color == cir2.color {
      cir1.velocity += new_dir * 1.0 * new_magnitude;
      cir2.velocity += new_dir * -1.0 * new_magnitude;
    }
    cir1.velocity += new_dir * -1.2 * new_magnitude;
    cir2.velocity += new_dir * 1.2 * new_magnitude;
  }
  // bounce on walls
  let origin = vec2f!(sys.win_size.x / 2.0, sys.win_size.y / 2.0);
  let screen_pos1 = cir1.pos + origin;
  if screen_pos1.x < 0.0 || screen_pos1.x > sys.win_size.x || screen_pos1.y < 0.0 || screen_pos1.y > sys.win_size.y {
    cir1.velocity = cir1.velocity * -0.9;
    cir1.pos += cir1.velocity * sys.frame_delta.as_secs_f32();
  }
  let screen_pos2 = cir2.pos + origin;
  if screen_pos2.x < 0.0 || screen_pos2.x > sys.win_size.x || screen_pos2.y < 0.0 || screen_pos2.y > sys.win_size.y {
    cir2.velocity = cir2.velocity * -0.9;
    cir2.pos += cir2.velocity * sys.frame_delta.as_secs_f32();
  }
  // cap velocity
  if cir1.velocity.magnitude() > 200.0 {
    cir1.velocity = cir1.velocity.normalize() * 200.0;
  }
  if cir2.velocity.magnitude() > 200.0 {
    cir2.velocity = cir2.velocity.normalize() * 200.0;
  }
  // finalize position
  cir1.pos += cir1.velocity * sys.frame_delta.as_secs_f32();
  cir2.pos += cir2.velocity * sys.frame_delta.as_secs_f32();
}