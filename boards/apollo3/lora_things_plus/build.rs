// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

fn main() {
    println!("cargo:rerun-if-changed=../layout.ld");
    println!("cargo:rerun-if-changed=../../kernel_layout.ld");
}
