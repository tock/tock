// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use std::{fmt::Debug, rc::Rc};

use super::NoSupport;

/// The [`Gpio`] trait applies to the peripherals that implement the Gpio-related traits defined in
/// Tock's HIL.
pub trait Gpio: std::fmt::Debug + PartialEq {
    /// The type that is used for indexing the pins. The `Copy` trait bound is present due to the
    /// implementations being either enums or primitives.
    type PinId: for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + std::fmt::Display
        + Debug
        + Copy
        + PartialEq;

    /// Return an array of the pins provided by the `Gpio` peripheral
    ///  FIXME: Change array to slice ASAP.
    fn pins(&self) -> Option<Rc<[Self::PinId]>> {
        None
    }
}

impl Gpio for NoSupport {
    type PinId = usize;
}
