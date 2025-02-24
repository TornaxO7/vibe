# Vibe

A desktop [glava] like desktop music visualizer by using shaders!

# Demo

[![Demo video](https://img.youtube.com/vi/557iYiWnXn0/maxresdefault.jpg)](https://www.youtube.com/watch?v=557iYiWnXn0)

# State

Currently not really user friendly. Still under active development.

# Usage

1. Install `pavucontrol` (or any other tool to see which programs are currently recording and change the source).
2. Compile and run the binary with `cargo run --release`
3. Your microphone is very likely be catched as the audio source.
   To fix that open up `pavucontrol` and set the audio source (see: https://github.com/TornaxO7/shady?tab=readme-ov-file#shady-audio-doesnt-listen-to-my-systems-audio)

If you'd like to tweak around:
Take a look at the output. It will give you the path where you can to tweak some things.
To see the changes, kill the process and start it again.

[glava]: https://github.com/jarcode-foss/glava
