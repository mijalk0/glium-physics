use crate::model::ModelHandle;
use crate::physics::PhysicsState;
use crate::renderer::RendererState;
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::ColliderBuilder;

use na::Point3;

pub struct Map {
    pub model: ModelHandle,
    pub trimesh: RigidBodyHandle,
}

impl Map {
    pub fn from_model(model: ModelHandle, rs: &RendererState, ps: &mut PhysicsState) -> Self {
        let mut vertices = Vec::new();
        let mut indices: Vec<Vec<u32>> = Vec::new();

        for mesh in rs.get_model(&model).meshes.iter() {
            let mut offset = 0;
            for primitive in mesh.primitives.iter() {
                let new_vertices = primitive.vbo.read().unwrap();
                let new_indices = primitive
                    .ibo
                    .read()
                    .unwrap()
                    .into_iter()
                    .map(|index| (index + offset as u32))
                    .collect();

                offset += new_vertices.len();

                vertices.push(new_vertices);
                indices.push(new_indices);
            }
        }

        let vertices: Vec<Point3<_>> = vertices
            .into_iter()
            .flatten()
            .map(|vertex| {
                let position = vertex.position();
                Point3::new(position[0], position[1], position[2])
            })
            .collect();

        let indices: Vec<u32> = indices.into_iter().flatten().collect();
        let mut indices_grouped: Vec<[u32; 3]> = Vec::with_capacity(indices.len() / 3);

        for i in 0..indices.len() / 3 {
            indices_grouped.push([indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]]);
        }

        //        let index_chunks: Vec<[u32; 3]> = index_chunks
        //            .into_iter()
        //            .map(|index_chunk: Vec<u32>| index_chunk.into_iter().map(|index| [index; 3]))
        //            .flatten()
        //            .collect();

        println!("started making convex shape");

        let rigidbody = RigidBodyBuilder::new_static().gravity_scale(0.0).build();
        let collider = ColliderBuilder::trimesh(vertices, indices_grouped)
            .restitution(0.3)
            .build();

        println!("finished making convex shape");

        let trimesh = ps.bodies.insert(rigidbody);
        ps.colliders.insert(collider, trimesh, &mut ps.bodies);

        Self { model, trimesh }
    }
}
