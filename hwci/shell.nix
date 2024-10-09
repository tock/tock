{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  name = "example-env";
  buildInputs = [
    pkgs.python311
    pkgs.python311Packages.venvShellHook
    pkgs.autoPatchelfHook
  ];
  propagatedBuildInputs = [
    pkgs.stdenv.cc.cc.lib
  ];

  venvDir = "./venv";
  postVenvCreation = ''
    unset SOURCE_DATE_EPOCH
    pip install -U pip setuptools wheel
    pip install -r requirements.txt
    pip install -e .
    autoPatchelf ./venv
  '';
}
