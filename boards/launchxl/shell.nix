# Shell expression for the Nix package manager
#
# This nix expression creates an environment with necessary packages installed:
#
#  * `tockloader`
#  * arm-none-eabi toolchain
#  * rustup
#  * openocd with support for the Launchxl's xds programmer
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
      version = "1.3.0-dev";
      name = "${pname}-${version}";

      propagatedBuildInputs = [ argcomplete colorama crcmod pyserial pytoml ];

      #fetchPypi {
      src = pkgs.fetchgit {
        url = "https://github.com/tock/tockloader";
        rev = "7ddf48669cb750ede5cacb4cb82e17a2841c14da";
        sha256 = "1rx893vj32f18pqlri7ikvcqbbvdpbi66xphdqkzbshljlwpr3xw";
        #inherit pname version;
        #sha256 = "0j15hrz45ay396n94m5i5pca5lrym1qjnj06b2lq9r67ks136333";
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
      (openocd.overrideAttrs (oldAttrs: {
        nativeBuildInputs = [ automake autoconf libtool pkgconfig which ];
        preConfigure = ''
          SKIP_SUBMODULE=1 ./bootstrap
        '';
        src = fetchgit {
          url = "https://github.com/ntfreak/openocd";
          rev = "4896c83ce8f28674a0beb34e7d475cb5b0ac7dab";
          sha256 = "0xlj2vbzbnzrxxxhm9ia0sh7b506m6disj72m494nqdcq0df1shy";
        };
      }))
      ];
     LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";
  }
