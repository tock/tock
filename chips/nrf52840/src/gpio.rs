// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
pub use nrf52::gpio::{GPIOPin, Pin, Port, GPIO_BASE_ADDRESS, GPIO_SIZE};
pub use nrf52::gpio::{GpioRegisters, GpioteRegisters};

pub const NUM_PINS: usize = 48;

pub const GPIOTE_BASE: StaticRef<GpioteRegisters> =
    unsafe { StaticRef::new(0x40006000 as *const GpioteRegisters) };

pub const GPIO_BASE_PORT0: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new((GPIO_BASE_ADDRESS) as *const GpioRegisters) };
pub const GPIO_BASE_PORT1: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new((GPIO_BASE_ADDRESS + GPIO_SIZE) as *const GpioRegisters) };

pub fn nrf52840_gpio_create_pin<'a>(pin: Pin) -> GPIOPin<'a> {
    let base_port = match pin {
        Pin::P0_00
        | Pin::P0_01
        | Pin::P0_02
        | Pin::P0_03
        | Pin::P0_04
        | Pin::P0_05
        | Pin::P0_06
        | Pin::P0_07
        | Pin::P0_08
        | Pin::P0_09
        | Pin::P0_10
        | Pin::P0_11
        | Pin::P0_12
        | Pin::P0_13
        | Pin::P0_14
        | Pin::P0_15
        | Pin::P0_16
        | Pin::P0_17
        | Pin::P0_18
        | Pin::P0_19
        | Pin::P0_20
        | Pin::P0_21
        | Pin::P0_22
        | Pin::P0_23
        | Pin::P0_24
        | Pin::P0_25
        | Pin::P0_26
        | Pin::P0_27
        | Pin::P0_28
        | Pin::P0_29
        | Pin::P0_30
        | Pin::P0_31 => GPIO_BASE_PORT0,
        Pin::P1_00
        | Pin::P1_01
        | Pin::P1_02
        | Pin::P1_03
        | Pin::P1_04
        | Pin::P1_05
        | Pin::P1_06
        | Pin::P1_07
        | Pin::P1_08
        | Pin::P1_09
        | Pin::P1_10
        | Pin::P1_11
        | Pin::P1_12
        | Pin::P1_13
        | Pin::P1_14
        | Pin::P1_15 => GPIO_BASE_PORT1,
    };
    GPIOPin::new(pin, GPIOTE_BASE, base_port)
}

pub const fn nrf52840_gpio_create<'a>() -> Port<'a, NUM_PINS> {
    Port::new([
        GPIOPin::new(Pin::P0_00, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_01, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_02, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_03, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_04, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_05, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_06, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_07, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_08, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_09, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_10, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_11, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_12, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_13, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_14, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_15, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_16, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_17, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_18, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_19, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_20, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_21, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_22, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_23, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_24, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_25, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_26, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_27, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_28, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_29, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_30, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P0_31, GPIOTE_BASE, GPIO_BASE_PORT0),
        GPIOPin::new(Pin::P1_00, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_01, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_02, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_03, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_04, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_05, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_06, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_07, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_08, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_09, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_10, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_11, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_12, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_13, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_14, GPIOTE_BASE, GPIO_BASE_PORT1),
        GPIOPin::new(Pin::P1_15, GPIOTE_BASE, GPIO_BASE_PORT1),
    ])
}
