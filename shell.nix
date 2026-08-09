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
      ## Rust build dependencies
      gcc
      openssl
      wmctrl
      pkg-config
      (pkgs.rust-bin.nightly."2026-08-08".default.override {
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
