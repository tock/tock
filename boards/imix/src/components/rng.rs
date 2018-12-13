//! Component for random number generator on imix board.
//!
//! This provides one Component, RngComponent, which implements a
//! userspace syscall interface to the RNG peripheral (TRNG) on the
//! SAM4L.
//!
//! Usage
//! -----
//! ```rust
//! let rng = RngComponent::new(board_kernel).finalize();
//! ```

// Author: Hudson Ayers <hayers@cs.stanford.edu>
// Last modified: 10/17/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::rng;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::entropy::Entropy32;
use kernel::hil::rng::Rng;

pub struct RngComponent {
    board_kernel: &'static kernel::Kernel,
}

impl RngComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> RngComponent {
        RngComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for RngComponent {
    type Output = &'static rng::RngDriver<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let entropy_to_random = static_init!(
            rng::Entropy32ToRandom<'static>,
            rng::Entropy32ToRandom::new(&sam4l::trng::TRNG)
        );
        let rng = static_init!(
            rng::RngDriver<'static>,
            rng::RngDriver::new(
                entropy_to_random,
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        sam4l::trng::TRNG.set_client(entropy_to_random);
        entropy_to_random.set_client(rng);

        rng
    }
}
