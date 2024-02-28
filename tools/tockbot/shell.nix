# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

with import <nixpkgs> {};

mkShell {
  name = "mirrorcheck-shell";
  buildInputs = [
    (python3.withPackages (pypkgs: with pypkgs; [
      requests-cache pygithub pyyaml
    ]))
  ];
}

