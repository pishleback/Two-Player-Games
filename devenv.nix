{ pkgs, ... }:

{
  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [ "wasm32-unknown-unknown" ];
  };

  packages = with pkgs; [
    # Rust/native build tooling
    clang
    llvmPackages_21.bintools
    gcc
    openssl
    pkg-config

    # WASM
    trunk

    # X11
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libxcb

    # Graphics / windowing
    libxkbcommon
    wayland
    wayland-protocols
    mesa
    libGL
    libglvnd
    vulkan-loader

    # bindgen / native dependencies
    glib
  ];

  env = {
    # Runtime shared libraries
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.xorg.libxcb
      pkgs.libxkbcommon
      pkgs.vulkan-loader
      pkgs.wayland
      pkgs.libGL
      pkgs.mesa
      pkgs.libglvnd
    ];

    # bindgen needs to be able to find libclang
    LIBCLANG_PATH =
      pkgs.lib.makeLibraryPath [
        pkgs.llvmPackages_latest.libclang.lib
      ];

    # Additional clang include paths required by bindgen
    BINDGEN_EXTRA_CLANG_ARGS =
      pkgs.lib.concatStringsSep " " [
        "-I${pkgs.glibc.dev}/include"
        "-I${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"
        "-I${pkgs.glib.dev}/include/glib-2.0"
        "-I${pkgs.glib.out}/lib/glib-2.0/include"
      ];
  };
}