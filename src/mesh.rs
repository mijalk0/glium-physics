use crate::primitive::Primitive;
use na::{Isometry3, Vector3};

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub base_isometry: Isometry3<f32>,
    pub updated_isometry: Isometry3<f32>,
    pub scaling: Vector3<f32>,
}

impl Mesh {
    pub fn new(
        primitives: Vec<Primitive>,
        base_isometry: Isometry3<f32>,
        scaling: Vector3<f32>,
    ) -> Self {
        Self {
            primitives,
            base_isometry,
            updated_isometry: base_isometry,
            scaling,
        }
    }

    pub fn base_isometry(&self) -> [[f32; 4]; 4] {
        self.base_isometry.to_homogeneous().into()
    }

    pub fn transformation(&self) -> [[f32; 4]; 4] {
        self.updated_isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scaling)
            .into()
    }

    pub fn update_isometry(&mut self, new_isometry: Isometry3<f32>) {
        self.updated_isometry = new_isometry;
    }
}
