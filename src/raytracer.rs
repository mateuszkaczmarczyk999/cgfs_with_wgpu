use std::ops::RangeInclusive;
use crate::utilities::{
    dot_product,
    vector_length,
    vector_subtraction,
    vector_addition,
    scale_vector,
    color_to_vector,
    reverse_vector,multiply_color
};
use crate::geometry::{
    Vertex,
    Sphere,
    Light,
    LightMode
};
pub struct Raytracer {
    state: Vec<Vertex>,
    scene: Vec<Sphere>,
    lights: Vec<Light>,
}
impl Raytracer {
    pub const CANVAS: [i32; 2] = [ 1600, 1600 ];
    // Canvas size

    const VIEWPORT: [f32; 3] = [1.0, 1.0, 1.0];
    // Viewport width, height and depth which is camera distance

    const CAMERA_POSITION: [f32; 3] = [0.0, 0.0, 0.0];
    // Camera 3D position

    const BACKGROUND_COLOR: [i32; 3] = [0, 0, 0];
    // Default color for scene

    pub fn new() -> Self {
        Self { state: vec![], scene: vec![], lights: vec![] }
    }
    pub fn put_pixel(&mut self, x: i32, y: i32, rgb: [f32; 3]) {
        let x_cord = x as f32 / (Self::CANVAS[0] / 2) as f32;
        let y_cord = y as f32 / (Self::CANVAS[1] / 2) as f32;
        let color = [ rgb[0] / 255.0, rgb[1] / 255.0, rgb[2] / 255.0 ];

        let result = multiply_color(color, 0.78);

        self.state.push(Vertex { position: [x_cord, y_cord, 0.0], color: result });
    }
    pub fn get_state(&mut self) -> &[Vertex] {
        return self.state.as_slice();
    }
    pub fn get_canvas_size(&mut self) -> [i32; 2] {
        return Self::CANVAS;
    }

    pub fn add_to_scene(&mut self, sphere: Sphere) {
        self.scene.push(sphere);
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    fn get_canvas_range(&mut self, axis: char) -> RangeInclusive<i32> {
        match axis {
            'x' => { ( -Self::CANVAS[0]/2 ..= Self::CANVAS[0]/2 ) },
            'y' => { ( -Self::CANVAS[1]/2 ..= Self::CANVAS[1]/2 ) },
            _ => { ( -1 ..= 1 ) }
        }
    }

    fn canvas_to_viewport(&mut self, x: i32, y: i32) -> [f32; 3] {
        let x_pos = x as f32 * Self::VIEWPORT[0] / Self::CANVAS[0] as f32;
        let y_pos = y as f32 * Self::VIEWPORT[1] / Self::CANVAS[1] as f32;
        let z_pos = Self::VIEWPORT[2];
        return [x_pos, y_pos, z_pos];
    }

    fn diffuse_reflection(&self, light_intensity: f32, light_vec: [f32; 3], normal_vec: [f32; 3]) -> f32 {
        let light_to_surface = dot_product(normal_vec, light_vec);
        let normalized_vectors = vector_length(normal_vec) * vector_length(light_vec);

        if light_to_surface > 0.0 {
            return light_intensity * light_to_surface / normalized_vectors;
        }
        return 0.0;
    }

    fn reflect_ray(&self, ray: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
        let projection_scale = dot_product(normal, ray);
        let translation = scale_vector(normal, 2.0 * projection_scale);
        return vector_subtraction(translation, ray);
    }

    fn specular_reflection(&self, light_intensity: f32, light_vec: [f32; 3], normal_vec: [f32; 3], bounce_vec: [f32; 3], specular_scale: f32) -> f32 {
        let reflection = self.reflect_ray(light_vec, normal_vec);
        let reflection_offset = dot_product(reflection, bounce_vec);
        let normalized_vectors = vector_length(reflection) * vector_length(bounce_vec);

        if reflection_offset > 0.0 {
            return light_intensity * (reflection_offset / normalized_vectors).powf(specular_scale);
        }
        return 0.0;
    }

    fn add_reflection(&self, color: [f32; 3], reflection: [f32; 3], reflective: f32) -> [f32; 3] {
        let base_color = scale_vector(color, 1.0 - reflective);
        let translation = scale_vector(reflection, reflective);
        return vector_addition(base_color, translation);
    }

    fn compute_lighting(&self, position: [f32; 3], normal: [f32; 3], bounce: [f32; 3], specular: f32) -> f32 {
        let mut light_accumulator = 0.0;
        for light in self.lights.iter() {
            match light.mode {
                LightMode::Ambient => {
                    light_accumulator += light.intensity
                }
                LightMode::Point => {
                    let light_vec = vector_subtraction(light.position, position);
                    let (shadow_sphere, _) = self.closest_intersection(position, light_vec, (0.001 ..= 1.0));
                    match shadow_sphere {
                        None => {
                            light_accumulator += self.diffuse_reflection(light.intensity, light_vec, normal);
                            light_accumulator += self.specular_reflection(light.intensity, light_vec, normal, bounce, specular);
                        }
                        Some(_) => { continue; }
                    }
                }
                LightMode::Directional => {
                    let (shadow_sphere, _) = self.closest_intersection(position, light.direction, (0.001 ..= f32::INFINITY));
                    match shadow_sphere {
                        None => {
                            light_accumulator += self.diffuse_reflection(light.intensity, light.direction, normal);
                            light_accumulator += self.specular_reflection(light.intensity, light.direction, normal, bounce, specular);
                        }
                        Some(_) => { continue; }
                    }
                }
            }
        }
        return light_accumulator;
    }

    fn closest_intersection(&self, origin: [f32; 3], direction: [f32; 3], ray_range: RangeInclusive<f32>) -> (Option<&Sphere>, f32) {
        let mut closest_t = f32::INFINITY;
        let mut closest_sphere: Option<&Sphere> = None;

        for sphere in self.scene.iter() {
            let (t1, t2) = sphere.intersect_ray(origin, direction);

            if ray_range.contains(&t1) && t1 < closest_t {
                closest_t = t1;
                closest_sphere = Some(sphere);
            }

            if ray_range.contains(&t2) && t2 < closest_t {
                closest_t = t2;
                closest_sphere = Some(sphere);
            }
        }

        return (closest_sphere, closest_t)
    }

    fn trace_ray(&self, origin: [f32; 3], direction: [f32; 3], t_min: f32, t_max: f32, depth: u32) -> [f32; 3] {
        let ray_range = (t_min ..= t_max);
        let (closest_sphere, closest_t) = self.closest_intersection(origin, direction, ray_range);

        match closest_sphere {
            Some(sphere) => {
                let position = vector_addition(Self::CAMERA_POSITION, scale_vector(direction, closest_t));
                let normal = sphere.get_normal(position);
                let reversed_direction = reverse_vector(direction);
                let light_accumulated = self.compute_lighting(position, normal, reversed_direction, sphere.specular);
                let sphere_color = color_to_vector(sphere.color);
                let local_color = scale_vector(sphere_color, light_accumulated);

                if sphere.reflective <= 0.0 || depth <= 0 { return local_color; }

                let reflected_ray = self.reflect_ray(reversed_direction, normal);
                let reflected_color = self.trace_ray(position, reflected_ray, 0.001, f32::INFINITY, depth - 1);

                return self.add_reflection(local_color, reflected_color, sphere.reflective);
            },
            None => color_to_vector(Self::BACKGROUND_COLOR),
        }
    }

    pub fn pass(&mut self) {
        for x in self.get_canvas_range('x').clone() {
            for y in self.get_canvas_range('y').clone() {
                let direction = self.canvas_to_viewport(x, y);
                let color = self.trace_ray(Self::CAMERA_POSITION, direction, 1.0, f32::INFINITY, 2);
                self.put_pixel(x, y, color);
            }
        }
    }
}

pub fn init_raytracer() -> Raytracer {
    let mut raytracer = Raytracer::new();

    raytracer.add_light(Light {
        mode: LightMode::Ambient,
        intensity: 0.1,
        position: [0.0, 0.0, 0.0],
        direction: [0.0, 0.0, 0.0],
    });

    raytracer.add_light(Light {
        mode: LightMode::Point,
        intensity: 0.4,
        position: [2.0, 1.0, 0.0],
        direction: [0.0, 0.0, 0.0],
    });

    raytracer.add_light(Light {
        mode: LightMode::Directional,
        intensity: 0.2,
        position: [0.0, 0.0, 0.0],
        direction: [1.0, 4.0, 4.0],
    });

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [0.0, -1.0, 3.0],
        // color: [219, 176, 127],
        color: [255, 0, 0],
        specular: 600.0,
        reflective: 0.1,
    });

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [2.0, 0.0, 4.0],
        // color: [116, 57, 59],
        color: [0, 0, 255],
        specular: 400.0,
        reflective: 0.2,
    });

    raytracer.add_to_scene(Sphere {
        radius: 1.0,
        center: [-2.0, 0.0, 4.0],
        // color: [122, 167, 203],
        color: [0, 255, 0],
        specular: 10.0,
        reflective: 0.3,
    });

    raytracer.add_to_scene(Sphere {
        radius: 5000.0,
        center: [0.0, -5001.0, 3.0],
        // color: [57, 87, 165],
        color: [255, 255, 0],
        specular: 1000.0,
        reflective: 0.4,
    });

    raytracer.pass();

    return raytracer;
}
