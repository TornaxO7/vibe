# Changelog

All notable changes to this project will be documented in this file.

## 1.0.1 - 2025-06-13

[b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163)...[6cf98ac](6cf98ac26f4da4e7c47bc9873b905e0f8478c5e1)

### Features

- Add smoothness option ([68a1b40](68a1b40ac4e56c6c4e8a898996b0460b4c5c86fc))
- Add full reference config with test ([65809fe](65809fe105a88b526f1c2a8b7903cb3666add39d))
- Add placement config ([f0d265f](f0d265f19bf78f402eb3cc346d7ff34e4761e60f))
- Add positioning ([d25ae5a](d25ae5a00161fde6b073dfcf024d858f8f781db7))
- Add position config ([8a8af3c](8a8af3c99d35b09ace499acae54e3d2ff5fc00de))
- Add command to list all output devices ([b41ac49](b41ac497ec670e16fcb3e7b3ce20851749c2c094))
- Add mirroring option ([41d948b](41d948b658150343e804772b4163b55faf0ab4fb))

### Miscellaneous Tasks

- Update dep and migrate to it ([35b0962](35b09629a2a5e0233ad89c2744b08ba818c70a29))
- Update changelog ([c0d21e9](c0d21e9f8d16c492464ab42cc1719082aa189f9d))
- Failed the release version... ([6cf98ac](6cf98ac26f4da4e7c47bc9873b905e0f8478c5e1))

### Refactor

- Create one single example binary instead of multiple equal ones ([9f444cd](9f444cde8c845ab911adc3060cda742e617a26ef))

## ibe-v0.0.7 - 2025-05-15

[a39d710](a39d710e6be162ec981b44c770114561466f2c2c)...[b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163)

### Bug Fixes

- Remove missused attribute ([e16e86c](e16e86c9120a3e399e19e16516d63b14d5d6eb92))

### Miscellaneous Tasks

- Release ([b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163))

## ibe-v0.0.6 - 2025-05-15

[571e7b3](571e7b3a8d19825c48ecf889910209e9b6db84c2)...[a39d710](a39d710e6be162ec981b44c770114561466f2c2c)

### Bug Fixes

- Apply changes for xdg ([e4a0ab6](e4a0ab68a5189b075860cf7f9842499f8efa0e18))

### Features

- Add graph circle ([aee713a](aee713aae7a87f9b1ea1a1aa34ef932f7e28bec9))
- Init config code ([92c5571](92c5571b9269584b7bf471bc2ce5b936ff2b9b16))

### Miscellaneous Tasks

- Release ([a39d710](a39d710e6be162ec981b44c770114561466f2c2c))

### Build

- Bump xdg from 2.5.2 to 3.0.0 ([50afcb9](50afcb98e78b624ee3240392e47bb09034917efb)), Signed-off-by:dependabot[bot] <support@github.com>

## ibe-v0.0.5 - 2025-05-09

[495fb38](495fb384cc55cf080e136d64abd0a9e09eacf118)...[571e7b3](571e7b3a8d19825c48ecf889910209e9b6db84c2)

### Miscellaneous Tasks

- Change default component to filled bars ([e4a79d1](e4a79d13b68d79d3a93de46c47b3991c2c3ff12a))
- First structure for graph ([22e1808](22e1808d2af20e53f878f5001bc95a8e3130193a))
- Update changelog ([ef5e2c0](ef5e2c05623720e8b2ab7c81e4208ffeafb8f155))
- Release ([571e7b3](571e7b3a8d19825c48ecf889910209e9b6db84c2))

## ibe-v0.0.4 - 2025-04-28

[c43a2d9](c43a2d99577703a2b833ce880079fa786fc8ccf9)...[495fb38](495fb384cc55cf080e136d64abd0a9e09eacf118)

### Bug Fixes

- Fix missing bracket in the default shader code ([4a67324](4a67324517204484e7beb1f620f60b49771abb32))

### Miscellaneous Tasks

- Update changelog ([8a9a214](8a9a21421f97096aa655b1d00eca9ba3ce4b47ef))
- Release ([495fb38](495fb384cc55cf080e136d64abd0a9e09eacf118))

## ibe-v0.0.3 - 2025-04-17

[2cd8024](2cd8024918f77b205e235b312cae56c64481291b)...[c43a2d9](c43a2d99577703a2b833ce880079fa786fc8ccf9)

### Bug Fixes

- Move `release.toml` to root of repository ([10e615d](10e615df0298f1d81764352f719987b0ebc93e8d))

### Miscellaneous Tasks

- Release ([c43a2d9](c43a2d99577703a2b833ce880079fa786fc8ccf9))

## ibe-v0.0.2 - 2025-04-17

[7696451](7696451247d7996f06d4a73528d1f440b816ae79)...[2cd8024](2cd8024918f77b205e235b312cae56c64481291b)

### Bug Fixes

- Remove gamma correction requirement for shaders ([c856ed9](c856ed9ad560078910d7b1cb7e448e250d6df832))

### Miscellaneous Tasks

- Release ([2cd8024](2cd8024918f77b205e235b312cae56c64481291b))

### Refactor

- Move release.toml ([9f73b36](9f73b36667a49d7c764f021ef832b3132cb545d7))

## ibe-v0.0.1 - 2025-04-10

### Bug Fixes

- Resolution is applied to component during hot reloading ([6c9077a](6c9077a0f08e1d60ef64243b15680d1081ba572e))

### Documentation

- Reformat description for `output_name` in the cli ([b740b34](b740b34a9f58a22910861e23e5fb9325297231db))

### Features

- Implement hot reloading ([d37d80a](d37d80ad35f26de06e0f439e3aa3e014eff7a59c))

### Miscellaneous Tasks

- Improve error handling for hot reloader ([a081788](a0817888392827f22364c16bd1006cd5edcdaa01))
- Prepare release ([7696451](7696451247d7996f06d4a73528d1f440b816ae79))

### Styling

- Make clippy happy ([b45cf02](b45cf02eeb04791d86be615687da16fc8bacb4aa))

### Testing

- Add test for external_paths() function of OutputConfig ([5ade3a7](5ade3a735d70f9f77565575404f40a2cdca6716f))

### Vibe

- Remove state ([7cef7f3](7cef7f39e7759d314f5715dd28f228917dae23d9))
- Adding file watcher for output changes ([17cba18](17cba18a31cb4c67a70acc53ed9696c0ac43cf7b))
- Implement registering of configs which got added/removed ([ddc6770](ddc67708dd672477d731605289dad1caaf7de974))
- Adding entry for amount of bars ([46bb5d2](46bb5d2674081b8190070452221122d1415289c0))
- Improve management for setting amount of bars ([e4cee32](e4cee32260859185fc3b44159793de0d2c0c15ca))

<!-- generated by git-cliff -->
