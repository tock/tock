// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::chip_specs::Stm32f429Specs;
use stm32f4xx::chip::Stm32f4xxDefaultPeripherals;

use crate::{can_registers, stm32f429zi_nvic, trng_registers};

pub struct Stm32f429ziDefaultPeripherals<'a> {
    pub stm32f4: Stm32f4xxDefaultPeripherals<'a, Stm32f429Specs>,
    // Once implemented, place Stm32f429zi specific peripherals here
    pub trng: stm32f4xx::trng::Trng<'a>,
    pub can1: stm32f4xx::can::Can<'a>,
    pub rtc: crate::rtc::Rtc<'a>,
}

impl<'a> Stm32f429ziDefaultPeripherals<'a> {
    pub unsafe fn new(
        clocks: &'a crate::clocks::Clocks<'a, Stm32f429Specs>,
        exti: &'a crate::exti::Exti<'a>,
        dma1: &'a crate::dma::Dma1<'a>,
        dma2: &'a crate::dma::Dma2<'a>,
    ) -> Self {
        Self {
            stm32f4: Stm32f4xxDefaultPeripherals::new(clocks, exti, dma1, dma2),
            trng: stm32f4xx::trng::Trng::new(trng_registers::RNG_BASE, clocks),
            can1: stm32f4xx::can::Can::new(clocks, can_registers::CAN1_BASE),
            rtc: crate::rtc::Rtc::new(clocks),
        }
    }
    // Necessary for setting up circular dependencies and registering deferred calls
    pub fn init(&'static self) {
        self.stm32f4.setup_circular_deps();
        kernel::deferred_call::DeferredCallClient::register(&self.can1);
        kernel::deferred_call::DeferredCallClient::register(&self.rtc);
    }
}
impl<'a> kernel::platform::chip::InterruptService for Stm32f429ziDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            // put Stm32f429zi specific interrupts here
            stm32f429zi_nvic::HASH_RNG => {
                self.trng.handle_interrupt();
                true
            }
            stm32f4xx::nvic::CAN1_TX => {
                self.can1.handle_transmit_interrupt();
                true
            }
            stm32f4xx::nvic::CAN1_RX0 => {
                self.can1.handle_fifo0_interrupt();
                true
            }
            stm32f4xx::nvic::CAN1_RX1 => {
                self.can1.handle_fifo1_interrupt();
                true
            }
            stm32f4xx::nvic::CAN1_SCE => {
                self.can1.handle_error_status_interrupt();
                true
            }
            _ => self.stm32f4.service_interrupt(interrupt),
        }
    }
}
