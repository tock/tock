//! Component for random number generator using `Entropy32ToRandom`.
//!
//! This provides one Component, RngComponent, which implements a
//! userspace syscall interface to the RNG peripheral (TRNG).
//!
//! Usage
//! -----
//! ```rust
//! let rng = components::rng::RngComponent::new(board_kernel, &sam4l::trng::TRNG).finalize(());
//! ```

// Author: Hudson Ayers <hayers@cs.stanford.edu>
// Last modified: 07/12/2019

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::rng;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::entropy::Entropy32;
use kernel::hil::rng::Rng;
use kernel::static_init;

pub struct RngComponent {
    board_kernel: &'static kernel::Kernel,
    trng: &'static dyn Entropy32<'static>,
}

impl RngComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        trng: &'static dyn Entropy32<'static>,
    ) -> RngComponent {
        RngComponent {
            board_kernel: board_kernel,
            trng: trng,
        }
    }
}

impl Component for RngComponent {
    type StaticInput = ();
    type Output = &'static rng::RngDriver<'static>;

    unsafe fn finalize(&mut self, _static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let entropy_to_random = static_init!(
            rng::Entropy32ToRandom<'static>,
            rng::Entropy32ToRandom::new(self.trng)
        );
        let rng = static_init!(
            rng::RngDriver<'static>,
            rng::RngDriver::new(
                entropy_to_random,
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        self.trng.set_client(entropy_to_random);
        entropy_to_random.set_client(rng);

        rng
    }
}
