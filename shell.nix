{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
      ## Tauri dependencies
      webkitgtk_4_1
      librsvg
      stdenv.cc.cc.lib
      gnumake
      cmake
      jdk17
      llvmPackages.libcxx # libc++.so
      python310Packages.libxml2 # libxml2.2.so
      libxml2
      libappindicator-gtk3

      ## Rust build dependencies
      rustup
      gcc
      openssl
      pkg-config

      ## Utilities
      zx
      just

      ## Versio
      gpgme
      gnupg
      libgpg-error

      ## GUI
      # We only install packages needed for local development
      libsoup
      webkitgtk
      wget
      nodejs_18
      nodePackages.typescript-language-server
   ];

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

      export PATH="$PATH:$(pwd)/.packages/bin/:$(pwd)/bin/";

      export LD_LIBRARY_PATH=${pkgs.libappindicator-gtk3}/lib:$LD_LIBRARY_PATH

      ## Linux development
      # Without this the ui may not display properly, see issue:
      # https://github.com/NixOS/nixpkgs/issues/32580
      export WEBKIT_DISABLE_COMPOSITING_MODE=1

      ## !! You may want to run this command aswell !!
      # rustup component add rust-analyzer --toolchain nightly
    '';
}
