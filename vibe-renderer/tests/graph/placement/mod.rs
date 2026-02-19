use crate::{Tester, RED, WHITE};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{
    Graph, GraphDescriptor, GraphFormat, GraphPlacement, GraphVariant,
};

#[test]
fn top() {
    test(
        GraphPlacement::Top,
        include_bytes!("./top.png"),
        "graph-top",
    );
}

#[test]
fn right() {
    test(
        GraphPlacement::Right,
        include_bytes!("./right.png"),
        "graph-right",
    );
}

#[test]
fn bottom() {
    test(
        GraphPlacement::Bottom,
        include_bytes!("./bottom.png"),
        "graph-bottom",
    );
}

#[test]
fn left() {
    test(
        GraphPlacement::Left,
        include_bytes!("./left.png"),
        "graph-left",
    );
}

#[test]
fn custom() {
    test(
        GraphPlacement::Custom {
            bottom_left_corner: [0.1, 0.9],
            rotation: cgmath::Deg(45.0),
            amount_bars: NonZero::new(300).unwrap(),
        },
        include_bytes!("./custom.png"),
        "graph-custom",
    );
}

fn test(placement: GraphPlacement, reference: &'static [u8], id: &'static str) {
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
        placement,
        format: GraphFormat::TrebleBassTreble,
        border: None,
    });

    tester.evaluate(&mut graph, reference, id);
}
