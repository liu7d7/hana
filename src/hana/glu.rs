use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::fs;
use std::ptr::{addr_of, addr_of_mut, null_mut};
use std::sync::atomic::{AtomicI32, Ordering};
use glam::{Mat4, Vec2, Vec3, Vec4};
use glfw::Window;

pub fn gl_viewport(width: i32, height: i32) {
  unsafe { gl::Viewport(0, 0, width, height); }
}

pub fn gl_clear_color(r: f32, g: f32, b: f32, a: f32) {
  unsafe { gl::ClearColor(r, g, b, a); }
}

pub fn gl_depth_func(func: u32) {
  unsafe { gl::DepthFunc(func) }
}

pub fn gl_draw_buffers(fbo: &Fbo, attachments: &[u32]) {
  unsafe { gl::NamedFramebufferDrawBuffers(fbo.id, attachments.len() as i32, attachments.as_ptr()) }
}

pub fn gl_read_buffers(fbo: &Fbo, attachment: u32) {
  unsafe { gl::NamedFramebufferReadBuffer(fbo.id, attachment) }
}

pub fn gl_clear(mask: u32) {
  unsafe { gl::Clear(mask) }
}

pub fn gl_bind_fbo(fbo: &Fbo) {
  unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, fbo.id) }
}

pub fn gl_blit_fbo(src: &Fbo, dst: &Fbo, src_attachment: u32, dst_attachment: u32, filter: u32) {
  let src_mask = match src_attachment {
    gl::COLOR_ATTACHMENT0..=gl::COLOR_ATTACHMENT31 => gl::COLOR_BUFFER_BIT,
    gl::DEPTH_ATTACHMENT => gl::DEPTH_BUFFER_BIT,
    _ => panic!("Invalid src_attachment: {}", src_attachment)
  };

  let dst_mask = match dst_attachment {
    gl::COLOR_ATTACHMENT0..=gl::COLOR_ATTACHMENT31 => gl::COLOR_BUFFER_BIT,
    gl::DEPTH_ATTACHMENT => gl::DEPTH_BUFFER_BIT,
    _ => panic!("Invalid dst_attachment: {}", dst_attachment)
  };

  if src_mask != dst_mask {
    panic!("Src and dst masks do not match! src_mask = {}, dst_mask = {}", src_attachment, dst_attachment)
  }

  let src_tex = if let Some(src_tex) = src.attachments.get(&src_attachment) {
    src_tex
  } else { panic!("src_attachment not found on src!"); };

  let dst_tex = if let Some(dst_tex) = dst.attachments.get(&dst_attachment) {
    dst_tex
  } else { panic!("dst_attachment not found on dst!"); };

  unsafe {
    if src_mask == gl::COLOR_BUFFER_BIT {
      gl::NamedFramebufferReadBuffer(src.id, src_attachment);
      gl::NamedFramebufferDrawBuffers(dst.id, 1, addr_of!(dst_attachment));
    }

    gl::BlitNamedFramebuffer(src.id, dst.id, 0, 0, src_tex.spec.width, src_tex.spec.height, 0, 0, dst_tex.spec.width, dst_tex.spec.height, src_mask, filter)
  }
}

trait GlObject {
  fn id(&self) -> u32;
}

pub struct Vao(u32);

pub struct Buf(u32, u32);

static mut CURRENT_SHADER: *const Shader = null_mut();

pub struct Shader(pub u32, pub HashMap<String, i32>);

#[derive(Clone)]
pub struct Fbo {
  pub id: u32,
  pub attachments: HashMap<u32, Tex>,
}

#[derive(Clone)]
pub struct Tex {
  pub id: u32,
  pub spec: TexSpec,
}

impl GlObject for Vao {
  fn id(&self) -> u32 {
    self.0
  }
}

impl GlObject for Buf {
  fn id(&self) -> u32 {
    self.0
  }
}

impl GlObject for Shader {
  fn id(&self) -> u32 {
    self.0
  }
}

pub const FLOAT_1: (i32, bool) = (1, true);
pub const FLOAT_2: (i32, bool) = (2, true);
pub const FLOAT_3: (i32, bool) = (3, true);
pub const FLOAT_4: (i32, bool) = (4, true);

pub fn gl_bind_vao(vao: &Vao) {
  unsafe { gl::BindVertexArray(vao.0) }
}

pub fn gl_bind_buf(buf: &Buf) {
  unsafe { gl::BindBuffer(buf.1, buf.0) }
}

pub fn gl_bind_sh(shader: &Shader) {
  unsafe {
    gl::UseProgram(shader.0);
    CURRENT_SHADER = shader as *const Shader;
  }
}

pub fn gl_resize_fbo_attachments(fbo: &mut Fbo, attachments: &[u32], new_width: i32, new_height: i32) {
  for it in attachments {
    let new_tex = gl_resize_tex(&fbo.attachments[it], new_width, new_height);
    unsafe { gl::DeleteTextures(1, addr_of!(fbo.attachments[it].id)) }
    unsafe { gl::NamedFramebufferTexture(fbo.id, *it, new_tex.id, 0) }
    *fbo.attachments.get_mut(it).unwrap() = new_tex;
  }
}

pub fn gl_resize_tex(tex: &Tex, new_width: i32, new_height: i32) -> Tex {
  gl_make_tex(&TexSpec {
    width: new_width,
    height: new_height,
    ..tex.spec.clone()
  })
}

pub fn gl_fbo_tex(fbo: &Fbo, attachment: u32) -> &Tex {
  fbo.attachments.get(&attachment).unwrap()
}

pub fn gl_bind_tex(tex: &Tex, unit: u32) {
  unsafe {
    gl::ActiveTexture(unit);
    gl::BindTexture(gl::TEXTURE_2D, tex.id);
  }
}

pub fn gl_make_vao() -> Vao {
  let mut vao = 0;
  unsafe { gl::CreateVertexArrays(1, addr_of_mut!(vao)); }
  Vao(vao)
}

#[derive(Clone)]
pub struct TexSpec {
  pub width: i32,
  pub height: i32,
  pub internal_format: u32,
  pub format: u32,
  pub min_filter: u32,
  pub mag_filter: u32,
  pub pixels: Option<Vec<u8>>,
}

impl TexSpec {
  pub fn invalid() -> TexSpec {
    TexSpec {
      width: 0,
      height: 0,
      internal_format: 0,
      format: 0,
      min_filter: 0,
      mag_filter: 0,
      pixels: None,
    }
  }

  pub fn rgba8_linear(width: i32, height: i32) -> TexSpec {
    TexSpec {
      width,
      height,
      internal_format: gl::RGBA8,
      format: gl::RGBA,
      min_filter: gl::LINEAR,
      mag_filter: gl::LINEAR,
      pixels: None,
    }
  }

  pub fn rgba8_nearest(width: i32, height: i32) -> TexSpec {
    TexSpec {
      min_filter: gl::NEAREST,
      mag_filter: gl::NEAREST,
      ..Self::rgba8_linear(width, height)
    }
  }

  pub fn rgba16_linear(width: i32, height: i32) -> TexSpec {
    TexSpec {
      width,
      height,
      internal_format: gl::RGBA16F,
      format: gl::RGBA,
      min_filter: gl::LINEAR,
      mag_filter: gl::LINEAR,
      pixels: None,
    }
  }

  pub fn rgba16_nearest(width: i32, height: i32) -> TexSpec {
    TexSpec {
      min_filter: gl::NEAREST,
      mag_filter: gl::NEAREST,
      ..Self::rgba16_linear(width, height)
    }
  }

  pub fn depth24_nearest(width: i32, height: i32) -> TexSpec {
    TexSpec {
      width,
      height,
      internal_format: gl::DEPTH_COMPONENT24,
      format: gl::DEPTH_COMPONENT,
      min_filter: gl::NEAREST,
      mag_filter: gl::NEAREST,
      pixels: None,
    }
  }
}

pub fn gl_make_tex(spec: &TexSpec) -> Tex {
  let mut tex = 0;
  unsafe {
    gl::CreateTextures(gl::TEXTURE_2D, 1, addr_of_mut!(tex));
    gl::TextureParameteri(tex, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as i32);
    gl::TextureParameteri(tex, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as i32);
    gl::TextureParameteri(tex, gl::TEXTURE_MIN_FILTER, spec.min_filter as i32);
    gl::TextureParameteri(tex, gl::TEXTURE_MAG_FILTER, spec.mag_filter as i32);

    gl::TextureStorage2D(tex, 1, spec.internal_format, spec.width, spec.height);

    if let Some(pixels) = &spec.pixels {
      gl::TextureSubImage2D(tex, 0, 0, 0, spec.width, spec.height, spec.format, gl::UNSIGNED_BYTE, pixels.as_ptr() as *const c_void)
    }
  }

  Tex { id: tex, spec: (*spec).clone() }
}

pub fn gl_make_fbo(attachments: &[(u32, TexSpec)]) -> Fbo {
  let mut fbo = 0;
  unsafe { gl::CreateFramebuffers(1, addr_of_mut!(fbo)) }

  let mut map = HashMap::new();
  for it in attachments {
    let tex = gl_make_tex(&it.1);
    map.insert(it.0, tex.clone());
    unsafe { gl::NamedFramebufferTexture(fbo, it.0, tex.id, 0) }
  }

  Fbo { id: fbo, attachments: map }
}

pub fn gl_draw_arrays(cap: u32, len: i32) {
  unsafe { gl::DrawArrays(cap, 0, len); }
}

pub fn gl_draw_elements(cap: u32, len: i32) {
  unsafe { gl::DrawElements(cap, len, gl::UNSIGNED_INT, 0 as *const c_void) }
}

pub fn gl_make_buf(usage: u32) -> Buf {
  let mut vbo = [0u32; 1];
  unsafe { gl::CreateBuffers(1, vbo.as_mut_ptr()); }
  Buf(vbo[0], usage)
}

pub fn gl_enable(cap: u32) {
  unsafe { gl::Enable(cap); }
}

pub fn gl_disable(cap: u32) {
  unsafe { gl::Disable(cap); }
}

pub fn gl_make_v(attribs: &[(i32, bool)]) -> (Vao, Buf) {
  let vao = gl_make_vao();
  let vbo = gl_make_buf(gl::ARRAY_BUFFER);

  unsafe {
    gl::VertexArrayVertexBuffer(vao.0, 0, vbo.0, 0, attribs.iter().map(|(size, _)| size).sum::<i32>() * 4);

    gl_select_attribs(attribs, &vao);
  }

  (vao, vbo)
}

pub fn gl_make_vi(attribs: &[(i32, bool)]) -> (Vao, Buf, Buf) {
  let vao = gl_make_vao();
  let vbo = gl_make_buf(gl::ARRAY_BUFFER);
  let ibo = gl_make_buf(gl::ELEMENT_ARRAY_BUFFER);

  unsafe {
    gl::VertexArrayVertexBuffer(vao.0, 0, vbo.0, 0, attribs.iter().map(|(size, _)| size).sum::<i32>() * 4);
    gl::VertexArrayElementBuffer(vao.0, ibo.0);

    gl_select_attribs(attribs, &vao);
  }

  (vao, vbo, ibo)
}

pub fn gl_buf_data<DataType>(buf: &Buf, usage: u32, data: &[DataType])
  where DataType: Sized {
  unsafe {
    gl::NamedBufferData(buf.0, (data.len() * std::mem::size_of::<DataType>()) as isize, data.as_ptr() as *const _, usage)
  }
}

unsafe fn gl_select_attribs(attribs: &[(i32, bool)], vao: &Vao) {
  let mut off = 0;
  for i in 0..attribs.len() {
    gl::EnableVertexArrayAttrib(vao.0, i as u32);
    gl::VertexArrayAttribFormat(vao.0, i as u32, attribs[i].0, if attribs[i].1 { gl::FLOAT } else { gl::INT }, gl::FALSE, off);
    gl::VertexArrayAttribBinding(vao.0, i as u32, 0);
    off += attribs[i].0 as u32 * 4;
  }
}

pub fn gl_uniform_1f(name: &'static str, val: f32) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::Uniform1f(CURRENT_SHADER.as_ref().unwrap().1[name], val);
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_uniform_2f(name: &'static str, val: &Vec2) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::Uniform2f(CURRENT_SHADER.as_ref().unwrap().1[name], val.x, val.y);
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_uniform_3f(name: &'static str, val: &Vec3) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::Uniform3f(CURRENT_SHADER.as_ref().unwrap().1[name], val.x, val.y, val.z);
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_uniform_3fv(name: &'static str, val: &[Vec3]) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::Uniform3fv(CURRENT_SHADER.as_ref().unwrap().1[name], val.len() as i32, addr_of!(val[0].x));
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_uniform_4f(name: &'static str, val: &Vec4) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::Uniform4f(CURRENT_SHADER.as_ref().unwrap().1[name], val.x, val.y, val.z, val.w);
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_uniform_1i(name: &'static str, val: i32) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::Uniform1i(CURRENT_SHADER.as_ref().unwrap().1[name], val);
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_uniform_mat4(name: &'static str, val: &Mat4) {
  if unsafe { CURRENT_SHADER.is_null() } {
    panic!("no shader bound");
  }

  if unsafe { CURRENT_SHADER.as_ref().unwrap().1.contains_key(name) } {
    unsafe {
      gl::UniformMatrix4fv(CURRENT_SHADER.as_ref().unwrap().1[name], 1, gl::FALSE, val.as_ref().as_ptr());
    }
    return;
  }

  panic!("uniform {} not found", name);
}

pub fn gl_read_sh(vert: &'static str, frag: &'static str, geom: Option<&'static str>) -> Result<Shader, String> {
  let vert_src = fs::read_to_string(vert).map_err(|e| e.to_string())?;
  let frag_src = fs::read_to_string(frag).map_err(|e| e.to_string())?;
  let geom_src = if let Some(geom) = geom {
    Some(fs::read_to_string(geom).map_err(|e| e.to_string())?.to_owned())
  } else {
    None
  };

  gl_make_sh(vert_src.as_str(), frag_src.as_str(), geom_src)
}

fn gl_attach_shader(prog: u32, src: &str, kind: u32) -> Result<u32, String> {
  let sh = unsafe { gl::CreateShader(kind) };
  gl_shader_source(sh, src);
  unsafe { gl::CompileShader(sh) };
  gl_check_compile(sh)?;
  unsafe { gl::AttachShader(prog, sh) };
  unsafe { gl::DeleteShader(sh) };
  Ok(sh)
}

pub fn gl_make_sh(vert_src: &str, frag_src: &str, geom_src: Option<String>) -> Result<Shader, String> {
  unsafe {
    let prog = gl::CreateProgram();

    gl_attach_shader(prog, vert_src, gl::VERTEX_SHADER)?;
    gl_attach_shader(prog, frag_src, gl::FRAGMENT_SHADER)?;
    if let Some(geom_src) = geom_src {
      gl_attach_shader(prog, geom_src.as_str(), gl::GEOMETRY_SHADER)?;
    }

    gl::LinkProgram(prog);
    gl_check_link(prog)?;

    let mut uniforms = HashMap::new();
    let mut n_uniforms = 0;
    gl::GetProgramiv(prog, gl::ACTIVE_UNIFORMS, addr_of_mut!(n_uniforms));
    for i in 0..n_uniforms {
      let mut len = 0;
      let mut chars = [0; 256];
      let mut ty = 0;
      let mut count = 0;
      gl::GetActiveUniform(prog, i as u32, 256, addr_of_mut!(len), addr_of_mut!(count), addr_of_mut!(ty), chars.as_mut_ptr());

      let name = CStr::from_ptr(chars.as_ptr()).to_str().map_err(|e| e.to_string())?.to_string();
      let loc = gl::GetUniformLocation(prog, chars.as_ptr());

      uniforms.insert(name, loc);
    }

    Ok(Shader(prog, uniforms))
  }
}

fn gl_shader_source(sh: u32, src: &str) {
  unsafe { gl::ShaderSource(sh, 1, [CStr::from_bytes_with_nul_unchecked(src.as_bytes()).as_ptr()].as_ptr(), [src.len() as i32].as_ptr()) }
}

fn gl_check_compile(sh: u32) -> Result<(), String> {
  let mut compiled = 0;
  unsafe { gl::GetShaderiv(sh, gl::COMPILE_STATUS, addr_of_mut!(compiled)) };

  if compiled == gl::FALSE as i32 {
    let mut len = 0;
    unsafe { gl::GetShaderiv(sh, gl::INFO_LOG_LENGTH, addr_of_mut!(len)) };

    let mut log = vec![0; len as usize];
    let mut real_len = 0;
    unsafe { gl::GetShaderInfoLog(sh, len, addr_of_mut!(real_len), log.as_mut_ptr()) };

    let cstr = unsafe { CStr::from_ptr(log.as_ptr()) };
    return Err(cstr.to_str().map_err(|e| e.to_string())?.to_string());
  }

  Ok(())
}

fn gl_check_link(prog: u32) -> Result<(), String> {
  let mut linked = 0;
  unsafe { gl::GetProgramiv(prog, gl::LINK_STATUS, addr_of_mut!(linked)) };

  if linked == gl::FALSE as i32 {
    let mut len = 0;
    unsafe { gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, addr_of_mut!(len)) };

    let mut log = vec![0; len as usize];
    let mut real_len = 0;
    unsafe { gl::GetProgramInfoLog(prog, len, addr_of_mut!(real_len), log.as_mut_ptr()) };

    let cstr = unsafe { CStr::from_ptr(log.as_ptr()) };
    return Err(cstr.to_str().map_err(|e| e.to_string())?.to_string());
  }

  Ok(())
}

fn gl_check_error() -> Result<(), Vec<String>> {
  let mut errs = Vec::new();
  loop {
    let err = unsafe { gl::GetError() };
    if err == gl::NO_ERROR {
      if !errs.is_empty() {
        return Err(errs);
      }

      return Ok(());
    }

    match err {
      gl::INVALID_ENUM => {
        errs.push("INVALID_ENUM".to_string())
      }
      gl::INVALID_VALUE => {
        errs.push("INVALID_VALUE".to_string())
      }
      gl::INVALID_OPERATION => {
        errs.push("INVALID_OPERATION".to_string())
      }
      gl::INVALID_FRAMEBUFFER_OPERATION => {
        errs.push("INVALID_FRAMEBUFFER_OPERATION".to_string())
      }
      gl::OUT_OF_MEMORY => {
        errs.push("OUT_OF_MEMORY".to_string())
      }
      gl::STACK_UNDERFLOW => {
        errs.push("STACK_UNDERFLOW".to_string())
      }
      gl::STACK_OVERFLOW => {
        errs.push("STACK_OVERFLOW".to_string())
      }
      _ => {
        panic!("wtf is going on? error value of {}", err)
      }
    }
  }
}

pub trait FramebufferAttached {
  fn fbo0(&self) -> &Fbo;
}

impl FramebufferAttached for Window {
  fn fbo0(&self) -> &Fbo {
    static mut FBO: Option<Fbo> = None;
    static LAST_WIDTH: AtomicI32 = AtomicI32::new(0);
    static LAST_HEIGHT: AtomicI32 = AtomicI32::new(0);

    let (width, height) = self.get_size();
    if LAST_WIDTH.load(Ordering::Relaxed) != width
      || LAST_HEIGHT.load(Ordering::Relaxed) != height
    {
      unsafe { FBO = None }
    }

    LAST_WIDTH.store(width, Ordering::Relaxed);
    LAST_HEIGHT.store(height, Ordering::Relaxed);

    if unsafe { &FBO }.is_none() {
      let tex = Tex { id: 0, spec: TexSpec { width, height, ..TexSpec::invalid() } };
      let new_fbo = Some(Fbo {
        id: 0,
        attachments: HashMap::from([
          (gl::COLOR_ATTACHMENT0, tex.clone()),
          (gl::COLOR_ATTACHMENT1, tex.clone()),
          (gl::COLOR_ATTACHMENT2, tex.clone()),
          (gl::COLOR_ATTACHMENT3, tex.clone()),
          (gl::COLOR_ATTACHMENT4, tex.clone()),
          (gl::COLOR_ATTACHMENT5, tex.clone()),
          (gl::COLOR_ATTACHMENT6, tex.clone()),
          (gl::COLOR_ATTACHMENT7, tex.clone()),
          (gl::COLOR_ATTACHMENT8, tex.clone()),
          (gl::COLOR_ATTACHMENT9, tex.clone()),
          (gl::COLOR_ATTACHMENT10, tex.clone()),
          (gl::COLOR_ATTACHMENT11, tex.clone()),
          (gl::COLOR_ATTACHMENT12, tex.clone()),
          (gl::COLOR_ATTACHMENT13, tex.clone()),
          (gl::COLOR_ATTACHMENT14, tex.clone()),
          (gl::COLOR_ATTACHMENT15, tex.clone()),
          (gl::COLOR_ATTACHMENT16, tex.clone()),
          (gl::COLOR_ATTACHMENT17, tex.clone()),
          (gl::COLOR_ATTACHMENT18, tex.clone()),
          (gl::COLOR_ATTACHMENT19, tex.clone()),
          (gl::COLOR_ATTACHMENT20, tex.clone()),
          (gl::COLOR_ATTACHMENT21, tex.clone()),
          (gl::COLOR_ATTACHMENT22, tex.clone()),
          (gl::COLOR_ATTACHMENT23, tex.clone()),
          (gl::COLOR_ATTACHMENT24, tex.clone()),
          (gl::COLOR_ATTACHMENT25, tex.clone()),
          (gl::COLOR_ATTACHMENT26, tex.clone()),
          (gl::COLOR_ATTACHMENT27, tex.clone()),
          (gl::COLOR_ATTACHMENT28, tex.clone()),
          (gl::COLOR_ATTACHMENT29, tex.clone()),
          (gl::COLOR_ATTACHMENT30, tex.clone()),
          (gl::COLOR_ATTACHMENT31, tex.clone()),
        ]),
      });

      unsafe {
        FBO = new_fbo;
      }
    }

    unsafe { &FBO }.as_ref().unwrap()
  }
}