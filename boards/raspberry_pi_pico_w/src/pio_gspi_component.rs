// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::mem::MaybeUninit;
use kernel::component::Component;
use rp2040::gpio::{RPGpio, RPGpioPin};
use rp2040::pio_gspi::PioGSpi;
use rp2040::{dma, pio, pio_gspi};

macro_rules! pio_gpsi_component_static {
    () => {{
        kernel::static_buf!(rp2040::pio_gspi::PioGSpi<'static>)
    }};
}
pub(super) use pio_gpsi_component_static;

pub struct PioGspiComponent {
    dma_channel: &'static dma::DmaChannel<'static>,
    dma_irq: dma::Irq,
    pio: &'static pio::Pio,
    pio_sm: pio::SMNumber,
    clk: RPGpio,
    dio: RPGpio,
    cs: &'static RPGpioPin<'static>,
}

impl PioGspiComponent {
    pub fn new(
        pio: &'static pio::Pio,
        pio_sm: pio::SMNumber,
        dma_channel: &'static dma::DmaChannel<'static>,
        dma_irq: dma::Irq,
        clk: RPGpio,
        dio: RPGpio,
        cs: &'static RPGpioPin<'static>,
    ) -> Self {
        Self {
            dma_channel,
            dma_irq,
            pio,
            pio_sm,
            clk,
            dio,
            cs,
        }
    }
}

impl Component for PioGspiComponent {
    type StaticInput = &'static mut MaybeUninit<pio_gspi::PioGSpi<'static>>;

    type Output = &'static pio_gspi::PioGSpi<'static>;

    fn finalize(self, static_memory: Self::StaticInput) -> Self::Output {
        self.dma_channel.enable_interrupt(self.dma_irq);

        let pio_gspi = static_memory.write(PioGSpi::new(
            self.pio,
            self.dma_channel,
            self.clk as _,
            self.dio as _,
            self.cs,
            self.pio_sm,
        ));

        self.dma_channel.set_client(pio_gspi);
        pio_gspi.init();
        self.pio.sm(self.pio_sm).set_sm_client(pio_gspi);

        pio_gspi
    }
}
