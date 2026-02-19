use crate::{Tester, RED};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement};

#[test]
fn bass_treble() {
    test(
        BarsFormat::BassTreble,
        include_bytes!("./bass-treble.png"),
        "bar-bass-treble",
    );
}

#[test]
fn bass_treble_bass() {
    test(
        BarsFormat::BassTrebleBass,
        include_bytes!("./bass-treble-bass.png"),
        "bar-bass-treble-bass",
    );
}

#[test]
fn treble_bass() {
    test(
        BarsFormat::TrebleBass,
        include_bytes!("./treble-bass.png"),
        "bar-treble-bass",
    );
}

#[test]
fn treble_bass_treble() {
    test(
        BarsFormat::TrebleBassTreble,
        include_bytes!("./treble-bass-treble.png"),
        "bar-treble-bass-treble",
    );
}

fn test(format: BarsFormat, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut bars = Bars::new(&BarsDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::Color(RED.into()),
        placement: BarsPlacement::Bottom,
        format,
        border: None,
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    tester.evaluate(&mut bars, reference, id);
}
