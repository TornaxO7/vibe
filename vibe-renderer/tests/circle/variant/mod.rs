use crate::{Tester, WHITE};
use test_fork::test;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{Circle, CircleDescriptor, CircleVariant};

#[test]
fn graph() {
    test(
        CircleVariant::Graph {
            spike_sensitivity: 0.1,
            color: WHITE.into(),
        },
        include_bytes!("./graph.png"),
        "circle-graph",
    );
}

fn test(variant: CircleVariant, reference: &'static [u8], id: &'static str) {
    let tester = Tester::default();

    let mut circle = Circle::new(&CircleDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        audio_conf: BarProcessorConfig::default(),
        texture_format: tester.output_texture_format(),
        variant,
        radius: 0.1,
        rotation: cgmath::Deg(90.),
        position: (0.5, 0.5),
    });

    tester.evaluate(&mut circle, reference, id);
}
