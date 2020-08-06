{ nixpkgs ? import <nixpkgs> { } }:
nixpkgs.rustPlatform.buildRustPackage {
  pname   = "nix-doc";
  version = "0.2.0";

  src = builtins.fetchGit ./.;

  cargoSha256 = "1n6kc82bisibkjkalc9q5fb4nq6x8a2y210x0s9fdcld1cl3x9a5";

  meta = with nixpkgs.stdenv.lib; {
    description = "A source-based Nix documentation tool";
    homepage    = "https://github.com/lf-/nix-doc";
    license     = licenses.lgpl3Plus;
    platforms   = platforms.all;
  };
}
