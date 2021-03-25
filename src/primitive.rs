use crate::vertex::Vertex;
use glium::backend::Facade;
use glium::IndexBuffer;
use glium::VertexBuffer;

pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_index: usize,
    pub vbo: VertexBuffer<Vertex>,
    pub ibo: IndexBuffer<u32>,
}

impl Primitive {
    pub fn new<F: ?Sized>(
        facade: &F,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        material_index: usize,
    ) -> Self
    where
        F: Facade,
    {
        let vbo = glium::VertexBuffer::new(facade, &vertices).unwrap();
        let ibo =
            glium::IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)
                .unwrap();

        Self {
            vertices,
            indices,
            material_index,
            vbo,
            ibo,
        }
    }
}
