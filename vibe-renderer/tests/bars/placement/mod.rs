use crate::{Tester, RED};
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
