use std::iter::zip;
use std::ops::Deref;
use glam::{Vec3, Vec4};
use russimp::scene::{PostProcess, Scene};
use crate::hana::cvt::ScuffedInto;
use crate::hana::glu::{Buf, FLOAT_3, FLOAT_4, gl_gen_vi, Vao};

#[repr(packed(4))]
#[derive(Clone)]
pub struct Vertex {
  pub pos: Vec3,
  pub norm: Vec3,
  pub tint: Vec4,
  // tex: Vec2,
}

impl Vertex {
  pub fn empty() -> Vertex {
    Vertex {
      pos: Vec3::ZERO,
      norm: Vec3::ZERO,
      tint: Vec4::ONE
    }
  }
}

pub struct Mesh {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u32>,
  pub gl: (Vao, Buf, Buf)
}

impl Mesh {
  pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
    let res = Mesh {
      vertices,
      indices,
      gl: gl_gen_vi(&[FLOAT_3, FLOAT_3, FLOAT_4])
    };

    let (_, vbo, ibo) = &res.gl;

    vbo.data(gl::DYNAMIC_DRAW, res.vertices.as_slice());
    ibo.data(gl::DYNAMIC_DRAW, res.indices.as_slice());

    res
  }
}

pub struct Model(pub Vec<Mesh>);

impl Model {
  pub fn new(path: &str) -> Result<Model, String> {
    let scene =
      Scene::
      from_file(path, vec![PostProcess::Triangulate, PostProcess::GenerateNormals])
        .map_err(|e| e.to_string())?;
    if let None = scene.root {
      return Err("Failed to load model!".into())
    }

    let root = scene.root.as_deref().unwrap();

    let res = Model(cvt_node(&root, &scene));
    Ok(res)
  }
}

fn cvt_node(root: &russimp::node::Node, scene: &Scene) -> Vec<Mesh> {
  let mut res = Vec::new();
  for mesh in &root.meshes {
    res.push(cvt_mesh(&scene.meshes[*mesh as usize]));
  }

  for child in root.children.borrow().deref() {
    res.append(&mut cvt_node(child, scene))
  }

  res
}

fn cvt_mesh(mesh: &russimp::mesh::Mesh) -> Mesh {
  let mut vertices = Vec::new();
  let mut indices = Vec::new();

  for i in zip(&mesh.vertices, &mesh.normals) {
    vertices.push(Vertex {
      pos: i.0.cvt() * Vec3::new(-1., 1., 1.),
      norm: i.1.cvt() * Vec3::new(-1., 1., 1.),
      tint: Vec4::ONE
    })
  }

  for face in &mesh.faces {
    for index in &face.0 {
      indices.push(*index)
    }
  }

  Mesh::new(vertices, indices)
}