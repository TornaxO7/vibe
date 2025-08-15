`shady-audio` is the audio backend for the other [shady tools].
Its interface let's you easily fetch the frequency presence of the given audio source
by selecting an audio source which implemented the `Fetcher` trait.

# Example

```rust
use std::num::NonZero;
use shady_audio::{SampleProcessor, BarProcessor, BarProcessorConfig, fetcher::DummyFetcher};

let mut sample_processor = SampleProcessor::new(DummyFetcher::new());

let mut bar_processor = BarProcessor::new(
    &sample_processor,
    BarProcessorConfig {
        amount_bars: NonZero::new(20).unwrap(),
        ..Default::default()
    }
);
let mut bar_processor2 = BarProcessor::new(
    &sample_processor,
    BarProcessorConfig {
        amount_bars: NonZero::new(10).unwrap(),
        ..Default::default()
    }
);

loop {
    // the sample processor needs to compute the new samples only once
    // for both bar processors (to reduce computation)
    sample_processor.process_next_samples();

    let bars = bar_processor.process_bars(&sample_processor);
    let bars2 = bar_processor2.process_bars(&sample_processor);

    assert_eq!(bars.len(), 20);
    assert_eq!(bars2.len(), 10);

    break;
}
```

[shady tools]: https://github.com/TornaxO7/shady
