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

- Create app_name.rs
- implement AppBase trait for root app struct
```rust
pub struct AppName {
  // app state
}
impl AppBase for AppName {
  fn init(&mut self, sys: SystemInfo, renderer: &mut Renderer) {
    // initialization of state
    // initialize render pipelines
    // initialize objects to render
  }
  fn resize(&mut self, renderer: &mut Renderer, width: u32, height: u32) {
    // optional handler for screen resizing
    // resize any textures meant to maintain screen size/aspect ratio
    // note: asynchronous from update loop
  }
  fn update(&mut self, sys: SystemInfo, renderer: &mut Renderer) -> Vec<RPipelineId> {
    // update state logic
    // update render object state
    // render to textures
    // output pipelines to render to screen
    vec![]
  }
  fn cleanup(&mut self) {
    // optional handler for calls on exit
    // diagonistics/analytics output
    // manually free memory
  }
}
```