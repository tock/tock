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
    PINMUX_PERIPH_OUTSEL_IDX_OFFSET,
};

pub const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(GPIO_BASE_ADDR as *const GpioRegisters) };

pub struct Port<'a> {
    pins: [GpioPin<'a, PadConfig>; 32],
}

impl From<GpioBitfield> for PinmuxPeripheralIn {
    fn from(pin: GpioBitfield) -> PinmuxPeripheralIn {
        // We used fact that first 0-31 values are directly maped to GPIO
        Self::try_from(pin.shift as u32).unwrap()
    }
}

impl From<GpioBitfield> for PinmuxOutsel {
    fn from(pin: GpioBitfield) -> Self {
        // We skip first 3 constans to convert value to output selector
        match Self::try_from(pin.shift as u32 + PINMUX_PERIPH_OUTSEL_IDX_OFFSET as u32) {
            Ok(outsel) => outsel,
            Err(_) => PinmuxOutsel::ConstantHighZ,
        }
    }
}

// This function use extract GPIO mapping from initial pinmux configurations
pub fn gpio_pad_config<Layout: EarlGreyPinmuxConfig>(pin: GpioBitfield) -> PadConfig {
    let inp: PinmuxPeripheralIn = PinmuxPeripheralIn::from(pin);
    match Layout::INPUT[inp as usize] {
        // Current implementation don't support Output only GPIO
        PinmuxInsel::ConstantZero | PinmuxInsel::ConstantOne => PadConfig::Unconnected,
        input_selector => {
            if let Ok(pad) = MuxedPads::try_from(
                input_selector as u32 - PINMUX_MIO_PERIPH_INSEL_IDX_OFFSET as u32,
            ) {
                let out: PinmuxOutsel = Layout::OUTPUT[pad as usize];
                // Checking for bi-directional I/O
                if out == PinmuxOutsel::from(pin) {
                    PadConfig::InOut(pad, inp, out)
                } else {
                    PadConfig::Input(pad, inp)
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
impl<'a> Port<'a> {
    pub fn new<Layout: EarlGreyPinmuxConfig>() -> Self {
        Self {
            // Intentionally prevent splitting GpioPin to multiple line
            #[rustfmt::skip]
            pins: [
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin0), pins::pin0),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin1), pins::pin1),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin2), pins::pin2),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin3), pins::pin3),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin4), pins::pin4),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin5), pins::pin5),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin6), pins::pin6),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin7), pins::pin7),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin8), pins::pin8),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin9), pins::pin9),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin10), pins::pin10),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin11), pins::pin11),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin12), pins::pin12),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin13), pins::pin13),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin14), pins::pin14),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin15), pins::pin15),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin16), pins::pin16),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin17), pins::pin17),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin18), pins::pin18),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin19), pins::pin19),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin20), pins::pin20),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin21), pins::pin21),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin22), pins::pin22),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin23), pins::pin23),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin24), pins::pin24),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin25), pins::pin25),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin26), pins::pin26),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin27), pins::pin27),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin28), pins::pin28),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin29), pins::pin29),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin30), pins::pin30),
                GpioPin::new(GPIO_BASE, gpio_pad_config::<Layout>(pins::pin31), pins::pin31),
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
