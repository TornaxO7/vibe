use crate::Tester;
use image::ImageReader;
use std::num::NonZero;
use test_fork::test;
use vibe_renderer::components::live_wallpaper::pulse_edges::{PulseEdges, PulseEdgesDescriptor};

#[test]
fn test() {
    let tester = Tester::default();

    let img = ImageReader::open("../assets/castle.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let mut pulse_edges = PulseEdges::new(&PulseEdgesDescriptor {
        renderer: &tester.renderer,
        sample_processor: &tester.sample_processor,
        texture_format: tester.output_texture_format(),

        img,
        freq_range: NonZero::new(100).unwrap()..NonZero::new(300).unwrap(),
        audio_sensitivity: 4.0,
        high_threshold_ratio: 0.7,
        low_threshold_ratio: 0.3,
        wallpaper_brightness: 0.2,
        edge_width: 4.0,
        pulse_brightness: 4.0,
        sigma: 1.,
        kernel_size: 9,
    })
    .unwrap();

    tester.evaluate(
        &mut pulse_edges,
        include_bytes!("./reference.png"),
        "pulse-edges",
    );
}
