// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! SPI master driver for the general-purpose SPI controller GP-SPI2.
//!
//! This driver uses the SPI controllers 64-byte data buffer (implemented as 16
//! 32-bit registers `W0` through `W15`). It does not use the GDMA controller.
//! Transfers longer than 64 bytes are split into 64-byte chunks with
//! chip-select held active over the entire transfer. Chip-select is implemented
//! through an (optional) conventional GPIO pin.
//!
//! The controller's bus clock must be enabled with
//! [`crate::sysreg::SysReg::enable_spi2`] before [`Spi::init`] is called.

use core::cell::Cell;
use core::cmp;

use kernel::ErrorCode;
use kernel::hil::gpio::Configure;
use kernel::hil::spi::cs::ChipSelectPolar;
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiMaster, SpiMasterClient};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

/// Number of bytes the controller's data buffer (`W0`-`W15`) holds.
const BUFFER_LEN: usize = 64;

/// GPIO matrix signal indices for GP-SPI2's clock and data lines.
///
/// (`FSPICLK_OUT_IDX`, `FSPIQ_IN_IDX`, and `FSPID_OUT_IDX` in Espressif's
/// `soc/gpio_sig_map.h`.)
const FSPICLK_SIGNAL: u32 = 63;
const FSPIQ_SIGNAL: u32 = 64;
const FSPID_SIGNAL: u32 = 65;

/// Controller clock, selected with `CLK_GATE::MST_CLK_SEL`.
const SPI_CLK_HZ: u32 = 80_000_000;

/// Default SPI clock rate.
const DEFAULT_RATE_HZ: u32 = 1_000_000;

pub const SPI2_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x6002_4000 as *const SpiRegisters) };

register_structs! {
    pub SpiRegisters {
        (0x00 => cmd: ReadWrite<u32, CMD::Register>),
        (0x04 => _reserved0),
        (0x0C => clock: ReadWrite<u32, CLOCK::Register>),
        (0x10 => user: ReadWrite<u32, USER::Register>),
        (0x14 => user1: ReadWrite<u32>),
        (0x18 => user2: ReadWrite<u32>),
        (0x1C => ms_dlen: ReadWrite<u32, MS_DLEN::Register>),
        (0x20 => misc: ReadWrite<u32, MISC::Register>),
        (0x24 => _reserved1),
        (0x30 => dma_conf: ReadWrite<u32>),
        (0x34 => dma_int_ena: ReadWrite<u32, DMA_INT::Register>),
        (0x38 => dma_int_clr: ReadWrite<u32, DMA_INT::Register>),
        (0x3C => dma_int_raw: ReadOnly<u32, DMA_INT::Register>),
        (0x40 => dma_int_st: ReadOnly<u32, DMA_INT::Register>),
        (0x44 => _reserved2),
        (0x98 => w: [ReadWrite<u32>; BUFFER_LEN / 4]),
        (0xD8 => _reserved3),
        (0xE0 => slave: ReadWrite<u32>),
        (0xE4 => _reserved4),
        (0xE8 => clk_gate: ReadWrite<u32, CLK_GATE::Register>),
        (0xEC => @END),
    }
}

register_bitfields![u32,
    CMD [
        /// Start a transfer.
        USR OFFSET(24) NUMBITS(1) [],
        /// Copy the configuration registers into the SPI clock domain; cleared
        /// by hardware when done.
        UPDATE OFFSET(23) NUMBITS(1) [],
    ],
    CLOCK [
        CLKCNT_L OFFSET(0) NUMBITS(6) [],
        CLKCNT_H OFFSET(6) NUMBITS(6) [],
        CLKCNT_N OFFSET(12) NUMBITS(6) [],
        CLKDIV_PRE OFFSET(18) NUMBITS(4) [],
        CLK_EQU_SYSCLK OFFSET(31) NUMBITS(1) [],
    ],
    USER [
        /// Full-duplex: receive while transmitting.
        DOUTDIN OFFSET(0) NUMBITS(1) [],
        CK_OUT_EDGE OFFSET(9) NUMBITS(1) [],
        USR_MOSI OFFSET(27) NUMBITS(1) [],
        USR_MISO OFFSET(28) NUMBITS(1) [],
    ],
    MS_DLEN [
        /// Number of data bits minus one.
        MS_DATA_BITLEN OFFSET(0) NUMBITS(18) [],
    ],
    MISC [
        CK_IDLE_EDGE OFFSET(29) NUMBITS(1) [],
    ],
    DMA_INT [
        TRANS_DONE OFFSET(12) NUMBITS(1) [],
    ],
    CLK_GATE [
        CLK_EN OFFSET(0) NUMBITS(1) [],
        MST_CLK_ACTIVE OFFSET(1) NUMBITS(1) [],
        /// 0: XTAL clock, 1: 80 MHz PLL clock.
        MST_CLK_SEL OFFSET(2) NUMBITS(1) [],
    ],
];

/// Chip select pin of a device.
pub type ChipSelect<'a> = Option<ChipSelectPolar<'a, esp32::gpio::GpioPin<'a>>>;

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,
    client: OptionalCell<&'a dyn SpiMasterClient>,
    chip_select: Cell<ChipSelect<'a>>,

    // Whether to hold CS low after a bus transaction:
    hold_low: Cell<bool>,

    rate: Cell<u32>,
    polarity: Cell<ClockPolarity>,
    phase: Cell<ClockPhase>,

    busy: Cell<bool>,
    write_buffer: MapCell<SubSliceMut<'static, u8>>,
    read_buffer: MapCell<SubSliceMut<'static, u8>>,
    /// Total number of bytes of the current transfer.
    transfer_len: Cell<usize>,
    /// Number of bytes of the current transfer started so far.
    transfer_pos: Cell<usize>,
}

impl<'a> Spi<'a> {
    pub fn new(registers: StaticRef<SpiRegisters>) -> Self {
        Spi {
            registers,
            client: OptionalCell::empty(),
            chip_select: Cell::new(None),
            hold_low: Cell::new(false),
            rate: Cell::new(DEFAULT_RATE_HZ),
            polarity: Cell::new(ClockPolarity::IdleLow),
            phase: Cell::new(ClockPhase::SampleLeading),
            busy: Cell::new(false),
            write_buffer: MapCell::empty(),
            read_buffer: MapCell::empty(),
            transfer_len: Cell::new(0),
            transfer_pos: Cell::new(0),
        }
    }

    /// Choose which GPIO pins are wired to the SPI clock (SCLK), controller
    /// output (MOSI), and, optionally, controller input (MISO).
    pub fn set_pins(
        &self,
        sclk: &esp32::gpio::GpioPin<'a>,
        mosi: &esp32::gpio::GpioPin<'a>,
        miso: Option<&esp32::gpio::GpioPin<'a>>,
    ) {
        sclk.make_output_peripheral_pin(FSPICLK_SIGNAL);
        mosi.make_output_peripheral_pin(FSPID_SIGNAL);
        if let Some(miso) = miso {
            miso.make_input_peripheral_pin(FSPIQ_SIGNAL);
        }
    }

    /// Compute the clock register settings for the fastest rate that does not
    /// exceed `rate`. Returns the register value and the actual rate.
    fn clock_config(rate: u32) -> Option<(u32, u32)> {
        if rate >= SPI_CLK_HZ {
            return Some((CLOCK::CLK_EQU_SYSCLK::SET.value, SPI_CLK_HZ));
        }

        // SPI clock = SPI_CLK_HZ / (pre * n), with pre in 1..=16, n in 2..=64.
        let mut best: Option<(u32, u32, u32)> = None;
        for pre in 1..=16 {
            let n = cmp::max(SPI_CLK_HZ.div_ceil(rate * pre), 2);
            if n > 64 {
                continue;
            }
            let actual = SPI_CLK_HZ / (pre * n);
            if best.is_none_or(|(_, _, best_rate)| actual > best_rate) {
                best = Some((pre, n, actual));
            }
        }

        best.map(|(pre, n, actual)| {
            let value = CLOCK::CLKDIV_PRE.val(pre - 1)
                + CLOCK::CLKCNT_N.val(n - 1)
                + CLOCK::CLKCNT_H.val(n / 2 - 1)
                + CLOCK::CLKCNT_L.val(n - 1);
            (value.value, actual)
        })
    }

    /// Apply the clock rate, polarity, and phase to the hardware.
    fn configure(&self) {
        let regs = self.registers;

        if let Some((clock, _)) = Self::clock_config(self.rate.get()) {
            regs.clock.set(clock);
        }

        let idle_high = self.polarity.get() == ClockPolarity::IdleHigh;
        let sample_trailing = self.phase.get() == ClockPhase::SampleTrailing;
        regs.misc.modify(MISC::CK_IDLE_EDGE.val(idle_high as u32));
        // SPI modes 1 and 2 shift data out on the rising edge.
        regs.user
            .modify(USER::CK_OUT_EDGE.val((idle_high != sample_trailing) as u32));
    }

    /// Start transferring the next chunk of the current transfer.
    fn start_chunk(&self) {
        let regs = self.registers;
        let pos = self.transfer_pos.get();
        let len = cmp::min(self.transfer_len.get() - pos, BUFFER_LEN);

        self.write_buffer.map(|write_buffer| {
            let chunk = &write_buffer[pos..pos + len];
            for (word, bytes) in regs.w.iter().zip(chunk.chunks(4)) {
                let mut value = [0; 4];
                value[..bytes.len()].copy_from_slice(bytes);
                word.set(u32::from_le_bytes(value));
            }
        });

        regs.ms_dlen
            .write(MS_DLEN::MS_DATA_BITLEN.val((len * 8 - 1) as u32));
        regs.cmd.write(CMD::UPDATE::SET);
        while regs.cmd.is_set(CMD::UPDATE) {}

        self.transfer_pos.set(pos + len);
        regs.dma_int_clr.write(DMA_INT::TRANS_DONE::SET);
        regs.cmd.write(CMD::USR::SET);
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        if !regs.dma_int_st.is_set(DMA_INT::TRANS_DONE) {
            return;
        }
        regs.dma_int_clr.write(DMA_INT::TRANS_DONE::SET);

        // Copy the received bytes of the finished chunk.
        let end = self.transfer_pos.get();
        let start = (end - 1) / BUFFER_LEN * BUFFER_LEN;
        self.read_buffer.map(|read_buffer| {
            for (word, bytes) in regs.w.iter().zip(read_buffer[start..end].chunks_mut(4)) {
                let value = word.get().to_le_bytes();
                bytes.copy_from_slice(&value[..bytes.len()]);
            }
        });

        if end < self.transfer_len.get() {
            self.start_chunk();
            return;
        }

        regs.dma_int_ena.modify(DMA_INT::TRANS_DONE::CLEAR);
        if !self.hold_low.get() {
            self.chip_select.get().map(|cs| cs.deactivate());
        }
        self.busy.set(false);

        let len = self.transfer_len.get();
        if let Some(write_buffer) = self.write_buffer.take() {
            let read_buffer = self.read_buffer.take();
            self.client
                .map(|client| client.read_write_done(write_buffer, read_buffer, Ok(len)));
        }
    }
}

impl<'a> SpiMaster<'a> for Spi<'a> {
    type ChipSelect = ChipSelect<'a>;

    fn init(&self) -> Result<(), ErrorCode> {
        let regs = self.registers;

        regs.clk_gate.write(
            CLK_GATE::CLK_EN::SET + CLK_GATE::MST_CLK_ACTIVE::SET + CLK_GATE::MST_CLK_SEL::SET,
        );
        // Master mode, no command/address/dummy phases, no DMA.
        regs.slave.set(0);
        regs.user1.set(0);
        regs.user2.set(0);
        regs.dma_conf.set(0);
        // Always transmit and receive; received bytes are only copied out when
        // there is a read buffer.
        regs.user
            .write(USER::DOUTDIN::SET + USER::USR_MOSI::SET + USER::USR_MISO::SET);
        regs.dma_int_ena.set(0);
        regs.dma_int_clr.set(u32::MAX);

        self.configure();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn SpiMasterClient) {
        self.client.set(client);
    }

    fn is_busy(&self) -> bool {
        self.busy.get()
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
        if self.busy.get() {
            return Err((ErrorCode::BUSY, write_buffer, read_buffer));
        }

        let len = match read_buffer {
            Some(ref read_buffer) => cmp::min(write_buffer.len(), read_buffer.len()),
            None => write_buffer.len(),
        };
        if len == 0 {
            return Err((ErrorCode::INVAL, write_buffer, read_buffer));
        }

        self.busy.set(true);
        self.transfer_len.set(len);
        self.transfer_pos.set(0);
        self.write_buffer.replace(write_buffer);
        if let Some(read_buffer) = read_buffer {
            self.read_buffer.replace(read_buffer);
        }

        self.chip_select.get().map(|cs| cs.activate());
        self.registers.dma_int_ena.modify(DMA_INT::TRANS_DONE::SET);
        self.start_chunk();

        Ok(())
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        if let Some(cs) = cs {
            cs.pin.make_output();
            cs.deactivate();
        }
        self.chip_select.set(cs);
        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        if rate == 0 {
            return Err(ErrorCode::INVAL);
        }
        let (_, actual) = Self::clock_config(rate).ok_or(ErrorCode::INVAL)?;
        self.rate.set(rate);
        self.configure();
        Ok(actual)
    }

    fn get_rate(&self) -> u32 {
        Self::clock_config(self.rate.get()).map_or(0, |(_, actual)| actual)
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        self.polarity.set(polarity);
        self.configure();
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        self.polarity.get()
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        self.phase.set(phase);
        self.configure();
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        self.phase.get()
    }

    fn hold_low(&self) {
        self.hold_low.set(true);
    }

    fn release_low(&self) {
        self.hold_low.set(false);
    }
}
