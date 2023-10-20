// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! GPIO instantiation.

use core::ops::{Index, IndexMut};

use kernel::utilities::StaticRef;
use lowrisc::gpio::GpioRegisters;
pub use lowrisc::gpio::{pins, GpioBitfield, GpioPin};

use crate::pinmux::PadConfig;
use crate::registers::top_earlgrey::GPIO_BASE_ADDR;
use crate::registers::top_earlgrey::{
    MuxedPads, PinmuxOutsel, PinmuxPeripheralIn, PINMUX_PERIPH_OUTSEL_IDX_OFFSET,
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

pub fn gpio_pad_config(pad: MuxedPads, pin: GpioBitfield) -> PadConfig {
    PadConfig::InOut(pad, PinmuxPeripheralIn::from(pin), PinmuxOutsel::from(pin))
}

// Configuring first 32 pad as GPIO
impl<'a> Port<'a> {
    pub const fn new() -> Self {
        Self {
            // Intentionally prevent splitting GpioPin to multi line definition
            #[rustfmt::skip]
            pins: [
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa0, PinmuxPeripheralIn::GpioGpio0, PinmuxOutsel::GpioGpio0), pins::pin0),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa1, PinmuxPeripheralIn::GpioGpio1, PinmuxOutsel::GpioGpio1), pins::pin1),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa2, PinmuxPeripheralIn::GpioGpio2, PinmuxOutsel::GpioGpio2), pins::pin2),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa3, PinmuxPeripheralIn::GpioGpio3, PinmuxOutsel::GpioGpio3), pins::pin3),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa4, PinmuxPeripheralIn::GpioGpio4, PinmuxOutsel::GpioGpio4), pins::pin4),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa5, PinmuxPeripheralIn::GpioGpio5, PinmuxOutsel::GpioGpio5), pins::pin5),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa6, PinmuxPeripheralIn::GpioGpio6, PinmuxOutsel::GpioGpio6), pins::pin6),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa7, PinmuxPeripheralIn::GpioGpio7, PinmuxOutsel::GpioGpio7), pins::pin7),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioa8, PinmuxPeripheralIn::GpioGpio8, PinmuxOutsel::GpioGpio8), pins::pin8),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob0, PinmuxPeripheralIn::GpioGpio9, PinmuxOutsel::GpioGpio9), pins::pin9),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob1, PinmuxPeripheralIn::GpioGpio10, PinmuxOutsel::GpioGpio10), pins::pin10),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob2, PinmuxPeripheralIn::GpioGpio11, PinmuxOutsel::GpioGpio11), pins::pin11),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob3, PinmuxPeripheralIn::GpioGpio12, PinmuxOutsel::GpioGpio12), pins::pin12),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob4, PinmuxPeripheralIn::GpioGpio13, PinmuxOutsel::GpioGpio13), pins::pin13),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob5, PinmuxPeripheralIn::GpioGpio14, PinmuxOutsel::GpioGpio14), pins::pin14),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob6, PinmuxPeripheralIn::GpioGpio15, PinmuxOutsel::GpioGpio15), pins::pin15),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob7, PinmuxPeripheralIn::GpioGpio16, PinmuxOutsel::GpioGpio16), pins::pin16),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob8, PinmuxPeripheralIn::GpioGpio17, PinmuxOutsel::GpioGpio17), pins::pin17),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob9, PinmuxPeripheralIn::GpioGpio18, PinmuxOutsel::GpioGpio18), pins::pin18),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob10, PinmuxPeripheralIn::GpioGpio19, PinmuxOutsel::GpioGpio19), pins::pin19),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob11, PinmuxPeripheralIn::GpioGpio20, PinmuxOutsel::GpioGpio20), pins::pin20),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Iob12, PinmuxPeripheralIn::GpioGpio21, PinmuxOutsel::GpioGpio21), pins::pin21),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc0, PinmuxPeripheralIn::GpioGpio22, PinmuxOutsel::GpioGpio22), pins::pin22),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc1, PinmuxPeripheralIn::GpioGpio23, PinmuxOutsel::GpioGpio23), pins::pin23),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc2, PinmuxPeripheralIn::GpioGpio24, PinmuxOutsel::GpioGpio24), pins::pin24),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc3, PinmuxPeripheralIn::GpioGpio25, PinmuxOutsel::GpioGpio25), pins::pin25),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc4, PinmuxPeripheralIn::GpioGpio26, PinmuxOutsel::GpioGpio26), pins::pin26),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc5, PinmuxPeripheralIn::GpioGpio27, PinmuxOutsel::GpioGpio27), pins::pin27),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc6, PinmuxPeripheralIn::GpioGpio28, PinmuxOutsel::GpioGpio28), pins::pin28),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc7, PinmuxPeripheralIn::GpioGpio29, PinmuxOutsel::GpioGpio29), pins::pin29),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc8, PinmuxPeripheralIn::GpioGpio30, PinmuxOutsel::GpioGpio30), pins::pin30),
                GpioPin::new(GPIO_BASE, PadConfig::InOut(MuxedPads::Ioc9, PinmuxPeripheralIn::GpioGpio31, PinmuxOutsel::GpioGpio31), pins::pin31),
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
