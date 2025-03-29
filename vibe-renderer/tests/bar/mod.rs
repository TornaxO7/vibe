use shady_audio::{fetcher::DummyFetcher, SampleProcessor};
use vibe_renderer::components::{Bars, BarsDescriptor};

use crate::Tester;

#[test]
fn wgsl_passes() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let mut bars = Bars::new(&BarsDescriptor {
        device: tester.renderer.device(),
        sample_processor: &sample_processor,
        audio_conf: shady_audio::Config::default(),
        texture_format: tester.output_texture_format(),
        fragment_source: wgpu::ShaderSource::Wgsl(include_str!("./frag.wgsl").into()),
    });

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
        audio_conf: shady_audio::Config::default(),
        texture_format: tester.output_texture_format(),
        fragment_source: wgpu::ShaderSource::Glsl {
            shader: include_str!("./frag.glsl").into(),
            stage: wgpu::naga::ShaderStage::Fragment,
            defines: wgpu::naga::FastHashMap::default(),
        },
    });

    bars.update_time(tester.renderer.queue(), 100.);

    let _img = tester.render(bars);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
