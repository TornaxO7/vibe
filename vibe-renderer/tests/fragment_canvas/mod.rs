use shady_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Component, FragmentCanvas, FragmentCanvasDescriptor, ShaderCode};

use crate::Tester;

// Check if the standard shaders are working
#[test]
fn wgsl_passes() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let mut frag_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        device: tester.renderer.device(),
        format: tester.output_texture_format(),
        fragment_code: ShaderCode {
            language: vibe_renderer::components::ShaderLanguage::Wgsl,
            source: vibe_renderer::components::ShaderSource::Code(
                include_str!("./frag.wgsl").into(),
            ),
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    frag_canvas.update_time(tester.renderer.queue(), 100.);

    let img = tester.render(frag_canvas);

    for &pixel in img.pixels() {
        let pixel_is_not_empty = pixel.0.iter().all(|value| *value != 0);
        assert!(pixel_is_not_empty);
    }
}

#[test]
fn glsl_passes() {
    let mut tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new());
    let mut frag_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        device: tester.renderer.device(),
        format: tester.output_texture_format(),
        fragment_code: ShaderCode {
            language: vibe_renderer::components::ShaderLanguage::Glsl,
            source: vibe_renderer::components::ShaderSource::Code(
                include_str!("./frag.glsl").into(),
            ),
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    frag_canvas.update_time(tester.renderer.queue(), 100.);

    let img = tester.render(frag_canvas);

    for &pixel in img.pixels() {
        let pixel_is_not_empty = pixel.0.iter().all(|value| *value != 0);
        assert!(pixel_is_not_empty);
    }
}
