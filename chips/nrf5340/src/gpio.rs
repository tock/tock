// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::utilities::StaticRef;
pub use nrf5x::gpio::{GPIOPin, GpioRegisters, GpioteRegisters, Pin, Port};

pub const NUM_PINS: usize = 48;

/// The secure GPIOTE instance.
pub const GPIOTE0_BASE: StaticRef<GpioteRegisters> =
    unsafe { StaticRef::new(0x5000D000 as *const GpioteRegisters) };

// `GpioRegisters` starts 0x500 bytes before OUT, which sits at 0x50842504.
pub const GPIO_BASE_PORT0: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x50842000 as *const GpioRegisters) };
pub const GPIO_BASE_PORT1: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x50842300 as *const GpioRegisters) };

pub const fn nrf5340_gpio_create<'a>() -> Port<'a, NUM_PINS> {
    Port::new([
        GPIOPin::new(Pin::P0_00, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_01, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_02, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_03, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_04, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_05, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_06, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_07, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_08, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_09, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_10, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_11, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_12, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_13, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_14, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_15, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_16, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_17, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_18, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_19, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_20, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_21, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_22, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_23, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_24, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_25, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_26, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_27, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_28, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_29, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_30, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_31, GPIOTE0_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P1_00, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_01, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_02, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_03, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_04, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_05, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_06, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_07, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_08, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_09, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_10, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_11, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_12, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_13, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_14, GPIOTE0_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_15, GPIOTE0_BASE, GPIO_BASE_PORT1),
    ])
}
