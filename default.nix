{ nixpkgs ? import <nixpkgs> { } }:
nixpkgs.rustPlatform.buildRustPackage {
  pname   = "nix-doc";
  version = "0.2.2";

  src = builtins.fetchGit ./.;

  nativeBuildInputs = with nixpkgs; [
    pkg-config
  ];

  buildInputs = with nixpkgs; [
    boost
    nix
  ];

  cargoSha256 = "1bh8076ig9ssh8w44vyybswnn66xnfh28jsy6v9g5k6jmdlhr3qm";

  meta = with nixpkgs.stdenv.lib; {
    description = "A source-based Nix documentation tool";
    homepage    = "https://github.com/lf-/nix-doc";
    license     = licenses.lgpl3Plus;
    platforms   = platforms.all;
  };
}
