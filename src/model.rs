use crate::material::Material;
use crate::mesh::Mesh;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>, materials: Vec<Material>) -> Self {
        Self { meshes, materials }
    }
}

pub struct ModelHandle {
    index: usize,
}

impl ModelHandle {
    pub fn new(index: usize) -> Self {
        ModelHandle { index }
    }
    pub fn index(&self) -> usize {
        self.index
    }
}
