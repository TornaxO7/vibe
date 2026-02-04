//! Everything related to the interpolation calculation.
mod context;
mod cubic_spline;
mod descriptor;
mod linear;
mod nothing;

use context::InterpolationCtx;
use std::slice::IterMut;

pub use cubic_spline::CubicSplineInterpolation;
pub use descriptor::*;
pub use linear::LinearInterpolation;
pub use nothing::NothingInterpolation;

/// Methods for the actual interpolating process.
pub trait Interpolater {
    fn interpolate(&mut self, buffer: &mut [f32]);

    fn get_ctx(&self) -> &InterpolationCtx;

    fn get_ctx_mut(&mut self) -> &mut InterpolationCtx;

    fn total_amount_bars(&self) -> usize {
        self.get_ctx().total_amount_bars()
    }

    fn supporting_points_unpadded_mut(&mut self) -> IterMut<'_, SupportingPoint> {
        self.get_ctx_mut().supporting_points_unpadded_mut()
    }
}

/// Trait to create new interpolations.
pub trait InterpolatorCreation: Interpolater + Sized {
    fn new(desc: InterpolatorDescriptor) -> Self;

    /// Same as `new` but puts the interpolator in a [`Box`].
    fn boxed(desc: InterpolatorDescriptor) -> Box<Self> {
        Box::new(Self::new(desc))
    }
}

// == Data structures ==

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SupportingPoint {
    /// The x value of the supporting point
    pub x: usize,

    /// The y value of the supporting point
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpolationSection {
    // assuming the supporting points are stored in an indexable data structure.
    // The attribute stores the index of the supporting point within the data sturcture.
    pub left_supporting_point_idx: usize,

    /// the amount of points which need to be interpolated
    /// within this section (up to the next supporting point)
    pub amount: usize,
}
