// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the WS2812 LED.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! ws2812_component_static {
    ($P:ty $(,)?) => {
        kernel::static_buf!(capsules_extra::ws2812::Ws2812<'static, $P>)
    };
}

pub struct Ws2812Component<P: 'static + hil::gpio::Pin> {
    led_pin: &'static P,
    write_frame: fn(&P, u32),
}

impl<P: 'static + hil::gpio::Pin> Ws2812Component<P> {
    pub fn new(led_pin: &'static P, write_frame: fn(&P, u32)) -> Self {
        Self {
            led_pin,
            write_frame,
        }
    }
}

impl<P: 'static + hil::gpio::Pin> Component for Ws2812Component<P> {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::ws2812::Ws2812<'static, P>>;
    type Output = &'static capsules_extra::ws2812::Ws2812<'static, P>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ws2812 = static_buffer.write(capsules_extra::ws2812::Ws2812::new(
            self.led_pin,
            self.write_frame,
        ));
        ws2812.init();
        ws2812
    }
}

#[macro_export]
macro_rules! ws2812_led_component_static {
    ($P:ty $(,)?) => {
        kernel::static_buf!(capsules_extra::ws2812::Ws2812Led<'static, $P>)
    };
}

pub type Ws2812LedComponentType<P> = capsules_extra::ws2812::Ws2812Led<'static, P>;

pub struct Ws2812LedComponent<P: 'static + hil::gpio::Pin> {
    ws2812: &'static capsules_extra::ws2812::Ws2812<'static, P>,
    index: usize,
}

impl<P: 'static + hil::gpio::Pin> Ws2812LedComponent<P> {
    pub fn new(ws2812: &'static capsules_extra::ws2812::Ws2812<'static, P>, index: usize) -> Self {
        Self { ws2812, index }
    }
}

impl<P: 'static + hil::gpio::Pin> Component for Ws2812LedComponent<P> {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::ws2812::Ws2812Led<'static, P>>;
    type Output = &'static capsules_extra::ws2812::Ws2812Led<'static, P>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ws2812_led = static_buffer.write(capsules_extra::ws2812::Ws2812Led::new(
            self.ws2812,
            self.index,
        ));
        ws2812_led
    }
}
