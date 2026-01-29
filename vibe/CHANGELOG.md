# Changelog

All notable changes to this project will be documented in this file.

## 2.5.0 - 2026-01-29

[e279be1](e279be1b21f8cc07b55acbac2e23c79177b253d8)...[333b9d0](333b9d0884ec4bfa26d4f8798e2d155555e1f4a3)

### Features

- Add texture support for glsl and wgsl ([3a1e696](3a1e696ec3eb0b717e593004abe36c0017866dc6))
- Improve error message for missing texture path option ([570d07e](570d07eb56edb13a31f77f1f24d90d434b656369))

### Refactor

- Setting layer back to "Background" ([8888232](88882328fbbdaa2bef1478e9ee0fb995f246602a))
- Move `texture_path` to own `texture.path` option ([21aaf76](21aaf76f30c5710788dd3c77c1d1750a44b7441a))

### Testing

- Add tests with and without images ([56d3f10](56d3f1048f99467d43852106aca946035148fb7d))
- Add more tests ([06727fd](06727fd01cb487df4c114d801f7a835fa142a8bb))
- Add texture_path to reference-config ([55d9f1e](55d9f1ecc4e0e7c49e647fe87c58c12eb8f1c94f))

### Build

- Trying out `cargo-dist` ([333b9d0](333b9d0884ec4bfa26d4f8798e2d155555e1f4a3))

## ibe-v2.4.0 - 2026-01-02

[a761509](a761509b6a3d94aca66d38030ac1ec41d1650e1d)...[e279be1](e279be1b21f8cc07b55acbac2e23c79177b253d8)

### Bug Fixes

- Implement aspect ratio correction for rotations ([ddb046a](ddb046a055892b55774265ff7aca148da4c2a817))
- Comment out unused function ([45b568d](45b568dd83612d529fdcbedc1666538a535bbe79))

### Documentation

- Add description for `iMouse` ([588e9ec](588e9ecca69932ecf449d9dd14f605f611a8c897))

### Features

- Add option for y-mirroring ([16fbe1e](16fbe1ec08f3f7dbb4c239c8f99319e54d28f630))
- Add mouse support ([d46cbe5](d46cbe5dc6e2b6c18989641abbb57fe0deede7c7))
- Add `pulse_edges` component ([3c0fb4d](3c0fb4d3b877fa44dcd64bdac3dde455dda6a295))
- Add light sources component ([c170d0e](c170d0eb792c51547a60c63ce50f9313786d59bb))
- Add default component option ([de0dbc1](de0dbc19ac9a63df07b77987168a145f89ec70f8))

### Miscellaneous Tasks

- Update changelog ([0341a27](0341a27dc819bc0d2185215cf0b26ddfd49baab1))
- Change sensitivity of default component ([285d0da](285d0daadefa9589f64091dff06835412fcf6531))
- Release ([e279be1](e279be1b21f8cc07b55acbac2e23c79177b253d8))

### Refactor

- Set the default audio channels to stereo ([84e54db](84e54dbfb6cd3baf93183b8674c7871bd24b7f41))
- Move `height_mirrored` to BarsPlacement::Custom ([1b421c5](1b421c5e27076f853deb0d96b889cd3b04994faa))
- Move height_mirrored to BarsPlacement::Custom and add gradients ([327b0ba](327b0baf31b1f778d749b9102de1c82764e57ef3))
- Port over to texture generation ([768e4c9](768e4c9fb1f4a1d47782e6c64cb62624d7fd5acf))
- Replace `high_...` and `low_` with `thresholds` ([75c9986](75c99863a17c45a8906a631fad5ee5489a1005b5))
- Add `gaussian_blur` attribute set ([bea5010](bea50105a511c2433b1113d6a3a25ea5f7260c1e))

### Testing

- Add bars gradient configs to reference config ([b74f797](b74f797ead06bd657d7f3a29d4e0398efcf175d9))

### Build

- Disable unused features for better compile times ([48b52c5](48b52c5ef883eb3a2cd7a279e512fa2da992658f))

## ibe-v2.3.0 - 2025-09-16

[8d5c03c](8d5c03c0c8b2229f0f106fb101ecee07d20bf897)...[a761509](a761509b6a3d94aca66d38030ac1ec41d1650e1d)

### Bug Fixes

- Fix desktop interaction ([6120b9c](6120b9c2dbb5760c03ba32b0d317235b53374644))
- Re-added `rand` and fixed value noise ([2465506](246550650f2825c05423a92bddbbcfdc58aa48e3))
- Fill up screen again ([145fb14](145fb14f5b71d6724e8bc49b4a9122a864e3344a))

### Features

- Add custom placement ([b2062a1](b2062a17908721abd6cacc37ba44dfa3b3ee4fc1))
- Add format options ([ddf357c](ddf357cefe1b02f2f15acadc942525b6da3a8c62))
- Add format option ([c69f942](c69f9426bd413557852c0e553e5195ebc25fc4a9))

### Miscellaneous Tasks

- Update changelog ([4c7f49c](4c7f49c8999b9a0df0d346627e1339ceba1a1348))
- Release ([a761509](a761509b6a3d94aca66d38030ac1ec41d1650e1d))

### Refactor

- Set exclusize zone level to a funnier number :P ([8302a4c](8302a4c1ee35aec6d48d640b850848f0a8c037e1))

## ibe-v2.2.0 - 2025-08-27

[e79ef80](e79ef806cb4778dbfb4743cd898f8cb3fc382a58)...[8d5c03c](8d5c03c0c8b2229f0f106fb101ecee07d20bf897)

### Features

- Add `height-gradient-variant` ([7f08b8d](7f08b8d2084fa5b35e9ff26f4256551d60658f63))

### Miscellaneous Tasks

- Update git cliff config file ([615f591](615f59192e92202c3bc37458b2439a6d758471ad))
- Release ([8d5c03c](8d5c03c0c8b2229f0f106fb101ecee07d20bf897))

## ibe-v2.1.0 - 2025-08-23

[769ccfb](769ccfb48282c182b42d5a0e8b82d9a4ae34c6f6)...[e79ef80](e79ef806cb4778dbfb4743cd898f8cb3fc382a58)

### Features

- Init `chessy` component ([d0b1a2a](d0b1a2ad2c2ced7eb0ba954d8bfc05f87c3a4c57))

### Miscellaneous Tasks

- Release ([e79ef80](e79ef806cb4778dbfb4743cd898f8cb3fc382a58))

## ibe-v2.0.1 - 2025-08-22

[d6a0732](d6a0732ccbe9ede24a725dd2ca4257fb2f7b03c6)...[769ccfb](769ccfb48282c182b42d5a0e8b82d9a4ae34c6f6)

### Miscellaneous Tasks

- Update changelog ([6784352](678435297bfdab37b34e60581096f96ebf5300cb))
- Explain breaking changes ([70db931](70db9315ee402987036b779a87be51d0cfc3c222))
- Update changelogs ([940f56a](940f56a8cfb101f26855b7d858e995a9cec4f166))
- Release ([769ccfb](769ccfb48282c182b42d5a0e8b82d9a4ae34c6f6))

### Refactor

- Refactor!(audio): improve audio smoothness ([13a379f](13a379fe397d3835196db4cc26dd74917167c9d4))
- Remove double updating audio ([d0ffc29](d0ffc298fbd693d0eefa24af4597f5f6dc30b174))

### Build

- Bump shady version ([de44b6d](de44b6dca52395dac1a74331151d82a86b859ab2))

## ibe-v2.0.0 - 2025-08-15

[2722da3](2722da3f1311862bd23ee9f86efea1a99d942df8)...[d6a0732](d6a0732ccbe9ede24a725dd2ca4257fb2f7b03c6)

### Bug Fixes

- Typos ([b4a58cf](b4a58cfd3cff443494c6884da23e7d008cb2c7dd)), fix:rollback easier -> easer

### Features

- Let window title include output name ([cdaeca7](cdaeca77cc746d73e0e14f627cd1ab48bcd12b7f))

### Miscellaneous Tasks

- Release ([d6a0732](d6a0732ccbe9ede24a725dd2ca4257fb2f7b03c6))

### Build

- Bump toml from 0.8.23 to 0.9.2 ([2451dac](2451dac0c9127b17c01cca109d9f8c0f38d9b25c)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump the rust-dependencies group with 8 updates ([12d6d54](12d6d5434f10e13b7ef0c5a04714ba3bc38648f7)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump the rust-dependencies group with 10 updates ([b2e0974](b2e0974a54fd1ebc53b751c77e4e314d66b02770)), Signed-off-by:dependabot[bot] <support@github.com>

## ibe-v1.0.1 - 2025-06-13

[b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163)...[2722da3](2722da3f1311862bd23ee9f86efea1a99d942df8)

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
- Release ([2722da3](2722da3f1311862bd23ee9f86efea1a99d942df8))

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
