#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

# Find chips based on folders with Cargo.toml
for b in $(find chips -maxdepth 4 -name 'Cargo.toml'); do
    b1=${b#chips/}
    b2=${b1%/*}
    # `stm32f4xx` crate is excluded in order to pass CI
    # This is due to the use of Rust features and the lack of support from
    # `cargo test` and `cargo clippy`
    echo $b2 | grep --invert-match "stm32f4xx"
done
