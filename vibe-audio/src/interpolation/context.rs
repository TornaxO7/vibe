use super::{InterpolationSection, SupportingPoint};
use crate::interpolation::{InterpolatorPaddingSide, InterpolatorPaddingSize};
use std::{ops::Range, slice::IterMut};

#[derive(Clone)]
pub struct InterpolationCtx {
    // Contains all supporting points (inclusive the ones for padding)
    pub supporting_points: Box<[SupportingPoint]>,
    pub sections: Box<[InterpolationSection]>,

    // Stores the range for `supporting_points` which are the actual supporting points,
    // so excluding the supporting points which create the padding
    supporting_points_unpadded_range: Range<usize>,
}

/// Constructing stuff
impl InterpolationCtx {
    pub fn new(desc: super::InterpolatorDescriptor) -> Self {
        if let Some(sp) = desc.supporting_points.first() {
            // For now, just panic
            debug_assert!(
                sp.x == 0,
                "First supporting point must start at 0 but first supporting point starts at '{}'",
                sp.x
            );
        }

        let (supporting_points, unpadded_range) = {
            let mut supporting_points = desc.supporting_points;
            let mut unpadded_range = 0..supporting_points.len();

            if !supporting_points.is_empty() {
                if let Some(padding) = desc.padding {
                    let padding_size: usize = match padding.size {
                        InterpolatorPaddingSize::Custom(amount) => amount.get() as usize,
                    };

                    let requires_left_padding =
                        [InterpolatorPaddingSide::Left, InterpolatorPaddingSide::Both]
                            .contains(&padding.side);

                    if requires_left_padding {
                        let mut new_supporting_points =
                            Vec::with_capacity(supporting_points.len() + 1);

                        new_supporting_points.push(SupportingPoint { x: 0, y: 0. });

                        for mut sp in supporting_points {
                            sp.x += padding_size;
                            new_supporting_points.push(sp);
                        }

                        supporting_points = new_supporting_points;
                        unpadded_range.start += 1;
                        unpadded_range.end += 1;
                    }

                    let requires_right_padding = [
                        InterpolatorPaddingSide::Both,
                        InterpolatorPaddingSide::Right,
                    ]
                    .contains(&padding.side);

                    if requires_right_padding {
                        let x = match supporting_points.last() {
                            Some(last) => last.x + padding_size,
                            None => padding_size,
                        };

                        supporting_points.push(SupportingPoint { x, y: 0. });
                    }
                }
            }

            (supporting_points.into_boxed_slice(), unpadded_range)
        };

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
            supporting_points_unpadded_range: unpadded_range,
        };

        tracing::debug!("{:?}", ctx);

        ctx
    }

    pub fn total_amount_bars(&self) -> usize {
        if self.supporting_points.is_empty() {
            0
        } else if self.supporting_points.len() == 1 {
            1
        } else {
            let first = self.supporting_points.first().unwrap();
            let last = self.supporting_points.last().unwrap();

            (last.x - first.x) + 1
        }
    }

    /// Returns all supporting points excluding the padded ones.
    pub fn supporting_points_unpadded_mut(&mut self) -> IterMut<'_, SupportingPoint> {
        self.supporting_points[self.supporting_points_unpadded_range.clone()].iter_mut()
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

    mod no_padding {
        use super::*;

        #[test]
        #[should_panic]
        fn first_support_point_not_starting_at_0() {
            let supporting_points = vec![SupportingPoint { x: 1, y: 0. }];

            InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points,
                ..Default::default()
            });
        }

        #[test]
        fn no_points_total_amount_bars() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![],
                ..Default::default()
            });

            assert_eq!(ctx.total_amount_bars(), 0);
        }

        #[test]
        fn no_points_no_sections() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![],
                ..Default::default()
            });

            assert!(ctx.supporting_points.is_empty());
            assert!(ctx.sections.is_empty());
            assert!(ctx.supporting_points_unpadded_range.is_empty());
        }

        #[test]
        fn one_point_no_sections() {
            let supporting_points = vec![SupportingPoint { x: 0, y: 0.0 }];

            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: supporting_points.clone(),
                ..Default::default()
            });

            assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
            assert!(ctx.sections.is_empty());
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
            );
        }

        #[test]
        fn one_point_total_amount_bars() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![SupportingPoint { x: 0, y: 0. }],
                ..Default::default()
            });

            assert_eq!(ctx.total_amount_bars(), 1);
        }

        #[test]
        fn two_points_no_sections() {
            let supporting_points = vec![
                SupportingPoint { x: 0, y: 0.0 },
                SupportingPoint { x: 1, y: 1.0 },
            ];

            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: supporting_points.clone(),
                ..Default::default()
            });

            assert_eq!(ctx.supporting_points.as_ref(), &supporting_points);
            assert!(ctx.sections.is_empty());
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
            );
        }

        #[test]
        fn two_points_one_section() {
            let supporting_points = vec![
                SupportingPoint { x: 0, y: 0.0 },
                SupportingPoint { x: 5, y: 1.0 },
            ];

            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: supporting_points.clone(),
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
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
            );
        }

        #[test]
        fn two_points_total_amount_bars() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 1, y: 0. },
                ],
                ..Default::default()
            });

            assert_eq!(ctx.total_amount_bars(), 2);
        }

        #[test]
        fn three_points_one_section_at_the_beginning() {
            let supporting_points = vec![
                SupportingPoint { x: 0, y: 0.0 },
                SupportingPoint { x: 2, y: 0.0 },
                SupportingPoint { x: 3, y: 0.0 },
            ];

            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: supporting_points.clone(),
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
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
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
                supporting_points: supporting_points.clone(),
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
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
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
                supporting_points: supporting_points.clone(),
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
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
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
                supporting_points: supporting_points.clone(),
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
            assert_eq!(
                ctx.supporting_points_unpadded_range,
                0..supporting_points.len()
            );
        }

        #[test]
        fn two_points_with_section_and_total_amount_bars() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 5, y: 0. },
                ],
                ..Default::default()
            });

            assert_eq!(ctx.total_amount_bars(), 6);
        }

        #[test]
        #[should_panic]
        fn invalid_supporting_points_ordering() {
            let supporting_points = vec![
                SupportingPoint { x: 1, y: 0.0 },
                SupportingPoint { x: 0, y: 0.0 },
            ];

            InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: supporting_points,
                ..Default::default()
            });
        }
    }

    mod with_padding {
        use super::*;
        use crate::interpolation::InterpolatorPadding;
        use std::num::NonZero;

        #[test]
        fn no_points_left_side_padding() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![],
                padding: Some(InterpolatorPadding {
                    side: InterpolatorPaddingSide::Left,
                    size: InterpolatorPaddingSize::Custom(NonZero::new(10).unwrap()),
                }),
            });

            assert!(ctx.supporting_points.is_empty());
            assert!(ctx.sections.is_empty());
            assert!(ctx.supporting_points_unpadded_range.is_empty());
        }

        #[test]
        fn no_points_right_side_padding() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![],
                padding: Some(InterpolatorPadding {
                    side: InterpolatorPaddingSide::Right,
                    size: InterpolatorPaddingSize::Custom(NonZero::new(10).unwrap()),
                }),
            });

            assert!(ctx.supporting_points.is_empty(),);
            assert!(ctx.sections.is_empty());
            assert!(ctx.supporting_points_unpadded_range.is_empty());
        }

        #[test]
        fn one_point_left_side_padding() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![SupportingPoint { x: 0, y: 0. }],
                padding: Some(InterpolatorPadding {
                    side: InterpolatorPaddingSide::Left,
                    size: InterpolatorPaddingSize::Custom(NonZero::new(10).unwrap()),
                }),
            });

            assert_eq!(
                ctx.supporting_points.as_ref(),
                &[
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 10, y: 0. }
                ]
            );
            assert_eq!(
                ctx.sections.as_ref(),
                &[InterpolationSection {
                    left_supporting_point_idx: 0,
                    amount: 9,
                }]
            );
            assert_eq!(ctx.supporting_points_unpadded_range, 1..2);
        }

        #[test]
        fn one_point_right_side_padding() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![SupportingPoint { x: 0, y: 0. }],
                padding: Some(InterpolatorPadding {
                    side: InterpolatorPaddingSide::Right,
                    size: InterpolatorPaddingSize::Custom(NonZero::new(5).unwrap()),
                }),
            });

            assert_eq!(
                ctx.supporting_points.as_ref(),
                &[
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 5, y: 0. }
                ]
            );
            assert_eq!(
                ctx.sections.as_ref(),
                &[InterpolationSection {
                    left_supporting_point_idx: 0,
                    amount: 4
                }]
            );
            assert_eq!(ctx.supporting_points_unpadded_range, 0..1);
        }

        #[test]
        fn one_point_both_sides_padding() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![SupportingPoint { x: 0, y: 0. }],
                padding: Some(InterpolatorPadding {
                    side: InterpolatorPaddingSide::Both,
                    size: InterpolatorPaddingSize::Custom(NonZero::new(5).unwrap()),
                }),
            });

            assert_eq!(
                ctx.supporting_points.as_ref(),
                &[
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 5, y: 0. },
                    SupportingPoint { x: 10, y: 0. }
                ]
            );
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
            assert_eq!(ctx.supporting_points_unpadded_range, 1..2);
        }

        #[test]
        fn two_points_both_sides_padding() {
            let ctx = InterpolationCtx::new(InterpolatorDescriptor {
                supporting_points: vec![
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 5, y: 0. },
                ],
                padding: Some(InterpolatorPadding {
                    side: InterpolatorPaddingSide::Both,
                    size: InterpolatorPaddingSize::Custom(NonZero::new(5).unwrap()),
                }),
            });

            assert_eq!(
                ctx.supporting_points.as_ref(),
                &[
                    SupportingPoint { x: 0, y: 0. },
                    SupportingPoint { x: 5, y: 0. },
                    SupportingPoint { x: 10, y: 0. },
                    SupportingPoint { x: 15, y: 0. }
                ]
            );
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
                    },
                    InterpolationSection {
                        left_supporting_point_idx: 2,
                        amount: 4
                    }
                ]
            );
            assert_eq!(ctx.supporting_points_unpadded_range, 1..3);
        }
    }
}
