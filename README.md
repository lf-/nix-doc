# nix-doc

A Nix developer tool leveraging the `rnix` Nix parser for intelligent
documentation search and tags generation.

## Features

* Nix plugin that adds a builtin that can display the signature and
  documentation for a lambda object in a friendly format
* Command line tool that searches Nix files in a directory for functions and
  shows documentation for matching ones
* Tags generator similar to `ctags` that can generate a vim compatible tags
  file for Nix source
* High performance, threaded implementation in Rust

## Setup

```
# nix-doc is in nixpkgs
$ nix-env -i nix-doc

# you can alternatively get it from git
$ nix-env -i -f https://github.com/lf-/nix-doc/archive/main.tar.gz

# or if you don't want to use nix (only includes the command line tool for
# search and tags)
$ cargo install --locked nix-doc
```

### Nix Plugin

To install the Nix plugin on a single-user installation of Nix, add this to
your Nix config at `~/.config/nix/nix.conf` after installing `nix-doc` with
`nix-env`:

```
plugin-files = /home/YOURUSERNAMEHERE/.nix-profile/lib/libnix_doc_plugin.so
```

For a multi-user installation, you will need to do something like this:

```
$ sudo ln -s $(nix-build '<nixpkgs>' -A nix-doc) /opt/nix-doc
```

and then put this into your `/etc/nix/nix.conf`:

```
plugin-files = /opt/nix-doc/lib/libnix_doc_plugin.so
```

## NixOS Installation

Link the plugin file using
`nix.extraOptions`:

```nix
{ pkgs, ... }:

{
  nix.extraOptions = ''
    plugin-files = ${pkgs.nix-doc}/lib/libnix_doc_plugin.so
  '';

  environment.systemPackages = with pkgs; [
    nix-doc
  ];
}
```


## Usage

### CLI

```
nix-doc <command>
```

#### `nix-doc tags [dir]`

Generates a vim-compatible `tags` file in the current directory, for all nix
script files below the directory `dir`.

Example:

```
nixpkgs$ nix-doc tags

nixpkgs$ file tags
tags: Exuberant Ctags tag file, ASCII text, with very long lines (502)

# opens vim to the function callCabal2nix
nixpkgs$ vim -t callCabal2nix
```

#### `nix-doc search <regex> [dir]`

Example output:

```
nixpkgs$ nix-doc search callPackage
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

- Tech: should update rnix to the latest major.

## Related work

- https://github.com/NixOS/nix/pull/1652: A PR implementing basically the same
  thing as this tool's plugin in Nix itself, which has been deferred
  indefinitely due to disagreements about what syntax to use in documentation
  comments.
- https://github.com/tazjin/nixdoc: A Rust tool producing DocBook documentation
  for Nix library functions.
- https://github.com/mlvzk/manix: An early fork of this tool with a stronger
  focus on CLI usage, with support for indexing and faster search. By
  comparison, their CLI is better, but they don't do tags or a Nix plugin.

## Project information

Everyone is expected to follow the [code of conduct](./CODE_OF_CONDUCT.md)
while participating in this project.
