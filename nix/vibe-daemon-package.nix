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
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = builtins.path {
    path = ../.;
  };

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [
    alsa-lib

    wayland

    libGL
    libxkbcommon

    vulkan-loader
    vulkan-validation-layers
    vulkan-tools
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = lib.licenses.gpl3;
    mainProgram = pname;
  };
}
