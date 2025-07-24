#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Find boards based on folders with Makefiles
for b in $(find boards -maxdepth 4 | grep -E 'Makefile$'); do
    b1=${b#boards/}
    b2=${b1%/*}
    echo $b2
done
