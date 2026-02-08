use crate::Tester;
use image::ImageReader;
use std::num::NonZero;
use vibe_renderer::components::live_wallpaper::light_sources::{
    LightSourceData, LightSources, LightSourcesDescriptor,
};

#[test]
fn test() {
    let tester = Tester::default();

    let img = ImageReader::open("../assets/castle.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let mut light_sources = LightSources::new(&LightSourcesDescriptor {
        renderer: &tester.renderer,
        format: tester.output_texture_format(),
        processor: &tester.sample_processor,

        wallpaper: img,

        freq_range: NonZero::new(150).unwrap()..NonZero::new(250).unwrap(),
        sensitivity: 4.0,

        sources: &[LightSourceData {
            center: [0f32; 2],
            radius: 0.2,
        }],
        uniform_pulse: false,
        debug_sources: false,
    });

    tester.evaluate(
        &mut light_sources,
        include_bytes!("./reference.png"),
        "light-sources",
        0.35,
    );
}
