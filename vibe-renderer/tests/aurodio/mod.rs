use std::num::NonZero;

use shady_audio::{fetcher::DummyFetcher, SampleProcessor};
use vibe_renderer::components::{Aurodio, AurodioDescriptor, AurodioLayerDescriptor, Component};

use crate::Tester;

const BLUE: [f32; 3] = [0., 0., 1.];

#[test]
fn test() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let mut aurodio = Aurodio::new(&AurodioDescriptor {
        renderer: &tester.renderer,
        sample_processor: &sample_processor,
        texture_format: tester.output_texture_format(),
        base_color: BLUE,
        movement_speed: 0.2,
        layers: &[AurodioLayerDescriptor {
            freq_range: NonZero::new(50).unwrap()..NonZero::new(200).unwrap(),
            zoom_factor: 5.,
        }],
        sensitivity: 0.2,
    });

    aurodio.update_resolution(&tester.renderer, [255, 255]);

    let _img = tester.render(aurodio);

    // we don't do anything else because all bars are at the bottom
    // but the fragment shader should work... trust me bro
}
