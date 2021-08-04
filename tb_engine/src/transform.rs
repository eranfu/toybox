use tb_core::*;
use tb_ecs::*;

#[component]
pub struct Rotation {
    pub euler: EulerAngle,
}

impl Rotation {
    pub fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self::from(EulerAngle::new(roll, pitch, yaw))
    }
}

impl From<EulerAngle> for Rotation {
    fn from(euler: EulerAngle) -> Self {
        Self { euler }
    }
}

#[component]
pub struct Location {
    pub location: Vec3,
}

impl Location {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from(Vec3::new(x, y, z))
    }
}

impl From<Vec3> for Location {
    fn from(vec: Vec3) -> Self {
        Self { location: vec }
    }
}
