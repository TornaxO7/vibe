use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Graph, GraphDescriptor, GraphFormat, GraphVariant};

use crate::{Tester, RED};

#[test]
fn test() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let graph = Graph::new(&GraphDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: GraphVariant::Color(RED.into()),
        placement: vibe_renderer::components::GraphPlacement::Bottom,
        format: GraphFormat::BassTreble,
    });

    let _img = tester.render(&graph);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
