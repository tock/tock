# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

{pkgs ? import <nixpkgs> {}}: let
  pythonEnv = pkgs.python311.withPackages (ps:
    with ps; [
      pip
      setuptools
      wheel
    ]);
in
  pkgs.mkShell {
    name = "treadmill-hwci-env";
    buildInputs = [
      pythonEnv
      pkgs.autoPatchelfHook
    ];
    propagatedBuildInputs = [
      pkgs.stdenv.cc.cc.lib
    ];

    venvDir = "./venv";

    shellHook = ''
      if [ ! -d "$venvDir" ]; then
        echo "Creating new venv..."
        ${pythonEnv.interpreter} -m venv "$venvDir"
      fi

      source "$venvDir/bin/activate"

      if [ ! -f "$venvDir/.requirements_installed" ] || [ requirements.txt -nt "$venvDir/.requirements_installed" ]; then
        echo "Installing/updating dependencies..."
        pip install -U pip setuptools wheel
        pip install -r requirements.txt
        pip install -e .
        autoPatchelf "$venvDir"
        touch "$venvDir/.requirements_installed"
      fi

      export PYTHONPATH="$PWD:$PYTHONPATH"

      echo "Virtual environment is ready!"
    '';
  }
