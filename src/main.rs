use std::sync::atomic::{AtomicBool, Ordering};
use atomic::Atomic;

use gl;
use glam::{Vec2, Vec3};
use glfw::{Action, Context, CursorMode, Key, MouseButton, SwapInterval, WindowEvent, WindowHint};
use rand::Rng;

use crate::hana::camera::Camera;
use crate::hana::glu::*;
use crate::hana::model::{Model};
use crate::hana::palette::{Color, hex_to_vec3};

mod hana;

fn main() -> Result<(), String> {
  let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
  glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(WindowHint::ContextVersion(4, 6));
  glfw.window_hint(WindowHint::Resizable(true));
  glfw.window_hint(WindowHint::Samples(Some(4)));

  let (mut width, mut height) = (1152i32, 720i32);
  let (mut win, evt) =
    glfw
      .create_window(width as u32, height as u32, "hana", glfw::WindowMode::Windowed)
      .expect("failed to make window.");

  win.make_current();
  win.set_cursor_mode(CursorMode::Disabled);
  win.set_cursor_pos_polling(true);
  win.set_mouse_button_polling(true);
  win.set_size_polling(true);
  win.set_key_polling(true);
  glfw.set_swap_interval(SwapInterval::Adaptive);

  gl::load_with(|s| win.get_proc_address(s) as *const _);
  glfw.make_context_current(Some(&win));

  // set up permanent gl state
  gl_enable(gl::MULTISAMPLE);
  gl_clear_color(0.0, 0.0, 0.0, 0.0);
  gl_depth_func(gl::LESS);
  gl_enable(gl::DEPTH_TEST);

  // palette
  let palette = [
    hex_to_vec3(0x66ffe3), hex_to_vec3(0x4da6ff), hex_to_vec3(0x4b5bab), hex_to_vec3(0x473b78), // blue
    hex_to_vec3(0xcfff70), hex_to_vec3(0x8fde5d), hex_to_vec3(0x3ca370), hex_to_vec3(0x3d6e70), // green
    hex_to_vec3(0xffe478), hex_to_vec3(0xf2a65e), hex_to_vec3(0xba6156), hex_to_vec3(0x8c3f5d), // yellow
    hex_to_vec3(0xffb570), hex_to_vec3(0xff9166), hex_to_vec3(0xeb564b), hex_to_vec3(0xb0305c), // orange
    hex_to_vec3(0xff6b97), hex_to_vec3(0xbd4882), hex_to_vec3(0x80366b), hex_to_vec3(0x5a265e), // pink
    hex_to_vec3(0xffffeb), hex_to_vec3(0xc2c2d1), hex_to_vec3(0x7e7e8f), hex_to_vec3(0x606070), // white
  ];

  // shaders
  let defer = Shader::new("res/shader/model.vert", "res/shader/g_buffer_cel.frag", None)?;
  let fin = Shader::new("res/shader/postprocess.vert", "res/shader/final_cel.frag", None)?;
  let blit = Shader::new("res/shader/postprocess.vert", "res/shader/blit.frag", None)?;
  let cel = Shader::new("res/shader/postprocess.vert", "res/shader/cel.frag", None)?;

  // camera
  let mut cam = Camera::new();

  // frame buffers
  let mut f_buf =
    Fbo::new(&[
      (gl::COLOR_ATTACHMENT0, TexSpec::rgba8_linear(width * 2, height * 2)),
      (gl::DEPTH_ATTACHMENT, TexSpec::depth24_nearest(width * 2, height * 2)),
    ]);

  let mut g_buf =
    Fbo::new(&[
      (gl::COLOR_ATTACHMENT0, TexSpec::rgba16_linear(width * 2, height * 2)), // POS
      (gl::COLOR_ATTACHMENT1, TexSpec::rgba16_linear(width * 2, height * 2)), // NORM
      (gl::COLOR_ATTACHMENT2, TexSpec::rg8_nearest(width * 2, height * 2)), // COLOR
      (gl::DEPTH_ATTACHMENT, TexSpec::depth24_nearest(width * 2, height * 2)), // DEPTH
    ]);

  let mut c_buf =
    Fbo::new(&[
      (gl::COLOR_ATTACHMENT0, TexSpec::rgba8_linear(width * 2, height * 2))
    ]);

  // character model
  let hana = Model::new("res/model/hana.obj")?;

  // post processing vertex array
  let (p_vao, p_vbo) = gl_gen_v(&[FLOAT_2]);
  p_vbo.data(
    gl::STATIC_DRAW,
    &[
      -1.0f32, -1.,
      -1., 1.,
      1., 1.,
      1., 1.,
      1., -1.,
      -1., -1.
    ]
  );

  // define tick delta
  let mut tick_delta = 0.;

  while !win.should_close() {
    // TODO: refactor all of this
    // begin ticking
    let n_ticks = {
      const TICK_LENGTH: f64 = 1./30.;
      static PREV_TIME: Atomic<f64> = Atomic::new(0.);
      let time = glfw.get_time();
      let last_frame = (time - PREV_TIME.load(Ordering::Relaxed)) / TICK_LENGTH;
      PREV_TIME.store(time, Ordering::Relaxed);
      tick_delta += last_frame as f32;
      let i = tick_delta as i64;
      tick_delta -= i as f32;
      i
    };

    for _ in 0..n_ticks.min(10) {
      cam.prev_pos = cam.pos;
      for key in [Key::W, Key::A, Key::S, Key::D, Key::Space, Key::LeftShift] {
        if ![Action::Repeat, Action::Press].contains(&win.get_key(key)) {
          continue;
        }

        cam.key(key);
      }
    }
    // end ticking

    // set up per-frame gl state
    gl_enable(gl::DEPTH_TEST);
    gl_disable(gl::BLEND);

    // begin g buffer pass
    gl_viewport(width * 2, height * 2);
    g_buf.bind();
    g_buf.draw_buffers(&[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2]);
    gl_clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    defer.bind();
    defer.set_mat4("u_proj", &cam.proj(width as f32 / height as f32));
    defer.set_mat4("u_look", &cam.look_at(tick_delta));
    defer.set_1i("tint", 5);

    for mesh in &hana.0 {
      let (vao, ..) = &mesh.gl;
      vao.bind();
      gl_draw_elements(gl::TRIANGLES, mesh.indices.len() as i32)
    }
    // end g buffer pass

    // begin lighting pass
    gl_viewport(width * 2, height * 2);
    f_buf.bind();
    gl_clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    fin.bind();
    g_buf.tex_at(gl::COLOR_ATTACHMENT0).bind(gl::TEXTURE0);
    fin.set_1i("f_pos", 0);
    g_buf.tex_at(gl::COLOR_ATTACHMENT1).bind(gl::TEXTURE1);
    fin.set_1i("f_norm", 1);
    g_buf.tex_at(gl::COLOR_ATTACHMENT2).bind(gl::TEXTURE2);
    fin.set_1i("f_tint", 2);
    fin.set_3f("u_eye", &cam.eye(tick_delta));
    fin.set_3fv("palette", &palette);
    p_vao.bind();
    gl_draw_arrays(gl::TRIANGLES, 6);
    // end lighting pass

    // begin blitting to backbuffer with cel shading
    gl_viewport(width, height);
    win.fbo0().bind();
    gl_clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    blit.bind();
    f_buf.tex_at(gl::COLOR_ATTACHMENT0).bind(gl::TEXTURE0);
    cel.set_1i("u_tex", 0);
    p_vao.bind();
    gl_draw_arrays(gl::TRIANGLES, 6);
    // end blitting to backbuffer

    win.swap_buffers();

    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&evt) {
      static LAST_X: Atomic<f64> = Atomic::new(0.);
      static LAST_Y: Atomic<f64> = Atomic::new(0.);
      static FIRST_MOUSE: AtomicBool = AtomicBool::new(true);

      match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
          FIRST_MOUSE.store(true, Ordering::Relaxed);
          win.set_cursor_mode(CursorMode::Normal);
        }
        WindowEvent::Size(new_width, new_height) => {
          width = new_width;
          height = new_height;
          gl_viewport(new_width, new_height);
          f_buf.resize_attachments(
            &[gl::COLOR_ATTACHMENT0, gl::DEPTH_ATTACHMENT],
            new_width * 2,
            new_height * 2
          );

          g_buf.resize_attachments(
            &[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2, gl::DEPTH_ATTACHMENT],
            new_width * 2,
            new_height * 2
          );
        }
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
          win.set_cursor_mode(CursorMode::Disabled);
        }
        WindowEvent::CursorPos(x, y) => {
          if win.get_cursor_mode() != CursorMode::Disabled {
            continue;
          }

          let x_off = if FIRST_MOUSE.load(Ordering::Relaxed) {
            0.0
          } else {
            x - LAST_X.load(Ordering::Relaxed)
          };

          let y_off = if FIRST_MOUSE.load(Ordering::Relaxed) {
            0.0
          } else {
            y - LAST_Y.load(Ordering::Relaxed)
          };

          cam.mouse_move(x_off as f32, y_off as f32);

          FIRST_MOUSE.store(false, Ordering::Relaxed);
          LAST_X.store(x, Ordering::Relaxed);
          LAST_Y.store(y, Ordering::Relaxed);
        }
        _ => {}
      }
    }
  }

  Ok(())
}