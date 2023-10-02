use std::sync::atomic::{AtomicBool, Ordering};

use atomic_float::AtomicF64;
use gl;
use glam::Vec3;
use glfw::{Action, Context, CursorMode, Key, MouseButton, SwapInterval, WindowEvent, WindowHint};

use crate::hana::camera::Camera;
use crate::hana::glu::*;
use crate::hana::model::load_model;

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
  // glfw.set_swap_interval(SwapInterval::None);

  gl::load_with(|s| win.get_proc_address(s) as *const _);
  glfw.make_context_current(Some(&win));

  gl_enable(gl::MULTISAMPLE);
  gl_clear_color(0.0, 0.0, 0.0, 0.0);
  gl_depth_func(gl::LESS);
  gl_enable(gl::DEPTH_TEST);

  let def = gl_read_sh("res/shader/model.vert", "res/shader/g_buffer.frag", None)?;
  let fin = gl_read_sh("res/shader/postprocess.vert", "res/shader/final.frag", None)?;
  let blit = gl_read_sh("res/shader/postprocess.vert", "res/shader/blit.frag", None)?;
  let motion_blur = gl_read_sh("res/shader/postprocess.vert", "res/shader/motion_blur.frag", None)?;

  let mut cam = Camera::new();
  let mut f_buf =
    gl_make_fbo(&[
      (gl::COLOR_ATTACHMENT0, TexSpec::rgba8_linear(width * 2, height * 2)),
      (gl::DEPTH_ATTACHMENT, TexSpec::depth24_nearest(width * 2, height * 2)),
    ]);

  let mut g_buf =
    gl_make_fbo(&[
      (gl::COLOR_ATTACHMENT0, TexSpec::rgba16_linear(width * 2, height * 2)), // POS
      (gl::COLOR_ATTACHMENT1, TexSpec::rgba16_linear(width * 2, height * 2)), // NORM
      (gl::COLOR_ATTACHMENT2, TexSpec::rgba8_linear(width * 2, height * 2)), // COLOR
      (gl::DEPTH_ATTACHMENT, TexSpec::depth24_nearest(width * 2, height * 2)), // DEPTH
    ]);

  let girl = load_model("res/model/girl.obj")?;

  let (p_vao, p_vbo) = gl_make_v(&[FLOAT_2]);
  gl_buf_data(&p_vbo, gl::STATIC_DRAW, &[
    -1.0f32, -1.,
    -1., 1.,
    1., 1.,
    1., 1.,
    1., -1.,
    -1., -1.
  ]);

  let mut tick_delta = 0.;

  while !win.should_close() {
    let ticks = {
      const TICK_LENGTH: f64 = 1./30.;
      static PREV_TIME: AtomicF64 = AtomicF64::new(0.);
      let time = glfw.get_time();
      let last_frame = (time - PREV_TIME.load(Ordering::Relaxed)) / TICK_LENGTH;
      PREV_TIME.store(time, Ordering::Relaxed);
      tick_delta += last_frame as f32;
      let i = tick_delta as i64;
      tick_delta -= i as f32;
      i
    };

    for _ in 0..ticks.min(10) {
      cam.prev_pos = cam.pos;
      for key in [Key::W, Key::A, Key::S, Key::D, Key::Space, Key::LeftShift] {
        if ![Action::Repeat, Action::Press].contains(&win.get_key(key)) {
          continue;
        }

        cam.key(key);
      }
    }

    gl_enable(gl::DEPTH_TEST);
    gl_disable(gl::BLEND);

    gl_viewport(width * 2, height * 2);
    gl_bind_fbo(&g_buf);
    gl_draw_buffers(&g_buf, &[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2]);
    gl_clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    gl_bind_sh(&def);
    gl_uniform_mat4("u_proj", &cam.proj(width as f32 / height as f32));
    gl_uniform_mat4("u_look", &cam.look_at(tick_delta));

    for mesh in &girl.0 {
      let (vao, ..) = &mesh.gl;
      gl_bind_vao(vao);
      gl_draw_elements(gl::TRIANGLES, mesh.indices.len() as i32)
    }

    gl_bind_fbo(&f_buf);
    gl_clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    gl_bind_sh(&fin);
    gl_bind_tex(gl_fbo_tex(&g_buf, gl::COLOR_ATTACHMENT0), gl::TEXTURE0);
    gl_uniform_1i("f_pos", 0);
    gl_bind_tex(gl_fbo_tex(&g_buf, gl::COLOR_ATTACHMENT1), gl::TEXTURE1);
    gl_uniform_1i("f_norm", 1);
    gl_bind_tex(gl_fbo_tex(&g_buf, gl::COLOR_ATTACHMENT2), gl::TEXTURE2);
    gl_uniform_1i("f_tint", 2);
    gl_uniform_3f("u_eye", &cam.eye(tick_delta));
    let light_poses = [
      Vec3 {
        x: (glfw.get_time() * 0.5 + 1.0).sin() as f32 * 5.,
        y: (glfw.get_time() * 0.5 + 1.2).sin() as f32 * 10. + 10.,
        z: (glfw.get_time() * 0.5 + 1.4).sin() as f32 * 10. + 10.,
      },
      Vec3 {
        x: (glfw.get_time() * 0.5 + 1.25).sin() as f32 * 5.,
        y: (glfw.get_time() * 0.5 + 1.45).sin() as f32 * 10. + 10.,
        z: (glfw.get_time() * 0.5 + 1.65).sin() as f32 * 10. + 10.,
      },
    ];

    let light_colors = [
      Vec3::new(0.25, 1., 1.),
      Vec3::new(1., 0.25, 1.)
    ];

    gl_uniform_3fv("u_light_positions[0]", &light_poses);
    gl_uniform_3fv("u_light_colors[0]", &light_colors);
    gl_uniform_1i("u_n_lights", 2);
    gl_bind_vao(&p_vao);
    gl_draw_arrays(gl::TRIANGLES, 6);

    gl_viewport(width, height);
    gl_bind_fbo(&win.fbo0());
    gl_clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    gl_bind_sh(&blit);
    gl_bind_tex(gl_fbo_tex(&f_buf, gl::COLOR_ATTACHMENT0), gl::TEXTURE0);
    gl_uniform_1i("u_tex", 0);
    gl_bind_vao(&p_vao);
    gl_draw_arrays(gl::TRIANGLES, 6);

    win.swap_buffers();

    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&evt) {
      static LAST_X: AtomicF64 = AtomicF64::new(0.0);
      static LAST_Y: AtomicF64 = AtomicF64::new(0.0);
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
          gl_resize_fbo_attachments(
            &mut f_buf,
            &[gl::COLOR_ATTACHMENT0, gl::DEPTH_ATTACHMENT],
            new_width * 2,
            new_height * 2
          );

          gl_resize_fbo_attachments(
            &mut g_buf,
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