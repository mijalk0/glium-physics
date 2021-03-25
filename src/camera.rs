use nalgebra::{IsometryMatrix3, Perspective3, Point3, Vector3};

#[derive(Copy, Clone)]
pub struct Camera {
    view_matrix: IsometryMatrix3<f32>,
    projection_matrix: Perspective3<f32>,
}

impl Camera {
    pub fn new(view_matrix: IsometryMatrix3<f32>, projection_matrix: Perspective3<f32>) -> Self {
        Self {
            view_matrix,
            projection_matrix,
        }
    }

    /// Current position of the camera.
    pub fn position(&self) -> [f32; 3] {
        let camera_pos = self.view_matrix.inverse().translation.vector;
        let x = camera_pos.x;
        let y = camera_pos.y;
        let z = camera_pos.z;
        [x, y, z]
    }

    /// Gets a view matrix which only has rotation and ignores position. Used for skybox
    /// calculations.
    pub fn skybox_view_matrix(&self) -> [[f32; 4]; 4] {
        self.view_matrix
            .rotation
            .to_homogeneous()
            .into()
    }

    /// View matrix of the camera. Represents rotation and position.
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        self.view_matrix.to_homogeneous().into()
    }

    /// Projection matrix of the camera. Represents factors like FOV/aspect ratio.
    pub fn projection_matrix(&self) -> [[f32; 4]; 4] {
        self.projection_matrix.to_homogeneous().into()
    }

    fn default_view_matrix() -> IsometryMatrix3<f32> {
        let view_eye = Point3::new(2.5, 2.5, -1.5);
        let view_target = Point3::new(-1.0, 1.5, -3.0);
        let view_up = Vector3::new(0.0, 1.0, 0.0);
        let view_matrix = IsometryMatrix3::look_at_rh(&view_eye, &view_target, &view_up);
        view_matrix
    }

    fn default_projection_matrix() -> Perspective3<f32> {
        let projection_aspect = 4.0 / 3.0;
        let projection_fov = 1.047198;
        let projection_near = 0.1;
        let projection_far = 100.0;
        let projection_matrix = Perspective3::new(
            projection_aspect,
            projection_fov,
            projection_near,
            projection_far,
        );
        projection_matrix
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_matrix: Self::default_view_matrix(),
            projection_matrix: Self::default_projection_matrix(),
        }
    }
}
