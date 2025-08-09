# Vibe

`vibe` (to have a nice vibe with your music) is a desktop music visualizer inspired by [glava] and [shadertoy] for wayland!

**Note:** Your compositor _must_ support the [`wlr-layer-shell`] protocol. See [here](https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support)
for a list of compositors on which `vibe` should be able to run.

# Demo

You can click on the image below to see a live demo.

[![Demo video](https://img.youtube.com/vi/OQXdHLKH3ok/maxresdefault.jpg)](https://www.youtube.com/watch?v=OQXdHLKH3ok)

# Features

- support for (multiple) [shadertoy]-_like_-shaders (you can probably use most shaders from [shadertoy], but you can't just simply copy+paste them)
- audio processing support for shaders
- [wgsl] and [glsl] support for shaders
- some [predefined effects](https://github.com/TornaxO7/vibe/wiki/Config#components) which you can choose from

# State

It works on my machine and I've implemented basically everything I wanted and now I'm open for some feedback. For example in form of

- finding bugs
- suggestions or more ideas
- better user experience

Feel free to create an issue if you found a bug and/or an idea discussion if you'd like to suggest something.
However I can't promise to work on every suggestion/bug :>

**Note:** I'm unsure if I'd declare the config file format(s) of `vibe` as "stable", so for the time being: Be prepared for breaking changes.

# Usage

See [USAGE.md](./USAGE.md).

# Configs

See the [`Config` wiki page](https://github.com/TornaxO7/vibe/wiki/Config).

[shady-toy]: https://github.com/TornaxO7/shady/tree/main/shady-toy
[glava]: https://github.com/jarcode-foss/glava
[shadertoy]: https://www.shadertoy.com/
[wgsl]: https://www.w3.org/TR/WGSL/
[glsl]: https://www.khronos.org/opengl/wiki/Core_Language_(GLSL)
[`wlr-layer-shell`]: https://wayland.app/protocols/wlr-layer-shell-unstable-v1
