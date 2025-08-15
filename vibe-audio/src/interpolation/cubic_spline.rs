use nalgebra::{Cholesky, DMatrix, DVector, Dyn};

use super::{context::InterpolationCtx, Interpolater, InterpolationInner};

type Width = usize;

#[derive(Debug, Clone)]
pub struct CubicSplineInterpolation {
    ctx: InterpolationCtx,

    section_widths: Box<[Width]>,

    matrix: Cholesky<f32, Dyn>,
    gradients: Box<[f32]>,
    gradient_diffs: Box<[f32]>,
}

impl InterpolationInner for CubicSplineInterpolation {
    fn new(supporting_points: impl IntoIterator<Item = super::SupportingPoint>) -> Self {
        let ctx = InterpolationCtx::new(supporting_points);

        let section_widths = if ctx.supporting_points.len() >= 2 {
            let mut section_widths = Vec::with_capacity(ctx.supporting_points.len() - 1);

            for (i, right) in ctx.supporting_points[1..].iter().enumerate() {
                let left = &ctx.supporting_points[i];

                let width = right.x - left.x;
                section_widths.push(width);
            }

            section_widths.into_boxed_slice()
        } else {
            Box::new([])
        };

        let amount_sections = section_widths.len();

        let matrix = {
            let matrix = get_matrix(&section_widths);

            ((1. / 6.) * matrix.clone()).cholesky().unwrap_or_else(|| panic!("Hold up! Looks like my numeric knowledge isn't really numericing ;-----;\nThe matrix which got calculated is: {}", matrix))
        };
        let gradients = vec![0f32; amount_sections].into_boxed_slice();
        let gradient_diffs = vec![0f32; amount_sections].into_boxed_slice();

        Self {
            ctx,
            section_widths,
            matrix,
            gradients,
            gradient_diffs,
        }
    }
}

impl Interpolater for CubicSplineInterpolation {
    fn interpolate(&mut self, buffer: &mut [f32]) {
        for point in self.ctx.supporting_points.iter() {
            buffer[point.x] = point.y;
        }

        if self.ctx.supporting_points.len() < 2 {
            return;
        }

        // == preparation ==
        debug_assert!(
            self.ctx.supporting_points.len() >= 2,
            "Starting from here, it's expected that we have at least 2 supporting points"
        );

        // update gradients
        {
            let prev_iter = self.ctx.supporting_points.iter();
            let next_iter = prev_iter.clone().skip(1);
            let gradient_iter = self.gradients.iter_mut();

            for ((gradient, prev), next) in gradient_iter.zip(prev_iter).zip(next_iter) {
                *gradient = (prev.y - next.y) / (prev.x as f32 - next.x as f32);
            }
        }

        // update gradient diffs
        {
            self.gradient_diffs[0] = self.gradients[0];

            {
                let diff_iter = self.gradient_diffs.iter_mut().skip(1);
                let prev_iter = self.gradients.iter();
                let next_iter = prev_iter.clone().skip(1);

                for ((diff, prev), next) in diff_iter.zip(prev_iter).zip(next_iter) {
                    *diff = next - prev;
                }
            }

            *self.gradient_diffs.last_mut().unwrap() = -self.gradients.last().unwrap();
        }

        // solve gamma
        let gammas = self
            .matrix
            .solve(&DVector::from_column_slice(&self.gradient_diffs));

        // == interpolation ==
        for section in self.ctx.sections.iter() {
            let n = section.left_supporting_point_idx + 1;

            let left = &self.ctx.supporting_points[n - 1];
            let right = &self.ctx.supporting_points[n];

            let prev_gamma = gammas[n - 1];
            // `None` appears, if we are in the last section.
            let next_gamma = gammas.get(n).cloned().unwrap_or(0.);

            let gradient = self.gradients[n - 1];
            let section_width = self.section_widths[n - 1];

            let amount = section.amount;
            for interpolated_idx in 0..amount {
                let bar_idx = interpolated_idx + 1 + left.x;
                let x = bar_idx as f32;

                let interpolated_value = left.y
                    + (x - left.x as f32) * gradient
                    + ((x - left.x as f32) * (x - right.x as f32)) / (6. * section_width as f32)
                        * ((prev_gamma + 2. * next_gamma) * (x - left.x as f32)
                            - (2. * prev_gamma + next_gamma) * (x - right.x as f32));

                buffer[bar_idx] = interpolated_value;
            }
        }
    }

    fn supporting_points_mut(&mut self) -> std::slice::IterMut<'_, super::SupportingPoint> {
        self.ctx.supporting_points.iter_mut()
    }
}

fn get_matrix(section_widths: &[usize]) -> DMatrix<f32> {
    let dimension = section_widths.len();

    let mut matrix = DMatrix::zeros(dimension, dimension);

    for n in 0..dimension {
        let mut row = matrix.row_mut(n);
        let prev_width = section_widths[n.saturating_sub(1)] as f32;
        let curr_width = section_widths[n] as f32;

        let is_in_first_row = n == 0;
        let is_in_last_row = n + 1 == dimension;

        if !is_in_first_row {
            row[n - 1] = prev_width;
        }

        if is_in_first_row || is_in_last_row {
            row[n] = 2. * curr_width;
        } else {
            row[n] = 2. * (prev_width + curr_width);
        }

        if !is_in_last_row {
            row[n + 1] = curr_width;
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use crate::interpolation::SupportingPoint;

    use super::*;

    fn validate_interpolation(interpolation: &[f32]) {
        // just check that the interpolation didn't panic (any unwraps)
        for value in interpolation.iter() {
            assert!(0. <= *value, "{:?}", interpolation);
        }
    }

    #[test]
    fn no_supporting_points() {
        let mut interpolator = CubicSplineInterpolation::new([]);
        let mut buffer = vec![];

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[]);
    }

    #[test]
    fn one_supporting_point() {
        let supporting_points = [SupportingPoint { x: 0, y: 1.0 }];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = CubicSplineInterpolation::new(supporting_points);

        interpolator.interpolate(&mut buffer);

        assert_eq!(&buffer, &[1.]);
    }

    #[test]
    fn two_supporting_points() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0. },
            SupportingPoint { x: 5, y: 1.0 },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = CubicSplineInterpolation::new(supporting_points);

        interpolator.interpolate(&mut buffer);

        validate_interpolation(&buffer);
    }

    #[test]
    fn three_supporting_points() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0. },
            SupportingPoint { x: 5, y: 0.25 },
            SupportingPoint { x: 10, y: 1. },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = CubicSplineInterpolation::new(supporting_points);

        interpolator.interpolate(&mut buffer);

        validate_interpolation(&buffer);
    }

    #[test]
    fn multiple_supporting_points() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0. },
            SupportingPoint { x: 5, y: 0.25 },
            SupportingPoint { x: 10, y: 0.3 },
            SupportingPoint { x: 15, y: 0.6 },
            SupportingPoint { x: 20, y: 1. },
        ];

        let mut buffer = vec![0f32; supporting_points.last().unwrap().x + 1];
        let mut interpolator = CubicSplineInterpolation::new(supporting_points);

        interpolator.interpolate(&mut buffer);

        validate_interpolation(&buffer);
    }

    mod matrix {
        use super::*;

        #[test]
        fn no_sections() {
            let matrix = get_matrix(&[]);
            let expected_matrix = DMatrix::from_row_slice(0, 0, &[]);

            assert_eq!(
                matrix, expected_matrix,
                "\nLeft:\n{}\nRight:\n{}",
                matrix, expected_matrix
            );
        }

        #[test]
        fn one_section() {
            const DIMENSION: usize = 1;
            let matrix = get_matrix(&[1]);
            let expected_matrix = DMatrix::from_row_slice(DIMENSION, DIMENSION, &[2.]);

            assert_eq!(
                matrix, expected_matrix,
                "\nLeft:\n{}\nRight:\n{}",
                matrix, expected_matrix
            );
        }

        #[test]
        fn two_sections() {
            const DIMENSION: usize = 2;
            let matrix = get_matrix(&[1; DIMENSION]);
            #[rustfmt::skip]
            let expected_matrix = DMatrix::from_row_slice(DIMENSION, DIMENSION,
                &[
                    2., 1.,
                    1., 2.
                ]
            );

            assert_eq!(
                matrix, expected_matrix,
                "\nLeft:\n{}\nRight:\n{}",
                matrix, expected_matrix
            );
        }

        #[test]
        fn three_sections() {
            const DIMENSION: usize = 3;
            let matrix = get_matrix(&[1; DIMENSION]);
            #[rustfmt::skip]
            let expected_matrix = DMatrix::from_row_slice(DIMENSION, DIMENSION,
                &[
                    2., 1., 0.,
                    1., 4., 1.,
                    0., 1., 2.,
                ]
            );

            assert_eq!(
                matrix, expected_matrix,
                "\nLeft:\n{}\nRight:\n{}",
                matrix, expected_matrix
            );
        }

        #[test]
        fn ten_sections() {
            const DIMENSION: usize = 10;
            let matrix = get_matrix(&[1; DIMENSION]);
            #[rustfmt::skip]
            let expected_matrix = DMatrix::from_row_slice(DIMENSION, DIMENSION,
                &[
                    2., 1., 0., 0., 0., 0., 0., 0., 0., 0.,
                    1., 4., 1., 0., 0., 0., 0., 0., 0., 0.,
                    0., 1., 4., 1., 0., 0., 0., 0., 0., 0.,
                    0., 0., 1., 4., 1., 0., 0., 0., 0., 0.,
                    0., 0., 0., 1., 4., 1., 0., 0., 0., 0.,
                    0., 0., 0., 0., 1., 4., 1., 0., 0., 0.,
                    0., 0., 0., 0., 0., 1., 4., 1., 0., 0.,
                    0., 0., 0., 0., 0., 0., 1., 4., 1., 0.,
                    0., 0., 0., 0., 0., 0., 0., 1., 4., 1.,
                    0., 0., 0., 0., 0., 0., 0., 0., 1., 2.,
                ]
            );

            assert_eq!(
                matrix, expected_matrix,
                "\nLeft:\n{}\nRight:\n{}",
                matrix, expected_matrix
            );
        }
    }
}
