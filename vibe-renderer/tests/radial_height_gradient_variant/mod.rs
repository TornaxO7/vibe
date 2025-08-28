use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Radial, RadialDescriptor, RadialVariant};

use crate::Tester;

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let radial = Radial::new(&RadialDescriptor {
        device: tester.renderer.device(),
        processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        variant: RadialVariant::HeightGradient {
            inner: super::RED,
            outer: super::WHITE,
        },

        init_rotation: cgmath::Deg(90.0),
        circle_radius: 0.01,
        bar_height_sensitivity: 0.3,
        bar_width: 0.1,
        position: (0.5, 0.5),
        format: vibe_renderer::components::RadialFormat::BassTrebleBass,
    });

    let _img = tester.render(radial);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
