// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the sk68xx LED.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

/// T0H clock divider for an ESP32-C3 with a 160MHz clock speed.
pub const ESP32C3_160MHZ_T0H: usize = 3;

#[macro_export]
macro_rules! sk68xx_component_static {
    ($P:ty, $T0H:expr $(,)?) => {
        kernel::static_buf!(capsules_extra::sk68xx::Sk68xx<'static, $P, $T0H>)
    };
}

/// Custom version of the static for the ESP32-C3 board with a fixed clock speed.
#[macro_export]
macro_rules! sk68xx_component_static_esp32c3_160mhz {
    ($P:ty $(,)?) => {
        $crate::sk68xx_component_static!($P, { $crate::sk68xx::ESP32C3_160MHZ_T0H })
    };
}

pub struct Sk68xxComponent<P: 'static + hil::gpio::Pin, const T0H: usize> {
    led_pin: &'static P,
    nop: fn(),
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Sk68xxComponent<P, T0H> {
    pub fn new(led_pin: &'static P, nop: fn()) -> Self {
        Self { led_pin, nop }
    }
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Component for Sk68xxComponent<P, T0H> {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::sk68xx::Sk68xx<'static, P, T0H>>;
    type Output = &'static capsules_extra::sk68xx::Sk68xx<'static, P, T0H>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let sk68xx =
            static_buffer.write(capsules_extra::sk68xx::Sk68xx::new(self.led_pin, self.nop));
        sk68xx.init();
        sk68xx
    }
}

#[macro_export]
macro_rules! sk68xx_led_component_static {
    ($P:ty,  $T0H:expr $(,)?) => {
        kernel::static_buf!(capsules_extra::sk68xx::Sk68xxLed<'static, $P, $T0H>)
    };
}

/// Custom version of the static for the ESP32-C3 board with a fixed clock speed.
#[macro_export]
macro_rules! sk68xx_led_component_static_esp32c3_160mhz {
    ($P:ty $(,)?) => {
        $crate::sk68xx_led_component_static!($P, { $crate::sk68xx::ESP32C3_160MHZ_T0H })
    };
}

pub type Sk68xxLedComponentType<P, const T0H: usize> =
    capsules_extra::sk68xx::Sk68xxLed<'static, P, T0H>;

pub struct Sk68xxLedComponent<P: 'static + hil::gpio::Pin, const T0H: usize> {
    sk68xx: &'static capsules_extra::sk68xx::Sk68xx<'static, P, T0H>,
    index: usize,
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Sk68xxLedComponent<P, T0H> {
    pub fn new(
        sk68xx: &'static capsules_extra::sk68xx::Sk68xx<'static, P, T0H>,
        index: usize,
    ) -> Self {
        Self { sk68xx, index }
    }
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Component for Sk68xxLedComponent<P, T0H> {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::sk68xx::Sk68xxLed<'static, P, T0H>>;
    type Output = &'static capsules_extra::sk68xx::Sk68xxLed<'static, P, T0H>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let sk68xx_led = static_buffer.write(capsules_extra::sk68xx::Sk68xxLed::new(
            self.sk68xx,
            self.index,
        ));
        sk68xx_led
    }
}
