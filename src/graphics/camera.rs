use std::f32::consts;

use glam::{Mat4, Vec3, Vec4};

pub struct Camera {
    projection: Mat4,
    view: Mat4,
}

const FOV_Y_RADS: f32 = consts::FRAC_PI_2;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4= Mat4::from_cols(
Vec4::new(1.0, 0.0, 0.0, 0.0),
Vec4::new(0.0, 1.0, 0.0, 0.0),
Vec4::new(0.0, 0.0, 0.5, 0.0),
Vec4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    /// Creates a new Camera with the given projection and view matrices.
    pub fn new(aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        let projection = Mat4::perspective_rh(FOV_Y_RADS, aspect_ratio, z_near, z_far);
        let view = Mat4::look_at_rh(Vec3::new(2.0, 1.0, 2.0), Vec3::ZERO, Vec3::Y);

        Self { projection, view }
    }

    /// Returns the projection matrix of the camera.
    pub fn projection(&self) -> Mat4 {
        self.projection
    }

    /// Returns the view matrix of the camera.
    pub fn view(&self) -> Mat4 {
        self.view
    }

    /// Returns the combined projection and view matrix of the camera.
    pub fn projection_view_matrix(&self) -> Mat4 {
        OPENGL_TO_WGPU_MATRIX * self.projection * self.view
    }
}
