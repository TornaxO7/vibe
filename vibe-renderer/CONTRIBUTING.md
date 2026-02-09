# Add a custom component
All components are in `src/components`. You can use everything in
- `src/components/mod.rs`
- `src/components/utils/*`
for your component.

## Convention

Each component must have a descriptor where you can configure the behaviour of your component.

## Example changes

Here's an example of how to add your own component:

1. Apply the following changes to the `src/components/mod.rs` file:
```rust
mod your_component;

pub use your_component::{your_component_descriptor, your_component};
```

2. Create the descriptor for your component:

`src/components/my_component/descriptor.rs`
```rust
use vibe_audio::{fetcher::Fetcher, BarProcessorConfig, SampleProcessor};
use crate::Renderer;

pub struct MyComponentDescriptor<&'a, F: Fetcher> {
  pub renderer: &'a Renderer,
  pub sample_processor: &'a SampleProcessor<F>, // to create the `BarProcessor`
  pub audio_config: BarProcessorConfig, // to create the `BarProcessor`
  pub texture_format: wgpu::TextureFormat, // the texture format of the output which should be rendered to

  // your additional configurations can be added here
  // ...
}
```

3. Create your component:

`src/components/my_component/mod.rs`
```rust
mod descriptor;

pub use descriptor::*;

use vibe_audio::Fetcher;

pub struct MyComponent {
  // stuff you need
}

impl MyComponent {
  pub fn new<F: Fetcher>(desc: &MyComponentDescriptor<F>) -> Self {
    // ...
  }
}

impl Renderable for MyComponent {
  // ...
}

impl Component for MyComponent {
  // ...
}

impl<F: Fetcher> ComponentAudio<F> for MyComponent {
  // ...
}
```

4. Go wild!