use crate::Tester;
use test_fork::test;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::{
    components::{Chessy, ChessyDescriptor},
    texture_generation::SdfPattern,
};

#[test]
fn r#box() {
    test(SdfPattern::Box, include_bytes!("./box.png"), "chessy-box");
}

#[test]
fn circle() {
    test(
        SdfPattern::Circle,
        include_bytes!("./circle.png"),
        "chessy-circle",
    );
}

#[test]
fn heart() {
    test(
        SdfPattern::Heart,
        include_bytes!("./heart.png"),
        "chessy-heart",
    );
}

fn test(pattern: SdfPattern, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut chessy = Chessy::new(&ChessyDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_config: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        movement_speed: 0.1,
        pattern,
        zoom_factor: 2.,
    });

    tester.evaluate(&mut chessy, reference, id);
}
