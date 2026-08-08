// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Adapted from embassy-rs/embassy/embassy-stm32/src/time.rs

use core::fmt::Display;
use core::ops::{Div, Mul};

/// Hertz
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug, Default)]
pub struct Hertz(pub u32);

impl Display for Hertz {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

impl Hertz {
    /// Create a `Hertz` from the given hertz.
    pub const fn hz(hertz: u32) -> Self {
        Self(hertz)
    }

    /// Create a `Hertz` from the given kilohertz.
    pub const fn khz(kilohertz: u32) -> Self {
        Self(kilohertz * 1_000)
    }

    /// Create a `Hertz` from the given megahertz.
    pub const fn mhz(megahertz: u32) -> Self {
        Self(megahertz * 1_000_000)
    }
}

pub(crate) trait Prescaler {
    fn num(&self) -> u32;
    fn denom(&self) -> u32;
}

impl<T: Prescaler> Div<T> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: T) -> Self::Output {
        self * rhs.denom() / rhs.num()
    }
}

impl<T: Prescaler> Mul<T> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: T) -> Self::Output {
        self * rhs.num() / rhs.denom()
    }
}

/// This is a convenience shortcut for [`Hertz::hz`]
pub const fn hz(hertz: u32) -> Hertz {
    Hertz::hz(hertz)
}

/// This is a convenience shortcut for [`Hertz::khz`]
pub const fn khz(kilohertz: u32) -> Hertz {
    Hertz::khz(kilohertz)
}

/// This is a convenience shortcut for [`Hertz::mhz`]
pub const fn mhz(megahertz: u32) -> Hertz {
    Hertz::mhz(megahertz)
}

impl Mul<u32> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u32) -> Self::Output {
        Hertz(self.0 * rhs)
    }
}

impl Div<u32> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: u32) -> Self::Output {
        Hertz(self.0 / rhs)
    }
}

impl Mul<u16> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u16) -> Self::Output {
        self * (rhs as u32)
    }
}

impl Div<u16> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: u16) -> Self::Output {
        self / (rhs as u32)
    }
}

impl Mul<u8> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u8) -> Self::Output {
        self * (rhs as u32)
    }
}

impl Div<u8> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: u8) -> Self::Output {
        self / (rhs as u32)
    }
}

impl Div<Hertz> for Hertz {
    type Output = u32;
    fn div(self, rhs: Hertz) -> Self::Output {
        self.0 / rhs.0
    }
}
