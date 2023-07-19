// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for collections of HMACs.
//!
//! Usage
//! -----
//! ```rust
//!    let hmac = components::hmac::HmacComponent::new(
//!        board_kernel,
//!        chip.hmac,
//!    )
//!    .finalize(components::hmac_component_static!(
//!        lowrisc::hmac::Hmac,
//!        32
//!    ));
//! ```

use capsules_extra::hmac::HmacDriver;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;

#[macro_export]
macro_rules! hmac_component_static {
    ($A:ty, $L:expr $(,)?) => {{
        let hmac = kernel::static_buf!(capsules_extra::hmac::HmacDriver<'static, $A, $L>);

        let data_buffer = kernel::static_buf!([u8; 64]);
        let dest_buffer = kernel::static_buf!([u8; $L]);

        (hmac, data_buffer, dest_buffer)
    };};
}

pub struct HmacComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    hmac: &'static A,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> HmacComponent<A, L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        hmac: &'static A,
    ) -> HmacComponent<A, L> {
        HmacComponent {
            board_kernel,
            driver_num,
            hmac,
        }
    }
}

impl<
        A: kernel::hil::digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + 'static
            + digest::Digest<'static, L>,
        const L: usize,
    > Component for HmacComponent<A, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<HmacDriver<'static, A, L>>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; L]>,
    );
    type Output = &'static HmacDriver<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let data_buffer = s.1.write([0; 64]);
        let dest_buffer = s.2.write([0; L]);

        let hmac = s.0.write(capsules_extra::hmac::HmacDriver::new(
            self.hmac,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        self.hmac.set_client(hmac);

        hmac
    }
}
