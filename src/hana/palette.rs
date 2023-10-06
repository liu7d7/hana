use glam::Vec3;

#[repr(packed(4))]
pub struct Color {
  pub highlight: Vec3,
  pub diffuse: Vec3,
  pub shade: Vec3,
}

pub fn hex_to_vec3(hex: u32) -> Vec3 {
  Vec3 {
    x: (hex >> 16) as u8 as f32 / 255.,
    y: (hex >> 08) as u8 as f32 / 255.,
    z: (hex >> 00) as u8 as f32 / 255.
  }
}

impl Color {
  pub fn new(highlight: Vec3, diffuse: Vec3, shade: Vec3) -> Color {
    Color { highlight, diffuse, shade }
  }

  pub fn hex(highlight: u32, diffuse: u32, shade: u32) -> Color {
    Self::new(hex_to_vec3(highlight), hex_to_vec3(diffuse), hex_to_vec3(shade))
  }
}