let
  sources = import ./nix/sources.nix;
in
{ nixpkgs ? import <nixpkgs> }:
  let
    nixpkgs-mozilla = import sources.nixpkgs-mozilla;
    pkgs = nixpkgs {
      overlays = [
        nixpkgs-mozilla
        (
          final: prev:
            let
              nightlyRust = final.latest.rustChannels.nightly.rust;
            in
              {
                rustPlatform = prev.makeRustPlatform {
                  rustc = nightlyRust;
                  cargo = nightlyRust;
                };
              }
        )
      ];
    };
    inherit (pkgs.nix-gitignore) gitignoreSource;
  in
    pkgs.rustPlatform.buildRustPackage {
      pname = "nix-doc";
      version = "0.5.1";

      cargoSha256 = "11bcn111j4iqwqwpp26w7rnycasbhiixaqb1rq4ry2402hvl90j7";

      src = gitignoreSource [] ./.;

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
