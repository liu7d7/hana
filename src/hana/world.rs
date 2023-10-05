use std::cell::{Ref, RefCell};
use std::rc::Rc;
use glam::{IVec2, Vec3, Vec3Swizzles};
use crate::hana::cvt::ScuffedInto;
use crate::hana::entity::Object;

pub struct World {
  pub objs: Vec<Rc<RefCell<Object>>>,
  pub space_part: [[Vec<Rc<RefCell<Object>>>; 32]; 32]
}

fn world_to_space_part(pos: Vec3) -> IVec2 {
  (pos.xz() / 16.).floor().cvt() + IVec2::new(16, 16)
}

impl World {
  fn do_space_part(&mut self) {
    for i in 0..32 {
      for j in 0..32 {
        self.space_part[i][j].clear();
      }
    }

    for it in &self.objs {
      let pos = world_to_space_part(*it.borrow().pos());
      if pos.x >= 32 || pos.y >= 32 || pos.y < 0 || pos.x < 0 {
        panic!("pos out of range! {}", it.borrow().pos());
      }

      self.space_part[pos.x as usize][pos.y as usize].push(it.clone());
    }
  }

  pub fn tick(&mut self, eye: Vec3, update_distance: i32) {
    let pos = world_to_space_part(eye);

    for i in pos.x - update_distance..pos.x + update_distance {
      for j in pos.y - update_distance..pos.y + update_distance {
        for it in &self.space_part[i as usize][j as usize] {
          it.borrow_mut().tick()
        }
      }
    }
  }

  pub fn draw(&self, tick_delta: f32, eye: Vec3, render_distance: i32) {
    let pos = world_to_space_part(eye);

    for i in pos.x - render_distance..pos.x + render_distance {
      for j in pos.y - render_distance..pos.y + render_distance {
        for it in &self.space_part[i as usize][j as usize] {
          it.borrow().draw(tick_delta);
        }
      }
    }
  }
}