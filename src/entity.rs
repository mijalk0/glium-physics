use crate::model::Model;
use glm::Mat4;
use std::convert::TryInto;

pub struct Entity {
    pub model: Model,
    pub model_matrix: Mat4,
}

impl Entity {
    pub fn new(model: Model) -> Self {
        let model_matrix = glm::Mat4::identity();

        Self {
            model,
            model_matrix,
        }
    }

    pub fn model_matrix_uniform(&self) -> [[f32; 4]; 4] {
        let data = self.model_matrix.as_slice();
        let column1 = data[0..4].try_into().unwrap();
        let column2 = data[4..8].try_into().unwrap();
        let column3 = data[8..12].try_into().unwrap();
        let column4 = data[12..16].try_into().unwrap();
        [column1, column2, column3, column4]
    }
}
