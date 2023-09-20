#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Find chips based on folders with Cargo.toml
for b in $(find chips -maxdepth 4 -name 'Cargo.toml'); do
    b1=${b#chips/}
    b2=${b1%/*}
    echo $b2
done
