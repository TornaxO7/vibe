use vibe_audio::{
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    BarProcessor, BarProcessorConfig, SampleProcessor,
};

fn main() {
    // Choose default settings
    let descriptor = SystemAudioFetcherDescriptor::default();

    let mut processor = SampleProcessor::new(SystemAudioFetcher::new(&descriptor).unwrap());
    let mut bar_processor = BarProcessor::new(&processor, BarProcessorConfig::default());

    // simply fetch
    processor.process_next_samples();
    bar_processor.process_bars(&processor);
}
