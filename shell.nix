{ nixpkgs ? import <nixpkgs> }:
let
  nix-doc = import ./default.nix { inherit nixpkgs; };
  pkgs = nixpkgs { };
in
nix-doc.overrideAttrs (old: {
  nativeBuildInputs = old.nativeBuildInputs ++ [
    pkgs.bear
    pkgs.niv
  ];
})
