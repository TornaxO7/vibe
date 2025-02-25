{ rustPlatform
, lib
, pkg-config

, alsa-lib

, libGL
, libxkbcommon
, wayland

, vulkan-loader
, vulkan-validation-layers
, vulkan-tools
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);

  dependencies = [
    pkg-config
    alsa-lib

    wayland

    libGL
    libxkbcommon

    vulkan-loader
    vulkan-validation-layers
    vulkan-tools
  ];
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = builtins.path {
    path = ../.;
  };

  buildInputs = dependencies;
  nativeBuildInputs = dependencies;

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = lib.licenses.gpl3;
    mainProgram = pname;
  };
}
