// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::utilities::StaticRef;
pub use nrf5x::gpio::{GPIOPin, GpioRegisters, GpioteRegisters, Pin, Port};

pub const NUM_PINS: usize = 48;

/// The secure GPIOTE instance.
/// Create the pins of both GPIO ports, with the GPIOTE instance and port
/// addresses for the current security state from
/// [`bases`](crate::chip::bases).
pub const fn nrf5340_gpio_create<'a>(
    gpiote: StaticRef<GpioteRegisters>,
    ports: [StaticRef<GpioRegisters>; 2],
) -> Port<'a, NUM_PINS> {
    Port::new([
        GPIOPin::new(Pin::P0_00, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_01, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_02, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_03, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_04, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_05, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_06, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_07, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_08, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_09, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_10, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_11, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_12, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_13, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_14, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_15, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_16, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_17, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_18, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_19, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_20, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_21, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_22, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_23, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_24, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_25, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_26, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_27, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_28, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_29, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_30, gpiote, ports[0]),
        GPIOPin::new(Pin::P0_31, gpiote, ports[0]),
        GPIOPin::new(Pin::P1_00, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_01, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_02, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_03, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_04, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_05, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_06, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_07, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_08, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_09, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_10, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_11, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_12, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_13, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_14, gpiote, ports[1]),
        GPIOPin::new(Pin::P1_15, gpiote, ports[1]),
    ])
}
