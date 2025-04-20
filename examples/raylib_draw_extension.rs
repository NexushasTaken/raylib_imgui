use raylib_imgui::RaylibGui;
use raylib::prelude::*;

impl<T> RaylibDrawUiExt for T where T: RaylibDraw {}
trait RaylibDrawUiExt: RaylibDraw {
  fn on_ui(&self, gui: &mut RaylibGui, open: &mut bool) {
    let ui = gui.new_frame();

    ui.show_demo_window(open);
    gui.render();
  }
}

fn main() {
  let (mut rl, thread) = raylib::init()
    .size(800, 600)
    .title("Using raylib draw")
    .build();

  let mut gui = RaylibGui::new(&mut rl, &thread);
  let mut open = true;

  while !rl.window_should_close() {
    gui.update(&mut rl);
    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::WHITE);
    d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    d.on_ui(&mut gui, &mut open);
  }
}
