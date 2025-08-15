use super::{context::InterpolationCtx, Interpolater, InterpolationInner};

#[derive(Debug)]
pub struct NothingInterpolation {
    ctx: InterpolationCtx,
}

impl InterpolationInner for NothingInterpolation {
    fn new(supporting_points: impl IntoIterator<Item = super::SupportingPoint>) -> Self {
        let ctx = InterpolationCtx::new(supporting_points);

        Self { ctx }
    }
}

impl Interpolater for NothingInterpolation {
    fn interpolate(&mut self, buffer: &mut [f32]) {
        for point in self.ctx.supporting_points.iter() {
            buffer[point.x] = point.y;
        }
    }

    fn supporting_points_mut(&mut self) -> std::slice::IterMut<'_, super::SupportingPoint> {
        self.ctx.supporting_points.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::interpolation::SupportingPoint;

    use super::*;

    #[test]
    fn general() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 3, y: 0.5 },
            SupportingPoint { x: 4, y: 1.0 },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = NothingInterpolation::new(supporting_points);

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[0., 0., 0., 0.5, 1.0,]);
    }
}
