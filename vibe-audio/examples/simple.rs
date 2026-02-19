use vibe_audio::{
    fetcher::DummyFetcher, BarProcessor, BarProcessorConfig, NothingInterpolation, SampleProcessor,
};

fn main() {
    let mut sample_processor = SampleProcessor::new(DummyFetcher::new(2));
    sample_processor.process_next_samples();

    let mut bar_processor: BarProcessor<NothingInterpolation> =
        BarProcessor::new(&sample_processor, BarProcessorConfig::default());
    bar_processor.process_bars(&sample_processor);
}
