use glam::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Transform(Mat4);

impl From<Mat4> for Transform {
    fn from(mat: Mat4) -> Self {
        Self(mat)
    }
}

impl Transform {
    pub fn mat4(&self) -> &Mat4 {
        &self.0
    }
    // pub fn forward(&self) -> Vec3 {
    //     let (_, rotation, _) = self.0.to_scale_rotation_translation();
    //     rotation * Vec3::new(0.0, 0.0, 0.0)
    // }
}

fn calculate_forward_direction(yaw: f32, pitch: f32) -> Vec3 {
    let result: Vec3 = [
        (-yaw).sin() * pitch.cos(),
        pitch.sin(),
        (-yaw).cos() * pitch.cos(),
    ]
    .into();
    result.normalize()
}