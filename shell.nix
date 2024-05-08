{ pkgs ? import <nixpkgs> {} }:

let
  androidEnv = pkgs.androidenv.composeAndroidPackages {
    includeEmulator = true;  # Set to true if you need to use the emulator
    includeSources = false;
    includeSystemImages = true;
    includeNDK = true;
    useGoogleAPIs = false;
    useGoogleTVAddOns = false;
    platformVersions = [ "29" "33" ];
    abiVersions = [ "x86_64" ];
    systemImageTypes = [ "google_apis_playstore" ];
    buildToolsVersions = [ "30.0.3" ];
    includeExtras = [];
  };
in pkgs.mkShell {
    buildInputs = [
      androidEnv.androidsdk
    ];

    nativeBuildInputs = [
      ## Tauri dependencies
      pkgs.webkitgtk_4_1
      pkgs.librsvg
      pkgs.stdenv.cc.cc.lib
      pkgs.gnumake
      pkgs.cmake
      pkgs.jdk17
      pkgs.llvmPackages.libcxx # libc++.so
      pkgs.python310Packages.libxml2 # libxml2.2.so
      pkgs.libxml2

      ## Rust build dependencies
      pkgs.rustup
      pkgs.gcc
      pkgs.openssl
      pkgs.pkg-config

      ## Utilities
      pkgs.zx
      pkgs.just

      ## Versio
      pkgs.gpgme
      pkgs.gnupg
      pkgs.libgpg-error

      ## GUI
      # We only install packages needed for local development
      pkgs.libsoup
      pkgs.webkitgtk
      pkgs.wget
      pkgs.nodejs_18
      pkgs.nodePackages.typescript-language-server
   ];
    # TODO
    # This is a good example on how to build rust projects in nix
    # https://github.com/NixOS/nixpkgs/blob/nixos-23.11/pkgs/applications/networking/feedreaders/newsflash/default.nix

    NIX_ENFORCE_PURITY = false;
    shellHook =
    ''
      if ! rustup toolchain list | grep default | grep -q nightly; then
        rustup default nightly
      fi;
      [ ! -f .packages/bin/cargo-expand ] && cargo install cargo-expand --root .packages/
      [ ! -f .packages/bin/cargo-tauri ] && cargo install tauri-cli --root .packages/
      [ ! -f .packages/bin/bacon ] && cargo install bacon --locked --root .packages/
      [ ! -f .packages/bin/cargo-watch ] && cargo install cargo-watch --root .packages/
      [ ! -f .packages/bin/versio ] && cargo install versio --root .packages/

      # rustup component add rust-analyzer-preview --toolchain nightly

      export PATH="$PATH:$(pwd)/.packages/bin/:$(pwd)/bin/";

      ## Android development
      export LD_LIBRARY_PATH="${pkgs.python310Packages.libxml2.out}/lib:${pkgs.llvmPackages.libcxx}/lib:$LD_LIBRARY_PATH"
      export JAVA_HOME="${pkgs.jdk17}"
      export ANDROID_HOME=${androidEnv.androidsdk}/libexec/android-sdk
      export ANDROID_SDK_ROOT=$ANDROID_HOME
      export NDK_HOME="${androidEnv.androidsdk}/libexec/android-sdk/ndk/$(ls ${androidEnv.androidsdk}/libexec/android-sdk/ndk/ | head -n 1)"

      # patchelf --set-interpreter /nix/store/ddwyrxif62r8n6xclvskjyy6szdhvj60-glibc-2.39-5/lib/ld-linux-x86-64.so.2 --set-rpath $(echo /nix/store/*-glibc-2.39-5/lib | tr ' ' ':') /home/emilien/.gradle/caches/transforms-3/b513380069e1d9d23b85e25896d2a7a2/transformed/aapt2-8.0.0-9289358-linux/aapt2

      # avdmanager create avd -n avd_name -k "system-images;android-29;google_apis_playstore;x86_64" --device "pixel_xl"

      ## Linux development
      # Without this the ui may not display properly, see issue:
      # https://github.com/NixOS/nixpkgs/issues/32580
      export WEBKIT_DISABLE_COMPOSITING_MODE=1
    '';
}
