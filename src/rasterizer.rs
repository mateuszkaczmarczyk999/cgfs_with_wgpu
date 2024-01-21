use crate::geometry::{ Vertex };
use crate::utilities::{Axis, interpolate, multiply_color, vector_multiplication, vector_addition, rotate_vector, to_translation_mat4, to_scale_mat4, to_rotation_mat4, mat4_default, multiply_mat4_mat4, to_inverse_translation_mat4, to_inverse_rotation_mat4};

const RED: [f32; 3] = [255.0, 0.0, 0.0];
const GREEN: [f32; 3] = [0.0, 255.0, 0.0];
const BLUE: [f32; 3] = [0.0, 0.0, 255.0];
const PURPLE: [f32; 3] = [255.0, 0.0, 255.0];
const YELLOW: [f32; 3] = [255.0, 255.0, 0.0];
const CYAN: [f32; 3] = [0.0, 255.0, 255.0];

pub struct Triangle {
    group: [usize; 3],
    color: [f32; 3],
}

pub struct Box {
    scale: [f32; 3],
    rotation: Option<(Axis, f32)>,
    position: [f32; 3],
}
impl Default for Box {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            rotation: None,
            position: [0.0, 0.0, 0.0],
        }
    }
}
impl Box {
    pub fn new(scale: [f32; 3], position: [f32; 3], rotation: Option<(Axis, f32)>) -> Self {
        Self { scale, position, rotation }
    }
    pub const VERTICES: [[f32; 3]; 8] = [
        [ 1.0,  1.0,  1.0],
        [-1.0,  1.0,  1.0],
        [-1.0, -1.0,  1.0],
        [ 1.0, -1.0,  1.0],
        [ 1.0,  1.0, -1.0],
        [-1.0,  1.0, -1.0],
        [-1.0, -1.0, -1.0],
        [ 1.0, -1.0, -1.0],
    ];
    pub const TRIANGLES: [Triangle; 12] = [
        Triangle { group: [0, 1, 2], color: RED },
        Triangle { group: [0, 2, 3], color: RED },
        Triangle { group: [4, 0, 3], color: GREEN },
        Triangle { group: [4, 3, 7], color: GREEN },
        Triangle { group: [5, 4, 7], color: BLUE },
        Triangle { group: [5, 7, 6], color: BLUE },
        Triangle { group: [1, 5, 6], color: YELLOW },
        Triangle { group: [1, 6, 2], color: YELLOW },
        Triangle { group: [4, 5, 1], color: PURPLE },
        Triangle { group: [4, 1, 0], color: PURPLE },
        Triangle { group: [2, 6, 7], color: CYAN },
        Triangle { group: [2, 7, 3], color: CYAN },
    ];

    pub fn get_geometry(&mut self) -> (Vec<[f32; 3]>, Vec<Triangle>) {
        let scale = self.scale;
        let position = self.position;

        let transformed = Self::VERTICES
            .iter()
            .map(|&f| vector_multiplication(f, scale))
            .map(|f| self.handle_rotation(f))
            .map(|f| vector_addition(f, position))
            .collect();

        return (transformed, Vec::from(Self::TRIANGLES));
    }

    fn handle_rotation(&mut self, vertex: [f32; 3]) -> [f32; 3] {
        return match &self.rotation {
            Some(rotation) => { rotate_vector(vertex, rotation) },
            None => vertex
        }
    }

    pub fn get_model_mat4(&mut self) -> [[f32; 4]; 4] {
        let origin = mat4_default();
        let translation_mat4 = to_translation_mat4(self.position);
        let scale_mat4 = to_scale_mat4(self.scale);
        let rotation_mat4 = self.handle_rotation_mat4();

        let translated_projection = multiply_mat4_mat4(origin, translation_mat4);
        let rotated_projection = multiply_mat4_mat4(translated_projection, rotation_mat4);
        let scaled_projection =  multiply_mat4_mat4(rotated_projection, scale_mat4);

        return scaled_projection;
    }

    fn handle_rotation_mat4(&mut self) -> [[f32; 4]; 4] {
        return match &self.rotation {
            Some((axis, angle)) => { to_rotation_mat4(*axis, angle.to_radians()) },
            None => mat4_default()
        }
    }
}

pub struct Rasterizer {
    state: Vec<Vertex>,
}

impl Rasterizer {
    pub const CANVAS: [i32; 2] = [ 1600, 1600 ];
    // Canvas size

    pub const VIEWPORT: [f32; 3] = [1.0, 1.0, 1.0];
    // Viewport width, height and depth which is camera distance

    pub const CAMERA_POSITION: [f32; 3] = [0.0, 0.0, 0.0];

    pub fn new() -> Self {
        Self { state: vec![] }
    }

    fn render_triangle(&mut self, indices: [usize; 3], projection: Vec<[i32; 2]>, rgb: [f32; 3]) {
        let point_a = projection[indices[0]];
        let point_b = projection[indices[1]];
        let point_c = projection[indices[2]];
        self.draw_wireframe_triangle(point_a, point_b, point_c, rgb);
    }

    fn render_object(&mut self, vertices: Vec<[f32; 3]>, geometries: Vec<Triangle>) {
        let mut projection: Vec<[i32; 2]> = vec![];

        for vertex in vertices.iter() {
            projection.push(self.project_vertex(vertex))
        }
        for geometry in geometries.iter() {
            self.render_triangle(geometry.group, projection.clone(), geometry.color);
        }
    }

    pub fn project_vertex(&mut self, vertex: &[f32; 3]) -> [i32; 2] {
        fn viewport_to_canvas(x: f32, y: f32) -> [i32; 2] {
            let x_pos = x * Rasterizer::CANVAS[0] as f32 / Rasterizer::VIEWPORT[0] as f32;
            let y_pos = y * Rasterizer::CANVAS[1] as f32 / Rasterizer::VIEWPORT[1] as f32;
            return [x_pos as i32, y_pos as i32];
        }

        let x_projection = vertex[0] * Rasterizer::VIEWPORT[2] / vertex[2];
        let y_projection = vertex[1] * Rasterizer::VIEWPORT[2] / vertex[2];

        return viewport_to_canvas(x_projection, y_projection);
    }

    pub fn get_state(&mut self) -> &[Vertex] {
        return self.state.as_slice();
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, rgb: [f32; 3]) {
        let x_cord = x as f32 / (Self::CANVAS[0] / 2) as f32;
        let y_cord = y as f32 / (Self::CANVAS[1] / 2) as f32;
        let color = [ rgb[0] / 255.0, rgb[1] / 255.0, rgb[2] / 255.0 ];
        self.state.push(Vertex { position: [x_cord, y_cord, 0.0], color });
    }

    pub fn draw_line(&mut self, point_a: [i32; 2], point_b: [i32; 2], rgb: [f32; 3]) {
        let mut a = point_a;
        let mut b = point_b;

        if (b[0] - a[0]).abs() > (b[1] - a[1]).abs() {
            if a[0] > b[0] { std::mem::swap(&mut a, &mut b); }
            let y_values = interpolate(a[0], a[1] as f32, b[0], b[1] as f32);
            for x in a[0] ..= b[0] {
                let y_idx = (x - a[0]) as usize;
                self.put_pixel(x, y_values[y_idx], rgb);
            }
        } else {
            if a[1] > b[1] { std::mem::swap(&mut a, &mut b); }
            let x_values = interpolate(a[1], a[0] as f32, b[1], b[0] as f32);
            for y in a[1] ..= b[1] {
                let x_idx = (y - a[1]) as usize;
                self.put_pixel(x_values[x_idx], y, rgb);
            }
        }
    }

    pub fn draw_wireframe_triangle(&mut self, point_a: [i32; 2], point_b: [i32; 2], point_c: [i32; 2], rgb: [f32; 3]) {
        self.draw_line(point_a, point_b, rgb);
        self.draw_line(point_b, point_c, rgb);
        self.draw_line(point_c, point_a, rgb);
    }

    pub fn draw_filled_triangle(&mut self, point_a: [i32; 2], point_b: [i32; 2], point_c: [i32; 2], rgb: [f32; 3]) {
        let mut a = point_a;
        let mut b = point_b;
        let mut c = point_c;

        if b[1] < a[1] { std::mem::swap(&mut b, &mut a); }
        if c[1] < a[1] { std::mem::swap(&mut c, &mut a); }
        if c[1] < b[1] { std::mem::swap(&mut c, &mut b); }

        let mut shade: [f32; 3] = [10.0, 0.0, 100.0];

        let mut x_vals_a_to_b = interpolate(a[1], a[0] as f32, b[1], b[0] as f32);
        let mut shade_a_to_b = interpolate(a[1], shade[0], b[1], shade[1]);

        let mut x_vals_b_to_c = interpolate(b[1], b[0] as f32, c[1], c[0] as f32);
        let mut shade_b_to_c = interpolate(b[1], shade[1], c[1], shade[2]);

        let mut x_vals_a_to_c = interpolate(a[1], a[0] as f32, c[1], c[0] as f32);
        let mut shade_a_to_c = interpolate(a[1], shade[0], c[1], shade[2]);

        let _ = x_vals_a_to_b.pop();
        let short_sides = [&x_vals_a_to_b[..], &x_vals_b_to_c[..]].concat();

        let _ = shade_a_to_b.pop();
        let shade_short_sides = [&shade_a_to_b[..], &shade_b_to_c[..]].concat();

        let mid_idx = short_sides.len() / 2;
        let short_sides_on_the_right = x_vals_a_to_c[mid_idx] < short_sides[mid_idx];
        let (x_left, x_right, shade_left, shade_right) =
            if short_sides_on_the_right {
                (x_vals_a_to_c, short_sides, shade_a_to_c, shade_short_sides)
            }
            else {
                (short_sides, x_vals_a_to_c, shade_short_sides, shade_a_to_c)
            };

        for y in a[1] ..= c[1] {
            let inverse_y_idx = (y - a[1]) as usize;
            let x_left_edge = x_left[inverse_y_idx];
            let x_right_edge = x_right[inverse_y_idx];

            let x_shades = interpolate(
                x_left_edge,
                shade_left[inverse_y_idx] as f32,
                x_right_edge,
                shade_right[inverse_y_idx] as f32,
            );

            for x in x_left_edge ..= x_right_edge {
                let inverse_x_idx = (x - x_left_edge) as usize;

                let shade_factor: f32 = x_shades[inverse_x_idx] as f32 / 100.0;
                let shaded_color = multiply_color(rgb, shade_factor);
                self.put_pixel(x, y, shaded_color);
            }
        }
    }
}

pub fn init_rasterizer() -> Rasterizer {
    let mut rasterizer = Rasterizer::new();

    let mut box_a = Box::new([1.0, 1.0, 1.0], [-1.5, 0.0, 7.0], Some((Axis::Y, 45.0)));
    let (box_a_position, box_a_triangles) = box_a.get_geometry();
    rasterizer.render_object(box_a_position, box_a_triangles);

    let mut box_b = Box::new([1.0, 4.0, 1.0], [1.25, 2.0, 7.5], Some((Axis::X, 45.0)));
    let (box_b_position, box_b_triangles) = box_b.get_geometry();
    rasterizer.render_object(box_b_position, box_b_triangles);

    return rasterizer;
}
