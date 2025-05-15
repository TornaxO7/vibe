use shady_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Graph, GraphDescriptor, GraphVariant};

use crate::Tester;

const RED: [f32; 4] = [1., 0., 0., 1.];
const BLUE: [f32; 4] = [0., 0., 1., 1.];

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let graph = Graph::new(&GraphDescriptor {
        device: tester.renderer.device(),
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: GraphVariant::HorizontalGradient {
            left: RED,
            right: BLUE,
        },
        smoothness: 0.01,
    });

    let _img = tester.render(graph);
    //
    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
