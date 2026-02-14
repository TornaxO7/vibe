use crate::{Tester, BLUE, RED};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{BarVariant, Bars, BarsDescriptor, BarsFormat};

#[test]
fn test() {
    let tester = Tester::default();

    let mut bars = Bars::new(&BarsDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        max_height: 1.,
        variant: BarVariant::PresenceGradient {
            high: RED.into(),
            low: BLUE.into(),
        },
        placement: vibe_renderer::components::BarsPlacement::Right,
        format: BarsFormat::TrebleBassTreble,
        border: None,
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    tester.evaluate(
        &mut bars,
        include_bytes!("./reference.png"),
        "bar-presence-gradient",
    );
}
