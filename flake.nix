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
            default = vibe-daemon;
            vibe-daemon = pkgs.callPackage (import ./nix/vibe-daemon-package.nix) { };
          };

          devShells.default =
            let
              rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
              dependencies = with pkgs; [
                mold
                clang
              ];
            in
            pkgs.mkShell rec {
              packages = with pkgs; [
                # for manual communiaction with the daemon socket
                netcat
              ] ++ [ rust-toolchain ];

              buildInputs = self'.packages.vibe-daemon.buildInputs ++ dependencies;

              shellHook = ''
                export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${lib.makeLibraryPath buildInputs}
              '';
            };
        };
      };
}
