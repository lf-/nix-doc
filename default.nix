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
  version = "0.5.2";

  cargoSha256 = "sha256-RxsH4bSAzBslK8MVGmCJxduf6MYOtQEKxt9QjgUCg1o=";

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
