use glam::{Mat4, Vec3};
use glfw::Key;

#[derive(Clone)]
pub struct Camera {
  pub pos: Vec3,
  pub prev_pos: Vec3,
  pub front: Vec3,
  pub right: Vec3,
  pub up: Vec3,
  pub world_up: Vec3,
  pub yaw: f32,
  pub pitch: f32,
  pub fov: f32,
  pub speed: f32,
  pub sensitivity: f32
}

impl Camera {
  pub fn new() -> Camera {
    Camera {
      pos: Vec3::ZERO,
      prev_pos: Vec3::ZERO,
      front: Vec3::ZERO,
      right: Vec3::ZERO,
      up: Vec3::Y,
      world_up: Vec3::Y,
      yaw: 0.,
      pitch: 0.,
      fov: 60.,
      speed: 1.,
      sensitivity: 0.3
    }
  }

  pub fn update(&mut self) {
    self.front = Vec3 {
      x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
      y: self.pitch.to_radians().sin(),
      z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
    };

    self.up = self.right.cross(self.front).normalize();
    self.right = self.world_up.cross(self.front).normalize();
  }

  pub fn key(&mut self, key: Key) {
    self.update();

    match key {
      Key::W => {
        self.pos += (self.front * Vec3::new(1., 0., 1.)).normalize() * 0.1;
      }
      Key::S => {
        self.pos -= (self.front * Vec3::new(1., 0., 1.)).normalize() * 0.1;
      }
      Key::A => {
        self.pos -= self.right * 0.1;
      }
      Key::D => {
        self.pos += self.right * 0.1;
      }
      Key::Space => {
        self.pos += self.world_up * 0.1;
      }
      Key::LeftShift => {
        self.pos -= self.world_up * 0.1;
      }
      _ => {}
    }
  }

  pub fn mouse_move(&mut self, mut x_off: f32, mut y_off: f32) {
    x_off *= self.sensitivity;
    y_off *= self.sensitivity;

    self.yaw -= x_off;
    self.pitch -= y_off;

    if self.pitch > 89.0 {
      self.pitch = 89.0;
    } else if self.pitch < -89.0 {
      self.pitch = -89.0;
    }

    self.update();
  }

  pub fn eye(&self, tick_delta: f32) -> Vec3 {
    Vec3::lerp(self.prev_pos, self.pos, tick_delta)
  }

  pub fn look_at(&self, tick_delta: f32) -> Mat4 {
    Mat4::look_at_lh(self.eye(tick_delta), self.eye(tick_delta) + self.front, self.world_up)
  }

  pub fn proj(&self, aspect: f32) -> Mat4 {
    Mat4::perspective_lh(self.fov.to_radians(), aspect, 0.1, 100.0)
  }
}