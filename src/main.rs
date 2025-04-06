mod utils;
mod wrapper;
use wrapper::{launch, SceneBase, WinitConfig};
mod render;
mod scene1;
use scene1::Scene1;
mod scene2;
use scene2::Scene2;

fn main() {
  launch(WinitConfig {
    size: (800, 600),
    max_fps: Some(120),
    title: "Re:Blueprint".to_owned(),
    icon: Some("icon.ico".to_owned()),
    ..Default::default()
  }, vec![Box::new(Scene1::new()), Box::new(Scene2::new())]);
}