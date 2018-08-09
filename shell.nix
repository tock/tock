# Shell expression for the Nix package manager
#
# This nix expression creates an environment with necessary packages installed:
#
#  * `tockloader`
#  * arm-none-eabi toolchain
#  * rustup
#
# To use:
#
#  $ nix-shell
#

{ pkgs ? import <nixpkgs> {} }:

with builtins;
let
  inherit (pkgs) stdenv;
  pythonPackages = stdenv.lib.fix' (self: with self; pkgs.python3Packages //
  {

    tockloader = buildPythonPackage rec {
      pname = "tockloader";
      version = "1.1.0";
      name = "${pname}-${version}";

      propagatedBuildInputs = [ argcomplete colorama crcmod pyserial pytoml ];

      src = fetchPypi {
        inherit pname version;
        sha256 = "0j15hrz45ay396n94m5i5pca5lrym1qjnj06b2lq9r67ks136333";
      };
    };
  });
in
  with pkgs;
  stdenv.mkDerivation {
    name = "tock-dev";
    buildInputs = [
      rustup
      gcc-arm-embedded
      python3Full
      pythonPackages.tockloader
      ];
  }
