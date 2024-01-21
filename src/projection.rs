use crate::utilities::{Axis, interpolate, multiply_color, vector_multiplication, vector_addition, rotate_vector, to_translation_mat4, to_scale_mat4, to_rotation_mat4, mat4_default, multiply_mat4_mat4, to_inverse_translation_mat4, to_inverse_rotation_mat4};

pub struct Camera {
    position: [f32; 3],
    rotation: Option<(Axis, f32)>,
}

impl Camera {
    pub fn new(position: [f32; 3], rotation: Option<(Axis, f32)>) -> Self {
        Self { position, rotation }
    }
    pub fn get_projection_mat4(&mut self) -> [[f32; 4]; 4] {
        let origin = mat4_default();
        let translation_mat4 = to_inverse_translation_mat4(self.position);
        let rotation_mat4 = self.handle_inverse_rotation_mat4();

        let rotated_projection = multiply_mat4_mat4(origin, rotation_mat4);
        let translated_projection = multiply_mat4_mat4(rotated_projection, translation_mat4);

        return translated_projection;
    }

    fn handle_inverse_rotation_mat4(&mut self) -> [[f32; 4]; 4] {
        return match &self.rotation {
            Some((axis, angle)) => { to_inverse_rotation_mat4(*axis, angle.to_radians()) },
            None => mat4_default()
        }
    }
}

pub struct PerspectiveProjection {
    field_of_view: f32,
    aspect_ratio: f32,
    near_clipping_plane: f32,
    far_clipping_plane: f32,
}

impl PerspectiveProjection {
    pub fn new(field_of_view: f32, aspect_ratio: f32, near_clipping_plane: f32, far_clipping_plane: f32) -> Self {
        Self { field_of_view: field_of_view.to_radians(), aspect_ratio, near_clipping_plane, far_clipping_plane }
    }

    pub fn get_projection_mat4(&self) -> [[f32; 4]; 4] {
        let tangent_mid_fov = (self.field_of_view / 2.0).tan();
        let perspective_scale = 1.0 / tangent_mid_fov;
        let depth_range_scale = 1.0 / (self.near_clipping_plane - self.far_clipping_plane);
        let ndc_depth_scale = (self.far_clipping_plane + self.near_clipping_plane) * depth_range_scale;
        let perspective_divide_factor = 2.0 * self.far_clipping_plane * self.near_clipping_plane * depth_range_scale;

        [
            [perspective_scale / self.near_clipping_plane, 0.0, 0.0, 0.0],
            [0.0, perspective_scale, 0.0, 0.0],
            [0.0, 0.0, ndc_depth_scale, perspective_divide_factor],
            [0.0, 0.0, -1.0, 0.0],
        ]
    }
}

pub struct Viewport {
    width: f32,
    height: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    pub fn to_canvas_mat4(&self) -> [[f32; 4]; 4] {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        [
            [half_width, 0.0, 0.0, half_width],
            [0.0, half_height, 0.0, half_height],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
