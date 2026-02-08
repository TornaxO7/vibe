use crate::{Tester, RED};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Radial, RadialDescriptor, RadialVariant};

#[test]
fn test() {
    let tester = Tester::default();

    let mut radial = Radial::new(&RadialDescriptor {
        renderer: &tester.renderer,
        processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        variant: RadialVariant::Color(RED.into()),

        init_rotation: cgmath::Deg(90.0),
        circle_radius: 0.2,
        bar_height_sensitivity: 1.0,
        bar_width: 0.01,
        position: (0.5, 0.5),
        format: vibe_renderer::components::RadialFormat::BassTreble,
    });

    tester.evaluate(
        &mut radial,
        include_bytes!("./reference.png"),
        "radial-color",
        0.9,
    );
}
