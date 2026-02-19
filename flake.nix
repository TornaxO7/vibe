{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; }
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem = { self', lib, system, pkgs, config, ... }: {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;

            overlays = with inputs; [
              rust-overlay.overlays.default

              (final: prev:
                let
                  version = "1.4.343.0";
                in
                {
                  vulkan-loader = prev.vulkan-loader.overrideAttrs (old: {
                    inherit version;
                    src = prev.fetchFromGitHub {
                      owner = "charles-lunarg";
                      repo = "Vulkan-Loader";
                      rev = "5b0ed940054d35d2b07840b02c1b8e0c2bf2e5cc";
                      hash = "sha256-AVd0Q2axESlFVShjNFUFoc1fQfvHP3/QfNtbD92jujg=";
                    };
                  });

                  vulkan-headers = prev.vulkan-headers.overrideAttrs (old: {
                    inherit version;
                    src = prev.fetchFromGitHub {
                      owner = "KhronosGroup";
                      repo = "Vulkan-Headers";
                      rev = "49f1a381e2aec33ef32adf4a377b5a39ec016ec4";
                      hash = "sha256-K/8X9D7B0o6+osOzScplwea+OsfM3srAtDKCgfZpcJU=";
                    };
                  });

                  # vulkan-validation-layers = prev.vulkan-validation-layers.overrideAttrs (old: {
                  #   inherit version;
                  #   src = pkgs.fetchFromGitHub {
                  #     owner = "KhronosGroup";
                  #     repo = "Vulkan-ValidationLayers";
                  #     rev = "11440fc0664718ac51646c63d6e321f61195e808";
                  #     hash = "sha256-iAOUwTAU8VdrMNDYlPHWqPKtzDZOHRxNq4nsEmDbsug=";
                  #   };

                  #   cmakeFlags = old.cmakeFlags ++ [ "-DUPDATE_DEPS=false" ];
                  # });
                })
            ];
          };

          packages = rec {
            default = vibe;
            vibe = pkgs.callPackage (import ./nix/vibe-package.nix) { };
          };

          devShells =
            let
              rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in
            {
              default =
                let
                  vibe = pkgs.callPackage (import ./nix/vibe-package.nix) { };
                in
                pkgs.mkShell {
                  packages = with pkgs; [
                    cargo-flamegraph
                    cargo-release
                    git-cliff
                  ] ++ [ rust-toolchain ];

                  buildInputs = vibe.buildInputs;
                  nativeBuildInputs = vibe.nativeBuildInputs;

                  LD_LIBRARY_PATH = vibe.LD_LIBRARY_PATH;
                };
            };
        };
      };
}
