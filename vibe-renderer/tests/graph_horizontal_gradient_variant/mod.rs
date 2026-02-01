use std::num::NonZero;

use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Graph, GraphDescriptor, GraphVariant};

use crate::{Tester, BLUE, RED};

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let graph = Graph::new(&GraphDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
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
    });

    let _img = tester.render(graph);
    //
    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
