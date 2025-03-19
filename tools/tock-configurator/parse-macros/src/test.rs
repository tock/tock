// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

#[test]
fn ui() {
    let tests = trybuild::TestCases::new();
    tests.pass("./tests/01.rs");
    //  TODO: Automate the writing of tests errors to a file and deleting them after because
    // the `compile fail` tests cannot be run due to the license checker.
}
