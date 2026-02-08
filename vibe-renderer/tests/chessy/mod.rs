use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::{
    components::{Chessy, ChessyDescriptor},
    texture_generation::SdfPattern,
};

use crate::Tester;

#[test]
fn test() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let mut chessy = Chessy::new(&ChessyDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
        audio_config: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        movement_speed: 0.01,
        pattern: SdfPattern::Heart,
        zoom_factor: 0.1,
    });

    let _img = tester.render(&mut chessy);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
