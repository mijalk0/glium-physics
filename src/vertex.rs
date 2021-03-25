use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{IndexBuffer, VertexBuffer};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    texture_coord: [f32; 2],
    normal: [f32; 3],
    tangent: [f32; 3],
}

impl Vertex {
    pub fn new(
        position: [f32; 3],
        texture_coord: [f32; 2],
        normal: [f32; 3],
        tangent: [f32; 3],
    ) -> Self {
        Self {
            position,
            texture_coord,
            normal,
            tangent,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }
}

pub fn calculate_tangent(
    position1: [f32; 3],
    position2: [f32; 3],
    position3: [f32; 3],
    texture_coord1: [f32; 2],
    texture_coord2: [f32; 2],
    texture_coord3: [f32; 2],
) -> [f32; 3] {
    let edge1 = [
        position2[0] - position1[0],
        position2[1] - position1[1],
        position2[2] - position1[2],
    ];

    let edge2 = [
        position3[0] - position1[0],
        position3[1] - position1[1],
        position3[2] - position1[2],
    ];

    let delta_u1 = texture_coord2[0] - texture_coord1[0];
    let delta_v1 = texture_coord2[1] - texture_coord1[1];
    let delta_u2 = texture_coord3[0] - texture_coord1[0];
    let delta_v2 = texture_coord3[1] - texture_coord1[1];

    let inverse_coeffecient = 1.0 / (delta_u1 * delta_v2 - delta_u2 * delta_v1);
    let tangent_x = inverse_coeffecient * (delta_v2 * edge1[0] - delta_v1 * edge2[0]);
    let tangent_y = inverse_coeffecient * (delta_v2 * edge1[1] - delta_v1 * edge2[1]);
    let tangent_z = inverse_coeffecient * (delta_v2 * edge1[2] - delta_v1 * edge2[2]);

    [tangent_x, tangent_y, tangent_z]
}

#[derive(Copy, Clone)]
pub struct SkyboxVertex {
    position: [f32; 3],
}

impl SkyboxVertex {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position }
    }
}

#[derive(Copy, Clone)]
pub struct QuadVertex {
    position: [f32; 3],
    texture_coord: [f32; 2],
}

impl QuadVertex {
    pub fn new(position: [f32; 3], texture_coord: [f32; 2]) -> Self {
        Self {
            position,
            texture_coord,
        }
    }

    pub fn texture_quad_vbo<F: ?Sized>(facade: &F) -> VertexBuffer<QuadVertex>
    where
        F: Facade,
    {
        let vertices = [
            Self::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
            Self::new([1.0, -1.0, 1.0], [1.0, 0.0]),
            Self::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
            Self::new([1.0, 1.0, 1.0], [1.0, 1.0]),
        ];

        VertexBuffer::new(facade, &vertices).unwrap()
    }

    pub fn texture_quad_ibo<F: ?Sized>(facade: &F) -> IndexBuffer<u32>
    where
        F: Facade,
    {
        let indices: [u32; 6] = [0, 1, 2, 2, 1, 3];
        IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices).unwrap()
    }
}

//#[derive(Copy, Clone, Eq, PartialEq, Hash)]
//struct VertexKey {
//    position: [NotNan<f32>; 3],
//    texture_coord: [NotNan<f32>; 2],
//    normal: [NotNan<f32>; 3],
//}
//
//impl VertexKey {
//    pub fn new(position: [f32; 3], texture_coord: [f32; 2], normal: [f32; 3]) -> Self {
//        let position_notnan = [
//            NotNan::new(position[0]).unwrap(),
//            NotNan::new(position[1]).unwrap(),
//            NotNan::new(position[2]).unwrap(),
//        ];
//
//        let texture_coord_notnan = [
//            NotNan::new(texture_coord[0]).unwrap(),
//            NotNan::new(texture_coord[1]).unwrap(),
//        ];
//        let normal_notnan = [
//            NotNan::new(normal[0]).unwrap(),
//            NotNan::new(normal[1]).unwrap(),
//            NotNan::new(normal[2]).unwrap(),
//        ];
//        Self {
//            position: position_notnan,
//            texture_coord: texture_coord_notnan,
//            normal: normal_notnan,
//        }
//    }
//}

//pub fn load_raw_vertex_data(
//    indices: &[(usize, usize, usize)],
//    positions: &[[f32; 3]],
//    texture_coords: &[[f32; 2]],
//    normals: &[[f32; 3]],
//) -> (Vec<u32>, Vec<Vertex>) {
//    let mut indices_vec: Vec<u32> = Vec::new();
//    let mut vertex_vec: Vec<Vertex> = Vec::new();
//
//    let mut data_to_indices_map: HashMap<VertexKey, u32> = HashMap::new();
//
//    let mut count: u32 = 0;
//    for (position_index, texture_coord_index, normal_index) in indices {
//        let position = positions[*position_index];
//        let texture_coord = texture_coords[*texture_coord_index];
//        let normal = normals[*normal_index];
//
//        let temp_vertex_key = VertexKey::new(position, texture_coord, normal);
//
//        match data_to_indices_map.get(&temp_vertex_key) {
//            Some(index) => indices_vec.push(*index),
//            None => {
//                indices_vec.push(count);
//                let temp_vertex = Vertex::new(position, texture_coord, normal, tange);
//                vertex_vec.push(temp_vertex);
//                data_to_indices_map.insert(temp_vertex_key, count);
//                count += 1;
//            }
//        }
//    }
//    (indices_vec, vertex_vec)
//}

implement_vertex!(Vertex, position, texture_coord, normal, tangent);
implement_vertex!(SkyboxVertex, position);
implement_vertex!(QuadVertex, position, texture_coord);
