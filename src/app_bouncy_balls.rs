use crate::*;
use renderer::*;

const RED: RColor = RColor { r: 1.0, g: 0.0, b: 0.0, a: 0.8 };
const BLUE: RColor = RColor { r: 0.0, g: 0.0, b: 1.0, a: 0.8 };

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
impl App {
  fn create_pipe(&mut self, renderer: &mut Renderer) {
    renderer.set_clear_color(RColor::hsv(0.65, 0.4, 0.02));
    let pipe = renderer.add_pipeline(RPipelineSetup {
      shader: RShader::FlatColor,
      ..Default::default()
    });
    self.pipe = pipe;
  }
  fn create_cir(&mut self, renderer: &mut Renderer, radius: f32, color: RColor, pos: Vec2, velocity: Vec2) {
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
  fn spawn_new_cir(&mut self, sys: &SystemInfo, renderer: &mut Renderer, pos: Vec2) {
    // spawn new cir in mouse dir
    let origin = vec2f!(sys.win_size.x / 2.0, sys.win_size.y / 2.0);
    let mouse_position = sys.m_inputs.position - origin;
    let delta = (mouse_position - pos).normalize();
    let n_pos = pos + delta * 20.0;
    let vel = vec2f!(0.0, 0.0);
    self.create_cir(renderer, 20.0, RED, n_pos, vel);
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
  fn new() -> Self {
    Self {
      pipe: RPipelineId(0),
      circles: Vec::new(),
    }
  }
  fn init(&mut self, _sys: SystemInfo, renderer: &mut Renderer) {
    self.create_pipe(renderer);
    self.create_cir(renderer, 20.0, RED, vec2f!(-100.0, -100.0), vec2f!(0.0, 0.0));
    self.create_cir(renderer, 30.0, BLUE, vec2f!(100.0, 100.0), vec2f!(0.0, 0.0));
    self.create_cir(renderer, 30.0, BLUE, vec2f!(-100.0, 100.0), vec2f!(0.0, 0.0));
    self.create_cir(renderer, 30.0, BLUE, vec2f!(100.0, -100.0), vec2f!(0.0, 0.0));
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    let origin = sys.win_size * 0.5;
    let mouse_pos = sys.m_pos_world_space_2d(None);
    // spawn new cirs
    for input in sys.kb_inputs {
      if input.0 == &KeyCode::Space && input.1 == &MKBState::Pressed {
        let mut pos_col = Vec::new();
        for cir in &self.circles {
          if cir.color == RED { pos_col.push(cir.pos); }
        }
        for pos in pos_col {
          self.spawn_new_cir(&sys, renderer, pos);
        }
      }
    }
    // collisions
    let l = self.circles.len();
    for i in 0..l {
      let (a, b) = self.circles.split_at_mut(i);
      if let Some(cir1) = a.last_mut() {
        for cir2 in b {
          collide_2_cirs(cir1, cir2, &sys);
        }
      };
    }
    // other considerations for pos/velocity
    for cir in &mut self.circles {
      // follow mouse
      if cir.color == RED {
        let mouse_dir = mouse_pos - cir.pos;
        cir.velocity += mouse_dir * 0.02;
      }
      // wall collisions
      let screen_pos = cir.pos + origin;
      if screen_pos.x < 0.0 && cir.velocity.x < 0.0 { cir.velocity.x = -1.0 * cir.velocity.x };
      if screen_pos.y < 0.0 && cir.velocity.y < 0.0 { cir.velocity.y = -1.0 * cir.velocity.y };
      if screen_pos.x > sys.win_size.x && cir.velocity.x > 0.0 { cir.velocity.x = -1.0 * cir.velocity.x };
      if screen_pos.y > sys.win_size.y && cir.velocity.y > 0.0 { cir.velocity.y = -1.0 * cir.velocity.y };
      // cap max velocity
      if cir.velocity.magnitude() > 60.0 {
        cir.velocity = cir.velocity.normalize() * 60.0;
      }
      // finalize position
      cir.pos += cir.velocity * sys.time_delta();
    }

    // draw to screen
    self.render_cir(renderer);
    vec!(self.pipe)
  }
}

fn collide_2_cirs(cir1: &mut Circle, cir2: &mut Circle, sys: &SystemInfo) {
  let pos_delta = cir1.pos - cir2.pos;
  let min_distance = cir1.radius + cir2.radius;
  let new_magnitude = cir1.velocity.magnitude() + cir2.velocity.magnitude();
  let new_dir = (cir2.pos - cir1.pos).normalize();
  // controlled circles
  if pos_delta.magnitude() < min_distance && cir1.color == RED && cir2.color == RED {
    cir1.pos += sys.time_delta() * pos_delta;
    cir2.pos += sys.time_delta() * -1.0 * pos_delta;
    cir1.velocity += new_dir * -0.2 * new_magnitude;
    cir2.velocity += new_dir *  0.2 * new_magnitude;
  }
  // regular collisions
  else if pos_delta.magnitude() < min_distance {
    cir1.velocity += new_dir * -0.5 * new_magnitude;
    cir2.velocity += new_dir *  0.5 * new_magnitude;
  }
}