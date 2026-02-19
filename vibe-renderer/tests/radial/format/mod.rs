use crate::{Tester, RED, WHITE};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Radial, RadialDescriptor, RadialFormat, RadialVariant};

#[test]
fn bass_treble() {
    test(
        RadialFormat::BassTreble,
        include_bytes!("./bass-treble.png"),
        "radial-bass-treble",
    );
}

#[test]
fn bass_treble_bass() {
    test(
        RadialFormat::BassTrebleBass,
        include_bytes!("./bass-treble-bass.png"),
        "radial-bass-treble-bass",
    );
}

#[test]
fn treble_bass() {
    test(
        RadialFormat::TrebleBass,
        include_bytes!("./treble-bass.png"),
        "radial-treble-bass",
    );
}

#[test]
fn treble_bass_treble() {
    test(
        RadialFormat::TrebleBassTreble,
        include_bytes!("./treble-bass-treble.png"),
        "radial-treble-bass-treble",
    );
}

fn test(format: RadialFormat, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut radial = Radial::new(&RadialDescriptor {
        renderer: &tester.renderer,
        processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        variant: RadialVariant::HeightGradient {
            inner: RED.into(),
            outer: WHITE.into(),
        },
        init_rotation: cgmath::Deg(90.0),
        circle_radius: 0.2,
        bar_height_sensitivity: 1.0,
        bar_width: 0.01,
        position: (0.5, 0.5),
        format,
    });

    tester.evaluate(&mut radial, reference, id);
}
