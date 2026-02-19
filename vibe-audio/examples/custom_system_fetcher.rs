use vibe_audio::{
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    util::DeviceType,
    BarProcessor, BarProcessorConfig, LinearInterpolation, SampleProcessor,
};

fn main() {
    // get a list of all available devices
    let available_output_devices = vibe_audio::util::get_device_ids(DeviceType::Output)
        .expect("Output devices exists for the given host");

    println!("{:#?}", available_output_devices);

    // choose one
    let device = vibe_audio::util::get_device(
        available_output_devices.first().unwrap().clone(),
        DeviceType::Output,
    )
    .unwrap()
    .unwrap();

    let descriptor = SystemAudioFetcherDescriptor {
        device,
        ..Default::default()
    };

    let mut sample_processor = SampleProcessor::new(SystemAudioFetcher::new(&descriptor).unwrap());
    let mut bar_processor: BarProcessor<LinearInterpolation> =
        BarProcessor::new(&sample_processor, BarProcessorConfig::default());

    // start creating the bars
    sample_processor.process_next_samples();
    bar_processor.process_bars(&sample_processor);
}
