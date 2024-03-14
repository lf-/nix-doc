# SPDX-FileCopyrightText: 2024 Jade Lovelace
#
# SPDX-License-Identifier: BSD-2-Clause OR MIT

{
  description = "Documentation plugin for Nix";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          inherit (pkgs) lib;

          nix-doc-for = nixVer: pkgs.callPackage ./package.nix {
            craneLib = crane.lib.${system};
            nix = pkgs.nixVersions.${nixVer};
          };

          versions = vs: builtins.listToAttrs (builtins.map
            (v: {
              name = "nix-doc_${v}";
              value = nix-doc-for "nix_${v}";
            })
            vs);

          shellFor = p: pkgs.mkShell {
            # make rust-analyzer work
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            inputsFrom = [ p ];

            # any dev tools you use in excess of the rust ones
            nativeBuildInputs = with pkgs; [
              rust-analyzer
              clang-tools_14
            ] ++ lib.optional pkgs.stdenv.isLinux pkgs.bear;
          } // lib.optionalAttrs pkgs.stdenv.isLinux {
            # so that you can load a mismatched version of nix-doc safely
            hardeningDisable = [ "relro" "bindnow" ];
            RUSTFLAGS = "-Z relro-level=partial";
            # this should have never been a -Z flag
            RUSTC_BOOTSTRAP = "1";
          };
        in
        {
          packages = rec {
            nix-doc = nix-doc-for "nix_2_19";
            default = nix-doc;
          } // versions [
            "2_19"
            "2_18"
            "2_17"
            "2_16"
            "2_15"
            "2_14"
            "2_13"
          ];
          checks = self.packages.${system};

          # for debugging
          inherit pkgs;

          devShells = builtins.mapAttrs (k: v: shellFor v) self.packages.${system};
        }
      )
  ;
}
