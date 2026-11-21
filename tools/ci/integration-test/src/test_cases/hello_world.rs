// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use super::Step::Expect;
use super::{App, Requirement, TESTS, TestCase};

#[linkme::distributed_slice(TESTS)]
static TEST: TestCase = TestCase {
    id: "hello_world",
    requires: &[Requirement::Uart],
    apps: &[App::LibtockCExample("c_hello")],
    steps: &[Expect("Hello World!")],
};
