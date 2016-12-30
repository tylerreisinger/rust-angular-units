//! Library for representing and manipulating angular quantities.
//! Provides type-safe wrapper types for each unit as well as helper
//! traits for abstracting over the concrete types.
//! Conversions between types is easy and safe, allowing highly flexible manipulation.
//!
//! ## Details
//!
//! ### Arithmetic
//!
//! Each angle type defines basic arithmetic operators. Multiplication and
//! division are between an angle and a scalar. Addition and subtraction
//! are between two angles and the two angles do not have to be represented using
//! the same units for example, the following is valid:
//!
//! ```
//! # use angular_units::*;
//! let angle = Turns(0.25) + Deg(30.0) - ArcMinutes(15.0);
//! ```
//!
//! When combining units like this, the left-hand side type will be the result.
//!
//! ### Normalization
//!
//! For performance, most operations do not normalize the results or inputs automatically.
//! This is mathematically sound, but it is often more convenient to have a single
//! value to represent each angle. Thus, for methods that expect an angle within
//! the standard domain, `normalize()` should be used to create an equivalent
//! angle that is less than one period.


extern crate num;
#[macro_use]
#[cfg(feature = "approx")]
extern crate approx;

use std::ops::*;
use std::f64::consts;
use std::fmt;
use std::convert::From;
use num::{Float, NumCast};

/// An angular quantity measured in degrees.
///
/// Degrees are uniquely defined from 0..360.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct Deg<T>(pub T);
/// An angular quantity measured in degrees.
///
/// Radians are uniquely defined from 0..2π.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct Rad<T>(pub T);
/// An angular quantity measured in "turns", or full rotations.
///
/// Turns are uniquely defined from 0..1.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct Turns<T>(pub T);
/// An angular quantity measured in arc minutes, which are
/// 1/60th of a degree.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct ArcMinutes<T>(pub T);
/// An angular quantity measured in arc seconds, which are
/// 1/60th of an arc minute.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct ArcSeconds<T>(pub T);

/// Construct `Self` from an angle.
///
/// Analogous to the traits in the standard library,
/// FromAngle and IntoAngle provide a way to convert between angle
/// types and to mix various angle types in a single operation.
pub trait FromAngle<T>
    where T: Angle
{
    /// Construct `Self` by converting a `T`.
    fn from_angle(from: T) -> Self;
}

/// Construct an angle by converting from another type.
///
/// IntoAngle is provided automatically based on FromAngle.
pub trait IntoAngle<To>
    where To: Angle<Scalar = Self::OutputScalar>
{
    type OutputScalar: Float;
    /// Construct an angle from `self`.
    fn into_angle(self) -> To;
}

/// Base functionality for all angle types.
pub trait Angle: Clone + FromAngle<Self> {
    /// Internal type storing the angle value.
    type Scalar: Float;

    /// The length of a full rotation.
    fn period() -> Self::Scalar;
    /// Return the scalar (unitless) value.
    ///
    /// Equivalent to `self.0` or to doing `let Deg(val) = self`
    fn scalar(&self) -> Self::Scalar;
    /// Normalize the angle, wrapping it back into the standard domain.
    ///
    /// After normalization, an angle will be in the range `[0, self.period())`.
    ///
    /// For performance reasons, normalization does not happen automatically
    /// during most operations. Thus, when passing an angle to a method that
    /// expects it to be within the standard domain, first normalize the angle.
    fn normalize(&self) -> Self;
    /// Whether the angle is in the standard domain.
    fn is_normalized(&self) -> bool;

    /// Compute the sine of an angle.
    fn sin(self) -> Self::Scalar;
    /// Compute the cosine of an angle.
    fn cos(self) -> Self::Scalar;
    /// Compute the tangent of an angle.
    fn tan(self) -> Self::Scalar;
    /// Simultaneously compute sine and cosine.
    fn sin_cos(self) -> (Self::Scalar, Self::Scalar);

    /// Compute the arcsine of a value, returning an angle.
    fn asin(value: Self::Scalar) -> Self;
    /// Compute the arccosine of a value, returning an angle.
    fn acos(value: Self::Scalar) -> Self;
    /// Compute the arctangent of a value, returning an angle.
    fn atan(value: Self::Scalar) -> Self;
    /// Compute the arctangent of a value, using information from
    /// the numerator and denominator in order to increase the domain.
    fn atan2(x: Self::Scalar, y: Self::Scalar) -> Self;

    /// Return one full rotation in some unit.
    ///
    /// Equivalent to `Self(Self::period())`.
    fn full_turn() -> Self;
    /// Return one half of a full rotation in some unit.
    fn half_turn() -> Self;
    /// Return one quarter of a full rotation in some unit.
    fn quarter_turn() -> Self;

    /// Return the inverse of an angle.
    ///
    /// The inverse is equivalent to adding half a rotation
    /// or inverting the unit vector pointing from the origin along the
    /// angle.
    fn invert(self) -> Self;
}

/// A trait for linear interpolation between angles.
pub trait Interpolate: Angle {
    /// Perform a linear interpolation between two angles.
    ///
    /// This method will always follow the shortest past between
    /// the two angles. This means it will go backward if the 
    /// angles are more than a half turn apart. To force the interpolation
    /// to go forward, use `interpolate_forward`. 
    /// The output is not normalized, and may exceed a 
    /// full turn if it interpolates backward,
    /// even if both inputs are normalized.
    /// The angles may be represented in different units.
    fn interpolate<U>(&self, right: &U, pos: Self::Scalar) -> Self
        where U: Clone + IntoAngle<Self, OutputScalar = Self::Scalar>;

    /// Perform a linear interpolation between two angles, 
    /// going forward from `self` to `right`.
    ///
    /// Unlike `interpolate` this will always go forward from `self` to `right`,
    /// even if going backward would take a shorter path. The output is not
    /// normalized, but should remain normalized if both `self` and `right` are.
    /// The angles may be represented in different units.
    fn interpolate_forward<U>(&self, right: &U, pos: Self::Scalar) -> Self
        where U: Clone + IntoAngle<Self, OutputScalar = Self::Scalar>;
}

macro_rules! impl_angle {
    ($Struct: ident, $period: expr) => {
        impl<T: Float> $Struct<T> {
            /// Construct a new angle. 
            /// 
            /// Equivalent to constructing the tuple struct directly, eg. `Deg(value)`.
            pub fn new(value: T) -> $Struct<T> {
                $Struct(value)
            }
        }

        impl<T: Float> Angle for $Struct<T>
        {
            type Scalar = T;
            fn period() -> T {
                cast($period).unwrap()
            }

            fn scalar(&self) -> T {
                self.0
            }
            fn is_normalized(&self) -> bool {
                self.0 >= T::zero() && self.0 < Self::period()
            }

            fn normalize(&self) -> $Struct<T> {
                if !self.is_normalized() {
                    let shifted = self.0 % Self::period();
                    if shifted < T::zero() {
                        $Struct(shifted + Self::period())
                    } else {
                        $Struct(shifted)
                    }
                } else {
                    self.clone()
                }
            }

            fn sin(self) -> T {
                Rad::from_angle(self).0.sin()
            }
            fn cos(self) -> T {
                Rad::from_angle(self).0.cos()
            }
            fn tan(self) -> T {
                Rad::from_angle(self).0.tan()
            }
            fn sin_cos(self) -> (T, T) {
                Rad::from_angle(self).0.sin_cos()
            }
            fn asin(value: T) -> $Struct<T> {
                $Struct::from_angle(Rad(value.asin()))
            }
            fn acos(value: T) -> $Struct<T> {
                $Struct::from_angle(Rad(value.acos()))
            }
            fn atan(value: T) -> $Struct<T> {
                $Struct::from_angle(Rad(value.atan()))
            }
            fn atan2(y: T, x: T) -> $Struct<T> {
                $Struct::from_angle(Rad(y.atan2(x)))
            }

            fn full_turn() -> Self {
                $Struct(Self::period())
            }
            fn half_turn() -> Self {
                $Struct(cast::<_, Self::Scalar>(0.5).unwrap() * Self::period())
            }
            fn quarter_turn() -> Self {
                $Struct(cast::<_, Self::Scalar>(0.25).unwrap() * Self::period())
            }
            fn invert(self) -> Self {
                self + Self::half_turn()
            }
        }

        impl<T: Float> Interpolate for $Struct<T> {
            fn interpolate<U>(&self, right: &U, pos: Self::Scalar) -> Self
                where U: Clone + IntoAngle<Self, OutputScalar=Self::Scalar>
            {
                let end = right.clone().into_angle();
                let forward_distance = (end.0 - self.0).abs();
                let inv_pos = cast::<_, Self::Scalar>(1.0).unwrap() - pos;
                
                if forward_distance > Self::half_turn().0 {
                    if *self > end {
                        $Struct(self.0 * inv_pos + (end.0 + Self::period()) * pos)
                    } else {
                        $Struct((self.0 + Self::period()) * inv_pos + end.0 * pos)
                    }
                } else {
                    $Struct(self.0 * inv_pos + end.0 * pos)
                }
            }

            fn interpolate_forward<U>(&self, right: &U, pos: Self::Scalar) -> Self
                where U: Clone + IntoAngle<Self, OutputScalar = Self::Scalar>
            {
                let inv_pos = cast::<_, Self::Scalar>(1.0).unwrap() - pos;
                $Struct(self.0 * inv_pos + right.clone().into_angle().0 * pos)
            }
        }

        #[cfg(feature = "approx")]
        impl<T: Float + approx::ApproxEq> approx::ApproxEq for $Struct<T> 
            where T::Epsilon: Clone,
        {
            type Epsilon = T::Epsilon;

            fn default_epsilon() -> Self::Epsilon {
                T::default_epsilon()
            }
            fn default_max_relative() -> Self::Epsilon {
                T::default_max_relative()
            }

            fn default_max_ulps() -> u32 {
                T::default_max_ulps()
            }
            fn relative_eq(&self, other: &Self, epsilon: Self::Epsilon, 
                           max_relative: Self::Epsilon) -> bool {
                self.0.relative_eq(&other.0, epsilon, max_relative)
            }
            fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
                self.0.ulps_eq(&other.0, epsilon, max_ulps)
            }
        }

        impl<T: Rem<T, Output=T>> Rem for $Struct<T> {
            type Output=$Struct<T>;
            fn rem(self, rhs: $Struct<T>) -> $Struct<T> {
                $Struct(self.0 % rhs.0)
            }
        }

        impl<T: RemAssign> RemAssign for $Struct<T> {
            fn rem_assign(&mut self, rhs: $Struct<T>) {
                self.0 %= rhs.0;
            }
        }

        impl<U, T> Add<U> for $Struct<T> 
            where T: Float + Add<T, Output=T>,
                  U: IntoAngle<$Struct<T>, OutputScalar=T>
        {
            type Output=$Struct<T>;
            fn add(self, rhs: U) -> $Struct<T> {
                $Struct(self.0 + rhs.into_angle().0)
            }
        }

        impl<U, T> AddAssign<U> for $Struct<T> 
            where T: Float + AddAssign<T>,
                  U: IntoAngle<$Struct<T>, OutputScalar=T>
        {
            fn add_assign(&mut self, rhs: U) {
                self.0 += rhs.into_angle().0;
            }
        }

        impl<U, T> Sub<U> for $Struct<T> 
            where T: Float + Sub<T, Output=T>,
                  U: IntoAngle<$Struct<T>, OutputScalar=T>
        {
            type Output=$Struct<T>;
            fn sub(self, rhs: U) -> $Struct<T> {
                $Struct(self.0 - rhs.into_angle().0)
            }
        }

        impl<U, T> SubAssign<U> for $Struct<T> 
            where T: Float + SubAssign<T>,
                  U: IntoAngle<$Struct<T>, OutputScalar=T>
        {
            fn sub_assign(&mut self, rhs: U) {
                self.0 -= rhs.into_angle().0;
            }
        }
        
        impl<T: Mul<T, Output=T>> Mul<T> for $Struct<T> {
            type Output=$Struct<T>;
            fn mul(self, rhs: T) -> $Struct<T> {
                $Struct(self.0 * rhs)
            }
        }

        impl<T: MulAssign<T>> MulAssign<T> for $Struct<T> {
            fn mul_assign(&mut self, rhs: T) {
                self.0 *= rhs;
            }
        }

        impl<T: Div<T, Output=T>> Div<T> for $Struct<T> {
            type Output=$Struct<T>;
            fn div(self, rhs: T) -> $Struct<T> {
                $Struct(self.0 / rhs)
            }
        }

        impl<T: DivAssign<T>> DivAssign<T> for $Struct<T> {
            fn div_assign(&mut self, rhs: T) {
                self.0 /= rhs;
            }
        }

        impl<T: Neg<Output=T>> Neg for $Struct<T> {
            type Output=$Struct<T>;
            fn neg(self) -> $Struct<T> {
                $Struct(-self.0)
            }
        }

        impl<T: Float> num::Zero for $Struct<T> {
            fn zero() -> $Struct<T> {
                $Struct(T::zero())
            }
            fn is_zero(&self) -> bool {
                self.0 == T::zero()
            }
        }

        impl<T: num::Zero> Default for $Struct<T> {
            fn default() -> $Struct<T> {
                $Struct(T::zero())
            }
        }

        impl<T, U> FromAngle<U> for $Struct<T>
            where U: Angle<Scalar=T>,
                  T: Float,
        {
            fn from_angle(from: U) -> $Struct<T> {
                $Struct(from.scalar() * $Struct::period() / U::period())
            }
        }
    }
}

macro_rules! impl_from_for_angle {
    ($from: ty, $to: ty) => {
        impl<T: Float> From<$from> for $to {
            fn from(from: $from) -> $to {Self::from_angle(from)}
        }
    }
}

impl_angle!(Deg, 360.0);
impl_angle!(Rad, consts::PI * 2.0);
impl_angle!(Turns, 1.0);
impl_angle!(ArcMinutes, 360.0 * 60.0);
impl_angle!(ArcSeconds, 360.0 * 3600.0);

impl_from_for_angle!(Deg<T>, Rad<T>);
impl_from_for_angle!(Deg<T>, Turns<T>);
impl_from_for_angle!(Rad<T>, Deg<T>);
impl_from_for_angle!(Rad<T>, Turns<T>);
impl_from_for_angle!(Turns<T>, Deg<T>);
impl_from_for_angle!(Turns<T>, Rad<T>);

impl_from_for_angle!(ArcMinutes<T>, Deg<T>);
impl_from_for_angle!(ArcSeconds<T>, Deg<T>);
impl_from_for_angle!(ArcSeconds<T>, ArcMinutes<T>);

impl<T: Float> Deg<T> {
    /// Construct a `Deg` instance from base degrees, minutes and seconds.
    ///
    /// The opposite of decompose. Equivalent to adding the components together:
    ///
    /// ```
    /// #   use angular_units::*;
    ///     let angle = Deg(50.0) + ArcMinutes(30.0) + ArcSeconds(10.0);
    ///     assert_eq!(angle, Deg::from_components(Deg(50.0),
    ///         ArcMinutes(30.0), ArcSeconds(10.0)));
    /// ```
    pub fn from_components(degs: Deg<T>, mins: ArcMinutes<T>, secs: ArcSeconds<T>) -> Self {
        degs + mins + secs
    }

    /// Split an angle in degrees into base degrees, minutes and seconds.
    ///
    /// If the decomposition would not be perfect, seconds will be
    /// a fractional value.
    pub fn decompose(self) -> (Deg<T>, ArcMinutes<T>, ArcSeconds<T>) {
        let sixty: T = cast(60.0).unwrap();
        let degs = self.0.floor();
        let rem = self.0 - degs;
        let mins = (rem * sixty).floor();
        let rem_s = rem * sixty - mins;
        let seconds = rem_s * sixty;

        (Deg(degs), ArcMinutes(mins), ArcSeconds(seconds))
    }
}

impl<T, U> IntoAngle<U> for T
    where U: Angle<Scalar = T::Scalar> + FromAngle<T>,
          T: Angle
{
    type OutputScalar = T::Scalar;
    fn into_angle(self) -> U {
        U::from_angle(self)
    }
}
impl<T: fmt::Display> fmt::Display for Deg<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°", self.0)
    }
}
impl<T: fmt::Display> fmt::Display for Rad<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}r", self.0)
    }
}
impl<T: fmt::Display> fmt::Display for Turns<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<T: fmt::Display> fmt::Display for ArcMinutes<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}'", self.0)
    }
}
impl<T: fmt::Display> fmt::Display for ArcSeconds<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\"", self.0)
    }
}

fn cast<T: NumCast, U: NumCast>(from: T) -> Option<U> {
    U::from(from)
}

#[cfg(test)]
mod test {
    use std::f64::consts;
    use std::f64;
    use super::*;

    #[test]
    fn test_convert() {
        assert_relative_eq!(ArcMinutes(120.0).into_angle(), Deg(2.0), epsilon=1e-6);
        assert_relative_eq!(ArcSeconds(30.0).into_angle(), ArcMinutes(0.5), epsilon=1e-6);
        assert_relative_eq!(Deg(30.0) + ArcMinutes(30.0) + ArcSeconds(30.0), 
            Deg(30.50833333333), epsilon=1e-6);
        assert_relative_eq!(Rad(consts::PI).into_angle(), Deg(180.0), epsilon=1e-6);
        assert_relative_eq!(Turns(0.25).into_angle(), Deg(90.0), epsilon=1e-6);
        assert_relative_eq!(Turns(0.25).into_angle(), Rad(consts::PI / 2.0), epsilon=1e-6);
        assert_relative_eq!(ArcMinutes(600.0).into_angle(), Deg(10.0), epsilon=1e-6);
        assert_relative_eq!(ArcMinutes(5400.0).into_angle(), Rad(consts::PI / 2.0), epsilon=1e-6);
    }

    #[test]
    fn test_arithmetic() {
        {
            let a1 = Rad(2.0);
            let a2 = Deg(100.0);

            let a3 = a2 + a1;
            assert_relative_eq!(a3.0, 214.59, epsilon=1e-2);

            let a4 = Deg(50.0);
            let a5 = a2 + a4;
            assert_ulps_eq!(a5.0, 150.0);

            let mut a6 = Deg(123.0);
            a6 += Deg(10.0);
            a6 += Rad(consts::PI);
            assert_ulps_eq!(a6.0, 313.0);

            let a7 = Deg(50.0);
            assert_ulps_eq!(a7 * 2.0, Deg(100.0));
        }
        {
            let a1 = Rad(2.0);
            let a2 = a1 % Rad(1.5);
            assert_ulps_eq!(a2, Rad(0.5));
            assert_ulps_eq!(Rad(1.0) * 2.0, Rad(2.0));
            assert_ulps_eq!(Rad(consts::PI * 2.0) / 2.0, Rad(consts::PI));
        }
    }

    #[test]
    fn test_trig() {
        assert_ulps_eq!(Deg(0.0).sin(), 0.0);
        assert_ulps_eq!(Rad(consts::PI / 2.0).sin(), 1.0);
        assert_ulps_eq!(Deg(90.0).sin(), 1.0);
        assert_ulps_eq!(Deg(45.0).tan(), 1.0);
        assert_relative_eq!(Deg(405.0).tan(), 1.0, epsilon=1e-6);
        let a1 = Rad(consts::PI * 1.25);
        assert_relative_eq!(a1.cos(), -f64::sqrt(2.0) / 2.0, epsilon=1e-6);
        assert_relative_eq!(a1.cos(), Deg(135.0).cos(), epsilon=1e-6);

        assert_relative_eq!(Deg::acos(1.0), Deg(0.0));
        assert_relative_eq!(Deg::acos(0.0), Deg(90.0));
        assert_relative_eq!(Rad::acos(0.0), Rad(consts::PI / 2.0));
    }

    #[test]
    fn test_equality() {
        let a1 = Rad(2.0);
        assert_ulps_eq!(a1, Rad(2.0));
        assert_ulps_eq!(Deg(200.0), Deg(200.0));
        assert!(!(Deg(200.0) == Deg(100.0)));

        assert!(Deg(200.0) < Deg(300.0));
        assert!(Deg(250.0) > Deg(100.0));
    }

    #[test]
    fn test_normalize() {
        let mut a1 = Deg(200.0);
        a1 += Deg(300.0);
        assert_ulps_eq!(a1, Deg(500.0));
        a1 = a1.normalize();
        assert_ulps_eq!(a1, Deg(140.0));
        assert_ulps_eq!(a1.normalize(), a1);

        let a2 = Deg(50.0);
        assert_ulps_eq!(a2 - Deg(150.0), Deg(-100.0));
        let a3 = a2 - Deg(100.0);
        assert!(!a3.is_normalized());
        assert_ulps_eq!(a3.normalize(), Deg(310.0));
        assert_ulps_eq!(a3.normalize().normalize(), a3.normalize());
        assert!(a3.normalize().is_normalized());

        let a4 = Rad(consts::PI);
        let a5 = a4 + Rad(consts::PI * 2.0);
        assert_ulps_eq!(a5, Rad(consts::PI * 3.0));
        assert!(!a3.is_normalized());
        assert_ulps_eq!(a5.normalize(), Rad(consts::PI));
        let a6 = a4 - Rad(consts::PI * 2.0);
        assert_ulps_eq!(a6, Rad(consts::PI * -1.0));
        assert!(!a6.is_normalized());
        assert_ulps_eq!(a6.normalize(), a5.normalize());

        assert_ulps_eq!(Deg(360.0).normalize(), Deg(0.0));
        assert_ulps_eq!(Deg(-1.0).normalize(), Deg(359.0));
    }

    #[test]
    fn decompose() {
        {
            let (deg, min, sec) = Deg(50.25).decompose();

            assert_ulps_eq!(deg, Deg(50.0));
            assert_ulps_eq!(min, ArcMinutes(15.0));
            assert_ulps_eq!(sec, ArcSeconds(0.0));
        }
        {
            let (deg, min, sec) = Deg(90.3131).decompose();

            assert_ulps_eq!(deg, Deg(90.0));
            assert_ulps_eq!(min, ArcMinutes(18.0));
            assert_relative_eq!(sec, ArcSeconds(47.16), epsilon=1e-6);
        }
    }

    #[test]
    fn test_interpolate() {
        assert_relative_eq!(Deg(60.0).interpolate(&Deg(120.0), 0.5), Deg(90.0));
        assert_relative_eq!(Deg(50.0).interpolate(&Rad(consts::PI), 0.75), 
                            Deg(147.5), epsilon=1e-6);
        assert_relative_eq!(Turns(0.50).interpolate(&Deg(30.0), 0.25), 
                            Turns(0.39583333333), epsilon=1e-6);

        assert_relative_eq!(Deg(100.0).interpolate(&Deg(310.0), 0.5).normalize(), Deg(25.0));
        assert_relative_eq!(Deg(100.0).interpolate_forward(&Deg(310.0), 0.5).normalize(), 
                            Deg(205.0));
    }

    #[test]
    fn test_constants() {
        assert_ulps_eq!(Deg::half_turn(), Deg(180.0));
        assert_ulps_eq!(Deg::quarter_turn(), Deg(90.0));
        assert_ulps_eq!(Rad::half_turn(), Rad(consts::PI));
        assert_ulps_eq!(Rad::<f32>::full_turn(), Rad(Rad::period()));
    }

    #[test]
    fn test_invert() {
        assert_ulps_eq!(Deg(0.0).invert(), Deg(180.0));
        assert_ulps_eq!(Deg(180.0).invert().normalize(), Deg(0.0));
        assert_ulps_eq!(Deg(80.0).invert(), Deg(260.0));
    }
}
