let
  sources = import ./nix/sources.nix;
in
{ nixpkgs ? import <nixpkgs> }:
let
  pkgs = nixpkgs { };
  inherit (import sources.gitignore { inherit (pkgs) lib; }) gitignoreSource;
in
pkgs.rustPlatform.buildRustPackage {
  pname   = "nix-doc";
  version = "0.4.0";

  cargoSha256 = "1xhv72466yrv57nj2whmyh3km0bz19q9bhgsi1z476k3l18bq3vg";

  src = gitignoreSource ./.;

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    boost
    nix
  ];

  # meta = with nixpkgs.stdenv.lib; {
  #   description = "A source-based Nix documentation tool";
  #   homepage    = "https://github.com/lf-/nix-doc";
  #   license     = licenses.lgpl3Plus;
  #   platforms   = platforms.all;
  # };
}
