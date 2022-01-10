use glam::{Mat4, Quat, Vec3, Vec4};

pub const GLOBAL_FORWARD: [f32; 3] = [0.0, 0.0, 1.0];
pub const GLOBAL_UP: [f32; 3] = [0.0, 1.0, 0.0];
pub const GLOBAL_LEFT: [f32; 3] = [1.0, 0.0, 0.0];

#[derive(Debug, Clone)]
pub struct Transform(Mat4);

impl From<Mat4> for Transform {
    fn from(mat: Mat4) -> Self {
        Self(mat)
    }
}

impl Transform {
    pub fn mat4(&self) -> Mat4 {
        self.0
    }

    pub fn scale(&self) -> Vec3 {
        let (scale, _, _) = self.0.to_scale_rotation_translation();
        scale
    }

    pub fn rotation(&self) -> Quat {
        let (_, rotation, _) = self.0.to_scale_rotation_translation();
        rotation
    }

    pub fn position(&self) -> Vec3 {
        let (_, _, position) = self.0.to_scale_rotation_translation();
        position
    }

    pub fn forward(&self) -> Vec3 {
        let fwd = self.0 * Vec4::new(GLOBAL_FORWARD[0], GLOBAL_FORWARD[1], GLOBAL_FORWARD[2], 0.0);
        Vec3::new(fwd.x, fwd.y, fwd.z).normalize()
    }

    pub fn left(&self) -> Vec3 {
        let fwd = self.0 * Vec4::new(GLOBAL_LEFT[0], GLOBAL_LEFT[1], GLOBAL_LEFT[2], 0.0);
        Vec3::new(fwd.x, fwd.y, fwd.z).normalize()
    }
}

// fn calculate_forward_direction(yaw: f32, pitch: f32) -> Vec3 {
//     let result: Vec3 = [
//         (-yaw).sin() * pitch.cos(),
//         pitch.sin(),
//         (-yaw).cos() * pitch.cos(),
//     ]
//     .into();
//     result.normalize()
// }
