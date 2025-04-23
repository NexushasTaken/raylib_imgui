use ffi;
use imgui::{
  self, internal::RawWrapper, BackendFlags, ClipboardBackend, ConfigFlags,
  Context, DrawCmd, DrawCmdParams, DrawVert, FontConfig, Key, Ui,
};
use raylib::ffi::{GetClipboardText, SetClipboardText};
use raylib::prelude::*;
use std::ffi::{c_char, CStr, CString};
use std::fs::{self, File};
use std::io::Write;

pub struct RaylibGui {
  pub context: Context,

  current_mouse_cursor: Option<imgui::MouseCursor>,

  last_frame_focused: bool,
  last_control_pressed: bool,
  last_shift_pressed: bool,
  last_alt_pressed: bool,
  last_super_pressed: bool,

  keys: [(KeyboardKey, Key); 105],
  gamepad_map: [(GamepadButton, Key); 16],
  gamepad_axis: [(GamepadAxis, Key, Key); 4],
}

pub enum Style {
  Dark,
  Light,
  Classic,
}

struct RaylibClipboardBackend;

impl ClipboardBackend for RaylibClipboardBackend {
  fn get(&mut self) -> Option<String> {
    unsafe {
      let c = GetClipboardText();
      let c = CStr::from_ptr(c as *mut c_char);
      c.to_str().map(|s| s.to_owned()).ok()
    }
  }

  fn set(&mut self, value: &str) {
    let s = CString::new(value);
    if let Ok(s) = s {
      unsafe {
        SetClipboardText(s.as_ptr());
      }
    }
  }
}

impl Drop for RaylibGui {
  fn drop(&mut self) {
    unsafe {
      let fonts = self.context.fonts();

      if fonts.tex_id != 0.into() {
        ffi::rlUnloadTexture(fonts.tex_id.id() as _);
        fonts.tex_id = 0.into();
      }

      save_ini_settings(&mut self.context);
    }
  }
}

pub struct Gui<'a> {
  gui: &'a mut RaylibGui,
}

impl<'a> Gui<'a> {
  pub fn begin(&mut self) -> &mut Ui {
    self.gui.context.new_frame()
  }
}

impl Drop for Gui<'_> {
  fn drop(&mut self) {
    self.gui.end();
  }
}

impl<T> RaylibDrawImGui for T where T: RaylibDraw {}
pub trait RaylibDrawImGui: RaylibDraw {
  fn prepare_imgui<'a>(&self, gui: &'a mut RaylibGui) -> Gui<'a> {
    Gui { gui }
  }
}

impl RaylibGui {
  pub fn new(rl: &mut RaylibHandle, _thread: &RaylibThread) -> RaylibGui {
    let mut ctx = Context::create();
    ctx.set_platform_name(Some(String::from("imgui_raylib_platform")));
    ctx.set_renderer_name(Some(String::from("imgui_raylib_renderer")));
    ctx.style_mut().use_dark_colors();
    ctx.set_clipboard_backend(RaylibClipboardBackend);
    ctx.fonts().add_font(&[imgui::FontSource::DefaultFontData {
      config: Some(FontConfig { ..Default::default() }),
    }]);

    load_ini_settings(&mut ctx);

    let io = ctx.io_mut();
    io.backend_flags.insert(BackendFlags::HAS_GAMEPAD);
    io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
    io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
    io.mouse_pos = [0.0, 0.0];

    let mut gui = RaylibGui {
      context: ctx,
      current_mouse_cursor: None,
      last_frame_focused: rl.is_window_focused(),
      last_control_pressed: false,
      last_shift_pressed: false,
      last_alt_pressed: false,
      last_super_pressed: false,
      keys: [
        // build up a map of raylib keys to ImGuiKeys
        (KeyboardKey::KEY_APOSTROPHE, Key::Apostrophe),
        (KeyboardKey::KEY_COMMA, Key::Comma),
        (KeyboardKey::KEY_MINUS, Key::Minus),
        (KeyboardKey::KEY_PERIOD, Key::Period),
        (KeyboardKey::KEY_SLASH, Key::Slash),
        (KeyboardKey::KEY_ZERO, Key::Keypad0),
        (KeyboardKey::KEY_ONE, Key::Keypad1),
        (KeyboardKey::KEY_TWO, Key::Keypad2),
        (KeyboardKey::KEY_THREE, Key::Keypad3),
        (KeyboardKey::KEY_FOUR, Key::Keypad4),
        (KeyboardKey::KEY_FIVE, Key::Keypad5),
        (KeyboardKey::KEY_SIX, Key::Keypad6),
        (KeyboardKey::KEY_SEVEN, Key::Keypad7),
        (KeyboardKey::KEY_EIGHT, Key::Keypad8),
        (KeyboardKey::KEY_NINE, Key::Keypad9),
        (KeyboardKey::KEY_SEMICOLON, Key::Semicolon),
        (KeyboardKey::KEY_EQUAL, Key::Equal),
        (KeyboardKey::KEY_A, Key::A),
        (KeyboardKey::KEY_B, Key::B),
        (KeyboardKey::KEY_C, Key::C),
        (KeyboardKey::KEY_D, Key::D),
        (KeyboardKey::KEY_E, Key::E),
        (KeyboardKey::KEY_F, Key::F),
        (KeyboardKey::KEY_G, Key::G),
        (KeyboardKey::KEY_H, Key::H),
        (KeyboardKey::KEY_I, Key::I),
        (KeyboardKey::KEY_J, Key::J),
        (KeyboardKey::KEY_K, Key::K),
        (KeyboardKey::KEY_L, Key::L),
        (KeyboardKey::KEY_M, Key::M),
        (KeyboardKey::KEY_N, Key::N),
        (KeyboardKey::KEY_O, Key::O),
        (KeyboardKey::KEY_P, Key::P),
        (KeyboardKey::KEY_Q, Key::Q),
        (KeyboardKey::KEY_R, Key::R),
        (KeyboardKey::KEY_S, Key::S),
        (KeyboardKey::KEY_T, Key::T),
        (KeyboardKey::KEY_U, Key::U),
        (KeyboardKey::KEY_V, Key::V),
        (KeyboardKey::KEY_W, Key::W),
        (KeyboardKey::KEY_X, Key::X),
        (KeyboardKey::KEY_Y, Key::Y),
        (KeyboardKey::KEY_Z, Key::Z),
        (KeyboardKey::KEY_SPACE, Key::Space),
        (KeyboardKey::KEY_ESCAPE, Key::Escape),
        (KeyboardKey::KEY_ENTER, Key::Enter),
        (KeyboardKey::KEY_TAB, Key::Tab),
        (KeyboardKey::KEY_BACKSPACE, Key::Backspace),
        (KeyboardKey::KEY_INSERT, Key::Insert),
        (KeyboardKey::KEY_DELETE, Key::Delete),
        (KeyboardKey::KEY_RIGHT, Key::RightArrow),
        (KeyboardKey::KEY_LEFT, Key::LeftArrow),
        (KeyboardKey::KEY_DOWN, Key::DownArrow),
        (KeyboardKey::KEY_UP, Key::UpArrow),
        (KeyboardKey::KEY_PAGE_UP, Key::PageUp),
        (KeyboardKey::KEY_PAGE_DOWN, Key::PageDown),
        (KeyboardKey::KEY_HOME, Key::Home),
        (KeyboardKey::KEY_END, Key::End),
        (KeyboardKey::KEY_CAPS_LOCK, Key::CapsLock),
        (KeyboardKey::KEY_SCROLL_LOCK, Key::ScrollLock),
        (KeyboardKey::KEY_NUM_LOCK, Key::NumLock),
        (KeyboardKey::KEY_PRINT_SCREEN, Key::PrintScreen),
        (KeyboardKey::KEY_PAUSE, Key::Pause),
        (KeyboardKey::KEY_F1, Key::F1),
        (KeyboardKey::KEY_F2, Key::F2),
        (KeyboardKey::KEY_F3, Key::F3),
        (KeyboardKey::KEY_F4, Key::F4),
        (KeyboardKey::KEY_F5, Key::F5),
        (KeyboardKey::KEY_F6, Key::F6),
        (KeyboardKey::KEY_F7, Key::F7),
        (KeyboardKey::KEY_F8, Key::F8),
        (KeyboardKey::KEY_F9, Key::F9),
        (KeyboardKey::KEY_F10, Key::F10),
        (KeyboardKey::KEY_F11, Key::F11),
        (KeyboardKey::KEY_F12, Key::F12),
        (KeyboardKey::KEY_LEFT_SHIFT, Key::LeftShift),
        (KeyboardKey::KEY_LEFT_CONTROL, Key::LeftCtrl),
        (KeyboardKey::KEY_LEFT_ALT, Key::LeftAlt),
        (KeyboardKey::KEY_LEFT_SUPER, Key::LeftSuper),
        (KeyboardKey::KEY_RIGHT_SHIFT, Key::RightShift),
        (KeyboardKey::KEY_RIGHT_CONTROL, Key::RightCtrl),
        (KeyboardKey::KEY_RIGHT_ALT, Key::RightAlt),
        (KeyboardKey::KEY_RIGHT_SUPER, Key::RightSuper),
        (KeyboardKey::KEY_KB_MENU, Key::Menu),
        (KeyboardKey::KEY_LEFT_BRACKET, Key::LeftBracket),
        (KeyboardKey::KEY_BACKSLASH, Key::Backslash),
        (KeyboardKey::KEY_RIGHT_BRACKET, Key::RightBracket),
        (KeyboardKey::KEY_GRAVE, Key::GraveAccent),
        (KeyboardKey::KEY_KP_0, Key::Keypad0),
        (KeyboardKey::KEY_KP_1, Key::Keypad1),
        (KeyboardKey::KEY_KP_2, Key::Keypad2),
        (KeyboardKey::KEY_KP_3, Key::Keypad3),
        (KeyboardKey::KEY_KP_4, Key::Keypad4),
        (KeyboardKey::KEY_KP_5, Key::Keypad5),
        (KeyboardKey::KEY_KP_6, Key::Keypad6),
        (KeyboardKey::KEY_KP_7, Key::Keypad7),
        (KeyboardKey::KEY_KP_8, Key::Keypad8),
        (KeyboardKey::KEY_KP_9, Key::Keypad9),
        (KeyboardKey::KEY_KP_DECIMAL, Key::KeypadDecimal),
        (KeyboardKey::KEY_KP_DIVIDE, Key::KeypadDivide),
        (KeyboardKey::KEY_KP_MULTIPLY, Key::KeypadMultiply),
        (KeyboardKey::KEY_KP_SUBTRACT, Key::KeypadSubtract),
        (KeyboardKey::KEY_KP_ADD, Key::KeypadAdd),
        (KeyboardKey::KEY_KP_ENTER, Key::KeypadEnter),
        (KeyboardKey::KEY_KP_EQUAL, Key::KeypadEqual),
      ],
      gamepad_map: [
        (GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP, Key::GamepadDpadUp),
        (GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT, Key::GamepadDpadRight),
        (GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN, Key::GamepadDpadDown),
        (GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT, Key::GamepadDpadLeft),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP, Key::GamepadFaceUp),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT, Key::GamepadFaceLeft),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN, Key::GamepadFaceDown),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT, Key::GamepadFaceRight),
        (GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1, Key::GamepadL1),
        (GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_2, Key::GamepadL2),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1, Key::GamepadR1),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2, Key::GamepadR2),
        (GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB, Key::GamepadL3),
        (GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB, Key::GamepadR3),
        (GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT, Key::GamepadStart),
        (GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT, Key::GamepadBack),
      ],
      gamepad_axis: [
        (
          GamepadAxis::GAMEPAD_AXIS_LEFT_X,
          Key::GamepadLStickLeft,
          Key::GamepadLStickRight,
        ),
        (
          GamepadAxis::GAMEPAD_AXIS_LEFT_Y,
          Key::GamepadLStickUp,
          Key::GamepadLStickDown,
        ),
        (
          GamepadAxis::GAMEPAD_AXIS_RIGHT_X,
          Key::GamepadRStickLeft,
          Key::GamepadRStickRight,
        ),
        (
          GamepadAxis::GAMEPAD_AXIS_RIGHT_Y,
          Key::GamepadRStickUp,
          Key::GamepadRStickDown,
        ),
      ],
    };

    gui.reload_fonts();
    gui
  }

  pub fn set_style(&mut self, style: Style) {
    match style {
      Style::Dark => {
        self.context.style_mut().use_dark_colors();
      },
      Style::Light => {
        self.context.style_mut().use_light_colors();
      },
      Style::Classic => {
        self.context.style_mut().use_classic_colors();
      },
    }
  }

  pub fn update(&mut self, rl: &mut RaylibHandle) {
    let delta_time = rl.get_frame_time();
    self.update_delta(rl, delta_time);
  }

  pub fn update_delta(&mut self, rl: &mut RaylibHandle, delta_time: f32) {
    self.handle_events(rl);
    self.prepare_frame(rl, delta_time);
  }

  pub fn begin(&mut self, rl: &mut RaylibHandle) -> &mut Ui {
    let delta_time = rl.get_frame_time();
    self.begin_delta(rl, delta_time)
  }

  pub fn begin_delta(
    &mut self,
    rl: &mut RaylibHandle,
    delta_time: f32,
  ) -> &mut Ui {
    self.handle_events(rl);
    self.prepare_frame(rl, delta_time);
    self.new_frame()
  }

  pub fn end(&mut self) {
    Renderer::render(&mut self.context);
  }

  pub fn reload_fonts(&mut self) {
    let fonts = self.context.fonts();
    let texture = fonts.build_rgba32_texture();

    unsafe {
      let data = texture.data.as_ptr() as *mut std::ffi::c_void;
      let [width, height] = [texture.width as i32, texture.height as i32];
      if fonts.tex_id == 0.into() {
        let id = ffi::rlLoadTexture(
          data,
          width,
          height,
          ffi::rlPixelFormat::RL_PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
          1,
        ) as u32;
        fonts.tex_id = (id as usize).into();
      } else {
        ffi::rlUpdateTexture(
          fonts.tex_id.id() as _,
          0,
          0,
          width,
          height,
          ffi::rlPixelFormat::RL_PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
          data,
        );
      }
    }
  }

  pub fn new_frame(&mut self) -> &mut Ui {
    self.context.new_frame()
  }

  pub fn prepare_frame(&mut self, rl: &mut RaylibHandle, delta_time: f32) {
    let imgui_cursor = self.context.mouse_cursor();
    let io = self.context.io_mut();

    if rl.is_window_fullscreen() {
      let monitor = window::get_current_monitor();
      io.display_size = [
        window::get_monitor_width(monitor) as f32,
        window::get_monitor_height(monitor) as f32,
      ];
    } else {
      io.display_size =
        [rl.get_screen_width() as f32, rl.get_screen_height() as f32];
    }

    if rl.get_window_state().window_highdpi() {
      let resolution_scale = rl.get_window_scale_dpi();
      io.display_framebuffer_scale = [resolution_scale.x, resolution_scale.y];
    } else {
      io.display_framebuffer_scale = [1.0, 1.0];
    }

    io.delta_time = delta_time;

    if io.want_set_mouse_pos {
      rl.set_mouse_position(Vector2::new(io.mouse_pos[0], io.mouse_pos[1]));
    } else {
      io.add_mouse_pos_event([
        rl.get_mouse_x() as f32,
        rl.get_mouse_y() as f32,
      ]);
    }

    let mut set_mouse_event =
      |ray_mouse: MouseButton, imgui_mouse: imgui::MouseButton| {
        if rl.is_mouse_button_pressed(ray_mouse) {
          io.add_mouse_button_event(imgui_mouse, true);
        } else if rl.is_mouse_button_released(ray_mouse) {
          io.add_mouse_button_event(imgui_mouse, false);
        }
      };

    set_mouse_event(MouseButton::MOUSE_BUTTON_LEFT, imgui::MouseButton::Left);
    set_mouse_event(MouseButton::MOUSE_BUTTON_RIGHT, imgui::MouseButton::Right);
    set_mouse_event(
      MouseButton::MOUSE_BUTTON_MIDDLE,
      imgui::MouseButton::Middle,
    );
    set_mouse_event(
      MouseButton::MOUSE_BUTTON_FORWARD,
      imgui::MouseButton::Extra1,
    );
    set_mouse_event(MouseButton::MOUSE_BUTTON_BACK, imgui::MouseButton::Extra2);

    let mouse_wheel = rl.get_mouse_wheel_move_v();
    io.add_mouse_wheel_event([mouse_wheel.x, mouse_wheel.y]);

    if io.backend_flags.intersects(BackendFlags::HAS_MOUSE_CURSORS) {
      if !io.config_flags.intersects(ConfigFlags::NO_MOUSE_CURSOR_CHANGE) {
        if imgui_cursor != self.current_mouse_cursor || io.mouse_draw_cursor {
          self.current_mouse_cursor = imgui_cursor;
          if io.mouse_draw_cursor || imgui_cursor == None {
            rl.hide_cursor();
          } else if let Some(cursor) = imgui_cursor {
            rl.show_cursor();
            rl.set_mouse_cursor(to_rl_cursor(cursor));
          }
        }
      }
    }
  }

  pub fn handle_events(&mut self, rl: &mut RaylibHandle) {
    let io = self.context.io_mut();

    //let focused = !rl.is_window_focused();
    //if focused != self.last_frame_focused {
    //  io.app_focus_lost = focused; // TODO: missing `add_focus_event()` function?
    //}
    //self.last_frame_focused = focused;

    // TODO: (int keyId = KEY_NULL; keyId < KeyboardKey::KEY_KP_EQUAL; keyId++)
    // get the pressed keys, just walk the keys so we don
    for (rl_key, imgui_key) in self.keys {
      if rl.is_key_pressed(rl_key) {
        io.add_key_event(imgui_key, true);
      }
    }

    // look for any keys that were down last frame and see if they were down and are released
    for (rl_key, imgui_key) in self.keys {
      if rl.is_key_released(rl_key) {
        io.add_key_event(imgui_key, false);
      }
    }

    let mut update_mod = |key: Key, current_state: &mut bool, state: bool| {
      if *current_state != state {
        io.add_key_event(key, state);
      }
      *current_state = state;
    };

    // handle the modifyer key events so that shortcuts work
    update_mod(
      Key::ModCtrl,
      &mut self.last_control_pressed,
      rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        || rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL),
    );
    update_mod(
      Key::ModShift,
      &mut self.last_shift_pressed,
      rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
        || rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT),
    );
    update_mod(
      Key::ModAlt,
      &mut self.last_alt_pressed,
      rl.is_key_down(KeyboardKey::KEY_RIGHT_ALT)
        || rl.is_key_down(KeyboardKey::KEY_LEFT_ALT),
    );
    update_mod(
      Key::ModSuper,
      &mut self.last_super_pressed,
      rl.is_key_down(KeyboardKey::KEY_RIGHT_SUPER)
        || rl.is_key_down(KeyboardKey::KEY_LEFT_SUPER),
    );

    if io.want_capture_keyboard {
      // add the text input in order
      while let Some(pressed) = rl.get_char_pressed() {
        io.add_input_character(pressed);
      }
    }

    if io.config_flags.intersects(ConfigFlags::NAV_ENABLE_GAMEPAD)
      && rl.is_gamepad_available(0)
    {
      for (btn, key) in self.gamepad_map {
        if rl.is_gamepad_button_pressed(0, btn) {
          io.add_key_event(key, true);
        } else if rl.is_gamepad_button_released(0, btn) {
          io.add_key_event(key, false);
        }
      }

      for (axis, pos_key, neg_key) in self.gamepad_axis {
        let dead_zone = 0.20;
        let axis_value = rl.get_gamepad_axis_movement(0, axis);

        io.add_key_analog_event(
          neg_key,
          axis_value < -dead_zone,
          if axis_value < -dead_zone { -axis_value } else { 0.0 },
        );
        io.add_key_analog_event(
          pos_key,
          axis_value > dead_zone,
          if axis_value > dead_zone { axis_value } else { 0.0 },
        );
      }
    }
  }

  pub fn render(&mut self) {
    self.end();
  }
}

fn save_ini_settings(ctx: &mut Context) {
  if let Some(ini_path) = ctx.ini_filename() {
    let mut settings = String::new();
    ctx.save_ini_settings(&mut settings);
    let mut file = File::create(&ini_path).unwrap();
    file.write_all(&settings.as_bytes()).unwrap();
  }
}

fn load_ini_settings(ctx: &mut Context) {
  if let Some(ini_path) = ctx.ini_filename() {
    if let Ok(contents) = fs::read_to_string(ini_path) {
      ctx.load_ini_settings(&contents.to_string());
    }
  }
}

fn to_rl_cursor(cursor: imgui::MouseCursor) -> MouseCursor {
  match cursor {
    imgui::MouseCursor::Arrow => MouseCursor::MOUSE_CURSOR_ARROW,
    imgui::MouseCursor::TextInput => MouseCursor::MOUSE_CURSOR_IBEAM,
    imgui::MouseCursor::ResizeAll => MouseCursor::MOUSE_CURSOR_RESIZE_ALL,
    imgui::MouseCursor::ResizeNS => MouseCursor::MOUSE_CURSOR_RESIZE_NS,
    imgui::MouseCursor::ResizeEW => MouseCursor::MOUSE_CURSOR_RESIZE_EW,
    imgui::MouseCursor::ResizeNESW => MouseCursor::MOUSE_CURSOR_RESIZE_NESW,
    imgui::MouseCursor::ResizeNWSE => MouseCursor::MOUSE_CURSOR_RESIZE_NWSE,
    imgui::MouseCursor::Hand => MouseCursor::MOUSE_CURSOR_POINTING_HAND,
    imgui::MouseCursor::NotAllowed => MouseCursor::MOUSE_CURSOR_NOT_ALLOWED,
  }
}

pub struct Renderer(());

impl Renderer {
  pub fn render(ctx: &mut Context) {
    unsafe {
      ffi::rlDrawRenderBatchActive();
      ffi::rlDisableBackfaceCulling();

      let [width, height] = ctx.io().display_size;
      let [scale_w, scale_h] = ctx.io().display_framebuffer_scale;
      let [fb_width, fb_height] = [width * scale_w, height * scale_h];

      let draw_data = ctx.render();
      if draw_data.draw_lists_count() > 0 {
        for draw_list in draw_data.draw_lists() {
          let idx_buffer = draw_list.idx_buffer();
          let vtx_buffer = draw_list.vtx_buffer();
          for cmd in draw_list.commands() {
            match cmd {
              DrawCmd::Elements {
                count,
                cmd_params:
                  DrawCmdParams { clip_rect, texture_id, idx_offset, .. },
              } => {
                enable_scissor(
                  clip_rect,
                  [scale_w, scale_h],
                  [fb_width, fb_height],
                );
                render_triangles(
                  count, idx_offset, idx_buffer, vtx_buffer, texture_id,
                );
                ffi::rlDrawRenderBatchActive();
              },
              DrawCmd::ResetRenderState => unimplemented!(),
              DrawCmd::RawCallback { callback, raw_cmd } => {
                let clip_rect = &(*raw_cmd).ClipRect;
                enable_scissor(
                  [clip_rect.x, clip_rect.y, clip_rect.z, clip_rect.w],
                  [scale_w, scale_h],
                  [fb_width, fb_height],
                );
                callback(draw_list.raw(), raw_cmd);
              },
            }
          }
        }
      }

      ffi::rlSetTexture(0);
      ffi::rlDisableScissorTest();
      ffi::rlEnableBackfaceCulling();
    }
  }
}

unsafe fn enable_scissor(
  clip_rect: [f32; 4],
  scale: [f32; 2],
  fb_size: [f32; 2],
) {
  let [x, y, z, w] = clip_rect;
  let [scale_w, scale_h] = scale;
  let [_fb_width, fb_height] = fb_size;
  ffi::rlEnableScissorTest();
  ffi::rlScissor(
    (x * scale_w) as i32,
    (fb_height - w * scale_h) as i32,
    ((z - x) * scale_w) as i32,
    ((w - y) * scale_h) as i32,
  );
}

fn render_triangles(
  count: usize,
  idx_offset: usize,
  idx_buffer: &[u16],
  vtx_buffer: &[DrawVert],
  texture_id: imgui::TextureId,
) {
  if count < 3 {
    return;
  }

  unsafe {
    ffi::rlBegin(ffi::RL_TRIANGLES as i32);
    ffi::rlSetTexture(texture_id.id() as _);

    for i in (0..=count - 3).step_by(3) {
      let index_a = idx_buffer[idx_offset + i];
      let index_b = idx_buffer[idx_offset + i + 1];
      let index_c = idx_buffer[idx_offset + i + 2];

      let vertex_a = vtx_buffer[index_a as usize];
      let vertex_b = vtx_buffer[index_b as usize];
      let vertex_c = vtx_buffer[index_c as usize];

      triangle_vert(vertex_a);
      triangle_vert(vertex_b);
      triangle_vert(vertex_c);
    }
    ffi::rlEnd();
  }
}

unsafe fn triangle_vert(idx_vert: DrawVert) {
  let [r, g, b, a] = idx_vert.col;
  let [uv_x, uv_y] = idx_vert.uv;
  let [x, y] = idx_vert.pos;
  ffi::rlColor4ub(r, g, b, a);
  ffi::rlTexCoord2f(uv_x, uv_y);
  ffi::rlVertex2f(x, y);
}
