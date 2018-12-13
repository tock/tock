# Shell expression for the Nix package manager
#
# This nix expression creates an environment with necessary packages installed:
#
#  * `tockloader`
#  * rust
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
      version = "1.3.1";
      name = "${pname}-${version}";

      propagatedBuildInputs = [ argcomplete colorama crcmod pyserial pytoml ];

      src = fetchPypi {
        inherit pname version;
        sha256 = "1gralnhvl82xr7rkrmxj0c1rxn1y9dlbmkkrklcdjahragbknivn";
      };
    };
  });
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rust_date = "2018-12-01";
  rust_channel = "nightly";
  rust_targets = [ "thumbv7em-none-eabi" "thumbv7em-none-eabihf" "thumbv6m-none-eabi" ];
  rust_build = nixpkgs.rustChannelOfTargets rust_channel rust_date rust_targets;
in
  with pkgs;
  stdenv.mkDerivation {
    name = "tock-dev";
    buildInputs = [
      python3Full
      pythonPackages.tockloader
      rust_build
      ];
  }
