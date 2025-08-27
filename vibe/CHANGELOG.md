## [vibe-v2.2.0] - 2025-08-27

### 🚀 Features

- _(radial)_ Add `height-gradient-variant`

### ⚙️ Miscellaneous Tasks

- _(cliff)_ Update git cliff config file
- Release

## [vibe-v2.1.0] - 2025-08-23

### 🚀 Features

- _(component)_ Init `chessy` component

### ⚙️ Miscellaneous Tasks

- Release

## [vibe-v2.0.1] - 2025-08-22

### 💼 Other

- Bump shady version

### 🚜 Refactor

- Remove double updating audio

### ⚙️ Miscellaneous Tasks

- Update changelog
- Explain breaking changes
- _(docs)_ Update changelogs
- Release

## [vibe-v2.0.0] - 2025-08-15

### 🚀 Features

- _(window)_ Let window title include output name

### 🐛 Bug Fixes

- Typos

### 💼 Other

- _(deps)_ Bump toml from 0.8.23 to 0.9.2
- _(deps)_ Bump the rust-dependencies group with 8 updates
- _(deps)_ Bump the rust-dependencies group with 10 updates

### ⚙️ Miscellaneous Tasks

- Release

## [vibe-v1.0.1] - 2025-06-13

### 🚀 Features

- _(graph)_ Add smoothness option
- _(output config)_ Add full reference config with test
- _(bars)_ Add placement config
- _(graph)_ Add positioning
- _(radial)_ Add position config
- _(cli)_ Add command to list all output devices
- _(bars)_ Add mirroring option

### 🚜 Refactor

- _(examples)_ Create one single example binary instead of multiple equal ones

### ⚙️ Miscellaneous Tasks

- _(shady-audio)_ Update dep and migrate to it
- Update changelog
- Failed the release version...
- Release

## [vibe-v0.0.7] - 2025-05-15

### 🐛 Bug Fixes

- _(config)_ Remove missused attribute

### ⚙️ Miscellaneous Tasks

- Release

## [vibe-v0.0.6] - 2025-05-15

### 🚀 Features

- _(circle)_ Add graph circle
- _(radial, config)_ Init config code

### 🐛 Bug Fixes

- _(deps)_ Apply changes for xdg

### 💼 Other

- _(deps)_ Bump xdg from 2.5.2 to 3.0.0

### ⚙️ Miscellaneous Tasks

- Release

## [vibe-v0.0.5] - 2025-05-09

### ⚙️ Miscellaneous Tasks

- _(default shader)_ Change default component to filled bars
- _(graph)_ First structure for graph
- _(changelog)_ Update changelog
- Release

## [vibe-v0.0.4] - 2025-04-28

### 🐛 Bug Fixes

- _(default shader)_ Fix missing bracket in the default shader code

### ⚙️ Miscellaneous Tasks

- Update changelog
- Release

## [vibe-v0.0.3] - 2025-04-17

### 🐛 Bug Fixes

- Move `release.toml` to root of repository

### ⚙️ Miscellaneous Tasks

- Release

## [vibe-v0.0.2] - 2025-04-17

### 🐛 Bug Fixes

- Remove gamma correction requirement for shaders

### 🚜 Refactor

- Move release.toml

### ⚙️ Miscellaneous Tasks

- Release

## [vibe-v0.0.1] - 2025-04-10

### 🚀 Features

- [**breaking**] Implement hot reloading

### 🐛 Bug Fixes

- Resolution is applied to component during hot reloading

### 💼 Other

- Remove state
- Adding file watcher for output changes
- Implement registering of configs which got added/removed
- Adding entry for amount of bars
- Improve management for setting amount of bars

### 📚 Documentation

- Reformat description for `output_name` in the cli

### 🎨 Styling

- Make clippy happy

### 🧪 Testing

- Add test for external_paths() function of OutputConfig

### ⚙️ Miscellaneous Tasks

- Improve error handling for hot reloader
- Prepare release
