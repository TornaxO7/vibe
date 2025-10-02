use std::num::NonZero;

use image::ImageReader;
use vibe_audio::{fetcher::DummyFetcher, SampleProcessor};
use vibe_renderer::components::live_wallpaper::pulse_edges::{PulseEdges, PulseEdgesDescriptor};

use crate::Tester;

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));

    let img = ImageReader::open("../assets/castle.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let pulse_edges = PulseEdges::new(&PulseEdgesDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
        texture_format: tester.output_texture_format(),

        img,
        freq_range: NonZero::new(100).unwrap()..NonZero::new(250).unwrap(),
        audio_sensitivity: 4.0,
        high_threshold_ratio: 0.6,
        low_threshold_ratio: 0.3,
        wallpaper_brightness: 0.5,
        edge_width: 4.0,
        pulse_brightness: 5.0,
        sigma: 10.,
        kernel_size: 49,
    })
    .unwrap();

    let _img = tester.render(pulse_edges);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
