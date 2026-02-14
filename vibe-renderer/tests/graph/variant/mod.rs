use crate::{Tester, RED, WHITE};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Graph, GraphDescriptor, GraphFormat, GraphVariant};

#[test]
fn color() {
    test(
        GraphVariant::Color(RED.into()),
        include_bytes!("./color.png"),
        "graph-color",
    );
}

#[test]
fn horizontal_gradient() {
    test(
        GraphVariant::HorizontalGradient {
            left: RED.into(),
            right: WHITE.into(),
        },
        include_bytes!("./horizontal-gradient.png"),
        "graph-horizontal-gradient",
    );
}

#[test]
fn vertical_gradient() {
    test(
        GraphVariant::VerticalGradient {
            top: WHITE.into(),
            bottom: RED.into(),
        },
        include_bytes!("./vertical-gradient.png"),
        "graph-vertical-gradient",
    );
}

fn test(variant: GraphVariant, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut graph = Graph::new(&GraphDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant,
        placement: vibe_renderer::components::GraphPlacement::Bottom,
        format: GraphFormat::TrebleBassTreble,
        border: None,
    });

    tester.evaluate(&mut graph, reference, id);
}
