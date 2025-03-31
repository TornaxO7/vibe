use shady_audio::{fetcher::DummyFetcher, SampleProcessor};
use vibe_renderer::components::{Bars, BarsDescriptor, ShaderCode};

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
        resolution: [0, 0],
        fragment_source: ShaderCode::Wgsl(include_str!("./frag.wgsl").into()),
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
        audio_conf: shady_audio::Config::default(),
        texture_format: tester.output_texture_format(),
        resolution: [0, 0],
        fragment_source: ShaderCode::Glsl(include_str!("./frag.glsl").into()),
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    bars.update_time(tester.renderer.queue(), 100.);

    let _img = tester.render(bars);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
