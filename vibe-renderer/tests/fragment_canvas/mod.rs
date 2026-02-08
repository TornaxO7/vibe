use std::io::Cursor;

use image::{DynamicImage, ImageReader};
use vibe_audio::{fetcher::DummyFetcher, BarProcessorConfig, SampleProcessor};
use vibe_renderer::components::{Component, FragmentCanvas, FragmentCanvasDescriptor, ShaderCode};

use crate::Tester;

fn load_img() -> DynamicImage {
    let img = Cursor::new(include_bytes!("./Bodiam_Castle_south.jpg"));

    ImageReader::with_format(img, image::ImageFormat::Jpeg)
        .decode()
        .unwrap()
}

// Check if the standard shaders are working
#[test]
fn wgsl_passes_without_img() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let mut frag_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        renderer: &tester.renderer,
        format: tester.output_texture_format(),

        img: None,
        fragment_code: ShaderCode {
            language: vibe_renderer::components::ShaderLanguage::Wgsl,
            source: vibe_renderer::components::ShaderSource::Code(
                include_str!("./frag_without_img.wgsl").into(),
            ),
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    frag_canvas.update_time(tester.renderer.queue(), 100.);

    let img = tester.render(&mut frag_canvas);

    for &pixel in img.pixels() {
        let pixel_is_not_empty = pixel.0.iter().all(|value| *value != 0);
        assert!(pixel_is_not_empty);
    }
}

#[test]
fn wgsl_passes_with_img() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let mut frag_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        renderer: &tester.renderer,
        format: tester.output_texture_format(),

        img: Some(load_img()),
        fragment_code: ShaderCode {
            language: vibe_renderer::components::ShaderLanguage::Wgsl,
            source: vibe_renderer::components::ShaderSource::Code(
                include_str!("./frag_with_img.wgsl").into(),
            ),
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    frag_canvas.update_time(tester.renderer.queue(), 100.);

    let img = tester.render(&mut frag_canvas);

    for &pixel in img.pixels() {
        let pixel_is_not_empty = pixel.0.iter().all(|value| *value != 0);
        assert!(pixel_is_not_empty);
    }
}

#[test]
fn glsl_passes_without_img() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let mut frag_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        renderer: &tester.renderer,
        format: tester.output_texture_format(),

        img: None,
        fragment_code: ShaderCode {
            language: vibe_renderer::components::ShaderLanguage::Glsl,
            source: vibe_renderer::components::ShaderSource::Code(
                include_str!("./frag_without_img.glsl").into(),
            ),
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    frag_canvas.update_time(tester.renderer.queue(), 100.);

    let img = tester.render(&mut frag_canvas);

    for &pixel in img.pixels() {
        let pixel_is_not_empty = pixel.0.iter().all(|value| *value != 0);
        assert!(pixel_is_not_empty);
    }
}

#[test]
fn glsl_passes_with_img() {
    let tester = Tester::default();

    let sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    let mut frag_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
        sample_processor: &sample_processor,
        audio_conf: BarProcessorConfig::default(),
        renderer: &tester.renderer,
        format: tester.output_texture_format(),

        img: Some(load_img()),
        fragment_code: ShaderCode {
            language: vibe_renderer::components::ShaderLanguage::Glsl,
            source: vibe_renderer::components::ShaderSource::Code(
                include_str!("./frag_with_img.glsl").into(),
            ),
        },
    })
    .unwrap_or_else(|msg| panic!("{}", msg));

    frag_canvas.update_time(tester.renderer.queue(), 100.);

    let img = tester.render(&mut frag_canvas);

    for &pixel in img.pixels() {
        let pixel_is_not_empty = pixel.0.iter().all(|value| *value != 0);
        assert!(pixel_is_not_empty);
    }
}
