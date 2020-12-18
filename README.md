# nix-doc

A Nix documentation search tool. This package provides two front ends for
documentation lookup: a Nix plugin that allows access to documentation directly
from `nix repl` and a command line tool.

## Setup

```
# installs both Nix plugin and command line tool
$ nix-env -i -f https://github.com/lf-/nix-doc/archive/main.tar.gz
# or if you don't want to use nix, (only includes the command line tool)
$ cargo install nix-doc
```

### Nix Plugin

To install the Nix plugin, add this to your Nix config at
`~/.config/nix/nix.conf` after installing `nix-doc` with `nix-env`:

```
plugin-files = /home/YOURUSERNAMEHERE/.nix-profile/lib/libnix_doc_plugin.so
```

## NixOS Installation

To install nix-doc and its plugin system-wide on NixOS, create a basic
derivation for `nix-doc` in a `default.nix` file, for example, in
`/etc/nixos/program/nix-doc/`:

```nix
{ stdenv, rustPlatform, fetchFromGitHub, pkgs, ... }:

rustPlatform.buildRustPackage rec {
  pname = "nix-doc";
  version = "v0.3.1";

  src = fetchFromGitHub {
    repo = pname;
    owner = "lf-";
    rev = version;
    sha256 = "1hiz0fsbsl74m585llg2n359gs8i2m6jh8zdikkwxd8xq7qmw032";
  };

  buildInputs = with pkgs; [ boost nix ];

  nativeBuildInputs = with pkgs; [ pkg-config ];

  cargoSha256 = "06b36jv8hk4gqqm4nbh9gfszncvixs3zcjph45wlx7z5m61y9sdg";

  meta = with stdenv.lib; {
    homepage = url;
    license = licenses.lgpl3;
    description = "A Nix documentation lookup tool that quickly dumps the docs of the library function";
  };
}
```

Expose the package using the following options in
`/etc/nixos/configuration.nix`, where `../program/nix-doc/default.nix` is the
path to the above `default.nix` file. Link the plugin file using
`nix.extraOptions`:

```nix
{ pkgs, ... }:

let
  nix-doc = pkgs.callPackage ./program/nix-doc/default.nix {};
in

{
  nix.extraOptions = ''
    plugin-files = ${nix-doc}/lib/libnix_doc_plugin.so
  '';

  environment.systemPackages = with pkgs; [
    nix-doc
  ];
}
```


## Usage

### CLI

```
nix-doc SearchRegex [Directory]
```

Example output:

```
nixpkgs$ nix-doc callPackage
   Call the package function in the file `fn' with the required
   arguments automatically.  The function is called with the
   arguments `args', but any missing arguments are obtained from
   `autoArgs'.  This function is intended to be partially
   parameterised, e.g.,

   callPackage = callPackageWith pkgs;
   pkgs = {
   libfoo = callPackage ./foo.nix { };
   libbar = callPackage ./bar.nix { };
   };

   If the `libbar' function expects an argument named `libfoo', it is
   automatically passed as an argument.  Overrides or missing
   arguments can be supplied in `args', e.g.

   libbar = callPackage ./bar.nix {
   libfoo = null;
   enableX11 = true;
   };
callPackageWith = autoArgs: fn: args: ...
# ./lib/customisation.nix:117
─────────────────────────────────────────────
   Like callPackage, but for a function that returns an attribute
   set of derivations. The override function is added to the
   individual attributes.
callPackagesWith = autoArgs: fn: args: ...
# ./lib/customisation.nix:127
─────────────────────────────────────────────
   Similar to callPackageWith/callPackage, but without makeOverridable
callPackageWith = autoArgs: fn: args: ...
# ./pkgs/development/beam-modules/lib.nix:7
```

### Nix plugin

The Nix plugin provides three builtins:

#### `builtins.doc f`

Prints the documentation of the function `f` to the screen. Returns `null`.

#### `builtins.getDoc f`

Returns the documentation message for the function `f` as a string (exactly the
same output as `builtins.doc`, just as a string).

#### `builtins.unsafeGetLambdaPos`

A backport of [NixOS/Nix#3912](https://github.com/NixOS/nix/pull/3912). Returns
the position of a lambda, in a similar fashion to `unsafeGetAttrPos` for
attributes.

#### Sample usage:

```
» nix repl
Welcome to Nix version 2.3.7. Type :? for help.

nix-repl> n=import <nixpkgs> {}

nix-repl> builtins.doc n.lib.callPackageWith
   Call the package function in the file `fn' with the required
   arguments automatically.  The function is called with the
   arguments `args', but any missing arguments are obtained from
   `autoArgs'.  This function is intended to be partially
   parameterised, e.g.,

     callPackage = callPackageWith pkgs;
     pkgs = {
       libfoo = callPackage ./foo.nix { };
       libbar = callPackage ./bar.nix { };
     };

   If the `libbar' function expects an argument named `libfoo', it is
   automatically passed as an argument.  Overrides or missing
   arguments can be supplied in `args', e.g.

     libbar = callPackage ./bar.nix {
       libfoo = null;
       enableX11 = true;
     };
func = autoArgs: fn: args: ...
# /nix/store/frpij1x0ihnyc4r5f7v0zxwpslkq6s27-nixpkgs-20.09pre237807.0dc87c6e54f/nixpkgs/lib/customisation.nix:117
null
```

## Development

This repository is set up as a Cargo workspace with the plugin and the command
line tool/library as parts.

It is not really possible to build the plugin outside a Nix shell since Nix
does not provide libraries outside the shell environment. As such, it is
suggested to use a nix shell while developing the plugin as follows:

```
$ nix-shell
[nix-shell]$ cargo build
[nix-shell]$ cargo check
[nix-shell]$ cargo test
# etc
```

## TODO

* Generate tags files/otherwise generate a database file to speed up result
  generation.

## Related work

- https://github.com/NixOS/nix/pull/1652: A PR implementing basically the same
  thing as this tool's plugin in Nix itself, which has been deferred
  indefinitely due to disagreements about what syntax to use in documentation
  comments.
- https://github.com/tazjin/nixdoc: A Rust tool producing DocBook documentation
  for Nix library functions.

## Project information

Everyone is expected to follow the [code of conduct](./CODE_OF_CONDUCT.md)
while participating in this project.
