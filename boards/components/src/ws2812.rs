// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the WS2812 LED.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

/// T0H clock divider for an ESP32-C3 with a 160MHz clock speed.
pub const ESP32C3_160MHZ_T0H: usize = 3;

#[macro_export]
macro_rules! ws2812_component_static {
    ($P:ty, $T0H:expr $(,)?) => {
        kernel::static_buf!(capsules_extra::ws2812::Ws2812<'static, $P, $T0H>)
    };
}

/// Custom version of the static for the ESP32-C3 board with a fixed clock speed.
#[macro_export]
macro_rules! ws2812_component_static_esp32c3_160mhz {
    ($P:ty $(,)?) => {
        $crate::ws2812_component_static!($P, { $crate::ws2812::ESP32C3_160MHZ_T0H })
    };
}

pub struct Ws2812Component<P: 'static + hil::gpio::Pin, const T0H: usize> {
    led_pin: &'static P,
    nop: fn(),
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Ws2812Component<P, T0H> {
    pub fn new(led_pin: &'static P, nop: fn()) -> Self {
        Self { led_pin, nop }
    }
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Component for Ws2812Component<P, T0H> {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::ws2812::Ws2812<'static, P, T0H>>;
    type Output = &'static capsules_extra::ws2812::Ws2812<'static, P, T0H>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ws2812 =
            static_buffer.write(capsules_extra::ws2812::Ws2812::new(self.led_pin, self.nop));
        ws2812.init();
        ws2812
    }
}

#[macro_export]
macro_rules! ws2812_led_component_static {
    ($P:ty,  $T0H:expr $(,)?) => {
        kernel::static_buf!(capsules_extra::ws2812::Ws2812Led<'static, $P, $T0H>)
    };
}

/// Custom version of the static for the ESP32-C3 board with a fixed clock speed.
#[macro_export]
macro_rules! ws2812_led_component_static_esp32c3_160mhz {
    ($P:ty $(,)?) => {
        $crate::ws2812_led_component_static!($P, { $crate::ws2812::ESP32C3_160MHZ_T0H })
    };
}

pub type Ws2812LedComponentType<P, const T0H: usize> =
    capsules_extra::ws2812::Ws2812Led<'static, P, T0H>;

pub struct Ws2812LedComponent<P: 'static + hil::gpio::Pin, const T0H: usize> {
    ws2812: &'static capsules_extra::ws2812::Ws2812<'static, P, T0H>,
    index: usize,
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Ws2812LedComponent<P, T0H> {
    pub fn new(
        ws2812: &'static capsules_extra::ws2812::Ws2812<'static, P, T0H>,
        index: usize,
    ) -> Self {
        Self { ws2812, index }
    }
}

impl<P: 'static + hil::gpio::Pin, const T0H: usize> Component for Ws2812LedComponent<P, T0H> {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::ws2812::Ws2812Led<'static, P, T0H>>;
    type Output = &'static capsules_extra::ws2812::Ws2812Led<'static, P, T0H>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ws2812_led = static_buffer.write(capsules_extra::ws2812::Ws2812Led::new(
            self.ws2812,
            self.index,
        ));
        ws2812_led
    }
}
