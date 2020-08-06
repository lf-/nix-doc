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

## TODO

* Generate tags files/otherwise generate a database file to speed up result
  generation.
* Use Nix itself with builtins.unsafeGetAttrPos to find our functions for us,
  but we handle pulling out the docs. (requires a fair refactor)
* Fix the dedent again. We are eating indents we should not be eating e.g. in
  the example above.
