{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = [
      # Rust build dependencies
      pkgs.rustup
      pkgs.gcc
      pkgs.openssl
      pkgs.pkg-config

      # Utilities
      pkgs.zx
      pkgs.just

      # Versio
      pkgs.gpgme
      pkgs.gnupg
      pkgs.libgpg-error

   ];

    NIX_ENFORCE_PURITY = false;
    shellHook =
    ''
      if ! rustup toolchain list | grep default | grep -q beta; then
        rustup default beta
      fi;
      [ ! -f .packages/bin/cargo-expand ] && cargo install cargo-expand --root .packages/
      [ ! -f .packages/bin/tauri-cli ] && cargo install tauri-cli --root .packages/
      [ ! -f .packages/bin/cargo-watch ] && cargo install cargo-watch --root .packages/
      [ ! -f .packages/bin/versio ] && cargo install versio --root .packages/
      export PATH="$PATH:$(pwd)/.packages/bin/:$(pwd)/bin/";
    '';
}
