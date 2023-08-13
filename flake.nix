{
  description = "Documentation plugin for Nix";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      # unused so break it
      inputs.rust-overlay.follows = "flake-utils";
      inputs.flake-utils.follows = "flake-utils";
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
              bear
              rust-analyzer
              clang-tools_14
            ];
          };
        in
        {
          packages = rec {
            nix-doc = nix-doc-for "nix_2_17";
            default = nix-doc;
          } // versions [
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
