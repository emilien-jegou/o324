{ pkgs, ... }:

let
  # Jist doc at: https://github.com/emilien-jegou/jist
  jistSrc = builtins.fetchTarball {
    url = "https://github.com/emilien-jegou/jist/archive/refs/tags/v0.0.2.tar.gz";
    sha256 = "1j0jpqwv1smrx95pl210rm8ylykfr964b46w2pvdnpdi9s46lvmy";
  };
  jist = import "${jistSrc}/lib/make-cli.nix" { inherit pkgs; };

  dev-cli = jist.cli (import ./make.nix);
in {
  packages = [
    pkgs.bacon
    dev-cli
  ];

  enterShell = "dev";

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "rust-src" "rustfmt" "rust-analyzer" "clippy" ];
    targets = [ "wasm32-unknown-unknown" "x86_64-unknown-linux-gnu" ];
  };

  dotenv.enable = true;
}
