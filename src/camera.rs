use glam::{Mat4, Vec2, Vec3};

use crate::{transform::{GLOBAL_UP, Transform}};


#[derive(Debug, Clone)]
pub struct Camera {
    position: Vec3,

    yaw: f32,   // radians
    pitch: f32, // radians

    speed: f32,
}

impl Camera {
    pub fn new(position: impl Into<Vec3>, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw,
            pitch,
            speed: 100.0,
        }
    }

    pub fn transform(&self) -> Transform {
        let rot = Mat4::from_rotation_ypr(self.yaw, self.pitch, 0.0);
        let tr = Mat4::from_translation(self.position);
        Transform::from(tr * rot)
    }

    // movement.x is sideways movement, movement.y is forward/back
    pub fn drive(&mut self, movement: Vec2) {
        let xform = self.transform();
        self.position += xform.forward() * movement.x() * self.speed;
        self.position += xform.left() * movement.y() * self.speed;
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.position,
            self.position + self.transform().forward(),
            GLOBAL_UP.into(),
        )
    }

    pub fn rotate(&mut self, rot: Vec2) {
        use std::f32::consts::PI;

        self.yaw -= rot.x();
        if self.yaw >= 2.0 * PI {
            self.yaw -= 2.0 * PI;
        }
        if self.yaw <= 0.0 {
            self.yaw += 2.0 * PI;
        }

        let freedom_y = 0.8;
        self.pitch += rot.y();
        self.pitch = self
            .pitch
            .max(-PI / 2.0 * freedom_y)
            .min(PI / 2.0 * freedom_y);
    }
}