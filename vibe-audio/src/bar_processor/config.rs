use std::{num::NonZero, ops::Range};

/// Decides which interpolation strategy for the bars.
#[derive(Debug, Clone, Copy, Hash)]
pub enum InterpolationVariant {
    /// No interpolation strategy should be used.
    ///
    /// Only the "supporting bars" (a.k.a. the bars which are picked up for a frequency range) which are calculated are going to be displayed.
    None,

    /// Use the linear interpolation.
    ///
    Linear,

    /// Use the cubic spline interpolation (recommended since it's the smoothest).
    CubicSpline,
}

/// Set the distribution of the bars.
#[derive(Debug, Clone, Copy, Hash, Default)]
pub enum BarDistribution {
    /// Tell the [`Barprocessor`] to distribute the bars so that the frequency spectrum
    /// looks like as if it would grow linear or in other words:
    /// To make the bars look "natural" to us.
    #[default]
    Uniform,

    /// Don't readjust the frequency bars so that it looks "natural" to us but
    /// physically correct.
    Natural,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaddingSide {
    Left,
    Right,
    Both,
}

impl PaddingSide {
    pub fn needs_left_padding(&self) -> bool {
        [Self::Left, Self::Both].contains(self)
    }

    pub fn needs_right_padding(&self) -> bool {
        [Self::Right, Self::Both].contains(self)
    }

    pub fn amount_padding_sides(&self) -> u8 {
        match self {
            PaddingSide::Left | PaddingSide::Right => 1,
            PaddingSide::Both => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaddingSize {
    // TODO: Add auto detection
    // Auto,
    // unit: "Bars"
    Custom(NonZero<u16>),
}

#[derive(Debug, Clone)]
pub struct PaddingConfig {
    pub side: PaddingSide,
    pub size: PaddingSize,
}

/// The config options for [crate::BarProcessor].
#[derive(Debug, Clone)]
pub struct BarProcessorConfig {
    /// Set the amount of bars which should be created.
    pub amount_bars: NonZero<u16>,

    /// Set the frequency range which the bar processor should consider.
    pub freq_range: Range<NonZero<u16>>,

    /// Decide how the bar values should be interpolated.
    pub interpolation: InterpolationVariant,

    /// Control how fast the bars should adjust to their new height.
    /// Default value: `2`.
    /// The higher the value, the "faster" the bars adjust to the new height.
    pub sensitivity: f32,

    /// Set the bar distribution.
    /// In general you needn't use another value than its default.
    pub bar_distribution: BarDistribution,

    pub padding: Option<PaddingConfig>,
}

impl Default for BarProcessorConfig {
    fn default() -> Self {
        Self {
            interpolation: InterpolationVariant::CubicSpline,
            amount_bars: NonZero::new(30).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: 2.,
            bar_distribution: BarDistribution::Uniform,
            padding: None,
        }
    }
}
