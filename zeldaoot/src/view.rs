use log::error;

pub fn func_800abe74(eye_x: f32, eye_y: f32, eye_z: f32) -> i32 {
    let mut error = 0i32;

    if eye_x.powi(2) + eye_y.powi(2) + eye_z.powi(2) > (32767.0f32).powi(2) {
        error = 3;
    } else {
        let abs_eye_x = eye_x.abs();
        let abs_eye_y = eye_y.abs();
        let abs_eye_z = eye_z.abs();

        if ((18900.0f32 < abs_eye_x) || (18900.0f32 < abs_eye_y)) || (18900.0f32 < abs_eye_z) {
            error = 2;
        } else if ((16000.0f32 < abs_eye_x) || (16000.0f32 < abs_eye_y)) || (16000.0f32 < abs_eye_z)
        {
            error = 1;
        }
    }

    if error != 0 {
        error!(
            "eye is too large eye=[{:.3} {:.3} {:.3}] error={}\n",
            eye_x, eye_y, eye_z, error
        );
    }

    return error;
}
