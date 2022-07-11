use rbx_types::Matrix3;

#[derive(Debug)]
pub struct AxisAngle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl AxisAngle {
    pub fn new(x: f32, y: f32, z: f32) -> AxisAngle {
        AxisAngle { x, y, z }
    }
}

pub fn matrix3_to_axis_angle(m: Matrix3) -> AxisAngle {
    // split matrix
    let r00 = m.x.x;
    let r01 = m.y.x;
    let r02 = m.z.x;
    let r10 = m.x.y;
    let r11 = m.y.y;
    let r12 = m.z.y;
    let r20 = m.x.z;
    let r21 = m.y.z;
    let r22 = m.z.z;

    // convert cf to quaternion
    let tr = 1f32 + r00 + r11 + r22;
    let ti = 1f32 + r00 - r11 - r22;
    let tj = 1f32 - r00 + r11 - r22;
    let tk = 1f32 - r00 - r11 + r22;

    let w: f32;
    let x: f32;
    let y: f32;
    let z: f32;

    if ti < tr && tj < tr && tk < tr {
        let s = 2f32 * tr.sqrt();
        w = s / 4f32;
        x = (r21 - r12) / s;
        y = (r02 - r20) / s;
        z = (r10 - r01) / s;
    } else if tj < ti && tk < ti {
        let s = 2f32 * ti.sqrt();
        w = (r21 - r12) / s;
        x = s / 4f32;
        y = (r10 + r01) / s;
        z = (r02 + r20) / s;
    } else if tk < tj {
        let s = 2f32 * tj.sqrt();
        w = (r02 - r20) / s;
        x = (r10 + r01) / s;
        y = s / 4f32;
        z = (r21 + r12) / s;
    } else {
        let s = 2f32 * tk.sqrt();
        w = (r10 - r01) / s;
        x = (r02 + r20) / s;
        y = (r21 + r12) / s;
        z = s / 4f32;
    }
    // bring this quaternion to aa
    let m = (x * x + y * y + z * z).sqrt();
    let a: f32 = if w < 0f32 {
        -2f32 * m.atan2(-w) / m
    } else {
        2f32 * m.atan2(w) / m
    };

    AxisAngle::new(a * x, a * y, a * z)
}
