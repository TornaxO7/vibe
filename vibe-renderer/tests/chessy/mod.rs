use crate::Tester;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::{
    components::{Chessy, ChessyDescriptor},
    texture_generation::SdfPattern,
};

#[test]
fn test() {
    let tester = Tester::default();

    let mut chessy = Chessy::new(&ChessyDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_config: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        movement_speed: 0.1,
        pattern: SdfPattern::Heart,
        zoom_factor: 2.,
    });

    tester.evaluate(
        &mut chessy,
        include_bytes!("./reference.png"),
        "chessy",
        0.9,
    );
}
