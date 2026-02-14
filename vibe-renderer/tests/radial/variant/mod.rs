use crate::{Tester, RED, WHITE};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Radial, RadialDescriptor, RadialVariant};

#[test]
fn color() {
    test(
        RadialVariant::Color(RED.into()),
        include_bytes!("./color.png"),
        "radial-color",
    );
}

#[test]
fn height_gradient() {
    test(
        RadialVariant::HeightGradient {
            inner: RED.into(),
            outer: WHITE.into(),
        },
        include_bytes!("./height-gradient.png"),
        "radial-height-gradient",
    );
}

fn test(variant: RadialVariant, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut radial = Radial::new(&RadialDescriptor {
        renderer: &tester.renderer,
        processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        output_texture_format: tester.output_texture_format(),
        variant,

        init_rotation: cgmath::Deg(90.0),
        circle_radius: 0.2,
        bar_height_sensitivity: 1.0,
        bar_width: 0.01,
        position: (0.5, 0.5),
        format: vibe_renderer::components::RadialFormat::BassTreble,
    });

    tester.evaluate(&mut radial, reference, id);
}
