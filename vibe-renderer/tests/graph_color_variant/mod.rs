use crate::{Tester, RED};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Graph, GraphDescriptor, GraphFormat, GraphVariant};

#[test]
fn test() {
    let tester = Tester::default();

    let mut graph = Graph::new(&GraphDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: GraphVariant::Color(RED.into()),
        placement: vibe_renderer::components::GraphPlacement::Bottom,
        format: GraphFormat::BassTreble,
        border: None,
    });

    tester.evaluate(&mut graph, include_bytes!("./reference.png"), "graph-color");
}
