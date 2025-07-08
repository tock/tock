// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::{SPI_HOST0_BASE_ADDR, SPI_HOST1_BASE_ADDR};
use kernel::utilities::StaticRef;
use lowrisc::spi_host::SpiHostRegisters;

pub const SPIHOST0_BASE: StaticRef<SpiHostRegisters> =
    unsafe { StaticRef::new(SPI_HOST0_BASE_ADDR as *const SpiHostRegisters) };

pub const SPIHOST1_BASE: StaticRef<SpiHostRegisters> =
    unsafe { StaticRef::new(SPI_HOST1_BASE_ADDR as *const SpiHostRegisters) };
