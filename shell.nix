{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = [
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
      [ ! -f .packages/bin/cargo-watch ] && cargo install cargo-watch --root .packages/
      [ ! -f .packages/bin/versio ] && cargo install versio --root .packages/
      export PATH="$PATH:$(pwd)/.packages/bin/:$(pwd)/bin/";

      # Without this the ui may not display properly, see issue:
      # https://github.com/NixOS/nixpkgs/issues/32580
      export WEBKIT_DISABLE_COMPOSITING_MODE=1
    '';
}
