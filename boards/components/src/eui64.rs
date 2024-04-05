// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for EUI-64 (Extended Unique Identifier).
//!
//! Usage
//! -----
//! ```rust
//!let eui64 = components::eui64::Eui64Component::new(u64::from_le_bytes(device_id))
//!     .finalize(components::eui64_component_static!());
//! ```

use capsules_extra::eui64::Eui64;
use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! eui64_component_static {
    () => {{
        let eui64_driver = kernel::static_buf!(capsules_extra::eui64::Eui64);
        let eui64_val = kernel::static_buf!(u64);
        (eui64_driver, eui64_val)
    };};
}

pub type Eui64ComponentType = capsules_extra::eui64::Eui64;

pub struct Eui64Component {
    eui64: u64,
}

impl Eui64Component {
    pub fn new(eui64: u64) -> Self {
        Self { eui64 }
    }
}

impl Component for Eui64Component {
    type StaticInput = (
        &'static mut MaybeUninit<Eui64>,
        &'static mut MaybeUninit<u64>,
    );
    type Output = &'static Eui64;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let eui64_val = s.1.write(self.eui64);
        s.0.write(Eui64::new(eui64_val))
    }
}
