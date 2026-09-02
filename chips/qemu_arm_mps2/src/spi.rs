// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! ARM PL022 (PrimeCell SSP) SPI controller, as found on the MPS2
//! AN385/AN386 FPGA images.
//!
//! Of the five PL022 instances on this machine, only the "Shield0" one
//! (`0x40026000`) is driven here.
//!
//! Limitations of this QEMU model shape this driver:
//!
//! - **No SSI slave device is attached to any of the five PL022 instances**
//!   in QEMU (`hw/arm/mps2.c` creates bare `TYPE_PL022` controllers with no
//!   `ssi_create_peripheral`), so a non-loopback transfer just reads back
//!   whatever QEMU's empty-bus default is, not meaningful data. This driver
//!   therefore always enables `CR1.LBM` (loopback) in [`Spi::init`].
//! - **No functional chip select.** GPIO on this machine is a QEMU stub
//!   (i.e., a no-op). [`ChipSelect`] is a zero-sized no-op placeholder.
//!
//! QEMU's PL022 also does not model SPI clock timing at all: `CR0`'s clock
//! format bits and `CPSR`'s prescaler are accepted and stored, but have no
//! effect on transfer behavior (transfers are synchronous and immediate in
//! the emulation).

use core::cell::Cell;
use core::cmp;

use kernel::ErrorCode;
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiMaster, SpiMasterClient};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

use crate::SYSCLK_FRQ;

pub const SPI_SHIELD0_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x4002_6000 as *const SpiRegisters) };

register_structs! {
    pub SpiRegisters {
        (0x000 => cr0: ReadWrite<u32, CR0::Register>),
        (0x004 => cr1: ReadWrite<u32, CR1::Register>),
        (0x008 => dr: ReadWrite<u32, DR::Register>),
        (0x00c => sr: ReadOnly<u32, SR::Register>),
        (0x010 => cpsr: ReadWrite<u32, CPSR::Register>),
        (0x014 => imsc: ReadWrite<u32, IMSC::Register>),
        (0x018 => ris: ReadOnly<u32, RIS::Register>),
        (0x01c => mis: ReadOnly<u32, MIS::Register>),
        (0x020 => icr: ReadWrite<u32, ICR::Register>),
        (0x024 => dmacr: ReadWrite<u32, DMACR::Register>),
        (0x028 => _reserved0),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    CR0 [
        Scr OFFSET(8) NUMBITS(8) [],
        Sph OFFSET(7) NUMBITS(1) [],
        Spo OFFSET(6) NUMBITS(1) [],
        Frf OFFSET(4) NUMBITS(2) [
            Motorola = 0b00,
        ],
        Dss OFFSET(0) NUMBITS(4) [
            Data8Bit = 0b0111,
        ],
    ],
    CR1 [
        Sod OFFSET(3) NUMBITS(1) [],
        Ms OFFSET(2) NUMBITS(1) [],
        Sse OFFSET(1) NUMBITS(1) [],
        Lbm OFFSET(0) NUMBITS(1) [],
    ],
    DR [
        Data OFFSET(0) NUMBITS(16) [],
    ],
    SR [
        Bsy OFFSET(4) NUMBITS(1) [],
        Rff OFFSET(3) NUMBITS(1) [],
        Rne OFFSET(2) NUMBITS(1) [],
        Tnf OFFSET(1) NUMBITS(1) [],
        Tfe OFFSET(0) NUMBITS(1) [],
    ],
    CPSR [
        Cpsdvsr OFFSET(0) NUMBITS(8) [],
    ],
    IMSC [
        Txim OFFSET(3) NUMBITS(1) [],
        Rxim OFFSET(2) NUMBITS(1) [],
        Rtim OFFSET(1) NUMBITS(1) [],
        Rorim OFFSET(0) NUMBITS(1) [],
    ],
    RIS [
        Txris OFFSET(3) NUMBITS(1) [],
        Rxris OFFSET(2) NUMBITS(1) [],
    ],
    MIS [
        Txmis OFFSET(3) NUMBITS(1) [],
        Rxmis OFFSET(2) NUMBITS(1) [],
    ],
    ICR [
        Rtic OFFSET(1) NUMBITS(1) [],
        Roric OFFSET(0) NUMBITS(1) [],
    ],
    DMACR [
        Txdmae OFFSET(1) NUMBITS(1) [],
        Rxdmae OFFSET(0) NUMBITS(1) [],
    ],
];

// PL022 clock dividers: the serial clock is
// `SYSCLK_FRQ / (CPSDVSR * (1 + SCR))`, where CPSDVSR is an even value in
// 2..=254 and SCR is in 0..=255.
const CPSDVSR_MIN: u32 = 2;
const CPSDVSR_MAX: u32 = 254;
const SCR_MAX: u32 = 255;

// The band of rates those dividers can realize, slowest and fastest.
const RATE_MIN: u32 = SYSCLK_FRQ.div_ceil(CPSDVSR_MAX * (SCR_MAX + 1));
const RATE_MAX: u32 = SYSCLK_FRQ / CPSDVSR_MIN;

// Default SPI clock speed, used by `init()`.
//
// The value is arbitrary within the band above: QEMU does not model
// transfer timing, so this only affects what `get_rate()` reports back.
const DEFAULT_RATE_HZ: u32 = 1_000_000;

// Bitfield of device logical state (i.e., non-exclusive states)
const SPI_IDLE: u8 = 0b000;
const SPI_WRITE_IN_PROGRESS: u8 = 0b001;
const SPI_READ_IN_PROGRESS: u8 = 0b010;
const SPI_IN_PROGRESS: u8 = 0b100;

/// Placeholder chip-select: this board has no functional GPIO to toggle a
/// real one, and none of the PL022 instances on this QEMU machine have a
/// slave device attached to select in the first place.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChipSelect;

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,
    client: OptionalCell<&'a dyn SpiMasterClient>,

    tx_buffer: MapCell<SubSliceMut<'static, u8>>,
    tx_position: Cell<usize>,
    rx_buffer: MapCell<SubSliceMut<'static, u8>>,
    rx_position: Cell<usize>,
    len: Cell<usize>,

    transfer_state: Cell<u8>,
}

impl<'a> Spi<'a> {
    pub const fn new(registers: StaticRef<SpiRegisters>) -> Spi<'a> {
        Spi {
            registers,
            client: OptionalCell::empty(),
            tx_buffer: MapCell::empty(),
            tx_position: Cell::new(0),
            rx_buffer: MapCell::empty(),
            rx_position: Cell::new(0),
            len: Cell::new(0),
            transfer_state: Cell::new(SPI_IDLE),
        }
    }

    fn enable(&self) {
        self.registers.cr1.modify(CR1::Sse::SET);
    }

    /// Shift one byte out and return the byte shifted in, leaving the device
    /// disabled and the receive FIFO drained.
    ///
    /// The PL022 only shifts data out while SSE is set, and nothing else on
    /// the synchronous path leaves it set: `init()` never enables the device.
    /// Without enabling it here a write would land in the TX FIFO and never
    /// be sent.
    fn transfer_byte_sync(&self, val: u8) -> u8 {
        self.enable();
        while !self.registers.sr.is_set(SR::Tnf) {}
        self.registers.dr.write(DR::Data.val(val as u32));
        while !self.registers.sr.is_set(SR::Rne) {}
        let byte = self.registers.dr.read(DR::Data) as u8;
        self.disable();
        byte
    }

    fn disable(&self) {
        self.registers.cr1.modify(CR1::Sse::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.sr.is_set(SR::Tfe) {
            if self.tx_buffer.is_some() {
                while self.registers.sr.is_set(SR::Tnf) && self.tx_position.get() < self.len.get() {
                    self.tx_buffer.map(|buf| {
                        self.registers
                            .dr
                            .write(DR::Data.val(buf[self.tx_position.get()] as u32));
                        self.tx_position.set(self.tx_position.get() + 1);
                    });
                }
                if self.tx_position.get() >= self.len.get() {
                    self.transfer_state
                        .set(self.transfer_state.get() & !SPI_WRITE_IN_PROGRESS);
                }
            } else {
                self.registers.imsc.modify(IMSC::Txim::CLEAR);
            }
        }

        while self.registers.sr.is_set(SR::Rne) {
            let byte = self.registers.dr.read(DR::Data) as u8;
            if self.rx_buffer.is_some() && self.rx_position.get() < self.len.get() {
                self.rx_buffer.map(|buf| {
                    buf[self.rx_position.get()] = byte;
                });
                self.rx_position.set(self.rx_position.get() + 1);
            }
        }
        if self.rx_position.get() >= self.len.get() {
            self.transfer_state
                .set(self.transfer_state.get() & !SPI_READ_IN_PROGRESS);
        }

        if self.transfer_state.get() == SPI_IN_PROGRESS
            && self.registers.sr.is_set(SR::Tfe)
            && !self.registers.sr.is_set(SR::Bsy)
        {
            // Tear the transfer down before handing the buffers back, not
            // inside the client callback: with no client registered,
            // `transfer_state` would otherwise never return to `SPI_IDLE`,
            // leaving the driver permanently busy while the level-triggered
            // TX interrupt re-asserts on every service pass.
            self.registers.imsc.modify(IMSC::Txim::CLEAR);
            self.registers.imsc.modify(IMSC::Rxim::CLEAR);
            self.disable();
            self.transfer_state.set(SPI_IDLE);

            if let Some(tx_buffer) = self.tx_buffer.take() {
                let rx_buffer = self.rx_buffer.take();
                self.client
                    .map(|client| client.read_write_done(tx_buffer, rx_buffer, Ok(self.len.get())));
            }
        }
    }
}

impl<'a> SpiMaster<'a> for Spi<'a> {
    type ChipSelect = ChipSelect;

    fn set_client(&self, client: &'a dyn SpiMasterClient) {
        self.client.set(client);
    }

    fn init(&self) -> Result<(), ErrorCode> {
        self.registers.cr0.modify(CR0::Dss::Data8Bit);
        self.registers.cr0.modify(CR0::Frf::Motorola);
        self.registers.cr0.modify(CR0::Spo::CLEAR);
        self.registers.cr0.modify(CR0::Sph::CLEAR);
        // See the module docs: no SSI slave is attached under QEMU, so
        // loopback is the only way to get a real transfer.
        self.registers.cr1.modify(CR1::Lbm::SET);
        // Master mode (Cr1::Ms::CLEAR); slave mode isn't implemented in
        // QEMU's PL022 model anyway.
        self.registers.cr1.modify(CR1::Ms::CLEAR);
        self.set_rate(DEFAULT_RATE_HZ)?;
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.transfer_state.get() != SPI_IDLE
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
        if self.is_busy() {
            return Err((ErrorCode::BUSY, write_buffer, read_buffer));
        }

        let len = match read_buffer.as_ref() {
            Some(rb) => cmp::min(write_buffer.len(), rb.len()),
            None => write_buffer.len(),
        };
        if len == 0 {
            return Err((ErrorCode::INVAL, write_buffer, read_buffer));
        }

        self.enable();

        self.len.set(len);
        let mut state = SPI_IN_PROGRESS | SPI_WRITE_IN_PROGRESS;

        self.tx_position.set(0);
        self.tx_buffer.replace(write_buffer);
        self.registers.imsc.modify(IMSC::Txim::SET);

        if let Some(rb) = read_buffer {
            state |= SPI_READ_IN_PROGRESS;
            self.rx_position.set(0);
            self.rx_buffer.replace(rb);
            self.registers.imsc.modify(IMSC::Rxim::SET);
        } else {
            self.registers.imsc.modify(IMSC::Rxim::CLEAR);
        }

        self.transfer_state.set(state);
        Ok(())
    }

    fn write_byte(&self, val: u8) -> Result<(), ErrorCode> {
        // With loopback-only SPI, our write pushes a read we should drop here
        // in the interest of looking like regular SPI for this interface.
        self.read_write_byte(val).map(|_| ())
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        self.read_write_byte(0)
    }

    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }
        Ok(self.transfer_byte_sync(val))
    }

    fn specify_chip_select(&self, _cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        // QEMU's PL022 does not model timing at all, so this only affects
        // what get_rate() reports back, not actual transfer speed. The
        // divider math mirrors the real PL022 formula so real-hardware
        // callers get a sane answer too.
        if !(RATE_MIN..=RATE_MAX).contains(&rate) {
            return Err(ErrorCode::INVAL);
        }

        // Round the divisor up, so the configured clock is never faster than
        // what the caller asked for.
        let divisor = SYSCLK_FRQ.div_ceil(rate);

        // The smallest legal CPSDVSR leaves the largest SCR, and so the
        // finest granularity and the closest achievable rate.
        let (cpsdvsr, scr) = (CPSDVSR_MIN..=CPSDVSR_MAX)
            .step_by(2)
            .find_map(|cpsdvsr| {
                let scr = divisor.div_ceil(cpsdvsr) - 1;
                (scr <= SCR_MAX).then_some((cpsdvsr, scr))
            })
            .ok_or(ErrorCode::INVAL)?;

        self.registers.cpsr.write(CPSR::Cpsdvsr.val(cpsdvsr));
        self.registers.cr0.modify(CR0::Scr.val(scr));
        Ok(SYSCLK_FRQ / (cpsdvsr * (scr + 1)))
    }

    fn get_rate(&self) -> u32 {
        // CPSDVSR reads back as 0 out of reset, before `init()` runs; clamp
        // so this reports a nonsense rate rather than dividing by zero.
        let prescale = self.registers.cpsr.read(CPSR::Cpsdvsr).max(1);
        let postdiv = self.registers.cr0.read(CR0::Scr) + 1;
        SYSCLK_FRQ / (prescale * postdiv)
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }
        match polarity {
            ClockPolarity::IdleHigh => self.registers.cr0.modify(CR0::Spo::SET),
            ClockPolarity::IdleLow => self.registers.cr0.modify(CR0::Spo::CLEAR),
        }
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        if self.registers.cr0.is_set(CR0::Spo) {
            ClockPolarity::IdleHigh
        } else {
            ClockPolarity::IdleLow
        }
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }
        match phase {
            ClockPhase::SampleTrailing => self.registers.cr0.modify(CR0::Sph::SET),
            ClockPhase::SampleLeading => self.registers.cr0.modify(CR0::Sph::CLEAR),
        }
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        if self.registers.cr0.is_set(CR0::Sph) {
            ClockPhase::SampleTrailing
        } else {
            ClockPhase::SampleLeading
        }
    }

    fn hold_low(&self) {}
    fn release_low(&self) {}
}
