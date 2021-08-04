use tb_core::*;

#[derive(Serialize, Deserialize)]
pub enum Bounds {
    Sphere(SphereBounds),
    Box(BoxBounds),
    Rect(RectBounds),
}

#[derive(Serialize, Deserialize)]
pub struct SphereBounds {
    center: Vec3,
    radius: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BoxBounds {
    center: Vec3,
    extends: Vec3,
}

#[derive(Serialize, Deserialize)]
pub struct RectBounds {
    center: Vec3,
    extends: Vec3,
}
