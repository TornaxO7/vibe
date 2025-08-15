//! Everything related to the interpolation calculation.

mod context;
mod cubic_spline;
mod linear;
mod nothing;

use std::slice::IterMut;

pub use cubic_spline::CubicSplineInterpolation;
pub use linear::LinearInterpolation;
pub use nothing::NothingInterpolation;

pub trait Interpolater {
    fn interpolate(&mut self, buffer: &mut [f32]);

    fn supporting_points_mut(&mut self) -> IterMut<'_, SupportingPoint>;
}

pub trait InterpolationInner: Interpolater + Sized {
    fn new(supporting_points: impl IntoIterator<Item = SupportingPoint>) -> Self;

    fn boxed(supporting_points: impl IntoIterator<Item = SupportingPoint>) -> Box<Self> {
        Box::new(Self::new(supporting_points))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupportingPoint {
    /// The x value of the supporting point
    pub x: usize,

    /// The y value of the supporting point
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InterpolationSection {
    // assuming the supporting points are stored in an indexable data structure.
    // The attribute stores the index of the supporting point within the data sturcture.
    pub left_supporting_point_idx: usize,

    /// the amount of points which need to be interpolated
    /// within this section (up to the next supporting point)
    pub amount: usize,
}
