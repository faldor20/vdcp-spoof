# nix/rust.nix
{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "nightly";
  date = "2021-02-23";
  targets = [ "x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu" ];
  chan = pkgs.rustChannelOfTargets channel date targets;
in chan