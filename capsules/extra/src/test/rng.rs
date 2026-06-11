// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Integration test for an RNG driver implementing the `Entropy32` trait.
//!
//! This test verifies that:
//!   - At least 8 `u32` values can be collected from the entropy source.
//!   - The driver correctly handles the `Continue` / `Done` return from the
//!     client callback, allowing the test to "wait" (re-request) until all
//!     requested values have been delivered.
//!   - Each collected value is stored and printed for manual inspection.
//!
//! # Usage
//!
//! Add this file alongside your driver under `capsules/extra/src/` (or
//! wherever your crate keeps driver tests), then register a test board that
//! wires the concrete `Entropy32` implementor to `RngEntropy32Test`.
//!
//! ```rust,ignore
//! // In your board's main.rs (or test harness):
//! let rng_test = static_init!(
//!     capsules_extra::rng_entropy32_test::RngEntropy32Test<'static, YourRngDriver>,
//!     capsules_extra::rng_entropy32_test::RngEntropy32Test::new(your_rng_driver)
//! );
//! your_rng_driver.set_client(rng_test);
//! rng_test.run();
//! ```

use core::cell::Cell;
use kernel::hil::entropy::{Client32, Entropy32};
use kernel::utilities::cells::OptionalCell;
use kernel::{debug, ErrorCode};

/// Number of `u32` words to collect before declaring the test a success.
const WORDS_REQUESTED: usize = 8;

// ---------------------------------------------------------------------------
// Test component
// ---------------------------------------------------------------------------

pub struct RngEntropy32Test<'a, E: Entropy32<'a>> {
    /// Reference to the entropy source under test.
    rng: &'a E,
    /// Accumulator for collected entropy words.
    collected: OptionalCell<[u32; WORDS_REQUESTED]>,
    /// How many words we have stored so far.
    count: Cell<usize>,
}

impl<'a, E: Entropy32<'a>> RngEntropy32Test<'a, E> {
    pub fn new(rng: &'a E) -> Self {
        Self {
            rng,
            collected: OptionalCell::new([0u32; WORDS_REQUESTED]),
            count: Cell::new(0),
        }
    }

    /// Kick off the entropy collection. Call once after wiring the client.
    pub fn run(&self) {
        debug!(
            "[RNG TEST] Starting Entropy32 test — requesting {} u32 words",
            WORDS_REQUESTED
        );
        match self.rng.get() {
            Ok(()) => {}
            Err(e) => {
                debug!("[RNG TEST] FAIL: rng.get() returned error: {:?}", e);
            }
        }
    }

    /// Print a summary once all words have been collected.
    fn finish(&self) {
        self.collected.map(|words| {
            debug!(
                "[RNG TEST] PASS: collected {} u32 entropy words:",
                WORDS_REQUESTED
            );
            for (i, w) in words.iter().enumerate() {
                debug!("  word[{}] = {:#010x}", i, w);
            }
            // Basic sanity: not *all* zeros (astronomically unlikely with a real RNG).
            let all_zero = words.iter().all(|&w| w == 0);
            if all_zero {
                debug!(
                    "[RNG TEST] WARNING: all collected words are zero — verify your RNG source!"
                );
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Entropy32 client implementation
// ---------------------------------------------------------------------------

impl<'a, E: Entropy32<'a>> Client32 for RngEntropy32Test<'a, E> {
    /// Called by the driver each time a new `u32` word of entropy is ready.
    ///
    /// Returns:
    /// - `Ok(Continue)` — ask the driver for another word (we still need more).
    /// - `Ok(Done)`     — we have collected enough; release the hardware.
    fn entropy_available(
        &self,
        entropy: &mut dyn Iterator<Item = u32>,
        error: Result<(), ErrorCode>,
    ) -> kernel::hil::entropy::Continue {
        // Surface driver-level errors but keep going — some peripheral drivers
        // (e.g. hardware FIFOs) report transient under-run errors yet can still
        // produce data; we treat them as non-fatal for test purposes.
        if let Err(e) = error {
            debug!(
                "[RNG TEST] entropy_available reported error: {:?} — retrying",
                e
            );
            return kernel::hil::entropy::Continue::More;
        }

        let mut done = false;

        self.collected.map(|mut words: [u32; 8]| {
            // Drain as many words as the iterator offers in this callback.
            for word in &mut *entropy {
                let idx = self.count.get();
                if idx >= WORDS_REQUESTED {
                    break;
                }
                words[idx] = word;
                self.count.set(idx + 1);
                debug!(
                    "[RNG TEST] word[{}] = {:#010x}  ({}/{} collected)",
                    idx,
                    word,
                    idx + 1,
                    WORDS_REQUESTED
                );
                if idx + 1 >= WORDS_REQUESTED {
                    break;
                }
            }
            done = self.count.get() >= WORDS_REQUESTED;
        });

        if done {
            self.finish();
            kernel::hil::entropy::Continue::Done
        } else {
            // Not enough words yet — tell the driver to keep going / call us
            // again when more entropy is available.
            kernel::hil::entropy::Continue::More
        }
    }
}
