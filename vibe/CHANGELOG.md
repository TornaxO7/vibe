# Changelog

All notable changes to this project will be documented in this file.

## vibe-v2.3.0 - 2025-09-16

[8d5c03c](8d5c03c0c8b2229f0f106fb101ecee07d20bf897)...[a761509](a761509b6a3d94aca66d38030ac1ec41d1650e1d)

### Bug Fixes

- Allow `BarProcessor` to utilize the full max amount of bars ([4ff491c](4ff491c333ce24a7c53520f4165c52477a1e9190))
- Fix desktop interaction ([6120b9c](6120b9c2dbb5760c03ba32b0d317235b53374644))
- Re-added `rand` and fixed value noise ([2465506](246550650f2825c05423a92bddbbcfdc58aa48e3))
- Fill up screen again ([145fb14](145fb14f5b71d6724e8bc49b4a9122a864e3344a))

### Features

- Add custom placement ([b2062a1](b2062a17908721abd6cacc37ba44dfa3b3ee4fc1))
- Add format options ([ddf357c](ddf357cefe1b02f2f15acadc942525b6da3a8c62))
- Add white noise component ([3285736](328573615d6ecd3e10e535e953321d02503f0b23))
- Add format option ([c69f942](c69f9426bd413557852c0e553e5195ebc25fc4a9))

### Miscellaneous Tasks

- Update changelog ([4c7f49c](4c7f49c8999b9a0df0d346627e1339ceba1a1348))
- Update flake ([40aec48](40aec484357f318680b342791125470c19f94d2d))
- Release ([a761509](a761509b6a3d94aca66d38030ac1ec41d1650e1d))

### Refactor

- Make clippy happy ([52dbbe9](52dbbe97b410a2577e6fbd61674b9997396d421a))
- Set exclusize zone level to a funnier number :P ([8302a4c](8302a4c1ee35aec6d48d640b850848f0a8c037e1))
- Add white noise component ([2413e0e](2413e0ee26c03bf3ab396d8f4281a592faf0c250))
- Use 3 vertices for full screen components instead of 4 ([dcb6462](dcb6462f3c152ac53cec983ffd5c472d672fb7b6))
- Make clippy happy ([371e8de](371e8de059aadcc02cc8305c4ca66e5322176872))

### Styling

- Remove unused files ([94edf0b](94edf0be67a17cdbb9ebaee1f7ceaf8ad224a958))

### Build

- Bump tracing-subscriber from 0.3.19 to 0.3.20 ([4105d5a](4105d5ad15c0bbbb4fed4ed317d6a894a9595cfa)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump clap in the rust-dependencies group ([688943c](688943cc51b07a4f154ebe4cdf528e2c16a6e434)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump the rust-dependencies group with 2 updates ([1c22d37](1c22d37a215120ffdef393eabe6bdaf883c7c5d2)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v2.2.0 - 2025-08-27

[e79ef80](e79ef806cb4778dbfb4743cd898f8cb3fc382a58)...[8d5c03c](8d5c03c0c8b2229f0f106fb101ecee07d20bf897)

### Bug Fixes

- Reduce circle radius ([1c3b288](1c3b2887a57bf965a231ce1025691c1f5e424312))

### Features

- Add `height-gradient-variant` ([7f08b8d](7f08b8d2084fa5b35e9ff26f4256551d60658f63))

### Miscellaneous Tasks

- Update git cliff config file ([615f591](615f59192e92202c3bc37458b2439a6d758471ad))
- Release ([8d5c03c](8d5c03c0c8b2229f0f106fb101ecee07d20bf897))

### Refactor

- Implement (analytical) anti-aliasing ([892b193](892b19346b486ac20cb79bb0e40c70fb6643050a))

### Styling

- Rename directories ([35d5022](35d5022a20a5c05941314f056e0458b0f5be076a))

### Testing

- Add chessy test ([4f5c212](4f5c21294fe0f246cfe2eb60f910f7083bebb651))

### Build

- Bump thiserror in the rust-dependencies group ([09a27dd](09a27dd3d7f0eac83fa5603d03aff7fe2e677736)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v2.1.0 - 2025-08-23

[769ccfb](769ccfb48282c182b42d5a0e8b82d9a4ae34c6f6)...[e79ef80](e79ef806cb4778dbfb4743cd898f8cb3fc382a58)

### Features

- Init `chessy` component ([d0b1a2a](d0b1a2ad2c2ced7eb0ba954d8bfc05f87c3a4c57))

### Miscellaneous Tasks

- Set it to only one job... ([76c542c](76c542c502ae847c58129811db2dfef014c5c028))
- Release ([e79ef80](e79ef806cb4778dbfb4743cd898f8cb3fc382a58))

## vibe-v2.0.1 - 2025-08-22

[d6a0732](d6a0732ccbe9ede24a725dd2ca4257fb2f7b03c6)...[769ccfb](769ccfb48282c182b42d5a0e8b82d9a4ae34c6f6)

### Miscellaneous Tasks

- Update changelog ([6784352](678435297bfdab37b34e60581096f96ebf5300cb))
- Explain breaking changes ([70db931](70db9315ee402987036b779a87be51d0cfc3c222))
- Update changelogs ([940f56a](940f56a8cfb101f26855b7d858e995a9cec4f166))
- Release ([769ccfb](769ccfb48282c182b42d5a0e8b82d9a4ae34c6f6))

### Refactor

- Refactor!(audio): improve audio smoothness ([13a379f](13a379fe397d3835196db4cc26dd74917167c9d4))
- Cleanup ([d5faab6](d5faab6f742d90519e86c52f84ebbcc9bdf8178c))
- Remove double updating audio ([d0ffc29](d0ffc298fbd693d0eefa24af4597f5f6dc30b174))

### Build

- Bump shady version ([de44b6d](de44b6dca52395dac1a74331151d82a86b859ab2))
- Bump the rust-dependencies group with 3 updates ([3c5694c](3c5694c576dddf890fde04b4e1add90d7cb9f0e3)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump flake dependencies ([e0392c8](e0392c8649ae9325f3df6e3f9e828d79c49aef8d))

## vibe-v2.0.0 - 2025-08-15

[2722da3](2722da3f1311862bd23ee9f86efea1a99d942df8)...[d6a0732](d6a0732ccbe9ede24a725dd2ca4257fb2f7b03c6)

### Bug Fixes

- Typos ([b4a58cf](b4a58cfd3cff443494c6884da23e7d008cb2c7dd)), fix:rollback easier -> easer

### Features

- Fix test ci ([2b5cb89](2b5cb893aee632ec0b56609e1c06b4a14647d524))
- Let window title include output name ([cdaeca7](cdaeca77cc746d73e0e14f627cd1ab48bcd12b7f))

### Miscellaneous Tasks

- Group updates to one single PR ([796149d](796149d78d16068a02464b54555d10aa155d0bc7))
- Group dependencies into one single pr ([4ede679](4ede679c1e844922437e240df965f9a966cd0f4f))
- Release ([d6a0732](d6a0732ccbe9ede24a725dd2ca4257fb2f7b03c6))

### Refactor

- Reduce jobs ([8f714e8](8f714e816644f2ec160ac066fa8553de73ec710b))

### Build

- Bump clap from 4.5.40 to 4.5.41 ([5d1cd38](5d1cd38b4a13b62724fde4ced8e84d39e8ff0528)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump notify from 8.0.0 to 8.1.0 ([b1db162](b1db162305e61b6a96b8f6bf9cea76920de34b6a)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump toml from 0.8.23 to 0.9.2 ([2451dac](2451dac0c9127b17c01cca109d9f8c0f38d9b25c)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump rand from 0.9.1 to 0.9.2 ([50e31c3](50e31c3fb4b91650436fa1f2acd7ff697034813e)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump clap from 4.5.41 to 4.5.42 ([2046524](2046524b859e59d2df4339137a374abb598c23c5)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump wayland-client from 0.31.10 to 0.31.11 ([2163700](2163700ff2516d56c4b6efd751a1dcde12f56f5e)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump the rust-dependencies group with 8 updates ([12d6d54](12d6d5434f10e13b7ef0c5a04714ba3bc38648f7)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump the rust-dependencies group with 10 updates ([b2e0974](b2e0974a54fd1ebc53b751c77e4e314d66b02770)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v1.0.1 - 2025-06-13

[b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163)...[2722da3](2722da3f1311862bd23ee9f86efea1a99d942df8)

### Bug Fixes

- Fix bar `max_height` attribute ([103854f](103854f246dec36840ec585ce030696640e8c521))
- Use `push_error_scope` instead of custom testing ([a66546b](a66546b3c687616b14cd08087919a2165e46119b))

### Documentation

- Extend feature section ([e3ddc92](e3ddc9249d203907774d231dcdda5488ce740931))

### Features

- Add smoothness option ([68a1b40](68a1b40ac4e56c6c4e8a898996b0460b4c5c86fc))
- Add full reference config with test ([65809fe](65809fe105a88b526f1c2a8b7903cb3666add39d))
- Add custom positioning ([e896735](e89673564e11743562463d79b0c439a8456a8489))
- Add placement config ([f0d265f](f0d265f19bf78f402eb3cc346d7ff34e4761e60f))
- Add positioning ([d25ae5a](d25ae5a00161fde6b073dfcf024d858f8f781db7))
- Add position config ([8a8af3c](8a8af3c99d35b09ace499acae54e3d2ff5fc00de))
- Add command to list all output devices ([b41ac49](b41ac497ec670e16fcb3e7b3ce20851749c2c094))
- Add mirroring option ([41d948b](41d948b658150343e804772b4163b55faf0ab4fb))

### Miscellaneous Tasks

- Add first stuff for positioning ([f0874b7](f0874b7098ac56a6a1abcd47809519b002a313ae))
- Update `shady-audio` ([9e90222](9e9022219b760b4de4df6e4c3078e0221023bd4e))
- Update shady-audio ([cd8cc11](cd8cc112f0c39b13b14c704a2d3c5be66e1674ce))
- Update dep and migrate to it ([35b0962](35b09629a2a5e0233ad89c2744b08ba818c70a29))
- Add window title ([a386ce1](a386ce1b02654cd84669a328043a573a64db5b2b))
- Update changelog ([c0d21e9](c0d21e9f8d16c492464ab42cc1719082aa189f9d))
- Failed the release version... ([6cf98ac](6cf98ac26f4da4e7c47bc9873b905e0f8478c5e1))
- Release ([2722da3](2722da3f1311862bd23ee9f86efea1a99d942df8))

### Refactor

- Create one single example binary instead of multiple equal ones ([9f444cd](9f444cde8c845ab911adc3060cda742e617a26ef))

### Build

- Bump winit from 0.30.10 to 0.30.11 ([a79817a](a79817a9367e503c115f6cec666d505bc7522bc6)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump wgpu from 24.0.3 to 24.0.5 ([eceb8fb](eceb8fb3fef0cc25a688d51f8540b59970156b9b)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump clap from 4.5.38 to 4.5.39 ([53c903e](53c903e7e577a5272283390173ec1b13a52f5393)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump clap from 4.5.39 to 4.5.40 ([f08861f](f08861fa73e0ebb86c8ea97d4dd3c8402b2b14ca)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump bytemuck from 1.23.0 to 1.23.1 ([7904e6e](7904e6e4f9f84a85b5aee50bfeee9b996771de5d)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump toml from 0.8.22 to 0.8.23 ([c951f25](c951f25c80b891c64cfc4fa3fa89155735fa8761)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v0.0.7 - 2025-05-15

[a39d710](a39d710e6be162ec981b44c770114561466f2c2c)...[b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163)

### Bug Fixes

- Remove missused attribute ([e16e86c](e16e86c9120a3e399e19e16516d63b14d5d6eb92))
- Icnrease the circle width ([4473980](447398083a77a8f58ae31afd1fa0bd486d9eba9e))

### Features

- Make the cells more random ([fd182af](fd182afff72d4cbd4adfd6580cdee4ffae4c0035))

### Miscellaneous Tasks

- Update changelog ([0540d85](0540d85f79e906f1c90c9a75ae6554bfc3fc1791))
- Release ([b677c27](b677c27e43b6bab131abfb7fe5a7425a9a758163))

### Refactor

- Improve wgsl shadercode ([ca8f971](ca8f9714ae7e0855bc0502cfd905dad4400fad76))

## vibe-v0.0.6 - 2025-05-15

[571e7b3](571e7b3a8d19825c48ecf889910209e9b6db84c2)...[a39d710](a39d710e6be162ec981b44c770114561466f2c2c)

### Bug Fixes

- Apply changes for xdg ([e4a0ab6](e4a0ab68a5189b075860cf7f9842499f8efa0e18))

### Documentation

- Add link to config doc ([80e8dbe](80e8dbeeebb000c5f0b18c98f026abe0d9b42512))

### Features

- Add graph circle ([aee713a](aee713aae7a87f9b1ea1a1aa34ef932f7e28bec9))
- Init code ([f7cc896](f7cc89612256107dc27fcccfa80e95ecc3f9049f))
- Add color variant ([5fc94b7](5fc94b7c0258287875badb2167b308b0488c5a78))
- Init config code ([92c5571](92c5571b9269584b7bf471bc2ce5b936ff2b9b16))

### Miscellaneous Tasks

- Release ([a39d710](a39d710e6be162ec981b44c770114561466f2c2c))

### Testing

- Init test ([80a6667](80a6667fde80437bfbb9ff8a341c4ff388dabc1a))

### Build

- Bump clap from 4.5.37 to 4.5.38 ([a7ce81d](a7ce81d6d89913ee93002d53daf560f20ab25801)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump xdg from 2.5.2 to 3.0.0 ([50afcb9](50afcb98e78b624ee3240392e47bb09034917efb)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v0.0.5 - 2025-05-09

[495fb38](495fb384cc55cf080e136d64abd0a9e09eacf118)...[571e7b3](571e7b3a8d19825c48ecf889910209e9b6db84c2)

### Documentation

- Adding docs to the preambles ([d29d8be](d29d8be37a74d0ef946e7bbbec09ba27a7454f25))

### Miscellaneous Tasks

- Change default component to filled bars ([e4a79d1](e4a79d13b68d79d3a93de46c47b3991c2c3ff12a))
- First structure for graph ([22e1808](22e1808d2af20e53f878f5001bc95a8e3130193a))
- Update changelog ([ef5e2c0](ef5e2c05623720e8b2ab7c81e4208ffeafb8f155))
- Release ([571e7b3](571e7b3a8d19825c48ecf889910209e9b6db84c2))

### Build

- Bump wayland-client from 0.31.8 to 0.31.9 ([077e747](077e74738847e6b7e1e44977148366611e1d36b7)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump wayland-backend from 0.3.8 to 0.3.9 ([9a21d2d](9a21d2d2bdd4794a241325946056f372bcb40eaa)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump toml from 0.8.20 to 0.8.21 ([a3c0a16](a3c0a160f975458465e4373357fe8d2a04842319)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump wayland-backend from 0.3.9 to 0.3.10 ([40e1470](40e147081f2f17b9f26d5382be72f76559e50bf3)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump bytemuck from 1.22.0 to 1.23.0 ([d2dea63](d2dea63215a76fc206bb516fc08fcb707c3b0170)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump toml from 0.8.21 to 0.8.22 ([c308268](c308268d3448975ff48171dcc8db783cfd9dee4d)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump winit from 0.30.9 to 0.30.10 ([3a31258](3a31258409a4de5f3facd4d486a4768498f41de4)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v0.0.4 - 2025-04-28

[c43a2d9](c43a2d99577703a2b833ce880079fa786fc8ccf9)...[495fb38](495fb384cc55cf080e136d64abd0a9e09eacf118)

### Bug Fixes

- Fix missing bracket in the default shader code ([4a67324](4a67324517204484e7beb1f620f60b49771abb32))

### Documentation

- Add some docs ([ff81984](ff8198402fd3214567e5598500d8420e2da6891f))

### Features

- Add release-plz ([0d123b9](0d123b96933cf4e0ff5f92ce922fce6a085d6fbc))

### Miscellaneous Tasks

- Update README ([1080d06](1080d0638987a506dccc3e97213e29e1d8b1e42f))
- Update changelog ([8a9a214](8a9a21421f97096aa655b1d00eca9ba3ce4b47ef))
- Release ([495fb38](495fb384cc55cf080e136d64abd0a9e09eacf118))

### Build

- Bump clap from 4.5.36 to 4.5.37 ([3de8d4b](3de8d4b38d254b56d2bb16735a683719bf748c6d)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump rand from 0.9.0 to 0.9.1 ([7143b98](7143b98a53914b0aab0393f8e37db14798f0c16e)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v0.0.3 - 2025-04-17

[2cd8024](2cd8024918f77b205e235b312cae56c64481291b)...[c43a2d9](c43a2d99577703a2b833ce880079fa786fc8ccf9)

### Bug Fixes

- Move `release.toml` to root of repository ([10e615d](10e615df0298f1d81764352f719987b0ebc93e8d))

### Miscellaneous Tasks

- Release ([c43a2d9](c43a2d99577703a2b833ce880079fa786fc8ccf9))

## vibe-v0.0.2 - 2025-04-17

[7696451](7696451247d7996f06d4a73528d1f440b816ae79)...[2cd8024](2cd8024918f77b205e235b312cae56c64481291b)

### Bug Fixes

- Remove gamma correction requirement for shaders ([c856ed9](c856ed9ad560078910d7b1cb7e448e250d6df832))

### Documentation

- Fix naming ([078383b](078383b0f376d9e7b259ed49d35d600722f587ef))
- Add full steps for pavucontrol ([cdf5fe4](cdf5fe4bc8e96b64f9d5208c34f147b3521cef89))

### Miscellaneous Tasks

- Remove unused file ([32a2d89](32a2d891deb3e7df15bc81fde8cbcc4ece529056))
- Release ([2cd8024](2cd8024918f77b205e235b312cae56c64481291b))

### Refactor

- Move release.toml ([9f73b36](9f73b36667a49d7c764f021ef832b3132cb545d7))

### Build

- Bump anyhow from 1.0.97 to 1.0.98 ([f54098b](f54098b15f688d9ec8f75407a58c2a70daebf2b5)), Signed-off-by:dependabot[bot] <support@github.com>
- Bump clap from 4.5.35 to 4.5.36 ([390163e](390163e332d4f92f01e0e633e9d5cc9041cc140a)), Signed-off-by:dependabot[bot] <support@github.com>

## vibe-v0.0.1 - 2025-04-10

### Bug Fixes

- Fix warnings ([50a4a41](50a4a41939d649ef32f5c0526d9d80f8b865889f))
- Fix warnings ([7d16326](7d16326dcb6635621c2be81647202a318218e974))
- Fix packaging and devshell ([081ee2e](081ee2e963750c3695a4884ab4dfa603840127ea))
- Fix kde bug ([c1f2829](c1f282975c5026ba11360a1261193cb837ad3c73))
- Fix examples ([607446f](607446f326ed0d0e6b0985bb5524ecec64d96e4e))
- Fix nix package ([50d9583](50d958344fbf4b546af56a06daf7f65e40cb3c5f))
- Fix formatting of error message ([330e22e](330e22eb116c177432f75d68f174ab6c10608fd1))
- Resolution is applied to component during hot reloading ([6c9077a](6c9077a0f08e1d60ef64243b15680d1081ba572e))
- Rename package ([ad1a1af](ad1a1af961b38784142ef7f924cb3c67e535234d)), BREAKING CHANGE:Renaming package and program from `vibe-daemon` to `vibe`

### Documentation

- Update config file format ([852fc98](852fc98e7ca87279bdf1d58d58266b2d5a1440cd))
- Update CONFIG.md ([513e345](513e34545f750dc802bd9ece9817fe782906b01b))
- Reformat description for `output_name` in the cli ([b740b34](b740b34a9f58a22910861e23e5fb9325297231db))

### Features

- Allow multiple shaders to be used for an output ([b606177](b606177e0c7b4913eef5faf541966ad2d4b4244d))
- Add frequency-range option ([c674dd2](c674dd274e284d414ef30e417ca6a1064d9279a0))
- Add optional grahics option `gpu_name` ([045e88b](045e88bb5ae399031bbe8e1bfb16676f309a5b96))
- Add option to use path instead of inline code for fragment code ([4f0ea6f](4f0ea6f4b56f8104a5507bac554bd2b85ea06670))
- Implement hot reloading ([d37d80a](d37d80ad35f26de06e0f439e3aa3e014eff7a59c))

### Miscellaneous Tasks

- Remove cargo check since it's already catched from clippy ([62270cf](62270cfb4b259bca1707a449263f795cff5de46a))
- Cleanup ([bfc4856](bfc485644b8d7deeb8fadebb1ed40e63bb276f43))
- Cleanup ([0c28503](0c28503cbf228e164c87b8cb1671b927cb6520ef))
- Simplify component descriptors ([3e22b55](3e22b55040b812684324c987d9eed4fe56fd85d5))
- Only add the output name to get its config ([7a8c51a](7a8c51ae9ed72027096af623b3e9496044d18063))
- Improve error handling for hot reloader ([a081788](a0817888392827f22364c16bd1006cd5edcdaa01))
- Add `git-cliff` and `cargo-release` ([53059ff](53059ffb6a9e2e6f3431bbbe902eaf151021c5fc))
- Add bug and feature request issue templates ([28986a8](28986a866712c73289538b8922777ecd5aefc771))
- Prepare release ([7696451](7696451247d7996f06d4a73528d1f440b816ae79))

### README

- Update ([6f96bd7](6f96bd70f78aca98c922d199fd51169429cd5e45))
- Update state ([222d778](222d778024a6e3c39c6d0ab7e526db4ccd07878b))
- Fix link ([bd24e2c](bd24e2cf1d9258ac698066cd933c3bde4d96e95e))
- Update link to config description ([02fd666](02fd666469570df6721f38de9085db54eeaf2ea5))
- Update demo video ([66c1b2b](66c1b2bd3a466505fdd901864c50fe56c9374d79))

### Refactor

- Refactor output code ([9e67f0c](9e67f0c3e422d6bc0c6553e03c629f18d6d125be))

### Styling

- Make clippy happy ([b45cf02](b45cf02eeb04791d86be615687da16fc8bacb4aa))

### Testing

- Add test for external_paths() function of OutputConfig ([5ade3a7](5ade3a735d70f9f77565575404f40a2cdca6716f))

### Aurodio

- Change config to allow to configure the layers as you want ([8bab533](8bab533623867a3fc8c9dc958a872f4d811a938e))
- Increase variation for the random seeds ([c51fca9](c51fca95d7274fbdb6ff9d00d7cebe7a364a2cc9))

### Bar

- Add presence gradient ([e9f1480](e9f1480b25f87cde66afe626bbccb73998626908))

### Bars

- Add iTime to fragment shader ([42478e0](42478e09acab92f168cc418714e220daf27307f7))
- Add iResolution to fragment shader ([60ba45f](60ba45ffe5105f43bc5b4c36e546722f6cf4b1c1))
- Add binding to set the max height ([1d59524](1d5952458d6a69a43d9ed2a4c4fa07d6b5140b27))
- Implement solid color variant ([d8493a8](d8493a8f11053c362789283309134c162b05f25a))

### Config

- Remove default wgsl shader ([c7f7645](c7f7645dc4decde0755db5bd960af7f553937f6b))
- Add entry for frequency range ([53a6793](53a67934ff1398e620c284ea40c57c6eb2c8c12a))

### Daemon

- Some cleanup (removed some files) ([cf26fa8](cf26fa835c689ef3725a77aeb0baa7010af01e96))

### Kde

- Workaround regarding desktop interactivity ([525716d](525716d5e57273ecba55aa0677636f5c7b6ceaca))

### Output-config

- Adding option to disable output ([baab65d](baab65d4155161141bc291b4b2a613d0b72ec0d2))

### Output-ctx

- Replace shader structs with components ([ad9a496](ad9a4964fbd1773171ff39037b0394930403e19c))

### Renderer

- Create working example ([02b03e5](02b03e55e7b529a06bdf99e70b534ef5637cd2bc))
- Add fragment_canvas ([3caa4bd](3caa4bd92487601d12ba5a57cd5eb3edc615c97b))
- Change order of the buffers to match the previous state ([8848c84](8848c84dd0b11fc309542b7a0437854ca6c5f04d))
- Add option to fallback to software rendering ([7a1651a](7a1651a9a61df5ae6693162e8cfdbd73f9b48cf2))
- Optimize bind group index ([75f23bd](75f23bd5152375fe6eee6e50daa67073c2db4146)), see:https://toji.dev/webgpu-best-practices/bind-groups#group-indices-matte
- Remove requirement to include bindings and bind groups in fragment shader ([cc49ef8](cc49ef8e055e5ec12ad18eed7484a84aed255f8a))

### Vibe

- Remove state ([7cef7f3](7cef7f39e7759d314f5715dd28f228917dae23d9))
- Adding file watcher for output changes ([17cba18](17cba18a31cb4c67a70acc53ed9696c0ac43cf7b))
- Implement registering of configs which got added/removed ([ddc6770](ddc67708dd672477d731605289dad1caaf7de974))
- Adding entry for amount of bars ([46bb5d2](46bb5d2674081b8190070452221122d1415289c0))
- Improve management for setting amount of bars ([e4cee32](e4cee32260859185fc3b44159793de0d2c0c15ca))

### Vibe-daemon

- Create default config for output if possible with default shader ([b579c99](b579c9975cc7344e7d372e7f1d97dd1eb8b063c2))

<!-- generated by git-cliff -->
