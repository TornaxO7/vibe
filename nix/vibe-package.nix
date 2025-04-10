{ rustPlatform
, lib
, pkg-config

, alsa-lib

, libGL
, libxkbcommon
, wayland

, mesa
, vulkan-loader
, vulkan-validation-layers
, vulkan-tools
, makeWrapper
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../vibe/Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = builtins.path {
    path = ../.;
  };

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = [
    alsa-lib

    wayland

    libGL
    libxkbcommon

    vulkan-loader
    vulkan-validation-layers
    vulkan-tools
  ];

  doCheck = false;

  postInstall = ''
    wrapProgram $out/bin/$pname --prefix LD_LIBRARY_PATH : ${builtins.toString (lib.makeLibraryPath [
      # Without wayland in library path, this warning is raised:
      # "No windowing system present. Using surfaceless platform"
      wayland
      # Without vulkan-loader present, wgpu won't find any adapter
      vulkan-loader
      mesa
    ])}
  '';

  LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${lib.makeLibraryPath buildInputs}";

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = lib.licenses.gpl3;
    mainProgram = pname;
  };
}
