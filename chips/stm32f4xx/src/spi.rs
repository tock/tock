// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::cell::Cell;
use core::cmp;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

use kernel::hil;
use kernel::hil::gpio::Output;
use kernel::hil::spi::{self, ClockPhase, ClockPolarity, SpiMasterClient};
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

use crate::clocks::phclk;
use crate::dma;
use crate::dma::{Dma1, Dma1Peripheral};

/// Serial peripheral interface
#[repr(C)]
pub struct SpiRegisters {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// status register
    sr: ReadWrite<u32, SR::Register>,
    /// data register
    dr: ReadWrite<u32, DR::Register>,
    /// CRC polynomial register
    crcpr: ReadWrite<u32>,
    /// RX CRC register
    rxcrcr: ReadOnly<u32>,
    /// TX CRC register
    txcrcr: ReadOnly<u32>,
    /// I2S configuration register
    i2scfgr: ReadWrite<u32, I2SCFGR::Register>,
    /// I2S prescaler register
    i2spr: ReadWrite<u32, I2SPR::Register>,
}

register_bitfields![u32,
    CR1 [
        /// Bidirectional data mode enable
        BIDIMODE OFFSET(15) NUMBITS(1) [],
        /// Output enable in bidirectional mode
        BIDIOE OFFSET(14) NUMBITS(1) [],
        /// Hardware CRC calculation enable
        CRCEN OFFSET(13) NUMBITS(1) [],
        /// CRC transfer next
        CRCNEXT OFFSET(12) NUMBITS(1) [],
        /// Data frame format
        DFF OFFSET(11) NUMBITS(1) [],
        /// Receive only
        RXONLY OFFSET(10) NUMBITS(1) [],
        /// Software slave management
        SSM OFFSET(9) NUMBITS(1) [],
        /// Internal slave select
        SSI OFFSET(8) NUMBITS(1) [],
        /// Frame format
        LSBFIRST OFFSET(7) NUMBITS(1) [],
        /// SPI enable
        SPE OFFSET(6) NUMBITS(1) [],
        /// Baud rate control
        BR OFFSET(3) NUMBITS(3) [],
        /// Master selection
        MSTR OFFSET(2) NUMBITS(1) [],
        /// Clock polarity
        CPOL OFFSET(1) NUMBITS(1) [],
        /// Clock phase
        CPHA OFFSET(0) NUMBITS(1) []
    ],
    CR2 [
        /// Tx buffer empty interrupt enable
        TXEIE OFFSET(7) NUMBITS(1) [],
        /// RX buffer not empty interrupt enable
        RXNEIE OFFSET(6) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(5) NUMBITS(1) [],
        /// Frame format
        FRF OFFSET(4) NUMBITS(1) [],
        /// SS output enable
        SSOE OFFSET(2) NUMBITS(1) [],
        /// Tx buffer DMA enable
        TXDMAEN OFFSET(1) NUMBITS(1) [],
        /// Rx buffer DMA enable
        RXDMAEN OFFSET(0) NUMBITS(1) []
    ],
    SR [
        /// TI frame format error
        TIFRFE OFFSET(8) NUMBITS(1) [],
        /// Busy flag
        BSY OFFSET(7) NUMBITS(1) [],
        /// Overrun flag
        OVR OFFSET(6) NUMBITS(1) [],
        /// Mode fault
        MODF OFFSET(5) NUMBITS(1) [],
        /// CRC error flag
        CRCERR OFFSET(4) NUMBITS(1) [],
        /// Underrun flag
        UDR OFFSET(3) NUMBITS(1) [],
        /// Channel side
        CHSIDE OFFSET(2) NUMBITS(1) [],
        /// Transmit buffer empty
        TXE OFFSET(1) NUMBITS(1) [],
        /// Receive buffer not empty
        RXNE OFFSET(0) NUMBITS(1) []
    ],
    DR [
        /// 8-bit data register
        DR OFFSET(0) NUMBITS(8) []
    ],
    I2SCFGR [
        /// I2S mode selection
        I2SMOD OFFSET(11) NUMBITS(1) [],
        /// I2S Enable
        I2SE OFFSET(10) NUMBITS(1) [],
        /// I2S configuration mode
        I2SCFG OFFSET(8) NUMBITS(2) [],
        /// PCM frame synchronization
        PCMSYNC OFFSET(7) NUMBITS(1) [],
        /// I2S standard selection
        I2SSTD OFFSET(4) NUMBITS(2) [],
        /// Steady state clock polarity
        CKPOL OFFSET(3) NUMBITS(1) [],
        /// Data length to be transferred
        DATLEN OFFSET(1) NUMBITS(2) [],
        /// Channel length (number of bits per audio channel)
        CHLEN OFFSET(0) NUMBITS(1) []
    ],
    I2SPR [
        /// Master clock output enable
        MCKOE OFFSET(9) NUMBITS(1) [],
        /// Odd factor for the prescaler
        ODD OFFSET(8) NUMBITS(1) [],
        /// I2S Linear prescaler
        I2SDIV OFFSET(0) NUMBITS(8) []
    ]
];

// for use by dma1
pub(crate) fn get_address_dr(regs: StaticRef<SpiRegisters>) -> u32 {
    core::ptr::addr_of!(regs.dr) as u32
}

pub const SPI3_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x40003C00 as *const SpiRegisters) };

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,
    clock: SpiClock<'a>,

    // SPI slave support not yet implemented
    master_client: OptionalCell<&'a dyn hil::spi::SpiMasterClient>,

    tx_dma: OptionalCell<&'a dma::Stream<'a, Dma1<'a>>>,
    tx_dma_pid: Dma1Peripheral,
    rx_dma: OptionalCell<&'a dma::Stream<'a, Dma1<'a>>>,
    rx_dma_pid: Dma1Peripheral,

    dma_len: Cell<usize>,
    transfers_in_progress: Cell<u8>,

    active_slave: OptionalCell<&'a crate::gpio::Pin<'a>>,

    active_after: Cell<bool>,
}

// for use by `set_dma`
pub struct TxDMA<'a>(pub &'a dma::Stream<'a, Dma1<'a>>);
pub struct RxDMA<'a>(pub &'a dma::Stream<'a, Dma1<'a>>);

impl<'a> Spi<'a> {
    pub const fn new(
        base_addr: StaticRef<SpiRegisters>,
        clock: SpiClock<'a>,
        tx_dma_pid: Dma1Peripheral,
        rx_dma_pid: Dma1Peripheral,
    ) -> Spi<'a> {
        Spi {
            registers: base_addr,
            clock,

            master_client: OptionalCell::empty(),

            tx_dma: OptionalCell::empty(),
            tx_dma_pid,
            rx_dma: OptionalCell::empty(),
            rx_dma_pid,

            dma_len: Cell::new(0),
            transfers_in_progress: Cell::new(0),

            active_slave: OptionalCell::empty(),

            active_after: Cell::new(false),
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn set_dma(&self, tx_dma: TxDMA<'a>, rx_dma: RxDMA<'a>) {
        self.tx_dma.set(tx_dma.0);
        self.rx_dma.set(rx_dma.0);
    }

    pub fn handle_interrupt(&self) {
        // Used only during debugging. Since we use DMA, we do not enable SPI
        // interrupts during normal operations
    }

    fn set_active_slave(&self, slave_pin: &'a crate::gpio::Pin<'a>) {
        self.active_slave.set(slave_pin);
    }

    fn set_cr<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        self.registers.cr1.modify(CR1::SPE::CLEAR);
        f();
        self.registers.cr1.modify(CR1::SPE::SET);
    }

    // IdleLow  = CPOL = 0
    // IdleHigh = CPOL = 1
    fn set_polarity(&self, polarity: ClockPolarity) {
        self.set_cr(|| match polarity {
            ClockPolarity::IdleLow => self.registers.cr1.modify(CR1::CPOL::CLEAR),
            ClockPolarity::IdleHigh => self.registers.cr1.modify(CR1::CPOL::SET),
        });
    }

    fn get_polarity(&self) -> ClockPolarity {
        if !self.registers.cr1.is_set(CR1::CPOL) {
            ClockPolarity::IdleLow
        } else {
            ClockPolarity::IdleHigh
        }
    }

    // SampleLeading  = CPHA = 0
    // SampleTrailing = CPHA = 1
    fn set_phase(&self, phase: ClockPhase) {
        self.set_cr(|| match phase {
            ClockPhase::SampleLeading => self.registers.cr1.modify(CR1::CPHA::CLEAR),
            ClockPhase::SampleTrailing => self.registers.cr1.modify(CR1::CPHA::SET),
        });
    }

    fn get_phase(&self) -> ClockPhase {
        if !self.registers.cr1.is_set(CR1::CPHA) {
            ClockPhase::SampleLeading
        } else {
            ClockPhase::SampleTrailing
        }
    }

    fn enable_tx(&self) {
        self.registers.cr2.modify(CR2::TXDMAEN::SET);
    }

    fn disable_tx(&self) {
        self.registers.cr2.modify(CR2::TXDMAEN::CLEAR);
    }

    fn enable_rx(&self) {
        self.registers.cr2.modify(CR2::RXDMAEN::SET);
    }

    fn disable_rx(&self) {
        self.registers.cr2.modify(CR2::RXDMAEN::CLEAR);
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
        self.active_slave.map(|p| {
            p.clear();
        });

        let mut count: usize = write_buffer.len();
        read_buffer
            .as_ref()
            .map(|buf| count = cmp::min(count, buf.len()));

        self.dma_len.set(count);

        self.transfers_in_progress.set(0);

        read_buffer.map(|rx_buffer| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.rx_dma.map(move |dma| {
                dma.do_transfer(rx_buffer);
            });
            self.enable_rx();
        });

        self.transfers_in_progress
            .set(self.transfers_in_progress.get() + 1);
        self.tx_dma.map(move |dma| {
            dma.do_transfer(write_buffer);
        });
        self.enable_tx();

        Ok(())
    }
}

impl<'a> spi::SpiMaster<'a> for Spi<'a> {
    type ChipSelect = &'a crate::gpio::Pin<'a>;

    fn set_client(&self, client: &'a dyn SpiMasterClient) {
        self.master_client.set(client);
    }

    fn init(&self) -> Result<(), ErrorCode> {
        // enable error interrupt (used only for debugging)
        // self.registers.cr2.modify(CR2::ERRIE::SET);

        // 2 line unidirectional mode
        // Select as master
        // Software slave management
        // 8 bit data frame format
        // Enable
        self.registers.cr1.modify(
            CR1::BIDIMODE::CLEAR
                + CR1::MSTR::SET
                + CR1::SSM::SET
                + CR1::SSI::SET
                + CR1::DFF::CLEAR
                + CR1::SPE::SET,
        );
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.registers.sr.is_set(SR::BSY)
    }

    fn write_byte(&self, out_byte: u8) -> Result<(), ErrorCode> {
        // loop till TXE (Transmit Buffer Empty) becomes 1
        while !self.registers.sr.is_set(SR::TXE) {}

        self.registers.dr.modify(DR::DR.val(out_byte as u32));
        Ok(())
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        self.read_write_byte(0)
    }

    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        match self.write_byte(val) {
            Ok(()) => {
                // loop till RXNE becomes 1
                while !self.registers.sr.is_set(SR::RXNE) {}

                Ok(self.registers.dr.read(DR::DR) as u8)
            }
            Err(e) => Err(e),
        }
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
        // If busy, don't start
        if self.is_busy() {
            return Err((ErrorCode::BUSY, write_buffer, read_buffer));
        }

        self.read_write_bytes(write_buffer, read_buffer)
    }

    /// We *only* support 1Mhz and 4MHz. If `rate` is set to any value other than
    /// `1_000_000` or `4_000_000`, then this function panics.
    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        match rate {
            1_000_000 => self.set_cr(|| {
                // HSI is 16Mhz and Fpclk is also 16Mhz. 0b011 is Fpclk / 16
                self.registers.cr1.modify(CR1::BR.val(0b011));
            }),
            4_000_000 => self.set_cr(|| {
                // HSI is 16Mhz and Fpclk is also 16Mhz. 0b001 is Fpclk / 4
                self.registers.cr1.modify(CR1::BR.val(0b001));
            }),
            _ => panic!("SPI rate must be 1_000_000 or 4_000_000"),
        }
        Ok(rate)
    }

    /// We *only* support 1Mhz and 4MHz. If we need to return any other value
    /// than `1_000_000` or `4_000_000`, then this function panics.
    fn get_rate(&self) -> u32 {
        // HSI is 16Mhz and Fpclk is also 16Mhz
        match self.registers.cr1.read(CR1::BR) {
            0b011 => 1_000_000, // 0b011 is Fpclk / 16 => 1 MHz
            0b001 => 4_000_000, // 0b001 is Fpclk / 4  => 4 MHz
            _ => panic!("Current SPI rate not supported by tock OS!"),
        }
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        self.set_polarity(polarity);
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        self.get_polarity()
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        self.set_phase(phase);
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        self.get_phase()
    }

    fn hold_low(&self) {
        self.active_after.set(true);
    }

    fn release_low(&self) {
        self.active_after.set(false);
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        self.set_active_slave(cs);
        Ok(())
    }
}

impl<'a> dma::StreamClient<'a, Dma1<'a>> for Spi<'a> {
    fn transfer_done(&self, pid: Dma1Peripheral) {
        if pid == self.tx_dma_pid {
            self.disable_tx();
        }

        if pid == self.rx_dma_pid {
            self.disable_rx();
        }

        self.transfers_in_progress
            .set(self.transfers_in_progress.get() - 1);

        if self.transfers_in_progress.get() == 0 {
            if !self.active_after.get() {
                self.active_slave.map(|p| {
                    p.set();
                });
            }

            let tx_buffer = self.tx_dma.and_then(|tx_dma| tx_dma.return_buffer());
            let rx_buffer = self.rx_dma.and_then(|rx_dma| rx_dma.return_buffer());

            let length = self.dma_len.get();
            self.dma_len.set(0);

            self.master_client.map(|client| {
                tx_buffer.map(|t| {
                    client.read_write_done(t, rx_buffer, Ok(length));
                });
            });
        }
    }
}

pub struct SpiClock<'a>(pub phclk::PeripheralClock<'a>);

impl ClockInterface for SpiClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
