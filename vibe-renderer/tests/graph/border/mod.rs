use crate::{Tester, GREEN, RED, WHITE};
use std::num::NonZero;
use test_fork::test;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{
    Graph, GraphBorder, GraphDescriptor, GraphFormat, GraphPlacement, GraphVariant,
};

#[test]
fn full() {
    test(
        GraphBorder {
            color: GREEN.into(),
            width: 1.0,
        },
        include_bytes!("./full.png"),
        "graph-border-full",
    );
}

#[test]
fn exceeding() {
    test(
        GraphBorder {
            color: GREEN.into(),
            width: 1.1,
        },
        // should equal "full"
        include_bytes!("./exceeding.png"),
        "graph-border-exceeding",
    );
}

#[test]
fn zero() {
    test(
        GraphBorder {
            color: GREEN.into(),
            width: 0.,
        },
        include_bytes!("./zero.png"),
        "graph-border-zero",
    );
}

#[test]
fn half() {
    test(
        GraphBorder {
            color: GREEN.into(),
            width: 0.5,
        },
        include_bytes!("./half.png"),
        "graph-border-half",
    );
}

fn test(border: GraphBorder, reference: &'static [u8], id: &'static str) {
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
        placement: GraphPlacement::Custom {
            bottom_left_corner: [0., 0.5],
            rotation: cgmath::Deg(0.),
            amount_bars: NonZero::new(256).unwrap(),
        },
        format: GraphFormat::TrebleBassTreble,
        border: Some(border),
    });

    tester.evaluate(&mut graph, reference, id);
}
