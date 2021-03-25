use glm::Mat4;
use std::convert::{From, TryInto};

pub fn as_array(matrix: &Mat4) -> [[f32; 4]; 4] {
    let data = matrix.as_slice();
    let column1 = data[0..4].try_into().unwrap();
    let column2 = data[4..8].try_into().unwrap();
    let column3 = data[8..12].try_into().unwrap();
    let column4 = data[12..16].try_into().unwrap();
    [column1, column2, column3, column4]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformationMatrix {
    data: [[f32; 4]; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewMatrix {
    data: [[f32; 4]; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionMatrix {
    data: [[f32; 4]; 4],
}

impl From<[[f32; 4]; 4]> for TransformationMatrix {
    fn from(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }
}

impl From<[[f32; 4]; 4]> for ViewMatrix {
    fn from(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }
}

impl From<[[f32; 4]; 4]> for ProjectionMatrix {
    fn from(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }
}

impl TransformationMatrix {}

impl ProjectionMatrix {}

impl ViewMatrix {}
