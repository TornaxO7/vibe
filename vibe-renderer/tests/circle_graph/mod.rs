use shady_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Circle, CircleDescriptor, CircleVariant};

use crate::Tester;

const WHITE: [f32; 4] = [1.; 4];

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let circle = Circle::new(&CircleDescriptor {
        device: tester.renderer.device(),
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        variant: CircleVariant::Graph {
            spike_sensitivity: 0.1,
            color: WHITE,
        },
        radius: 0.1,
        rotation: cgmath::Deg(90.),
    });

    let _img = tester.render(circle);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
