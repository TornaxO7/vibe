use super::{
    context::InterpolationCtx, Interpolater, InterpolatorCreation, InterpolatorDescriptor,
};

/// Interpolates nothing... as the name says...
/// which basically means that it won't fill any other values.
#[derive(Debug)]
pub struct NothingInterpolation {
    ctx: InterpolationCtx,
}

impl InterpolatorCreation for NothingInterpolation {
    fn new(desc: InterpolatorDescriptor) -> Self {
        let ctx = InterpolationCtx::new(desc);

        Self { ctx }
    }
}

impl Interpolater for NothingInterpolation {
    fn interpolate(&mut self, buffer: &mut [f32]) {
        for point in self.ctx.supporting_points.iter() {
            buffer[point.x] = point.y;
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
    use crate::interpolation::SupportingPoint;

    use super::*;

    #[test]
    fn general() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 3, y: 0.5 },
            SupportingPoint { x: 4, y: 1.0 },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = NothingInterpolation::new(InterpolatorDescriptor {
            supporting_points,
            ..Default::default()
        });

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[0., 0., 0., 0.5, 1.0,]);
    }
}
