use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Circle, CircleDescriptor, CircleVariant};

use crate::{Tester, WHITE};

#[test]
fn test() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let circle = Circle::new(&CircleDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        variant: CircleVariant::Graph {
            spike_sensitivity: 0.1,
            color: WHITE.into(),
        },
        radius: 0.1,
        rotation: cgmath::Deg(90.),
        position: (0.5, 0.5),
    });

    let _img = tester.render(&circle);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
