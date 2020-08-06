{ nixpkgs ? import <nixpkgs> { } }:
nixpkgs.rustPlatform.buildRustPackage {
  pname   = "nix-doc";
  version = "0.2.2";

  src = builtins.fetchGit ./.;

  cargoSha256 = "0sbybqr8g9bn1nrsm2azlyw0cp9bx2yz9lhanymw5cx9c1hpam5d";

  meta = with nixpkgs.stdenv.lib; {
    description = "A source-based Nix documentation tool";
    homepage    = "https://github.com/lf-/nix-doc";
    license     = licenses.lgpl3Plus;
    platforms   = platforms.all;
  };
}
