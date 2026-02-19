use crate::{Tester, RED};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement};

#[test]
fn top() {
    test(BarsPlacement::Top, include_bytes!("./top.png"), "bar-top");
}

#[test]
fn right() {
    test(
        BarsPlacement::Right,
        include_bytes!("./right.png"),
        "bar-right",
    );
}

#[test]
fn bottom() {
    test(
        BarsPlacement::Bottom,
        include_bytes!("./bottom.png"),
        "bar-bottom",
    );
}

#[test]
fn left() {
    test(
        BarsPlacement::Left,
        include_bytes!("./left.png"),
        "bar-left",
    );
}

#[test]
fn custom() {
    test(
        BarsPlacement::Custom {
            bottom_left_corner: (0.1, 0.9),
            width: NonZero::new(300).unwrap(),
            rotation: cgmath::Deg(45.0),
            height_mirrored: true,
        },
        include_bytes!("./custom.png"),
        "bar-custom",
    );
}

fn test(placement: BarsPlacement, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut bars = Bars::new(&BarsDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::Color(RED.into()),
        placement,
        format: BarsFormat::BassTreble,
        border: None,
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    tester.evaluate(&mut bars, reference, id);
}
