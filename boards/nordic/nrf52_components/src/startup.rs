// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for starting up nrf52 platforms.
//! Contains 3 components, NrfStartupComponent, NrfClockComponent,
//! and UartChannelComponent, as well as two helper structs for
//! intializing Uart on Nordic boards.

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use nrf52::gpio::Pin;
use nrf52::uicr::Regulator0Output;
use segger::rtt::SeggerRtt;

pub struct NrfStartupComponent<'a> {
    nfc_as_gpios: bool,
    button_rst_pin: Pin,
    reg_vout: Regulator0Output,
    nvmc: &'a nrf52::nvmc::Nvmc,
}

impl<'a> NrfStartupComponent<'a> {
    pub fn new(
        nfc_as_gpios: bool,
        button_rst_pin: Pin,
        reg_vout: Regulator0Output,
        nvmc: &'a nrf52::nvmc::Nvmc,
    ) -> Self {
        Self {
            nfc_as_gpios,
            button_rst_pin,
            reg_vout,
            nvmc,
        }
    }
}

impl Component for NrfStartupComponent<'_> {
    type StaticInput = ();
    type Output = ();
    fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        // Disable APPROTECT in software. This is required as of newer nRF52
        // hardware revisions. See
        // https://devzone.nordicsemi.com/nordic/nordic-blog/b/blog/posts/working-with-the-nrf52-series-improved-approtect.
        // If run on older HW revisions this function will do nothing.
        let approtect = nrf52::approtect::Approtect::new();
        approtect.sw_disable_approtect();

        // Make non-volatile memory writable and activate the reset button
        let uicr = nrf52::uicr::Uicr::new();

        // Check if we need to erase UICR memory to re-program it
        // This only needs to be done when a bit needs to be flipped from 0 to 1.
        let psel0_reset: u32 = uicr.get_psel0_reset_pin().map_or(0, |pin| pin as u32);
        let psel1_reset: u32 = uicr.get_psel1_reset_pin().map_or(0, |pin| pin as u32);
        let mut erase_uicr = ((!psel0_reset & (self.button_rst_pin as u32))
            | (!psel1_reset & (self.button_rst_pin as u32))
            | (!(uicr.get_vout() as u32) & (self.reg_vout as u32)))
            != 0;

        // Only enabling the NFC pin protection requires an erase.
        if self.nfc_as_gpios {
            erase_uicr |= !uicr.is_nfc_pins_protection_enabled();
        }

        // On new nRF52 variants we need to ensure that the APPROTECT field in UICR is
        // set to `HwDisable`.
        if uicr.is_ap_protect_enabled() {
            erase_uicr = true;
        }

        if erase_uicr {
            self.nvmc.erase_uicr();
        }

        self.nvmc.configure_writeable();
        while !self.nvmc.is_ready() {}

        let mut needs_soft_reset: bool = false;

        // Configure reset pins
        if uicr
            .get_psel0_reset_pin()
            .is_none_or(|pin| pin != self.button_rst_pin)
        {
            uicr.set_psel0_reset_pin(self.button_rst_pin);
            while !self.nvmc.is_ready() {}
            needs_soft_reset = true;
        }
        if uicr
            .get_psel1_reset_pin()
            .is_none_or(|pin| pin != self.button_rst_pin)
        {
            uicr.set_psel1_reset_pin(self.button_rst_pin);
            while !self.nvmc.is_ready() {}
            needs_soft_reset = true;
        }

        // Configure voltage regulator output
        if uicr.get_vout() != self.reg_vout {
            uicr.set_vout(self.reg_vout);
            while !self.nvmc.is_ready() {}
            needs_soft_reset = true;
        }

        // Check if we need to free the NFC pins for GPIO
        if self.nfc_as_gpios {
            uicr.set_nfc_pins_protection(true);
            while !self.nvmc.is_ready() {}
            needs_soft_reset = true;
        }

        // If APPROTECT was not already disabled, ensure it is set to disabled.
        if uicr.is_ap_protect_enabled() {
            uicr.disable_ap_protect();
            while !self.nvmc.is_ready() {}
            needs_soft_reset = true;
        }

        // Any modification of UICR needs a soft reset for the changes to be taken into account.
        if needs_soft_reset {
            unsafe {
                cortexm4::scb::reset();
            }
        }
    }
}

pub struct NrfClockComponent<'a> {
    clock: &'a nrf52::clock::Clock,
}

impl<'a> NrfClockComponent<'a> {
    pub fn new(clock: &'a nrf52::clock::Clock) -> Self {
        Self { clock }
    }
}

impl Component for NrfClockComponent<'_> {
    type StaticInput = ();
    type Output = ();
    fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        // Start all of the clocks. Low power operation will require a better
        // approach than this.
        self.clock.low_stop();
        self.clock.high_stop();

        self.clock
            .low_set_source(nrf52::clock::LowClockSource::XTAL);
        self.clock.low_start();
        self.clock.high_start();
        while !self.clock.low_started() {}
        while !self.clock.high_started() {}
    }
}

#[macro_export]
macro_rules! uart_channel_component_static {
    ($A:ty $(,)?) => {{
        components::segger_rtt_component_static!($A)
    };};
}

/// Pins for the UART
#[derive(Debug)]
pub struct UartPins {
    rts: Option<Pin>,
    txd: Pin,
    cts: Option<Pin>,
    rxd: Pin,
}

impl UartPins {
    pub fn new(rts: Option<Pin>, txd: Pin, cts: Option<Pin>, rxd: Pin) -> Self {
        Self { rts, txd, cts, rxd }
    }
}

/// Uart chanel representation depends on whether USB debugging is
/// enabled.
pub enum UartChannel<'a> {
    Pins(UartPins),
    Rtt(components::segger_rtt::SeggerRttMemoryRefs<'a>),
}

pub struct UartChannelComponent {
    uart_channel: UartChannel<'static>,
    mux_alarm: &'static MuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    uarte0: &'static nrf52::uart::Uarte<'static>,
}

impl UartChannelComponent {
    pub fn new(
        uart_channel: UartChannel<'static>,
        mux_alarm: &'static MuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
        uarte0: &'static nrf52::uart::Uarte<'static>,
    ) -> Self {
        Self {
            uart_channel,
            mux_alarm,
            uarte0,
        }
    }
}

impl Component for UartChannelComponent {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>>,
        &'static mut MaybeUninit<
            SeggerRtt<'static, VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>>,
        >,
    );
    type Output = &'static dyn kernel::hil::uart::Uart<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        match self.uart_channel {
            UartChannel::Pins(uart_pins) => {
                unsafe {
                    self.uarte0.initialize(
                        nrf52::pinmux::Pinmux::new(uart_pins.txd as u32),
                        nrf52::pinmux::Pinmux::new(uart_pins.rxd as u32),
                        uart_pins.cts.map(|x| nrf52::pinmux::Pinmux::new(x as u32)),
                        uart_pins.rts.map(|x| nrf52::pinmux::Pinmux::new(x as u32)),
                    )
                };
                self.uarte0
            }
            UartChannel::Rtt(rtt_memory) => {
                let rtt =
                    components::segger_rtt::SeggerRttComponent::new(self.mux_alarm, rtt_memory)
                        .finalize(s);
                rtt
            }
        }
    }
}
