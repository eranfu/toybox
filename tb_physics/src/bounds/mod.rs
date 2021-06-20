use tb_core::math::*;
use tb_core::serde::*;

#[derive(Serialize, Deserialize)]
pub enum Bounds {
    Sphere(SphereBounds),
    Box(BoxBounds),
    Rect(RectBounds),
}

#[derive(Serialize, Deserialize)]
pub struct SphereBounds {
    center: Point3<f32>,
    radius: f32,
}

#[derive(Serialize, Deserialize)]
pub struct BoxBounds {
    center: Point3<f32>,
    extends: Vector3<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct RectBounds {
    center: Point2<f32>,
    extends: Vector2<f32>,
}
