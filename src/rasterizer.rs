use crate::geometry::{ Vertex };
use crate::utilities::{interpolate, multiply_color};

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

    pub fn project_vertex(&mut self, vertex: [f32; 3]) -> [i32; 2] {
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

    // let point_a = [-700, 600];
    // let point_b = [540, 120];
    // let point_c = [-50, -700];
    //
    // rasterizer.draw_wireframe_triangle(point_a, point_b, point_c, [0.0, 0.0, 0.0]);
    // rasterizer.draw_filled_triangle(point_a, point_b, point_c, [0.0, 255.0, 0.0]);

    // The four "front" vertices
    let vAf = [-2.0, -0.5, 5.0];
    let vBf = [-2.0,  0.5, 5.0];
    let vCf = [-1.0,  0.5, 5.0];
    let vDf = [-1.0, -0.5, 5.0];

    // The four "back" vertices
    let vAb = [-2.0, -0.5, 6.0];
    let vBb = [-2.0,  0.5, 6.0];
    let vCb = [-1.0,  0.5, 6.0];
    let vDb = [-1.0, -0.5, 6.0];

    const PURPLE: [f32; 3] = [255.0, 0.0, 255.0];
    const RED: [f32; 3] = [255.0, 0.0, 0.0];
    const GREEN: [f32; 3] = [0.0, 255.0, 0.0];

    let a = rasterizer.project_vertex(vAf);
    let b = rasterizer.project_vertex(vBf);
    let c = rasterizer.project_vertex(vCf);
    let d = rasterizer.project_vertex(vDf);

    let e = rasterizer.project_vertex(vAb);
    let f = rasterizer.project_vertex(vBb);
    let g = rasterizer.project_vertex(vCb);
    let h = rasterizer.project_vertex(vDb);

    // The front face
    rasterizer.draw_line(a, b, PURPLE);
    rasterizer.draw_line(b, c, PURPLE);
    rasterizer.draw_line(c, d, PURPLE);
    rasterizer.draw_line(d, a, PURPLE);

    // The back face
    rasterizer.draw_line(e, f, RED);
    rasterizer.draw_line(f, g, RED);
    rasterizer.draw_line(g, h, RED);
    rasterizer.draw_line(h, e, RED);

    // The front-to-back edges
    rasterizer.draw_line(a, e, GREEN);
    rasterizer.draw_line(b, f, GREEN);
    rasterizer.draw_line(c, g, GREEN);
    rasterizer.draw_line(d, h, GREEN);

    return rasterizer;
}
