use crate::geometry::{ Vertex };
use crate::utilities::{ interpolate };

pub struct Rasterizer {
    state: Vec<Vertex>,
}

impl Rasterizer {
    pub const CANVAS: [i32; 2] = [ 1600, 1600 ];
    pub fn new() -> Self {
        Self { state: vec![] }
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
        self.draw_line(point_a, point_b, [0.0, 0.0, 0.0]);
        self.draw_line(point_b, point_c, [0.0, 0.0, 0.0]);
        self.draw_line(point_c, point_a, [0.0, 0.0, 0.0]);
    }
}

pub fn init_rasterizer() -> Rasterizer {
    let mut rasterizer = Rasterizer::new();

    let point_a = [-200, -100];
    let point_b = [240, 120];
    let point_c = [-50, -200];

    rasterizer.draw_wireframe_triangle(point_a, point_b, point_c, [0.0, 0.0, 0.0]);

    return rasterizer;
}
