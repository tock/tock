// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interfaces for interacting with debug hardware integrated in various SoCs.
//! Currently allows reading the cycle counter.

pub trait CycleCounter {
    /// Enable and start the cycle counter.
    /// Depending on the underlying hardware, it may be necessary to call reset
    /// before starting the cycle counter for the first time to get accurate results.
    fn start(&self);

    /// Stop the cycle counter.
    /// Does nothing if the cycle counter is not present.
    fn stop(&self);

    /// Return the current value of the cycle counter.
    fn count(&self) -> u64;

    /// Reset the counter to zero and stop the cycle counter.
    fn reset(&self);

    /// Benchmark the number of cycles to run a passed closure.
    /// This function is intended for use debugging in-kernel routines.
    fn profile_closure<F: FnOnce()>(&self, f: F) -> u64 {
        self.reset();
        self.start();
        f();
        self.stop();
        self.count()
    }
}
