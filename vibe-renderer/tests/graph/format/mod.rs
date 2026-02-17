use crate::{Tester, RED, WHITE};
use test_fork::test;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Graph, GraphDescriptor, GraphFormat, GraphVariant};

#[test]
fn bass_treble() {
    test(
        GraphFormat::BassTreble,
        include_bytes!("./bass-treble.png"),
        "graph-bass-treble",
    );
}

#[test]
fn bass_treble_bass() {
    test(
        GraphFormat::BassTrebleBass,
        include_bytes!("./bass-treble-bass.png"),
        "graph-bass-treble-bass",
    );
}

#[test]
fn treble_bass() {
    test(
        GraphFormat::TrebleBass,
        include_bytes!("./treble-bass.png"),
        "graph-treble-bass",
    );
}

#[test]
fn treble_bass_treble() {
    test(
        GraphFormat::TrebleBassTreble,
        include_bytes!("./treble-bass-treble.png"),
        "graph-treble-bass-treble",
    );
}
fn test(format: GraphFormat, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut graph = Graph::new(&GraphDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: GraphVariant::HorizontalGradient {
            left: WHITE.into(),
            right: RED.into(),
        },
        placement: vibe_renderer::components::GraphPlacement::Bottom,
        format,
        border: None,
    });

    tester.evaluate(&mut graph, reference, id);
}
