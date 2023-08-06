{ craneLib, lib, pkg-config, nix, boost }:
let
  src = lib.cleanSource ./.;
  args = {
    pname = "nix-doc";

    inherit src;
    nativeBuildInputs = [
      pkg-config
    ];
    buildInputs = [];
  };
  cargoArtifacts = craneLib.buildDepsOnly args;

  argsOurs = args // {
    inherit cargoArtifacts;
    # deliberately only include C++ libs in the binary one, so we are only
    # compiling our deps once per nix version.
    nativeBuildInputs = args.nativeBuildInputs ++ [
      nix
    ];
    buildInputs = args.buildInputs ++ [
      nix
      boost
    ];
  };
  crate = craneLib.buildPackage argsOurs;
in
crate
