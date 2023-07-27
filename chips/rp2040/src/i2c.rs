// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::clocks;
use crate::resets;
use core::cell::Cell;
use kernel::debug;
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

// NOTE:
//
// This driver is based on the logic from the official pico-sdk:
// https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/hardware_i2c/i2c.c
// and most of the technical comments were copied verbatim from there.
//
// The register operations are almost exactly the same as in the official driver,
// but have been modified to be non-blocking through the use of IRQs instead of polling.
// A future improvement would be to use DMA instead for even less overhead.
//
// Currently this driver only supports master mode.
// Reads and slave support are part of the pico-sdk and still need to be ported here.

register_structs! {
    I2cRegisters {
        (0x00 => ic_con: ReadWrite<u32, IC_CON::Register>),
        (0x04 => ic_tar: ReadWrite<u32, IC_TAR::Register>),
        (0x08 => ic_sar: ReadWrite<u32, IC_SAR::Register>),
        (0x0c => _reserved0),
        (0x10 => ic_data_cmd: ReadWrite<u32, IC_DATA_CMD::Register>),
        (0x14 => ic_ss_scl_hcnt: ReadWrite<u32, IC_SS_SCL_HCNT::Register>),
        (0x18 => ic_ss_scl_lcnt: ReadWrite<u32, IC_SS_SCL_LCNT::Register>),
        (0x1c => ic_fs_scl_hcnt: ReadWrite<u32, IC_FS_SCL_HCNT::Register>),
        (0x20 => ic_fs_scl_lcnt: ReadWrite<u32, IC_FS_SCL_LCNT::Register>),
        (0x24 => _reserved1),
        (0x2c => ic_intr_stat: ReadOnly<u32, IC_INTR_STAT::Register>),
        (0x30 => ic_intr_mask: ReadWrite<u32, IC_INTR_MASK::Register>),
        (0x34 => ic_raw_intr_stat: ReadOnly<u32, IC_RAW_INTR_STAT::Register>),
        (0x38 => ic_rx_tl: ReadWrite<u32, IC_RX_TL::Register>),
        (0x3c => ic_tx_tl: ReadWrite<u32, IC_TX_TL::Register>),
        (0x40 => ic_clr_intr: ReadOnly<u32, IC_CLR_INTR::Register>),
        (0x44 => _reserved2), // TODO: there are still some registers to list in this gap
        (0x54 => ic_clr_tx_abrt: ReadOnly<u32, IC_CLR_TX_ABRT::Register>),
        (0x58 => _reserved3), // TODO: there are still some registers to list in this gap
        (0x60 => ic_clr_stop_det: ReadOnly<u32, IC_CLR_STOP_DET::Register>),
        (0x64 => _reserved4), // TODO: there are still some registers to list in this gap
        (0x6c => ic_enable: ReadWrite<u32, IC_ENABLE::Register>),
        (0x70 => _reserved5), // TODO: there are still some registers to list in this gap
        (0x7c => ic_sda_hold: ReadWrite<u32, IC_SDA_HOLD::Register>),
        (0x80 => ic_tx_abrt_source: ReadOnly<u32, IC_TX_ABRT_SOURCE::Register>),
        (0x84 => _reserved6), // TODO: there are still some registers to list in this gap
        (0x88 => ic_dma_cr: ReadWrite<u32, IC_DMA_CR::Register>),
        (0x8c => _reserved7), // TODO: there are still some registers to list in this gap
        (0xa0 => ic_fs_spklen: ReadWrite<u32, IC_FS_SPKLEN::Register>),
        (0xa4 => @END), // TODO: there are still some more registers to list here
    }
}

register_bitfields! [u32,
    /// I2C Control Register
    IC_CON [
        MASTER_MODE OFFSET(0) NUMBITS(1) [],
        SPEED OFFSET(1) NUMBITS(2) [
            STANDARD = 0x1,
            FAST = 0x2,
            HIGH = 0x3,
        ],
        IC_10BITADDR_SLAVE OFFSET(3) NUMBITS(1) [],
        IC_10BITADDR_MASTER OFFSET(4) NUMBITS(1) [],
        IC_RESTART_EN OFFSET(5) NUMBITS(1) [],
        IC_SLAVE_DISABLE OFFSET(6) NUMBITS(1) [],
        STOP_DET_IFADDRESSED OFFSET(7) NUMBITS(1) [],
        TX_EMPTY_CTRL OFFSET(8) NUMBITS(1) [],
        RX_FIFO_FULL_HLD_CTRL OFFSET(9) NUMBITS(1) [],
        STOP_DET_IF_MASTER_ACTIVE OFFSET(10) NUMBITS(1) [],
    ],
    /// I2C Target Address Register
    IC_TAR [
        IC_TAR OFFSET(0) NUMBITS(10) [],
        GC_OR_START OFFSET(10) NUMBITS(1) [],
        SPECIAL OFFSET(11) NUMBITS(1) [],
    ],
    /// I2C Slave Address Register
    IC_SAR [
        IC_SAR OFFSET(0) NUMBITS(10) [],
    ],
    /// I2C Rx/Tx Data Buffer and Command Register
    IC_DATA_CMD [
        DAT OFFSET(0) NUMBITS(8) [],
        CMD OFFSET(8) NUMBITS(1) [],
        STOP OFFSET(9) NUMBITS(1) [],
        RESTART OFFSET(10) NUMBITS(1) [],
        FIRST_DATA_BYTE OFFSET(11) NUMBITS(1) [],
    ],
    /// Standard Speed I2C Clock SCL High Count Register
    IC_SS_SCL_HCNT [
        IC_SS_SCL_HCNT OFFSET(0) NUMBITS(16) [],
    ],
    /// Standard Speed I2C Clock SCL Low Count Register
    IC_SS_SCL_LCNT [
        IC_SS_SCL_LCNT OFFSET(0) NUMBITS(16) [],
    ],
    /// Fast Mode or Fast Mode Plus I2C Clock SCL High Count Register
    IC_FS_SCL_HCNT [
        IC_FS_SCL_HCNT OFFSET(0) NUMBITS(16) [],
    ],
    /// Fast Mode or Fast Mode Plus I2C Clock SCL Low Count Register
    IC_FS_SCL_LCNT [
        IC_FS_SCL_LCNT OFFSET(0) NUMBITS(16) [],
    ],
    /// I2C Interrupt Status Register
    IC_INTR_STAT [
        R_RX_UNDER OFFSET(0) NUMBITS(1) [],
        R_RX_OVER OFFSET(1) NUMBITS(1) [],
        R_RX_FULL OFFSET(2) NUMBITS(1) [],
        R_TX_OVER OFFSET(3) NUMBITS(1) [],
        R_TX_EMPTY OFFSET(4) NUMBITS(1) [],
        R_RD_REQ OFFSET(5) NUMBITS(1) [],
        R_TX_ABRT OFFSET(6) NUMBITS(1) [],
        R_RX_DONE OFFSET(7) NUMBITS(1) [],
        R_ACTIVITY OFFSET(8) NUMBITS(1) [],
        R_STOP_DET OFFSET(9) NUMBITS(1) [],
        R_START_DET OFFSET(10) NUMBITS(1) [],
        R_GEN_CALL OFFSET(11) NUMBITS(1) [],
        R_RESTART_DET OFFSET(12) NUMBITS(1) [],
    ],
    /// I2C Interrupt Mask Register
    IC_INTR_MASK [
        M_RX_UNDER OFFSET(0) NUMBITS(1) [],
        M_RX_OVER OFFSET(1) NUMBITS(1) [],
        M_RX_FULL OFFSET(2) NUMBITS(1) [],
        M_TX_OVER OFFSET(3) NUMBITS(1) [],
        M_TX_EMPTY OFFSET(4) NUMBITS(1) [],
        M_RD_REQ OFFSET(5) NUMBITS(1) [],
        M_TX_ABRT OFFSET(6) NUMBITS(1) [],
        M_RX_DONE OFFSET(7) NUMBITS(1) [],
        M_ACTIVITY OFFSET(8) NUMBITS(1) [],
        M_STOP_DET OFFSET(9) NUMBITS(1) [],
        M_START_DET OFFSET(10) NUMBITS(1) [],
        M_GEN_CALL OFFSET(11) NUMBITS(1) [],
        M_RESTART_DET OFFSET(12) NUMBITS(1) [],
    ],
    /// I2C Raw Interrupt Status Register
    IC_RAW_INTR_STAT [
        RX_UNDER OFFSET(0) NUMBITS(1) [],
        RX_OVER OFFSET(1) NUMBITS(1) [],
        RX_FULL OFFSET(2) NUMBITS(1) [],
        TX_OVER OFFSET(3) NUMBITS(1) [],
        TX_EMPTY OFFSET(4) NUMBITS(1) [],
        RD_REQ OFFSET(5) NUMBITS(1) [],
        TX_ABRT OFFSET(6) NUMBITS(1) [],
        RX_DONE OFFSET(7) NUMBITS(1) [],
        ACTIVITY OFFSET(8) NUMBITS(1) [],
        STOP_DET OFFSET(9) NUMBITS(1) [],
        START_DET OFFSET(10) NUMBITS(1) [],
        GEN_CALL OFFSET(11) NUMBITS(1) [],
        RESTART_DET OFFSET(12) NUMBITS(1) [],
    ],
    /// I2C Receive FIFO Threshold Register
    IC_RX_TL [
        IC_RX_TL OFFSET(0) NUMBITS(8) [],
    ],
    /// I2C Transmit FIFO Threshold Register
    IC_TX_TL [
        IC_TX_TL OFFSET(0) NUMBITS(8) [],
    ],
    /// Clear Combined and Individual Interrupt Register
    IC_CLR_INTR [
        CLR_INTR OFFSET(0) NUMBITS(1) [],
    ],
    /// Clear TX_ABRT Interrupt Register
    IC_CLR_TX_ABRT [
        CLR_TX_ABRT OFFSET(0) NUMBITS(1) [],
    ],
    /// Clear STOP_DET Interrupt Register
    IC_CLR_STOP_DET [
        CLR_STOP_DET OFFSET(0) NUMBITS(1) [],
    ],
    /// I2C Enable Register
    IC_ENABLE [
        ENABLE OFFSET(0) NUMBITS(1) [],
        ABORT OFFSET(1) NUMBITS(1) [],
        TX_CMD_BLOCK OFFSET(2) NUMBITS(1) [],
    ],
    /// I2C SDA Hold Time Length Register
    IC_SDA_HOLD [
        IC_SDA_TX_HOLD OFFSET(0) NUMBITS(16) [],
        IC_SDA_RX_HOLD OFFSET(16) NUMBITS(8) [],
    ],
    /// I2C Transmit Abort Source Register
    IC_TX_ABRT_SOURCE [
        ABRT_7B_ADDR_NOACK OFFSET(0) NUMBITS(1) [],
        ABRT_10ADDR1_NOACK OFFSET(1) NUMBITS(1) [],
        ABRT_10ADDR2_NOACK OFFSET(2) NUMBITS(1) [],
        ABRT_TXDATA_NOACK OFFSET(3) NUMBITS(1) [],
        ABRT_GCALL_NOACK OFFSET(4) NUMBITS(1) [],
        ABRT_GCALL_READ OFFSET(5) NUMBITS(1) [],
        ABRT_HS_ACKDET OFFSET(6) NUMBITS(1) [],
        ABRT_SBYTE_ACKDET OFFSET(7) NUMBITS(1) [],
        ABRT_HS_NORSTRT OFFSET(8) NUMBITS(1) [],
        ABRT_SBYTE_NORSTRT OFFSET(9) NUMBITS(1) [],
        ABRT_10B_RD_NORSTRT OFFSET(10) NUMBITS(1) [],
        ABRT_MASTER_DIS OFFSET(11) NUMBITS(1) [],
        ARB_LOST OFFSET(12) NUMBITS(1) [],
        ABRT_SLVFLUSH_TXFIFO OFFSET(13) NUMBITS(1) [],
        ABRT_SLV_ARBLOST OFFSET(14) NUMBITS(1) [],
        ABRT_SLVRD_INTX OFFSET(15) NUMBITS(1) [],
        ABRT_USER_ABRT OFFSET(16) NUMBITS(1) [],
        TX_FLUSH_CNT OFFSET(23) NUMBITS(9) [],
    ],
    /// DMA Control Register
    IC_DMA_CR [
        RDMAE OFFSET(0) NUMBITS(1) [],
        TDMAE OFFSET(1) NUMBITS(1) [],
    ],
    /// I2C SS, FS or FM+ spike suppression limit
    IC_FS_SPKLEN [
        IC_FS_SPKLEN OFFSET(0) NUMBITS(8) [],
    ],
];

const INSTANCES: [StaticRef<I2cRegisters>; 2] = unsafe {
    [
        StaticRef::new(0x40044000 as *const I2cRegisters),
        StaticRef::new(0x40048000 as *const I2cRegisters),
    ]
};

#[derive(Clone, Copy, PartialEq)]
enum State {
    Uninitialized,
    Idle,
    WaitingToWriteNextByte,
    WaitingToReadNextByte,
    WaitingToStartReading,
    WaitingForStop,
}

pub struct I2c<'a, 'c> {
    instance_num: u8,
    registers: StaticRef<I2cRegisters>,
    clocks: OptionalCell<&'a clocks::Clocks>,
    resets: OptionalCell<&'a resets::Resets>,

    client: OptionalCell<&'c dyn hil::i2c::I2CHwMasterClient>,
    buf: TakeCell<'static, [u8]>,

    state: Cell<State>,
    addr: Cell<u8>,
    write_len: Cell<i32>,
    read_len: Cell<i32>,
    rw_index: Cell<i32>,

    abort_reason: OptionalCell<LocalRegisterCopy<u32, IC_TX_ABRT_SOURCE::Register>>,
}

impl<'a, 'c> I2c<'a, 'c> {
    fn new(instance_num: u8) -> Self {
        Self {
            instance_num,
            registers: INSTANCES[instance_num as usize],
            clocks: OptionalCell::empty(),
            resets: OptionalCell::empty(),

            client: OptionalCell::empty(),
            buf: TakeCell::empty(),

            state: Cell::new(State::Uninitialized),
            addr: Cell::new(0),
            write_len: Cell::new(0),
            read_len: Cell::new(0),
            rw_index: Cell::new(0),

            abort_reason: OptionalCell::empty(),
        }
    }

    pub fn new_i2c0() -> Self {
        I2c::new(0)
    }

    pub fn new_i2c1() -> Self {
        I2c::new(1)
    }

    pub fn resolve_dependencies(&self, clocks: &'a clocks::Clocks, resets: &'a resets::Resets) {
        self.clocks.set(clocks);
        self.resets.set(resets);
    }

    fn reset(&self) {
        self.resets.map_or_else(
            || panic!("You should call resolve_dependencies before reset."),
            |resets| match self.instance_num {
                0 => resets.reset(&[resets::Peripheral::I2c0]),
                1 => resets.reset(&[resets::Peripheral::I2c1]),
                _ => unreachable!(),
            },
        );
    }

    fn unreset(&self) {
        self.resets.map_or_else(
            || panic!("You should call resolve_dependencies before unreset."),
            |resets| match self.instance_num {
                0 => resets.unreset(&[resets::Peripheral::I2c0], true),
                1 => resets.unreset(&[resets::Peripheral::I2c1], true),
                _ => unreachable!(),
            },
        );
    }

    fn disable(&self) {
        self.registers.ic_enable.set(0);
    }

    fn enable(&self) {
        self.registers.ic_enable.modify(IC_ENABLE::ENABLE::SET);
    }

    fn set_baudrate(&self, baudrate: u32) -> u32 {
        assert!(baudrate != 0);

        // I2C is synchronous design that runs from clk_sys
        let freq_in = self
            .clocks
            .map(|clocks| clocks.get_frequency(clocks::Clock::System))
            .unwrap(); // Unwrap fail = You should call resolve_dependencies before set_baudrate.

        // TODO: as per the comments in the pico-sdk, this block is not 100% correct
        let period = (freq_in + baudrate / 2) / baudrate;
        let lcnt = period * 3 / 5;
        let hcnt = period - lcnt;
        assert!(hcnt >= 8);
        assert!(lcnt >= 8);

        // Per I2C-bus specification a device in standard or fast mode must
        // internally provide a hold time of at least 300ns for the SDA signal to
        // bridge the undefined region of the falling edge of SCL. A smaller hold
        // time of 120ns is used for fast mode plus.
        let sda_tx_hold_count;
        if baudrate < 1000000 {
            // sda_tx_hold_count = freq_in [cycles/s] * 300ns * (1s / 1e9ns)
            // Reduce 300/1e9 to 3/1e7 to avoid numbers that don't fit in uint.
            // Add 1 to avoid division truncation.
            sda_tx_hold_count = ((freq_in * 3) / 10000000) + 1;
        } else {
            // sda_tx_hold_count = freq_in [cycles/s] * 120ns * (1s / 1e9ns)
            // Reduce 120/1e9 to 3/25e6 to avoid numbers that don't fit in uint.
            // Add 1 to avoid division truncation.
            sda_tx_hold_count = ((freq_in * 3) / 25000000) + 1;
        }
        assert!(sda_tx_hold_count <= lcnt - 2);

        self.registers.ic_enable.modify(IC_ENABLE::ENABLE::CLEAR);
        // Always use "fast" mode (<= 400 kHz, works fine for standard mode too)
        self.registers.ic_con.modify(IC_CON::SPEED::FAST);
        self.registers.ic_fs_scl_hcnt.set(hcnt);
        self.registers.ic_fs_scl_lcnt.set(lcnt);
        self.registers.ic_fs_spklen.set({
            if lcnt < 16 {
                1
            } else {
                lcnt / 16
            }
        });
        self.registers
            .ic_sda_hold
            .modify(IC_SDA_HOLD::IC_SDA_TX_HOLD.val(sda_tx_hold_count));

        freq_in / period
    }

    pub fn init(&self, baudrate: u32) {
        self.reset();
        self.unreset();
        self.disable();

        // Only enable interrupts that we care about
        self.registers
            .ic_intr_mask
            .write(IC_INTR_MASK::M_STOP_DET::SET);

        // Configure as a fast-mode master with RepStart support, 7-bit addresses
        self.registers.ic_con.write(
            IC_CON::SPEED::FAST
                + IC_CON::MASTER_MODE::SET
                + IC_CON::IC_SLAVE_DISABLE::SET
                + IC_CON::IC_RESTART_EN::SET
                + IC_CON::TX_EMPTY_CTRL::SET,
        );

        // Set the TX and RX thresholds to 1 (encoded by the value 0) so that we
        // get an interrupt whenever a byte is available to be read or written.
        //
        // TODO: this is obviously not optimal for efficiency
        self.registers.ic_tx_tl.set(0);
        self.registers.ic_rx_tl.set(0);

        // Always enable the DREQ signalling -- harmless if DMA isn't listening
        self.registers
            .ic_dma_cr
            .write(IC_DMA_CR::TDMAE::SET + IC_DMA_CR::RDMAE::SET);

        self.set_baudrate(baudrate);
        self.enable();
        self.state.set(State::Idle);
    }

    fn write_then_read(
        &self,
        addr: u8,
        write_len: usize,
        read_len: usize,
    ) -> Result<(), hil::i2c::Error> {
        let state = self.state.get();
        assert!(state != State::Uninitialized);
        if state != State::Idle {
            return Err(hil::i2c::Error::Busy);
        }

        // Synopsys hw accepts start/stop flags alongside data items in the same
        // FIFO word, so no 0 byte transfers.
        let write_len = write_len as i32;
        assert!(write_len >= 1);

        self.addr.set(addr);
        self.rw_index.set(0);
        self.write_len.set(write_len);
        self.read_len.set(read_len as i32);

        self.registers.ic_enable.set(0);
        self.registers.ic_tar.set(addr as u32);
        self.registers.ic_enable.set(1);

        // The first byte will be written in response to an IRQ
        self.state.set(State::WaitingToWriteNextByte);
        self.registers
            .ic_intr_mask
            .modify(IC_INTR_MASK::M_TX_EMPTY::SET);

        Ok(())
    }

    fn write_next_byte(&self) {
        assert!(self.state.get() == State::WaitingToWriteNextByte);

        // As long as the mask bit is not cleared, this function gets called repeatedly.
        // We thus set it again later if there are still bytes that we want to write.
        self.registers
            .ic_intr_mask
            .modify(IC_INTR_MASK::M_TX_EMPTY::CLEAR);

        let idx = self.rw_index.get();
        let len = self.write_len.get();

        let first = idx == 0;
        let last = idx == len - 1;
        let read_to_follow = self.read_len.get() != 0;

        if first {
            self.abort_reason.clear();
        } else {
            let abort_reason = self.registers.ic_tx_abrt_source.extract();
            if abort_reason.get() != 0 {
                self.abort_reason.set(abort_reason);

                // NOTE:
                //
                // Clearing the abort flag also clears the reason, and
                // this instance of flag is clear-on-read! Note also the
                // IC_CLR_TX_ABRT register always reads as 0.
                self.registers.ic_clr_tx_abrt.get();

                // If the transaction was aborted or if it completed
                // successfully wait until the STOP condition has occurred.
                //
                // Handled by IRQ and process_stop_det()
                self.state.set(State::WaitingForStop);
                return;
            }
        }

        let byte = self
            .buf
            .map_or(None, |buf| Some(buf[idx as usize]))
            .unwrap(); // Unwrap fail = I2C buffer was not set before a write.

        let data_cmd = IC_DATA_CMD::DAT.val(byte as u32) + IC_DATA_CMD::RESTART::CLEAR;
        let data_cmd = {
            if last && !read_to_follow {
                data_cmd + IC_DATA_CMD::STOP::SET
            } else {
                data_cmd + IC_DATA_CMD::STOP::CLEAR
            }
        };

        if last {
            if read_to_follow {
                // This will cause a read to start once the write buffer is empty
                self.state.set(State::WaitingToStartReading);
                self.registers
                    .ic_intr_mask
                    .modify(IC_INTR_MASK::M_TX_EMPTY::SET);
            } else {
                // If the transaction was aborted or if it completed
                // successfully wait until the STOP condition has occurred.
                //
                // Handled by IRQ and process_stop_det()
                self.state.set(State::WaitingForStop);
            }
        } else {
            // Wait until the transmission of the address/data from the internal
            // shift register has completed. For this to function correctly, the
            // TX_EMPTY_CTRL flag in IC_CON must be set. The TX_EMPTY_CTRL flag
            // was set in i2c_init.
            //
            // This is handled in IRQ.
            self.state.set(State::WaitingToWriteNextByte);
            self.registers
                .ic_intr_mask
                .modify(IC_INTR_MASK::M_TX_EMPTY::SET);
        }

        self.registers.ic_data_cmd.write(data_cmd);
        self.rw_index.set(idx + 1);
    }

    fn read(&self, addr: u8, len: usize) -> Result<(), hil::i2c::Error> {
        let state = self.state.get();
        assert!(state != State::Uninitialized);
        if state != State::Idle {
            return Err(hil::i2c::Error::Busy);
        }

        let len = len as i32;
        assert!(len >= 1);

        self.addr.set(addr);
        self.read_len.set(len);
        self.start_reading();

        Ok(())
    }

    fn start_reading(&self) {
        self.abort_reason.clear();
        self.rw_index.set(0);

        self.registers.ic_enable.set(0);
        self.registers.ic_tar.set(self.addr.get() as u32);
        self.registers.ic_enable.set(1);

        // The first byte will be read in response to an IRQ
        self.state.set(State::WaitingToReadNextByte);
        self.registers
            .ic_intr_mask
            .modify(IC_INTR_MASK::M_RX_FULL::SET);

        // Set the first read in motion (CMD::SET indicates a read)
        let data_cmd = IC_DATA_CMD::CMD::SET;
        let data_cmd = {
            if self.read_len.get() == 1 {
                // We need to issue the stop bit together with the last read bit
                data_cmd + IC_DATA_CMD::STOP::SET
            } else {
                data_cmd
            }
        };
        self.registers.ic_data_cmd.write(data_cmd);
    }

    fn read_next_byte(&self) {
        assert!(self.state.get() == State::WaitingToReadNextByte);

        // As long as the mask bit is not cleared, this function gets called repeatedly.
        // We thus set it again later if there are still bytes that we want to read.
        self.registers
            .ic_intr_mask
            .modify(IC_INTR_MASK::M_RX_FULL::CLEAR);

        let idx = self.rw_index.get();
        let len = self.read_len.get();

        // We copy the register before reading the bit that clears it
        let abort_reason = self.registers.ic_tx_abrt_source.extract();
        if self.registers.ic_clr_tx_abrt.get() != 0 {
            self.abort_reason.set(abort_reason);
            return;
        }

        let byte = self.registers.ic_data_cmd.read(IC_DATA_CMD::DAT) as u8;
        self.buf.map(|buf| buf[idx as usize] = byte);

        let idx = idx + 1;
        if idx > len - 1 {
            // We have just read the last byte and the stop condition has already
            // been issued so now we just need to wait for it to be recognized.
            self.state.set(State::WaitingForStop);
            return;
        }
        self.rw_index.set(idx);

        let data_cmd = IC_DATA_CMD::CMD::SET; // Read direction
        let data_cmd = {
            if idx == len - 1 {
                // The stop bit is issued together with the read bit for the last byte
                data_cmd + IC_DATA_CMD::STOP::SET
            } else {
                data_cmd
            }
        };

        self.state.set(State::WaitingToReadNextByte);
        self.registers
            .ic_intr_mask
            .modify(IC_INTR_MASK::M_RX_FULL::SET);
        self.registers.ic_data_cmd.write(data_cmd);
    }

    fn start_reading_after_write(&self) {
        assert!(self.state.get() == State::WaitingToStartReading);

        // In reading mode we no longer want to know when the TX buffer is empty
        self.registers
            .ic_intr_mask
            .modify(IC_INTR_MASK::M_TX_EMPTY::CLEAR);

        self.start_reading();
    }

    fn process_stop_det(&self) {
        assert!(self.state.get() == State::WaitingForStop);

        // Reset by read
        self.registers.ic_clr_stop_det.get();

        let status = {
            if let Some(reason) = self.abort_reason.take() {
                if reason.matches_all(IC_TX_ABRT_SOURCE::ABRT_7B_ADDR_NOACK::SET) {
                    Err(hil::i2c::Error::AddressNak)
                } else if reason.matches_all(IC_TX_ABRT_SOURCE::ABRT_TXDATA_NOACK::SET) {
                    Err(hil::i2c::Error::DataNak)
                } else if reason.matches_all(IC_TX_ABRT_SOURCE::ARB_LOST::SET) {
                    Err(hil::i2c::Error::ArbitrationLost)
                } else {
                    Err(hil::i2c::Error::NotSupported)
                }
            } else {
                Ok(())
            }
        };

        // Reset state before the callback in case the client wants to start a
        // new command inside the callback
        self.state.set(State::Idle);

        self.client.map(|client| match self.buf.take() {
            None => {}
            Some(buf) => {
                client.command_complete(buf, status);
            }
        });

        // NOTE:
        //
        // The hardware issues a STOP automatically on an abort condition.
        // Note also the hardware clears RX FIFO as well as TX on abort,
        // because we set hwparam IC_AVOID_RX_FIFO_FLUSH_ON_TX_ABRT to 0.
    }

    pub fn handle_interrupt(&self) {
        match self.state.get() {
            State::Uninitialized => debug!(
                "Unexpected IRQ for uninitialized I2C device {}",
                self.instance_num
            ),
            State::Idle => debug!("Unexpected IRQ for idle I2C device {}", self.instance_num),
            State::WaitingToWriteNextByte => self.write_next_byte(),
            State::WaitingToReadNextByte => self.read_next_byte(),
            State::WaitingToStartReading => self.start_reading_after_write(),
            State::WaitingForStop => self.process_stop_det(),
        }
    }
}

impl<'a, 'c> hil::i2c::I2CMaster<'c> for I2c<'a, 'c> {
    fn set_master_client(&self, client: &'c dyn hil::i2c::I2CHwMasterClient) {
        self.client.set(client);
    }

    fn enable(&self) {
        self.enable();
        // TODO: set as master once we support slave mode too
    }

    fn disable(&self) {
        self.disable();
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.buf.put(Some(data));

        if let Err(error) = self.write_then_read(addr, write_len, read_len) {
            // The unwrap should not fail because we have just assigned to buf
            Err((error, self.buf.take().unwrap()))
        } else {
            Ok(())
        }
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        // Setting read_len to 0 will result in having just a write
        self.write_read(addr, data, len, 0)
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.buf.put(Some(buffer));

        if let Err(error) = self.read(addr, len) {
            // The unwrap should not fail because we have just assigned to buf
            Err((error, self.buf.take().unwrap()))
        } else {
            Ok(())
        }
    }
}
