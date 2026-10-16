{
  lib,
  system,
  makeRustPlatform,
  pkg-config,
  systemd,
  openssl,
  cmake,
  clang,
  llvmPackages,
  fontconfig,
  vulkan-loader,
  libglvnd,
  libxkbcommon,
  xorg,
  wayland,
  wayland-protocols,
  wayland-scanner,
  alsa-lib,
  libjack2,
  pipewire,
  qt6
}:
let
  rustToolchain = "nightly-2026-07-19";
  fenix = import (fetchTarball {
  	url = "https://github.com/nix-community/fenix/archive/2013c981829bd5e93db06749b5639450c81bece3.tar.gz";
   	sha256 = "sha256-ZNw7FDQm2PVOIyE9WaLFgK1QYw8vrOchxMYHzDDxlUY=";
  }) { inherit system; };
  fenixToolchain = fenix.fromToolchainName { name = rustToolchain; sha256 = "sha256-ziDE7hJrDjZINklZE8gmLwXjmCa2B1aR2Z97NUcA77Y="; };
  rustPlatform = makeRustPlatform {
  	rustc = fenixToolchain.rustc;
    cargo = fenixToolchain.cargo;
  };
  qtEnv = with qt6; env "qt-custom-${qtbase.version}" [
    qtdeclarative
    qtwayland
    qtshadertools
    qtsvg
    qtvirtualkeyboard
  ];
in
rustPlatform.buildRustPackage {
  pname = "slipstream-embedded";
  version = "1.0.0";
  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "rekordcrate-0.3.0" = "sha256-bOcjGdotVeLDn/0kqlLOR1t5lABuNk401Ky/oP2B4yI=";
    };
  };

  buildInputs = [
    systemd
    openssl

    fontconfig
    vulkan-loader
    libglvnd

    libxkbcommon
    xorg.libxcb
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libXxf86vm

    wayland
    wayland-protocols
    wayland-scanner

    alsa-lib
    libjack2
    pipewire
    pipewire.jack

    qtEnv
  ];

  nativeBuildInputs = [
    pkg-config
    cmake

    clang
    llvmPackages.bintools

    qtEnv
  ];

  RUSTC_VERSION = rustToolchain;

  LIBCLANG_PATH = lib.makeLibraryPath [llvmPackages.libclang.lib];

  meta = {
    description = "DJ Deck Application for Embedded Systems";
    homepage = "https://github.com/fizzyapple12/djdeck";
    maintainers = with lib.maintainers; [ fizzyapple12 ];
    mainProgram = "slipstream-embedded";
    platforms = lib.platforms.all;
  };
}
