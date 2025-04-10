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
                  ] ++ [ rust-toolchain ];

                  buildInputs = vibe.buildInputs;
                  nativeBuildInputs = vibe.nativeBuildInputs;

                  LD_LIBRARY_PATH = vibe.LD_LIBRARY_PATH;
                };
            };
        };
      };
}
