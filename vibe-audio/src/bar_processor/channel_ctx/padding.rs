use crate::{interpolation::SupportingPoint, PaddingSide};

#[derive(Debug, Clone)]
pub struct PaddingCtx {
    side: PaddingSide,
    factors: Box<[f32]>,
}

impl PaddingCtx {
    pub fn new(size: usize, side: PaddingSide) -> Self {
        let factors = {
            let mut factors = vec![0f32; size].into_boxed_slice();

            for step in 0..size {
                factors[step] = compute_factor(step as f32 / size as f32);
            }

            factors
        };

        PaddingCtx { side, factors }
    }

    /// Returns the amount of bars which it additional creates
    pub fn amount_bars(&self) -> usize {
        let factor = self.side.amount_padding_sides();

        self.padding_offset() * factor as usize
    }

    pub fn adjust_supporting_points(&self, sps: &mut [SupportingPoint]) {
        if self.side.needs_left_padding() {
            for sp in sps.iter_mut() {
                sp.x += self.factors.len();
            }
        }
    }

    fn padding_offset(&self) -> usize {
        self.factors.len()
    }

    pub fn apply(&self, bar_values: &mut [f32]) {
        let padding_offset = self.padding_offset();

        if self.side.needs_left_padding() {
            let reference_y = bar_values.get(padding_offset).cloned().unwrap();

            for (factor, bar_value) in self.factors.iter().cloned().zip(bar_values.iter_mut()) {
                *bar_value = factor * reference_y;
            }
        }

        if self.side.needs_right_padding() {
            let reference_y = bar_values
                .iter()
                .rev()
                .nth(padding_offset)
                .cloned()
                .unwrap();

            for (factor, bar_value) in self
                .factors
                .iter()
                .cloned()
                .zip(bar_values.iter_mut().rev())
            {
                *bar_value = factor * reference_y;
            }
        }
    }
}

fn compute_factor(x: f32) -> f32 {
    assert!((0. ..=1.).contains(&x));

    x.powf(2.)
}
