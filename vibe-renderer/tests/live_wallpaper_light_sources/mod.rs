use std::num::NonZero;

use image::ImageReader;
use vibe_audio::{fetcher::DummyFetcher, SampleProcessor};
use vibe_renderer::components::live_wallpaper::light_sources::{
    LightSourceData, LightSources, LightSourcesDescriptor,
};

use crate::Tester;

#[test]
fn test() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));

    let img = ImageReader::open("../assets/castle.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let mut light_sources = LightSources::new(&LightSourcesDescriptor {
        renderer: &tester.renderer,
        format: tester.output_texture_format(),
        processor: &sample_processor,

        wallpaper: img,

        freq_range: NonZero::new(150).unwrap()..NonZero::new(250).unwrap(),
        sensitivity: 4.0,

        sources: &[LightSourceData {
            center: [0f32; 2],
            radius: 2f32,
        }],
        uniform_pulse: false,
        debug_sources: false,
    });

    let _img = tester.render(&mut light_sources);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
