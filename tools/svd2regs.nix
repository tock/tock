#
# Nix environment to run svd2regs.py
#
# To install the environment
#
# $ nix-env --file svd2regs.nix --install env-svd2regs
#
# To load the environment
#
# $ load-env-svd2regs
#
with import <nixpkgs> {};
let
  cmsis-svd = python37.pkgs.buildPythonPackage rec {
    pname = "cmsis-svd";
    version = "0.4";

    src = python37.pkgs.fetchPypi {
      inherit pname version;
      sha256 = "b5f439fc6bbc43c9b56dd822f1f764359d503c685a42f913a1cfc2a3c6c42b2c";
    };

    buildInputs = [ python37.pkgs.six ];

    doCheck = false;

    meta = {
      homepage = "https://github.com/posborne/cmsis-svd";
      description = "CMSIS SVD data files and parser";
    };
  };

  pydentifier = python37.pkgs.buildPythonPackage rec {
    pname = "pydentifier";
    version = "0.1.3";

    src = python37.pkgs.fetchPypi {
      inherit pname version;
      sha256 = "981f9705f71e0307a22030d3908369847b99a40caa5dba99aea9989400eb56a6";
    };

    doCheck = false;

    meta = {
      homepage = "https://github.com/nathforge/pydentifier";
      description = "Generate Python identifiers from English text";
    };
  };

  svd2regsPythonEnv = python37.withPackages (_: [ cmsis-svd
                                                  pydentifier
                                                  python37.pkgs.six ]);
in
myEnvFun {
  name = "svd2regs";

  buildInputs = [ svd2regsPythonEnv ];
}
