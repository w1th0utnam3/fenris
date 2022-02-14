use nalgebra::{Point3, RealField, Scalar, Unit, Vector3};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Plane3d<T: Scalar> {
    point: Point3<T>,
    normal: Unit<Vector3<T>>,
}

impl<T> Plane3d<T>
where
    T: RealField,
{
    pub fn normal(&self) -> &Unit<Vector3<T>> {
        &self.normal
    }

    pub fn point(&self) -> &Point3<T> {
        &self.point
    }

    pub fn from_point_and_normal(point: Point3<T>, normal: Unit<Vector3<T>>) -> Self {
        Self { point, normal }
    }
}