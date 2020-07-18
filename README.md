# nix-doc

A Nix documentation search tool.

## Usage

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
./lib/customisation.nix:117  callPackageWith

   Like callPackage, but for a function that returns an attribute
   set of derivations. The override function is added to the
   individual attributes.
./lib/customisation.nix:127  callPackagesWith

   Similar to callPackageWith/callPackage, but without makeOverridable
./pkgs/development/beam-modules/lib.nix:7  callPackageWith
```

## TODO

* Print arguments to functions (requires implementing an rnix pretty-printer)
* Generate tags files/otherwise generate a database file to speed up result
  generation
