#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Find crates based on folders with Cargo.lock files
for b in $(find . -maxdepth 4 | grep -E 'Cargo.lock$'); do
    b2=${b%/*}
    echo $b2
done
