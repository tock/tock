// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a NULL process checking machine that approves all
//! processes.

use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! app_checker_null_component_static {
    () => {{
        kernel::static_buf!(capsules_system::process_checker::basic::AppCheckerNull)
    };};
}

pub type AppCheckerNullComponentType = capsules_system::process_checker::basic::AppCheckerNull;

pub struct AppCheckerNullComponent {}

impl AppCheckerNullComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for AppCheckerNullComponent {
    type StaticInput =
        &'static mut MaybeUninit<capsules_system::process_checker::basic::AppCheckerNull>;
    type Output = &'static capsules_system::process_checker::basic::AppCheckerNull;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(capsules_system::process_checker::basic::AppCheckerNull::new())
    }
}
