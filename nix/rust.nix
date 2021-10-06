# nix/rust.nix
{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.rust-overlay) ]; };
 
  chan = pkgs.rust-bin.nightly."2021-09-06".rust.override{ 
  extensions = [ "rust-src" "rust-analysis" ];
  targets = [ "x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu" ];};
in chan
