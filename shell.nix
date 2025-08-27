{ pkgs ? import (fetchTarball {
    url = "https://channels.nixos.org/nixos-25.05/nixexprs.tar.xz";
  }) {} }:

let
  rust-overlay = import (builtins.fetchTarball {
    url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  });

  pkgs = import <nixpkgs> {
    overlays = [ rust-overlay ];
  };
in
pkgs.mkShell {
    buildInputs = with pkgs; [
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
      gcc
      openssl
      wmctrl
      pkg-config
      (pkgs.rust-bin.nightly."2025-08-26".default.override {
        extensions = ["rust-src" "rustfmt" "rust-analyzer" "clippy"];
        targets = ["wasm32-unknown-unknown" "x86_64-unknown-linux-gnu" ];
      })

      ## Utilities
      zx
      just
      wget
      elixir
      qemu
      buildPackages.gcc
      elixir-ls

      ## Versio
      #gpgme
      #gnupg
      #libgpg-error

      ## GUI
      # We only install packages needed for local development
      #libsoup
      #webkitgtk
      wget
      nodejs_24
      nodePackages.typescript-language-server
      vscode-langservers-extracted
   ];

    NIX_ENFORCE_PURITY = false;

    shellHook =
    ''
      #[ ! -f .packages/bin/cargo-tauri ] && cargo install tauri-cli --root .packages/
      [ ! -f .packages/bin/cargo-expand ] && cargo install cargo-expand --root .packages/
      [ ! -f .packages/bin/bacon ] && cargo install bacon --locked --root .packages/
      [ ! -f .packages/bin/cargo-watch ] && cargo install cargo-watch --root .packages/

      [ ! -f .packages/bin/cargo-audit ] && cargo install cargo-audit --root .packages/
      [ ! -f .packages/bin/cargo-deny ] && cargo install cargo-deny --root .packages/
      [ ! -f .packages/bin/cargo-udeps ] && cargo install cargo-udeps --root .packages/
      [ ! -f .packages/bin/cargo-outdated ] && cargo install cargo-outdated --root .packages/

      #if [ ! -f .packages/bin/versio ]; then
      #  echo "Building versio from source..."
      #  build_dir=$(mktemp -d -t versio-build-XXXXXX)
      #  current_path=$(pwd)
      #  git clone https://github.com/emilien-jegou/versio.git $build_dir/versio
      #  cd $build_dir/versio && cargo build --release --bin versio
      #  cp target/release/versio $current_path/.packages/bin
      #  cd $current_path
      #  rm -rf $build_dir
      #fi

      export PATH="$PATH:$(pwd)/.packages/bin/:$(pwd)/bin/";
      export LD_LIBRARY_PATH=${pkgs.libappindicator-gtk3}/lib:$LD_LIBRARY_PATH
      export VM_ISO_OUT_PATH="$(pwd)/.packages/iso/"

      # Without this the ui may not display properly, see issue:
      # https://github.com/NixOS/nixpkgs/issues/32580
      # export WEBKIT_DISABLE_COMPOSITING_MODE=1

      # Use this to override configurations
      [ -f .localrc ] && source .localrc
    '';
}
