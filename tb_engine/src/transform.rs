use tb_core::*;
use tb_ecs::*;

#[component]
pub struct Rotation {
    pub euler: Euler,
}

impl Rotation {
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self::from(Euler::new(pitch, yaw, roll))
    }
}

impl From<Euler> for Rotation {
    fn from(euler: Euler) -> Self {
        Self { euler }
    }
}

#[component]
pub struct Location {
    pub location: Point3,
}

impl Location {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from(Point3::new(x, y, z))
    }
}

impl From<Point3> for Location {
    fn from(location: Point3) -> Self {
        Self { location }
    }
}
