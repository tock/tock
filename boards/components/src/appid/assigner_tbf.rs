// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for AppID assigners based on TBF headers.

use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! appid_assigner_tbf_header_component_static {
    () => {{
        kernel::static_buf!(capsules_system::process_checker::tbf::AppIdAssignerTbfHeader)
    };};
}

pub struct AppIdAssignerTbfHeaderComponent {}

impl AppIdAssignerTbfHeaderComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for AppIdAssignerTbfHeaderComponent {
    type StaticInput =
        &'static mut MaybeUninit<capsules_system::process_checker::tbf::AppIdAssignerTbfHeader>;

    type Output = &'static capsules_system::process_checker::tbf::AppIdAssignerTbfHeader;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(capsules_system::process_checker::tbf::AppIdAssignerTbfHeader {})
    }
}
