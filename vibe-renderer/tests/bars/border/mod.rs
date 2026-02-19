use crate::{Tester, BLUE, GREEN, RED};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{
    BarBorder, BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement,
};

#[test]
fn full() {
    test(
        BarBorder {
            color: GREEN.into(),
            width: 1.0,
        },
        include_bytes!("./full.png"),
        "bar-border-full",
    );
}

#[test]
fn exceeding() {
    test(
        BarBorder {
            color: GREEN.into(),
            width: 1.1,
        },
        // should equal "full"
        include_bytes!("./full.png"),
        "bar-border-exceeding",
    );
}

#[test]
fn zero() {
    test(
        BarBorder {
            color: GREEN.into(),
            width: 0.,
        },
        include_bytes!("./zero.png"),
        "bar-border-zero",
    );
}

#[test]
fn half() {
    test(
        BarBorder {
            color: GREEN.into(),
            width: 0.5,
        },
        include_bytes!("./half.png"),
        "bar-border-half",
    );
}

fn test(border: BarBorder, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut bars = Bars::new(&BarsDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::PresenceGradient {
            high: RED.into(),
            low: BLUE.into(),
        },
        placement: BarsPlacement::Custom {
            bottom_left_corner: (0., 0.5),
            width: NonZero::new(256).unwrap(),
            rotation: cgmath::Deg(0.),
            height_mirrored: true,
        },
        format: BarsFormat::BassTreble,
        border: Some(border),
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    tester.evaluate(&mut bars, reference, id);
}
