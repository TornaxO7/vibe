use crate::{Tester, BLUE, RED};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Graph, GraphDescriptor, GraphVariant};

#[test]
fn test() {
    let tester = Tester::default();

    let mut graph = Graph::new(&GraphDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: GraphVariant::HorizontalGradient {
            left: RED.into(),
            right: BLUE.into(),
        },
        placement: vibe_renderer::components::GraphPlacement::Custom {
            bottom_left_corner: [0.2, 0.5],
            rotation: cgmath::Deg(90.),
            amount_bars: NonZero::new(50).unwrap(),
        },
        format: vibe_renderer::components::GraphFormat::BassTrebleBass,
        border: None,
    });

    tester.evaluate(
        &mut graph,
        include_bytes!("./reference.png"),
        "graph-horizontal-gradient",
    );
}
