use tb_core::*;

#[derive(Serialize, Deserialize)]
pub enum Bounds {
    Sphere(SphereBounds),
    Box(BoxBounds),
    Rect(RectBounds),
}

#[derive(Serialize, Deserialize)]
pub struct SphereBounds {
    center: Point3,
    radius: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BoxBounds {
    center: Point3,
    extends: Vector3,
}

#[derive(Serialize, Deserialize)]
pub struct RectBounds {
    center: Point3,
    extends: Vector3,
}
