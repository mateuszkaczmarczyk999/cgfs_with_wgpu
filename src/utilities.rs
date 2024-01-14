pub fn dot_product(v1: [f32; 3], v2: [f32; 3]) -> f32 {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

pub fn vector_length(vector: [f32; 3]) -> f32 {
    (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt()
}

pub fn vector_subtraction(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [ v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2] ]
}

pub fn vector_addition(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [ v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2] ]
}

pub fn scale_vector(v1: [f32; 3], scalar: f32) -> [f32; 3] {
    [ v1[0] * scalar, v1[1] * scalar, v1[2] * scalar ]
}

pub fn divide_vector(v1: [f32; 3], scalar: f32) -> [f32; 3] {
    [ v1[0] / scalar, v1[1] / scalar, v1[2] / scalar ]
}

pub fn color_to_vector(color: [i32; 3]) -> [f32; 3] {
    [ color[0] as f32, color[1] as f32, color[2] as f32 ]
}

pub fn reverse_vector(v1: [f32; 3]) -> [f32; 3]  {
    [ v1[0] * -1.0, v1[1] * -1.0, v1[2] * -1.0 ]
}

pub fn multiply_color(color: [f32; 3], factor: f32) -> [f32; 3] {
    [
        (color[0] * factor).clamp(0.0, 255.0),
        (color[1] * factor).clamp(0.0, 255.0),
        (color[2] * factor).clamp(0.0, 255.0),
    ]
}