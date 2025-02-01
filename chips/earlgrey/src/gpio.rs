// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! GPIO instantiation.

use core::ops::{Index, IndexMut};

use kernel::utilities::StaticRef;
use lowrisc::gpio::GpioRegisters;
pub use lowrisc::gpio::{pins, GpioBitfield, GpioPin};

use crate::pinmux::PadConfig;
use crate::pinmux_config::EarlGreyPinmuxConfig;
use crate::registers::top_earlgrey::GPIO_BASE_ADDR;
use crate::registers::top_earlgrey::{
    MuxedPads, PinmuxInsel, PinmuxOutsel, PinmuxPeripheralIn, PINMUX_MIO_PERIPH_INSEL_IDX_OFFSET,
};

pub const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(GPIO_BASE_ADDR as *const GpioRegisters) };

pub struct Port<'a> {
    pins: [GpioPin<'a, PadConfig>; 32],
}

impl From<PinmuxPeripheralIn> for PinmuxOutsel {
    fn from(pin: PinmuxPeripheralIn) -> Self {
        match pin {
            PinmuxPeripheralIn::GpioGpio0 => PinmuxOutsel::GpioGpio0,
            PinmuxPeripheralIn::GpioGpio1 => PinmuxOutsel::GpioGpio1,
            PinmuxPeripheralIn::GpioGpio2 => PinmuxOutsel::GpioGpio2,
            PinmuxPeripheralIn::GpioGpio3 => PinmuxOutsel::GpioGpio3,
            PinmuxPeripheralIn::GpioGpio4 => PinmuxOutsel::GpioGpio4,
            PinmuxPeripheralIn::GpioGpio5 => PinmuxOutsel::GpioGpio5,
            PinmuxPeripheralIn::GpioGpio6 => PinmuxOutsel::GpioGpio6,
            PinmuxPeripheralIn::GpioGpio7 => PinmuxOutsel::GpioGpio7,
            PinmuxPeripheralIn::GpioGpio8 => PinmuxOutsel::GpioGpio8,
            PinmuxPeripheralIn::GpioGpio9 => PinmuxOutsel::GpioGpio9,
            PinmuxPeripheralIn::GpioGpio10 => PinmuxOutsel::GpioGpio10,
            PinmuxPeripheralIn::GpioGpio11 => PinmuxOutsel::GpioGpio11,
            PinmuxPeripheralIn::GpioGpio12 => PinmuxOutsel::GpioGpio12,
            PinmuxPeripheralIn::GpioGpio13 => PinmuxOutsel::GpioGpio13,
            PinmuxPeripheralIn::GpioGpio14 => PinmuxOutsel::GpioGpio14,
            PinmuxPeripheralIn::GpioGpio15 => PinmuxOutsel::GpioGpio15,
            PinmuxPeripheralIn::GpioGpio16 => PinmuxOutsel::GpioGpio16,
            PinmuxPeripheralIn::GpioGpio17 => PinmuxOutsel::GpioGpio17,
            PinmuxPeripheralIn::GpioGpio18 => PinmuxOutsel::GpioGpio18,
            PinmuxPeripheralIn::GpioGpio19 => PinmuxOutsel::GpioGpio19,
            PinmuxPeripheralIn::GpioGpio20 => PinmuxOutsel::GpioGpio20,
            PinmuxPeripheralIn::GpioGpio21 => PinmuxOutsel::GpioGpio21,
            PinmuxPeripheralIn::GpioGpio22 => PinmuxOutsel::GpioGpio22,
            PinmuxPeripheralIn::GpioGpio23 => PinmuxOutsel::GpioGpio23,
            PinmuxPeripheralIn::GpioGpio24 => PinmuxOutsel::GpioGpio24,
            PinmuxPeripheralIn::GpioGpio25 => PinmuxOutsel::GpioGpio25,
            PinmuxPeripheralIn::GpioGpio26 => PinmuxOutsel::GpioGpio26,
            PinmuxPeripheralIn::GpioGpio27 => PinmuxOutsel::GpioGpio27,
            PinmuxPeripheralIn::GpioGpio28 => PinmuxOutsel::GpioGpio28,
            PinmuxPeripheralIn::GpioGpio29 => PinmuxOutsel::GpioGpio29,
            PinmuxPeripheralIn::GpioGpio30 => PinmuxOutsel::GpioGpio30,
            PinmuxPeripheralIn::GpioGpio31 => PinmuxOutsel::GpioGpio31,
            _ => PinmuxOutsel::ConstantHighZ,
        }
    }
}

// This function use extract GPIO mapping from initial pinmux configurations
pub fn gpio_pad_config<Layout: EarlGreyPinmuxConfig>(pin: PinmuxPeripheralIn) -> PadConfig {
    match Layout::INPUT[pin as usize] {
        // Current implementation don't support Output only GPIO
        PinmuxInsel::ConstantZero | PinmuxInsel::ConstantOne => PadConfig::Unconnected,
        input_selector => {
            if let Ok(pad) = MuxedPads::try_from(
                input_selector as u32 - PINMUX_MIO_PERIPH_INSEL_IDX_OFFSET as u32,
            ) {
                let out: PinmuxOutsel = Layout::OUTPUT[pad as usize];
                // Checking for bi-directional I/O
                if out == PinmuxOutsel::from(pin) {
                    PadConfig::InOut(pad, pin, out)
                } else {
                    PadConfig::Input(pad, pin)
                }
            } else {
                // Upper match checked for unconnected pad so in this
                // place we probably have some invalid value in INPUT array.
                PadConfig::Unconnected
            }
        }
    }
}

// Configuring first all GPIO based on board layout
impl Port<'_> {
    pub fn new<Layout: EarlGreyPinmuxConfig>() -> Self {
        Self {
            // Intentionally prevent splitting GpioPin to multiple line
            #[rustfmt::skip]
            pins: [
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio0), pins::pin0),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio1), pins::pin1),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio2), pins::pin2),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio3), pins::pin3),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio4), pins::pin4),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio5), pins::pin5),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio6), pins::pin6),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio7), pins::pin7),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio8), pins::pin8),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio9), pins::pin9),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio10), pins::pin10),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio11), pins::pin11),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio12), pins::pin12),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio13), pins::pin13),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio14), pins::pin14),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio15), pins::pin15),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio16), pins::pin16),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio17), pins::pin17),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio18), pins::pin18),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio19), pins::pin19),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio20), pins::pin20),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio21), pins::pin21),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio22), pins::pin22),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio23), pins::pin23),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio24), pins::pin24),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio25), pins::pin25),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio26), pins::pin26),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio27), pins::pin27),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio28), pins::pin28),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio29), pins::pin29),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio30), pins::pin30),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(PinmuxPeripheralIn::GpioGpio31), pins::pin31),
            ],
        }
    }
}

impl<'a> Index<usize> for Port<'a> {
    type Output = GpioPin<'a, PadConfig>;

    fn index(&self, index: usize) -> &GpioPin<'a, PadConfig> {
        &self.pins[index]
    }
}

impl<'a> IndexMut<usize> for Port<'a> {
    fn index_mut(&mut self, index: usize) -> &mut GpioPin<'a, PadConfig> {
        &mut self.pins[index]
    }
}
