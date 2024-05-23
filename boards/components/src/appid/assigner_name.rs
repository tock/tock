// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for AppID assigners based on names.

use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! appid_assigner_names_component_static {
    () => {{
        kernel::static_buf!(
            capsules_system::process_checker::basic::AppIdAssignerNames<fn(&'static str) -> u32>
        )
    };};
}

pub struct AppIdAssignerNamesComponent {}

impl AppIdAssignerNamesComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for AppIdAssignerNamesComponent {
    type StaticInput = &'static mut MaybeUninit<
        capsules_system::process_checker::basic::AppIdAssignerNames<
            'static,
            fn(&'static str) -> u32,
        >,
    >;

    type Output = &'static capsules_system::process_checker::basic::AppIdAssignerNames<
        'static,
        fn(&'static str) -> u32,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(
            capsules_system::process_checker::basic::AppIdAssignerNames::new(
                &((|s| kernel::utilities::helpers::crc32_posix(s.as_bytes()))
                    as fn(&'static str) -> u32),
            ),
        )
    }
}
