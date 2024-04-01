// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for EUI-64 (Extended Unique Identifier).
//!
//! Usage
//! -----
//! ```rust
//! let eui64 = components::eui64::Eui64Component::new(
//!     board_kernel,
//!     capsules_extra::eui64::DRIVER_NUM,
//!     device_id)
//! .finalize(components::eui64_component_static!());
//! ```

use capsules_extra::eui64::{Eui64, EUI64_BUF_SIZE};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::{capabilities, create_capability};

#[macro_export]
macro_rules! eui64_component_static {
    () => {{
        let eui64 = kernel::static_buf!(capsules_extra::eui64::Eui64);
        let eui64_buf = kernel::static_buf!([u8; capsules_extra::eui64::EUI64_BUF_SIZE]);
        (eui64, eui64_buf)
    };};
}

pub type Eui64ComponentType = capsules_extra::eui64::Eui64;

pub struct Eui64Component {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    eui64: [u8; 8],
}

impl Eui64Component {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, eui64: [u8; 8]) -> Self {
        Self {
            board_kernel,
            driver_num,
            eui64,
        }
    }
}

impl Component for Eui64Component {
    type StaticInput = (
        &'static mut MaybeUninit<Eui64>,
        &'static mut MaybeUninit<[u8; EUI64_BUF_SIZE]>,
    );
    type Output = &'static Eui64;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);
        let eui64_buf = s.1.write(self.eui64) as &[u8; EUI64_BUF_SIZE];

        s.0.write(Eui64::new(eui64_buf, grant))
    }
}
