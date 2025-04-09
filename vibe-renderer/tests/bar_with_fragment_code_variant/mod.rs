use shady_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, Component, ShaderCode};

use crate::Tester;

#[test]
fn wgsl_passes() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let mut bars = Bars::new(&BarsDescriptor {
        device: tester.renderer.device(),
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::FragmentCode {
            resolution: [0, 0],
            code: ShaderCode {
                language: vibe_renderer::components::ShaderLanguage::Wgsl,
                source: vibe_renderer::components::ShaderSource::Code(
                    include_str!("./frag.wgsl").into(),
                ),
            },
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    bars.update_time(tester.renderer.queue(), 100.);

    let _img = tester.render(bars);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}

#[test]
fn glsl_passes() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let mut bars = Bars::new(&BarsDescriptor {
        device: tester.renderer.device(),
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::FragmentCode {
            resolution: [0, 0],
            code: ShaderCode {
                language: vibe_renderer::components::ShaderLanguage::Glsl,
                source: vibe_renderer::components::ShaderSource::Code(
                    include_str!("./frag.glsl").into(),
                ),
            },
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    bars.update_time(tester.renderer.queue(), 100.);

    let _img = tester.render(bars);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
