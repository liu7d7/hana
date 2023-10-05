use glam::Vec3;
use crate::hana::model::Model;

pub enum Object {
  Any { pos: Vec3, model: Model },
  Player { pos: Vec3, model: Model },
}

impl Object {
  pub fn tick(&mut self) {
    match self {
      Object::Any { .. } => {}
      Object::Player { .. } => {}
    }
  }

  pub fn draw(&self, tick_delta: f32) {
    match self {
      Object::Any { .. } => {

      }
      Object::Player { .. } => {

      }
    }
  }

  pub fn pos(&self) -> &Vec3 {
    match self {
      Object::Any { pos, .. } => {
        pos
      }
      Object::Player { pos, .. } => {
        pos
      }
    }
  }
}