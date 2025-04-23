# raylib_imgui

[![Crates.io Version](https://img.shields.io/crates/v/raylib_imgui)][crates.io]

[crates.io]: https://crates.io/crates/raylib_imgui

# <img src="https://github.com/raysan5/raylib/raw/master/logo/raylib_logo_animation.gif" width="64" alt="raylib logo animated"> A Raylib integration with DearImGui in Rust!

## Based on [rlImgui](https://github.com/raylib-extras/rlImGui)

### In main.rs:
```rust
use raylib_imgui::RaylibGui;
use raylib::prelude::*;

fn main() {
  let (mut rl, thread) = raylib::init()
    .size(800, 600)
    .title("Demo window")
    .build();

  let mut gui = RaylibGui::new(&mut rl, &thread);
  let mut open = true;

  while !rl.window_should_close() {
    let ui = gui.begin(&mut rl);
    ui.show_demo_window(&mut open);
    // Draw ImGui stuff here

    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::WHITE);
    // Draw raylib stuff here

    gui.end();
  }
}
```
