#[derive(Clone, Copy)]
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

pub fn vector_multiplication(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [ v1[0] * v2[0], v1[1] * v2[1], v1[2] * v2[2] ]
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

pub fn mat4_default() -> [[f32; 4]; 4] {
    return [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
}

pub fn to_scale_mat4(scale: [f32; 3]) -> [[f32; 4]; 4] {
    return [
        [scale[0], 0.0,      0.0,      0.0],
        [0.0,      scale[1], 0.0,      0.0],
        [0.0,      0.0,      scale[2], 0.0],
        [0.0,      0.0,      0.0,      1.0],
    ];
}

pub fn to_translation_mat4(translation: [f32; 3]) -> [[f32; 4]; 4] {
    return [
        [1.0, 0.0, 0.0, translation[0]],
        [0.0, 1.0, 0.0, translation[1]],
        [0.0, 0.0, 1.0, translation[2]],
        [0.0, 0.0, 0.0,            1.0],
    ];
}
pub fn to_inverse_translation_mat4(translation: [f32; 3]) -> [[f32; 4]; 4] {
    return [
        [1.0, 0.0, 0.0, -translation[0]],
        [0.0, 1.0, 0.0, -translation[1]],
        [0.0, 0.0, 1.0, -translation[2]],
        [0.0, 0.0, 0.0,             1.0],
    ];
}

fn to_x_rotation_mat4(radian: f32) -> [[f32; 4]; 4] {
    return [
        [1.0, 0.0,          0.0,           0.0],
        [0.0, radian.cos(), -radian.sin(), 0.0],
        [0.0, radian.sin(),  radian.cos(), 0.0],
        [0.0, 0.0,          0.0,           1.0],
    ];
}
fn to_x_inverse_rotation_mat4(radian: f32) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0,           0.0,          0.0],
        [0.0,  radian.cos(), radian.sin(), 0.0],
        [0.0, -radian.sin(), radian.cos(), 0.0],
        [0.0, 0.0,           0.0,          1.0],
    ]
}
fn to_y_rotation_mat4(radian: f32) -> [[f32; 4]; 4] {
    return [
        [ radian.cos(), 0.0, radian.sin(), 0.0],
        [0.0,           1.0, 0.0,          0.0],
        [-radian.sin(), 0.0, radian.cos(), 0.0],
        [0.0,           0.0, 0.0,          1.0],
    ];
}

fn to_y_inverse_rotation_mat4(radian: f32) -> [[f32; 4]; 4] {
    [
        [radian.cos(), 0.0, -radian.sin(), 0.0],
        [0.0,          1.0, 0.0,           0.0],
        [radian.sin(), 0.0,  radian.cos(), 0.0],
        [0.0,          0.0, 0.0,           1.0],
    ]
}
fn to_z_rotation_mat4(radian: f32) -> [[f32; 4]; 4] {
    return [
        [radian.cos(), -radian.sin(), 0.0, 0.0],
        [radian.sin(),  radian.cos(), 0.0, 0.0],
        [0.0,          0.0,           1.0, 0.0],
        [0.0,          0.0,           0.0, 1.0],
    ];
}
fn to_z_inverse_rotation_mat4(radian: f32) -> [[f32; 4]; 4] {
    [
        [ radian.cos(), radian.sin(), 0.0, 0.0],
        [-radian.sin(), radian.cos(), 0.0, 0.0],
        [0.0,           0.0,          1.0, 0.0],
        [0.0,           0.0,          0.0, 1.0],
    ]
}

pub fn to_rotation_mat4(axis: Axis, radian: f32) -> [[f32; 4]; 4] {
    return match axis {
        Axis::X => { to_x_rotation_mat4(radian) }
        Axis::Y => { to_y_rotation_mat4(radian) }
        Axis::Z => { to_z_rotation_mat4(radian) }
    }
}

pub fn to_inverse_rotation_mat4(axis: Axis, radian: f32) -> [[f32; 4]; 4] {
    return match axis {
        Axis::X => { to_x_inverse_rotation_mat4(radian) }
        Axis::Y => { to_y_inverse_rotation_mat4(radian) }
        Axis::Z => { to_z_inverse_rotation_mat4(radian) }
    }
}

pub fn multiply_mat4_vec(mat4: [[f32; 4]; 4], vec: [f32; 4]) -> [f32; 4] {
    return [
        mat4[0][0] * vec[0] + mat4[0][1] * vec[1] + mat4[0][2] * vec[2] + mat4[0][3] * vec[3],
        mat4[1][0] * vec[0] + mat4[1][1] * vec[1] + mat4[1][2] * vec[2] + mat4[1][3] * vec[3],
        mat4[2][0] * vec[0] + mat4[2][1] * vec[1] + mat4[2][2] * vec[2] + mat4[2][3] * vec[3],
        mat4[3][0] * vec[0] + mat4[3][1] * vec[1] + mat4[3][2] * vec[2] + mat4[3][3] * vec[3],
    ];
}

pub fn multiply_mat4_mat4(mat_a: [[f32; 4]; 4], mat_b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0f32; 4]; 4];

    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = 0.0;
            for k in 0..4 {
                result[i][j] += mat_a[i][k] * mat_b[k][j];
            }
        }
    }

    return result;
}
