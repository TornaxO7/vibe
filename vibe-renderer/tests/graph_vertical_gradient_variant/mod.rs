use crate::{Tester, BLUE, RED};
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
        variant: GraphVariant::VerticalGradient {
            top: BLUE.into(),
            bottom: RED.into(),
        },
        placement: vibe_renderer::components::GraphPlacement::Top,
        format: vibe_renderer::components::GraphFormat::TrebleBassTreble,
    });

    tester.evaluate(
        &mut graph,
        include_bytes!("./reference.png"),
        "graph-vertical-gradient",
    );
}
