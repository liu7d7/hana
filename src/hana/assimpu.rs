use glam::{Vec2, Vec3};
use russimp::{Vector2D, Vector3D};

pub trait ScuffedInto<T> {
  fn cvt(&self) -> T;
}

impl ScuffedInto<Vec3> for Vector3D {
  fn cvt(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}

impl ScuffedInto<Vec2> for Vector2D {
  fn cvt(&self) -> Vec2 {
    Vec2::new(self.x, self.y)
  }
}