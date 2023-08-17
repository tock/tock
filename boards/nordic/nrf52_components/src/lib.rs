// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

#![no_std]

pub mod startup;

pub use self::startup::{
    NrfClockComponent, NrfRadioACKBufComponent, NrfStartupComponent, UartChannel,
    UartChannelComponent, UartPins, ACK_BUF_SIZE,
};
