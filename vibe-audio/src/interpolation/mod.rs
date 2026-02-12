//! Everything related to the interpolation calculation.
mod context;
mod cubic_spline;
mod linear;
mod nothing;

use context::InterpolationCtx;
use std::ops::Range;

pub use cubic_spline::CubicSplineInterpolation;
pub use linear::LinearInterpolation;
pub use nothing::NothingInterpolation;

/// Methods for the actual interpolating process.
pub trait Interpolater {
    fn interpolate(&mut self, buffer: &mut [f32]);

    fn get_ctx(&self) -> &InterpolationCtx;

    fn get_ctx_mut(&mut self) -> &mut InterpolationCtx;

    fn covered_bar_range(&self) -> Range<usize> {
        self.get_ctx().covered_bar_range()
    }

    fn supporting_points_mut(&mut self) -> &mut [SupportingPoint] {
        self.get_ctx_mut().supporting_points_mut()
    }
}

/// Descriptor to create new interpolations.
#[derive(Default)]
pub struct InterpolatorDescriptor {
    pub supporting_points: Box<[SupportingPoint]>,
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
