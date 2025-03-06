# Usage

1. Install the dependencies (see the [`Dependencies`](https://github.com/TornaxO7/vibe/blob/main/USAGE.md#dependencies) section below).
2. (optional) set the rust toolchain `rustup default stable`
3. Compile and run the binary with `cargo run --release`.
4. Your microphone is very likely be catched as the audio source.
   To fix that start the application `pavucontrol` and set the audio source (see: https://github.com/TornaxO7/shady?tab=readme-ov-file#shady-audio-doesnt-listen-to-my-systems-audio).

# Dependencies

Here's a list of package manager commands which you can copy+paste to install the required dependencies.
If your package manager isn't listed here, feel free to create a PR!

## `apt`

```
sudo apt install rustup librust-wayland-client-dev librust-alsa-sys-dev libxkbcommon-dev pavucontrol
```

## `dnf`

```
sudo dnf install rustup rust-wayland-client-devel rust-alsa-sys-devel libxkbcommon-devel pavucontrol
```
