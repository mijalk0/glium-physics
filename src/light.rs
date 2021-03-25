use glm::Vec3;

#[derive(Copy, Clone)]
pub struct Light {
    pub position: [f32; 3],
    pub colour: [f32; 3],
}

impl Light {
    pub fn new(position: Vec3, colour: Option<Vec3>) -> Self {
        let position = [position.x, position.y, position.z];
        let colour = if let Some(colour) = colour {
            [colour.x, colour.y, colour.z]
        } else {
            [1.0, 1.0, 1.0]
        };
        Self { position, colour }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self{
            position: [std::f32::MAX, std::f32::MAX, std::f32::MAX],
            colour: [std::f32::MAX, std::f32::MAX, std::f32::MAX],
        }
    }
}
