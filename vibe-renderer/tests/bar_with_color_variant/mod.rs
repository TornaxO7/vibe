use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement};

use crate::Tester;

const RED: [f32; 4] = [1., 0., 0., 1.];

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let bars = Bars::new(&BarsDescriptor {
        device: tester.renderer.device(),
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::Color(RED),
        placement: BarsPlacement::Top,
        format: BarsFormat::BassTreble,
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    let _img = tester.render(bars);
    //
    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
