use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, BarsFormat};

use crate::{Tester, BLUE, RED};

#[test]
fn test() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let bars = Bars::new(&BarsDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::PresenceGradient {
            high: RED.into(),
            low: BLUE.into(),
        },
        placement: vibe_renderer::components::BarsPlacement::Right,
        format: BarsFormat::TrebleBassTreble,
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    let _img = tester.render(&bars);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
