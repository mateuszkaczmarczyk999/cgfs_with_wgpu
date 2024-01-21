pub enum Axis { X, Y, Z }

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

pub fn interpolate(start_idx: i32, start_val: f32, end_idx: i32, end_val: f32) -> Vec<i32> {
    if start_idx == end_idx { return vec![start_val as i32]; }

    let mut values = Vec::new();
    let step = (end_val - start_val) / (end_idx - start_idx) as f32;
    let mut value = start_val;

    for _ in start_idx ..= end_idx {
        values.push(value.round() as i32);
        value += step;
    }

    return values;
}

pub fn rotate_vector(v1: [f32; 3], rotation: &(Axis, f32)) -> [f32; 3] {
    let (axis, angle) = rotation;
    let radian = angle.to_radians();

    return match axis {
        Axis::X => {
            let x = v1[0];
            let y = v1[1] * radian.cos() - v1[2] * radian.sin();
            let z = v1[1] * radian.sin() + v1[2] * radian.cos();
            [x, y, z]
        },
        Axis::Y => {
            let x = v1[2] * radian.sin() + v1[0] * radian.cos();
            let y = v1[1];
            let z = v1[2] * radian.cos() - v1[0] * radian.sin();
            [x, y, z]
        },
        Axis::Z => {
            let x = v1[0] * radian.cos() - v1[1] * radian.sin();
            let y = v1[0] * radian.sin() + v1[1] * radian.cos();
            let z = v1[2];
            [x, y, z]
        },
    }
}
