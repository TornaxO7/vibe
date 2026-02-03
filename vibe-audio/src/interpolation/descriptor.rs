use super::SupportingPoint;
use std::num::NonZero;

#[derive(Debug, PartialEq)]
pub enum InterpolatorPaddingSide {
    Left,
    Right,
    Both,
}

impl From<crate::PaddingSide> for InterpolatorPaddingSide {
    fn from(value: crate::PaddingSide) -> Self {
        match value {
            crate::PaddingSide::Left => Self::Left,
            crate::PaddingSide::Right => Self::Right,
            crate::PaddingSide::Both => Self::Both,
        }
    }
}

#[derive(Debug)]
pub enum InterpolatorPaddingSize {
    /// A custom amount of bars before the first and after the last supporting point
    /// (depending on `PaddingSide`).
    // should be enough... I think
    Custom(NonZero<u16>),
}

impl From<crate::PaddingSize> for InterpolatorPaddingSize {
    fn from(value: crate::PaddingSize) -> Self {
        match value {
            crate::PaddingSize::Custom(amount) => Self::Custom(amount),
        }
    }
}

#[derive(Debug)]
pub struct InterpolatorPadding {
    pub side: InterpolatorPaddingSide,
    pub size: InterpolatorPaddingSize,
}

impl From<crate::PaddingConfig> for InterpolatorPadding {
    fn from(value: crate::PaddingConfig) -> Self {
        Self {
            side: value.side.into(),
            size: value.size.into(),
        }
    }
}

/// Descriptor to create new interpolations.
#[derive(Default)]
pub struct InterpolatorDescriptor {
    pub supporting_points: Vec<SupportingPoint>,
    pub padding: Option<InterpolatorPadding>,
}
