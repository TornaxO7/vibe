use super::{InterpolationSection, SupportingPoint};
use std::ops::Range;

#[derive(Clone)]
pub struct InterpolationCtx {
    // Contains all supporting points (inclusive the ones for padding)
    pub supporting_points: Box<[SupportingPoint]>,
    pub sections: Box<[InterpolationSection]>,
}

/// Constructing stuff
impl InterpolationCtx {
    pub fn new(desc: super::InterpolatorDescriptor) -> Self {
        let supporting_points = desc.supporting_points;

        let sections = {
            let mut sections = Vec::new();

            if supporting_points.len() > 1 {
                for (i, supporting_point) in supporting_points[1..].iter().enumerate() {
                    let prev_supporting_point = supporting_points.get(i).unwrap();

                    let gap_size = supporting_point.x - prev_supporting_point.x - 1;
                    let there_is_a_gap = gap_size > 0;
                    if there_is_a_gap {
                        sections.push(InterpolationSection {
                            left_supporting_point_idx: i,
                            amount: gap_size,
                        });
                    }
                }
            }

            sections.into_boxed_slice()
        };

        let ctx = Self {
            supporting_points,
            sections,
        };

        tracing::debug!("{:?}", ctx);

        ctx
    }

    pub fn covered_bar_range(&self) -> Range<usize> {
        if self.supporting_points.is_empty() {
            0..0
        } else {
            let first = self.supporting_points.first().unwrap();
            let last = self.supporting_points.last().unwrap();

            first.x..(last.x + 1)
        }
    }

    /// Returns all supporting points excluding the padded ones.
    pub fn supporting_points_mut(&mut self) -> &mut [SupportingPoint] {
        &mut self.supporting_points
    }
}

impl std::fmt::Debug for InterpolationCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sp_iter = self.supporting_points.iter().enumerate().peekable();
        let mut s_iter = self.sections.iter().peekable();

        writeln!(f)?;
        writeln!(
            f,
            "Amount supporting points: {}",
            self.supporting_points.len()
        )?;
        writeln!(f, "Amount sections: {}", self.sections.len())?;
        writeln!(f, "Supporting point and sections:")?;

        loop {
            match (sp_iter.peek(), s_iter.peek()) {
                (Some((sp_idx, sp)), Some(s)) => {
                    if *sp_idx <= s.left_supporting_point_idx {
                        write!(f, "{:?}", sp)?;
                        sp_iter.next();
                    } else {
                        write!(f, "{:?}", s)?;
                        s_iter.next();
                    }
                }
                (Some((_sp_idx, sp)), None) => {
                    write!(f, "{:?}", sp)?;
                    sp_iter.next();
                }
                (None, Some(s)) => {
                    write!(f, "{:?}", s)?;
                    s_iter.next();
                }
                (None, None) => break,
            };

            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolation::InterpolatorDescriptor;

    #[test]
    fn no_points_total_amount_bars() {
        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: vec![].into(),
            ..Default::default()
        });

        assert!(ctx.covered_bar_range().is_empty());
    }

    #[test]
    fn no_points_no_sections() {
        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: vec![].into(),
            ..Default::default()
        });

        assert!(ctx.supporting_points.is_empty());
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn one_point_no_sections() {
        let supporting_points = vec![SupportingPoint { x: 0, y: 0.0 }];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn one_point_total_amount_bars() {
        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: vec![SupportingPoint { x: 0, y: 0. }].into(),
            ..Default::default()
        });

        assert_eq!(ctx.covered_bar_range(), 0..1);
    }

    #[test]
    fn two_points_no_sections() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 1, y: 1.0 },
        ];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn two_points_one_section() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 5, y: 1.0 },
        ];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[InterpolationSection {
                left_supporting_point_idx: 0,
                amount: 4
            }]
        );
    }

    #[test]
    fn two_points_total_amount_bars() {
        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: vec![
                SupportingPoint { x: 0, y: 0. },
                SupportingPoint { x: 1, y: 0. },
            ]
            .into(),
            ..Default::default()
        });

        assert_eq!(ctx.covered_bar_range(), 0..2);
    }

    #[test]
    fn three_points_one_section_at_the_beginning() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 0.0 },
            SupportingPoint { x: 3, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[InterpolationSection {
                left_supporting_point_idx: 0,
                amount: 1
            }]
        );
    }

    #[test]
    fn three_points_one_section_in_the_end() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 1, y: 0.0 },
            SupportingPoint { x: 3, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[InterpolationSection {
                left_supporting_point_idx: 1,
                amount: 1
            }]
        );
    }

    #[test]
    fn three_points_two_sections() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 0.0 },
            SupportingPoint { x: 4, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
        assert_eq!(
            ctx.sections.as_ref(),
            &[
                InterpolationSection {
                    left_supporting_point_idx: 0,
                    amount: 1
                },
                InterpolationSection {
                    left_supporting_point_idx: 1,
                    amount: 1
                }
            ]
        );
    }

    #[test]
    fn three_points_two_big_sections() {
        let supporting_points = vec![
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 5, y: 0.0 },
            SupportingPoint { x: 10, y: 0.0 },
        ];

        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.clone().into(),
            ..Default::default()
        });

        assert_eq!(
            ctx.sections.as_ref(),
            &[
                InterpolationSection {
                    left_supporting_point_idx: 0,
                    amount: 4
                },
                InterpolationSection {
                    left_supporting_point_idx: 1,
                    amount: 4
                }
            ]
        );
    }

    #[test]
    fn two_points_with_section_and_total_amount_bars() {
        let ctx = InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: vec![
                SupportingPoint { x: 0, y: 0. },
                SupportingPoint { x: 5, y: 0. },
            ]
            .into(),
            ..Default::default()
        });

        assert_eq!(ctx.covered_bar_range(), 0..6);
    }

    #[test]
    #[should_panic]
    fn invalid_supporting_points_ordering() {
        let supporting_points = vec![
            SupportingPoint { x: 1, y: 0.0 },
            SupportingPoint { x: 0, y: 0.0 },
        ];

        InterpolationCtx::new(InterpolatorDescriptor {
            supporting_points: supporting_points.into(),
            ..Default::default()
        });
    }
}
