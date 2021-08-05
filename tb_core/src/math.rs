use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::*;

use cgmath::num_traits::Num;
pub use cgmath::prelude::*;
use cgmath::{AbsDiffEq, RelativeEq, Ulps, UlpsEq};

use crate::serde::*;

pub type NumType = f32;

pub type Vector1 = cgmath::Vector1<NumType>;
pub type Vector2 = cgmath::Vector2<NumType>;
pub type Vector3 = cgmath::Vector3<NumType>;
pub type Vector4 = cgmath::Vector4<NumType>;

pub type Point1 = cgmath::Point1<NumType>;
pub type Point2 = cgmath::Point2<NumType>;
pub type Point3 = cgmath::Point3<NumType>;

pub type Quaternion = cgmath::Quaternion<NumType>;

pub type Matrix2 = cgmath::Matrix2<NumType>;
pub type Matrix3 = cgmath::Matrix3<NumType>;
pub type Matrix4 = cgmath::Matrix4<NumType>;

pub type Deg = cgmath::Deg<NumType>;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Euler(cgmath::Euler<Deg>);

impl Euler {
    pub fn new(pitch: NumType, yaw: NumType, roll: NumType) -> Self {
        Self(cgmath::Euler::<Deg>::new(
            cgmath::Deg::<NumType>(pitch),
            cgmath::Deg::<NumType>(yaw),
            cgmath::Deg::<NumType>(roll),
        ))
    }
}

impl Deref for Euler {
    type Target = cgmath::Euler<Deg>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Euler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Sum for Euler {
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        let mut sum = match iter.next() {
            None => {
                return Euler::zero();
            }
            Some(first) => first,
        };
        for euler in iter {
            sum = sum + euler;
        }
        sum
    }
}

impl Div<NumType> for Euler {
    type Output = Self;

    fn div(mut self, rhs: NumType) -> Self::Output {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self
    }
}

impl Div for Euler {
    type Output = NumType;

    fn div(mut self, _rhs: Self) -> Self::Output {
        unreachable!()
    }
}

impl Mul<NumType> for Euler {
    type Output = Self;

    fn mul(mut self, rhs: NumType) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self
    }
}

impl Mul for Euler {
    type Output = NumType;

    fn mul(self, _rhs: Self) -> Self::Output {
        unreachable!()
    }
}

impl Rem for Euler {
    type Output = Self;

    fn rem(mut self, rhs: Self) -> Self::Output {
        self.x %= rhs.x;
        self.y %= rhs.y;
        self.z %= rhs.z;
        self
    }
}

impl Sub for Euler {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self
    }
}

impl Neg for Euler {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl Add for Euler {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

impl AddAssign for Euler {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.z;
        self.z += rhs.z;
    }
}

impl cgmath::Zero for Euler {
    fn zero() -> Self {
        Self::new(0f32, 0f32, 0f32)
    }

    fn is_zero(&self) -> bool {
        self.x == Deg::zero() && self.y == Deg::zero() && self.z == Deg::zero()
    }
}

impl AbsDiffEq for Euler {
    type Epsilon = <Deg as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        Deg::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.x.abs_diff_eq(&other.x, epsilon)
            && self.y.abs_diff_eq(&other.y, epsilon)
            && self.z.abs_diff_eq(&other.z, epsilon)
    }
}

impl UlpsEq for Euler {
    fn default_max_ulps() -> u32 {
        Deg::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.x.ulps_eq(&other.x, epsilon, max_ulps)
            && self.y.ulps_eq(&other.y, epsilon, max_ulps)
            && self.z.ulps_eq(&other.z, epsilon, max_ulps)
    }
}

impl RelativeEq for Euler {
    fn default_max_relative() -> Self::Epsilon {
        Deg::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.x.relative_eq(&other.x, epsilon, max_relative)
            && self.y.relative_eq(&other.y, epsilon, max_relative)
            && self.z.relative_eq(&other.z, epsilon, max_relative)
    }
}

impl PartialOrd for Euler {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unreachable!()
    }
}

impl Angle for Euler {
    type Unitless = NumType;

    fn full_turn() -> Self {
        todo!()
    }

    fn sin(self) -> Self::Unitless {
        todo!()
    }

    fn cos(self) -> Self::Unitless {
        todo!()
    }

    fn tan(self) -> Self::Unitless {
        todo!()
    }

    fn sin_cos(self) -> (Self::Unitless, Self::Unitless) {
        todo!()
    }

    fn asin(ratio: Self::Unitless) -> Self {
        todo!()
    }

    fn acos(ratio: Self::Unitless) -> Self {
        todo!()
    }

    fn atan(ratio: Self::Unitless) -> Self {
        todo!()
    }

    fn atan2(a: Self::Unitless, b: Self::Unitless) -> Self {
        todo!()
    }
}
