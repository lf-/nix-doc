# SPDX-FileCopyrightText: 2024 Jade Lovelace
#
# SPDX-License-Identifier: BSD-2-Clause OR MIT

{ craneLib, lib, pkg-config, nix, boost, stdenv }:
let
  src = lib.cleanSource ./.;
  args = {
    pname = "nix-doc";

    inherit src;
    nativeBuildInputs = [
      pkg-config
    ];
    buildInputs = [ ];
  } // lib.optionalAttrs stdenv.isLinux {
    # so that you can load a mismatched version of nix-doc safely
    hardeningDisable = [ "relro" "bindnow" ];
    RUSTFLAGS = "-Z relro-level=partial";
    # this should have never been a -Z flag
    RUSTC_BOOTSTRAP = "1";
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
