let
  sources = import ./nix/sources.nix;
in
{ nixpkgs ? import <nixpkgs> }:
let
  pkgs = nixpkgs {
    overlays = [
      (import sources.nixpkgs-mozilla)
      (self: super:
        {
          rustc = self.latest.rustChannels.stable.rust;
          cargo = self.latest.rustChannels.stable.rust;
        }
      )
    ];
  };
  inherit (import sources.gitignore { inherit (pkgs) lib; }) gitignoreSource;
  naersk = pkgs.callPackage sources.naersk {};
in
naersk.buildPackage {
  name   = "nix-doc";
  version = "0.3.3";

  src = gitignoreSource ./.;

  # I am about to commit a great crime against the rust stability policy:
  # https://github.com/rust-lang/cargo/issues/6790
  # has been unresolved for a long time and I don't want to use a nightly
  # compiler
  cargoBuild = def: "RUSTC_BOOTSTRAP=1 " + def;
  # Poof, a squadron of angry Ferrises have been dispatched to my location and
  # are about to attack me with their cute little clawbs

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    boost
    nix
  ];

  targets = [
    "nix-doc"
    "plugin"
  ];

  copyLibs = true;

  # meta = with nixpkgs.stdenv.lib; {
  #   description = "A source-based Nix documentation tool";
  #   homepage    = "https://github.com/lf-/nix-doc";
  #   license     = licenses.lgpl3Plus;
  #   platforms   = platforms.all;
  # };
}
