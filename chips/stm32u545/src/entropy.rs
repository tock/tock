// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

// specified in the documentation (NIST compliant RNG configuration table in AN4230 available from www.st.com.)
// that values for the CR, HTCR and NSCR should be 0x00F11F00, 0x76B3 and 0x24C2 respectivly. CR config
// is that value, 0x00F11F00, plus the CONDRST bit
pub const RNG_CR_CONFIG_U545: u32 = 0x40F11F00;
pub const RNG_HTCR_CONFIG_U545: u32 = 0x76B3;
pub const RNG_NSCR_CONFIG_U545: u32 = 0x24C2;

pub type Trng<'a> =
    stm32u5xx::entropy::Trng<'a, RNG_CR_CONFIG_U545, RNG_HTCR_CONFIG_U545, RNG_NSCR_CONFIG_U545>;

pub use stm32u5xx::entropy::RNG_BASE;
