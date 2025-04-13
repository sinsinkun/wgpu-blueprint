# Wgpu Blueprint

A general purpose application library, built on top of winit + wgpu.

Continuation of previous project: https://github.com/sinsinkun/wgpu-app

Text rendering has been improved upon, but joint based animations may
have regressed. Custom shader buffers have been moved into group(0) and
limited to 64 bytes, but can be expanded upon later.

IME and international language rendering is implemented, but needs improvement
for usability.

Currently still a testing ground for features, but considering packaging up as
a generalized crate that can be used as an actual library.

## How to use

- Import `SceneBase`, `WinitConfig`, and `launch` components into main.rs
- Setup a new scene by implementing `SceneBase` for a new struct
- in `fn main()`, call `launch(winitConfig, scene_collection)`
- scene_collection is a `Vec<Box<dyn SceneBase>>` collection of scene structs
- scenes implementing `SceneBase` will be passed system data and gpu accessors 

```rust
pub struct TestScene {
  // app state
}
impl SceneBase for TestScene {
  fn new() -> Self {
    // initialize state without system data
    // (use Option<T> for system dependent state)
  }
  fn init(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    // initialization of state
    // initialize render pipelines
    // initialize objects to render
  }
  fn resize(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess, width: u32, height: u32) {
    // optional handler for screen resizing
    // resize any textures meant to maintain screen size/aspect ratio
    // update cameras if necessary
    // note: asynchronous from update loop
  }
  fn update(&mut self, sys: &mut SystemAccess, gpu: &mut GpuAccess) {
    // update state logic
    // update render object state
    // render to textures
    // render to screen
  }
  fn cleanup(&mut self) {
    // optional handler for calls on exit
    // diagonistics/analytics output
    // manually free memory
  }
}

fn main() {
  launch(
    WinitConfig {
      title: "Test Window".to_owned(),
      ..Default::default()
    },
    vec![ Box::new(TestScene::new()) ]
  );
}
```
