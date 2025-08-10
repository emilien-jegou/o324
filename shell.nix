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
      if ! rustup toolchain list | grep default | grep -q nightly; then
        rustup default nightly
      fi;
      [ ! -f .packages/bin/cargo-expand ] && cargo install cargo-expand --root .packages/
      [ ! -f .packages/bin/cargo-tauri ] && cargo install tauri-cli --root .packages/
      [ ! -f .packages/bin/bacon ] && cargo install bacon --locked --root .packages/
      [ ! -f .packages/bin/cargo-watch ] && cargo install cargo-watch --root .packages/

      if [ ! -f .packages/bin/versio ]; then
        echo "Building versio from source..."
        build_dir=$(mktemp -d -t versio-build-XXXXXX)
        current_path=$(pwd)
        git clone https://github.com/emilien-jegou/versio.git $build_dir/versio
        cd $build_dir/versio && cargo build --release --bin versio
        cp target/release/versio $current_path/.packages/bin
        cd $current_path
        rm -rf $build_dir
      fi

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
