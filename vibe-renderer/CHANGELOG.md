# Changelog

All notable changes to this project will be documented in this file.

## [0.0.2] - 2025-05-15

### 🚀 Features

- *(circle)* Add graph circle
- *(radial)* Init code
- *(readial)* Add color variant
- *(radial, config)* Init config code

### 🐛 Bug Fixes

- Remove gamma correction requirement for shaders

### 💼 Other

- *(deps)* Bump bytemuck from 1.22.0 to 1.23.0

### 📚 Documentation

- *(renderer)* Add some docs
- *(preambles)* Adding docs to the preambles

### 🧪 Testing

- *(radial, color)* Init test

### ⚙️ Miscellaneous Tasks

- Remove unused file
- *(graph)* First structure for graph

## [ibe-v0.0.1] - 2025-04-10

### 🚀 Features

- Add optional grahics option `gpu_name`
- Add option to use path instead of inline code for fragment code
- [**breaking**] Implement hot reloading

### 💼 Other

- Create working example
- Add iTime to fragment shader
- Add fragment_canvas
- Change order of the buffers to match the previous state
- Add option to fallback to software rendering
- Optimize bind group index
- Some cleanup (removed some files)
- Remove requirement to include bindings and bind groups in fragment shader
- Add iResolution to fragment shader
- Add binding to set the max height
- Implement solid color variant
- Add presence gradient
- Change config to allow to configure the layers as you want
- Increase variation for the random seeds

### ⚙️ Miscellaneous Tasks

- Cleanup
- Simplify component descriptors
- Prepare release

<!-- generated by git-cliff -->
