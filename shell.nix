{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
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

      ## Linux development
      # Without this the ui may not display properly, see issue:
      # https://github.com/NixOS/nixpkgs/issues/32580
      export WEBKIT_DISABLE_COMPOSITING_MODE=1

      ## !! You may want to run this command aswell !!
      # rustup component add rust-analyzer --toolchain nightly
    '';
}
