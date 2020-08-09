{ nixpkgs ? import <nixpkgs> { } }:
let
  gitignoreSrc = nixpkgs.fetchFromGitHub {
      owner = "hercules-ci";
      repo = "gitignore";
      rev = "647d0821b590ee96056f4593640534542d8700e5";
      sha256 = "sha256:0ks37vclz2jww9q0fvkk9jyhscw0ial8yx2fpakra994dm12yy1d";
    };
  inherit (import gitignoreSrc { inherit (nixpkgs) lib; }) gitignoreSource;
in
nixpkgs.rustPlatform.buildRustPackage {
  pname   = "nix-doc";
  version = "0.3.0";

  src = gitignoreSource ./.;

  nativeBuildInputs = with nixpkgs; [
    pkg-config
  ];

  buildInputs = with nixpkgs; [
    boost
    nix
  ];

  cargoSha256 = "1xxjw94dfqimcf74gyaf4iqq99r1rsqp95imczfhpkx8kvf99xyn";

  meta = with nixpkgs.stdenv.lib; {
    description = "A source-based Nix documentation tool";
    homepage    = "https://github.com/lf-/nix-doc";
    license     = licenses.lgpl3Plus;
    platforms   = platforms.all;
  };
}
