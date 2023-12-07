{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = [
      pkgs.gcc
      pkgs.openssl
      pkgs.pkg-config
      pkgs.just
      pkgs.rustup
   ];

    NIX_ENFORCE_PURITY = false;
    shellHook =
    ''
      if ! rustup toolchain list | grep default | grep -q beta; then
        rustup default beta
      fi;
      [ ! -f .packages/bin/cargo-expand ] && cargo install cargo-expand --root .packages/
      [ ! -f .packages/bin/cargo-watch ] && cargo install cargo-watch --root .packages/
      export PATH="$PATH:$(pwd)/.packages/bin/:$(pwd)/bin/";
    '';
}
