use super::{context::InterpolationCtx, Interpolater, InterpolatorDescriptor};
use tracing::debug;

/// Interpolates linearly between two supporting points.
#[derive(Debug)]
pub struct LinearInterpolation {
    ctx: InterpolationCtx,
}

impl Interpolater for LinearInterpolation {
    fn new(desc: InterpolatorDescriptor) -> Self {
        let ctx = InterpolationCtx::new(desc);

        Self { ctx }
    }

    fn interpolate(&mut self, buffer: &mut [f32]) {
        for point in self.ctx.supporting_points.iter() {
            buffer[point.x] = point.y;
        }

        debug!("{:?}", self.ctx);

        for section in self.ctx.sections.iter() {
            let left = &self.ctx.supporting_points[section.left_supporting_point_idx];
            let right = &self.ctx.supporting_points[section.left_supporting_point_idx + 1];

            let amount = section.amount;
            for interpolate_idx in 0..amount {
                let t = (interpolate_idx + 1) as f32 / (amount + 1) as f32;

                let idx = left.x + interpolate_idx + 1;
                buffer[idx] = t * right.y + (1. - t) * left.y;
            }
        }
    }

    fn get_ctx(&self) -> &InterpolationCtx {
        &self.ctx
    }

    fn get_ctx_mut(&mut self) -> &mut InterpolationCtx {
        &mut self.ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolation::SupportingPoint;

    #[test]
    fn zero_supporting_points_and_zero_sections() {
        let mut interpolator = LinearInterpolation::new(InterpolatorDescriptor {
            supporting_points: vec![].into(),
            ..Default::default()
        });
        let mut buffer = vec![];

        interpolator.interpolate(&mut buffer);
        assert!(buffer.is_empty());
    }

    #[test]
    fn one_supporting_point_and_zero_sections() {
        let supporting_points = vec![SupportingPoint { x: 0, y: 0.5 }];

        let mut interpolator = LinearInterpolation::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });
        let mut buffer = [0f32];

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[0.5]);
    }

    #[test]
    fn two_supporting_points_and_one_section() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 4, y: 1.0 },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = LinearInterpolation::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn three_supporting_points_and_one_section() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 1.0 },
            SupportingPoint { x: 3, y: 0.0 },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = LinearInterpolation::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[0.0, 0.5, 1.0, 0.0]);
    }

    #[test]
    fn three_supporting_points_and_two_sections() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 1.0 },
            SupportingPoint { x: 6, y: 0.0 },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = LinearInterpolation::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[0.0, 0.5, 1.0, 0.75, 0.5, 0.25, 0.0],);
    }
}
