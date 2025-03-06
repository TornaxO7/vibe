# Vibe

A desktop music visualizer inspired by [glava] and [shadertoy] for wayland!

**Note:** Your compositor _must_ support the [`wlr-layer-shell`] protocol. See [here](https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support)
for a list of compositors on which `vibe` should be able to run.

# Demo

[![Demo video](https://img.youtube.com/vi/557iYiWnXn0/maxresdefault.jpg)](https://www.youtube.com/watch?v=557iYiWnXn0)

# Features

- support for (multiple) [shadertoy]-_like_-shaders (you can probably use most shaders from [shadertoy], but you can't just simply copy+paste them)
- audio processing support for shaders
- [wgsl] and [glsl] support for shaders

# State

It works on my machine and I've implemented basicaly everything I wanted and now I'm looking for some feedback. For example in form of

- finding bugs
- suggestions or more ideas
- better user experience

Feel free to create an issue if you found a bug and/or an idea discussion if you'd like to suggest something.
However I can't promise to work on every suggestion :>

**Note:** `vibe` isn't stable yet (maybe in the future?) so be prepared for breaking changes.

# Usage

See [USAGE.md](./USAGE.md).

# Configs

See [CONFIG.md](./CONFIG.md).

[shady-toy]: https://github.com/TornaxO7/shady/tree/main/shady-toy
[glava]: https://github.com/jarcode-foss/glava
[shadertoy]: https://www.shadertoy.com/
[wgsl]: https://www.w3.org/TR/WGSL/
[glsl]: https://www.khronos.org/opengl/wiki/Core_Language_(GLSL)
[wlr-layer-shell]: https://wayland.app/protocols/wlr-layer-shell-unstable-v1
