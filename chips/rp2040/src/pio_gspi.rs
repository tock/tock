// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! PIO gSPI (generic SPI) support

use crate::dma::{self, DmaChannel, DmaChannelClient};
use crate::gpio::RPGpioPin;
use crate::pio::{Pio, PioSmClient, SMNumber, StateMachineConfiguration};
use kernel::hil::gpio::{self, Output as _};
use kernel::hil::spi::{self, SpiMasterDevice};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

/// The PIO program
const PROG: [u16; 11] = [
    0x6020, //  0: out    x, 32           side 0
    0x6040, //  1: out    y, 32           side 0
    0xe081, //  2: set    pindirs, 1      side 0
    //     .wrap_target
    0x6001, //  3: out    pins, 1         side 0
    0x1043, //  4: jmp    x--, 3          side 1
    0xe080, //  5: set    pindirs, 0      side 0
    0xa042, //  6: nop                    side 0
    0x5001, //  7: in     pins, 1         side 1
    0x0087, //  8: jmp    y--, 7          side 0
    0x20a0, //  9: wait   1 pin, 0        side 0
    0xc000, // 10: irq    nowait 0        side 0
            //     .wrap
];

/// The gSPI PIO peripheral driver
pub struct PioGSpi<'a> {
    pio: &'a Pio,
    dma: &'a DmaChannel<'a>,
    clock_pin: u32,
    dio_pin: u32,
    cs_pin: RPGpioPin<'a>,
    sm_number: SMNumber,
    write_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    read_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    client: OptionalCell<&'a dyn spi::SpiMasterClient>,
    irq_client: OptionalCell<&'a dyn gpio::Client>,
    pending: OptionalCell<Pending>,
}

#[derive(Debug)]
enum Pending {
    Write,
    Read,
}

impl<'a> PioGSpi<'a> {
    /// Create a new `PioCyw43Spi` instance
    pub fn new(
        pio: &'a Pio,
        dma: &'a DmaChannel<'a>,
        clock_pin: u32,
        dio_pin: u32,
        cs_pin: RPGpioPin<'a>,
        sm_number: SMNumber,
    ) -> Self {
        Self {
            pio,
            dma,
            clock_pin,
            dio_pin,
            cs_pin,
            sm_number,
            pending: OptionalCell::empty(),
            client: OptionalCell::empty(),
            irq_client: OptionalCell::empty(),
            write_buffer: OptionalCell::empty(),
            read_buffer: OptionalCell::empty(),
        }
    }

    /// Return SM number
    pub fn sm_number(&self) -> SMNumber {
        self.sm_number
    }

    pub fn set_irq_client(&self, client: &'a dyn gpio::Client) {
        self.irq_client.set(client);
    }

    /// Configure the PIO peripheral as an SPI device to communicate with the CYW43439 chip
    pub fn init(&self) {
        self.pio.init();
        self.cs_pin.set();

        // load program
        self.pio.add_program16(None::<usize>, &PROG).unwrap();

        let config = StateMachineConfiguration {
            out_pins_count: 1,
            out_pins_base: self.dio_pin,
            set_pins_count: 1,
            set_pins_base: self.dio_pin,
            in_pins_base: self.dio_pin,
            side_set_base: self.clock_pin,
            side_set_bit_count: 1,
            in_push_threshold: 0,
            out_pull_threshold: 0,
            div_int: 2u32,
            div_frac: 0u32,
            wrap: 10,
            wrap_to: 3,
            in_autopush: true,
            out_autopull: true,
            in_shift_direction_right: false,
            out_shift_direction_right: false,
            ..Default::default()
        };

        self.pio
            .cyw43_spi_program_init(self.sm_number, self.clock_pin, self.dio_pin, &config);

        self.pio
            .set_irq_source(0, crate::pio::InterruptSources::Interrupt0, true);
    }
}

impl DmaChannelClient for PioGSpi<'_> {
    fn transfer_done(&self) {
        let Some(pending) = self.pending.take() else {
            return;
        };

        if let Pending::Write = pending {
            let read_buffer = self.read_buffer.take();
            if let Some(read_buffer) = read_buffer {
                self.dma_pull(read_buffer.as_ptr() as u32, read_buffer.len() as u32 / 4);
                self.pending.set(Pending::Read);
                self.read_buffer.set(read_buffer);
                return;
            }
        }

        let write_buffer = self.write_buffer.take().unwrap();
        let len = write_buffer.len();

        self.cs_pin.set();
        self.client
            .map(|client| client.read_write_done(write_buffer, self.read_buffer.take(), Ok(len)));
    }
}

impl PioGSpi<'_> {
    fn dma_pull(&self, addr: u32, len: u32) {
        assert!(addr % 4 == 0);
        let current_sm = self.pio.sm(self.sm_number);
        self.dma
            .set_read_addr(current_sm.rxf_addr(self.pio.number()));
        self.dma.set_write_addr(addr);

        self.dma.set_len(len);
        self.dma.enable(
            dma::DmaPeripheral::PioRxFifo(self.pio.number(), self.sm_number),
            dma::DataSize::Word,
            dma::Transfer::PeripheralToMemory,
            false,
        );
    }

    fn dma_push(&self, addr: u32, len: u32) {
        assert!(addr % 4 == 0);
        let current_sm = self.pio.sm(self.sm_number);
        self.dma.set_read_addr(addr);
        self.dma
            .set_write_addr(current_sm.txf_addr(self.pio.number()));

        self.dma.set_len(len);

        self.dma.enable(
            dma::DmaPeripheral::PioTxFifo(self.pio.number(), self.sm_number),
            dma::DataSize::Word,
            dma::Transfer::MemoryToPeripheral,
            false,
        );
    }
}

impl<'a> SpiMasterDevice<'a> for PioGSpi<'a> {
    fn set_client(&self, client: &'a dyn spi::SpiMasterClient) {
        self.client.set(client);
    }

    fn read_write_bytes(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    > {
        assert!(write_buffer.len() % 4 == 0);

        let current_sm = self.pio.sm(self.sm_number);
        self.cs_pin.clear();
        current_sm.set_enabled(false);

        // Try to push the number of bits
        let write_bits = (write_buffer.len() as u32) * 8 - 1;
        let read_bits = read_buffer
            .as_ref()
            .map(|rx| rx.len() * 8 - 1)
            .unwrap_or_default();

        current_sm.push_blocking(write_bits).unwrap();
        current_sm.push_blocking(read_bits as _).unwrap();

        current_sm.exec(0);
        current_sm.set_enabled(true);

        self.dma_push(write_buffer.as_ptr() as u32, write_buffer.len() as u32 / 4);

        self.write_buffer.set(write_buffer);
        self.read_buffer.insert(read_buffer);
        self.pending.set(Pending::Write);

        Ok(())
    }

    fn set_rate(&self, _: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_polarity(&self, _: kernel::hil::spi::ClockPolarity) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_phase(&self, _: kernel::hil::spi::ClockPhase) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn configure(
        &self,
        _: kernel::hil::spi::ClockPolarity,
        _: kernel::hil::spi::ClockPhase,
        _: u32,
    ) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_polarity(&self) -> kernel::hil::spi::ClockPolarity {
        kernel::hil::spi::ClockPolarity::IdleLow
    }

    fn get_phase(&self) -> kernel::hil::spi::ClockPhase {
        kernel::hil::spi::ClockPhase::SampleLeading
    }

    fn get_rate(&self) -> u32 {
        0
    }
}

impl PioSmClient for PioGSpi<'_> {
    fn on_irq(&self) {
        // Clear interrupt
        self.pio.interrupt_clear(0);
        self.irq_client.map(|client| client.fired());
    }
}
