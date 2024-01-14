use crate::utilities::{
    dot_product,
    vector_length,
    vector_subtraction,
    divide_vector,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3
        }
    ];
    pub fn describe() -> wgpu::VertexBufferLayout<'static> {
        return wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        };
    }
}

pub enum LightMode {
    Ambient,
    Point,
    Directional,
}

pub struct Light {
    pub mode: LightMode,
    pub intensity: f32,
    pub position: [f32; 3],
    pub direction: [f32; 3],
}

pub struct Sphere {
    pub radius: f32,
    pub center: [f32; 3],
    pub color: [i32; 3],
    pub specular: f32,
    pub reflective: f32,
}

impl Sphere {
    pub fn intersect_ray(&self, camera_origin: [f32; 3], ray_direction: [f32; 3]) -> (f32, f32) {
        let distance_to_center = vector_subtraction(camera_origin, self.center);

        let a = dot_product(ray_direction, ray_direction);
        let b = 2.0 * dot_product(distance_to_center, ray_direction);
        let c = dot_product(distance_to_center, distance_to_center) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return (f32::INFINITY, f32::INFINITY);
        }

        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

        return (t1, t2);
    }

    pub fn get_normal(&self, position: [f32; 3]) -> [f32; 3] {
        let normal_vector = vector_subtraction(position, self.center);
        return divide_vector(normal_vector, vector_length(normal_vector));
    }
}
