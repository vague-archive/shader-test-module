//! Helper functions for math operations.

use std::{
    f32::consts::PI,
    ops::{Deref, Div, Rem},
};

use void_public::{Aspect, Mat2, Vec2};

pub fn division_result<T: Copy + Div<Output = T> + Rem<Output = T>>(
    dividend: T,
    divisor: T,
) -> (T, T) {
    (dividend / divisor, dividend % divisor)
}

/// Creates a rotation matrix that represents the rotation from splitting a circle
/// into `rotation_split_factor` equal parts. If `rotation_split_factor` is 2.,
/// then the rotation matrix will represent a rotation of 180 degrees or π radians.
/// 3. would represent 120 degrees or 2/3 π radians.
pub fn generate_equal_parts_rotation_matrix(rotation_split_factor: f32) -> Mat2 {
    let rotation_split_factor = if rotation_split_factor == 0. {
        1.
    } else {
        rotation_split_factor
    };
    Mat2::from_angle(2. * PI * (1. / (rotation_split_factor)))
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ZeroToHundredPercent(f32);

impl ZeroToHundredPercent {
    pub const fn new(value: f32) -> Self {
        // Once we upgrade to Rust 1.85 +, this should get changed to f32::clamp
        // since it will be const then
        let final_value = if value < 0.0 {
            0.
        } else if value > 1.0 {
            1.
        } else {
            value
        };
        Self(final_value)
    }
}

impl From<f32> for ZeroToHundredPercent {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl Deref for ZeroToHundredPercent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn screen_space_coordinate_by_percent(
    aspect: &Aspect,
    x_percent: ZeroToHundredPercent,
    y_percent: ZeroToHundredPercent,
) -> Vec2 {
    let half_width = aspect.width / 2.;
    let half_height = aspect.height / 2.;

    Vec2::new(
        -half_width + *x_percent * aspect.width,
        -half_height + *y_percent * aspect.height,
    )
}
