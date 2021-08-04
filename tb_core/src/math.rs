use std::ops::{AddAssign, Mul};

pub use nalgebra_glm::*;

use crate::serde::*;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct EulerAngle(Vec3);

impl EulerAngle {
    pub fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self::from(Vec3::new(roll, pitch, yaw))
    }
}

impl From<Vec3> for EulerAngle {
    fn from(vec: Vec3) -> Self {
        Self(vec)
    }
}

impl Mul<f32> for EulerAngle {
    type Output = EulerAngle;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.0 *= rhs;
        self
    }
}

impl AddAssign for EulerAngle {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
