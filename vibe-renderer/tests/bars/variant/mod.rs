use crate::{Tester, BLUE, RED};
use test_fork::test;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement};

#[test]
fn color() {
    test(
        BarVariant::Color(RED.into()),
        include_bytes!("./color.png"),
        "bar-color",
    );
}

#[test]
fn presence_gradient() {
    test(
        BarVariant::PresenceGradient {
            high: BLUE.into(),
            low: RED.into(),
        },
        include_bytes!("./presence-gradient.png"),
        "bar-presence-gradient",
    );
}

#[test]
fn horizontal_gradient() {
    test(
        BarVariant::HorizontalGradient {
            left: RED.into(),
            right: BLUE.into(),
        },
        include_bytes!("./horizontal-gradient.png"),
        "bar-horizontal-gradient",
    );
}

#[test]
fn vertical_gradient() {
    test(
        BarVariant::VerticalGradient {
            top: RED.into(),
            bottom: BLUE.into(),
        },
        include_bytes!("./vertical-gradient.png"),
        "bar-vertical-gradient",
    );
}

fn test(variant: BarVariant, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut bars = Bars::new(&BarsDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant,
        placement: BarsPlacement::Bottom,
        format: BarsFormat::BassTreble,
        border: None,
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    tester.evaluate(&mut bars, reference, id);
}
