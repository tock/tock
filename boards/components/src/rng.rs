// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for random number generator.
//!
//! `RngComponent`
//! --------------
//!
//! `RngComponent` implements a userspace syscall interface to the RNG
//! peripheral (TRNG) using `Entropy32ToRandom`.
//!
//! ### Usage
//! ```rust
//! let rng = components::rng::RngComponent::new(board_kernel, capsules_core::rng::DRIVER_NUM, rng)
//!     .finalize(rng_component_static!(nrf52840::trng::Trng));
//! ```
//!
//! `RngRandomComponent`
//! --------------------
//!
//! `RngRandomComponent` implements a userspace syscall interface to an RNG.
//!
//! ### Usage
//! ```rust
//! let rng = components::rng::RngRandomComponent::new(board_kernel, capsules_core::rng::DRIVER_NUM, rng)
//!     .finalize(rng_random_component_static!(qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng));
//! ```

// Author: Hudson Ayers <hayers@cs.stanford.edu>
// Last modified: 07/12/2019

use capsules_core::rng;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::entropy::Entropy32;
use kernel::hil::rng::Rng;
use kernel::{capabilities, DriverNumber};

#[macro_export]
macro_rules! rng_component_static {
    ($E: ty $(,)?) => {{
        let etr = kernel::static_buf!(capsules_core::rng::Entropy32ToRandom<'static, $E>);
        let rng = kernel::static_buf!(
            capsules_core::rng::RngDriver<
                'static,
                capsules_core::rng::Entropy32ToRandom<'static, $E>,
            >
        );

        (etr, rng)
    };};
}

pub type RngComponentType<E> =
    rng::RngDriver<'static, capsules_core::rng::Entropy32ToRandom<'static, E>>;

pub struct RngComponent<E: Entropy32<'static> + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: DriverNumber,
    trng: &'static E,
}

impl<E: Entropy32<'static>> RngComponent<E> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: DriverNumber,
        trng: &'static E,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            trng,
        }
    }
}

impl<E: Entropy32<'static>> Component for RngComponent<E> {
    type StaticInput = (
        &'static mut MaybeUninit<capsules_core::rng::Entropy32ToRandom<'static, E>>,
        &'static mut MaybeUninit<
            capsules_core::rng::RngDriver<
                'static,
                capsules_core::rng::Entropy32ToRandom<'static, E>,
            >,
        >,
    );
    type Output =
        &'static rng::RngDriver<'static, capsules_core::rng::Entropy32ToRandom<'static, E>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let entropy_to_random = static_buffer
            .0
            .write(rng::Entropy32ToRandom::new(self.trng));
        let rng = static_buffer.1.write(rng::RngDriver::new(
            entropy_to_random,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        self.trng.set_client(entropy_to_random);
        entropy_to_random.set_client(rng);

        rng
    }
}

#[macro_export]
macro_rules! rng_random_component_static {
    ($R: ty $(,)?) => {{
        let rng = kernel::static_buf!(capsules_core::rng::RngDriver<'static, $R>);

        rng
    };};
}

pub type RngRandomComponentType<R> = rng::RngDriver<'static, R>;

pub struct RngRandomComponent<R: Rng<'static> + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: DriverNumber,
    rng: &'static R,
}

impl<R: Rng<'static>> RngRandomComponent<R> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: DriverNumber,
        rng: &'static R,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            rng,
        }
    }
}

impl<R: Rng<'static>> Component for RngRandomComponent<R> {
    type StaticInput = &'static mut MaybeUninit<capsules_core::rng::RngDriver<'static, R>>;
    type Output = &'static rng::RngDriver<'static, R>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let rng_driver = static_buffer.write(rng::RngDriver::new(
            self.rng,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        self.rng.set_client(rng_driver);

        rng_driver
    }
}
