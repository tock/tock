// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! I2C Master-Slave Driver

use crate::i2c::i2c::I2CHwSlaveClient;
use crate::registers::i2c_regs::{
    ACQDATA, CTRL, FDATA, FIFO_CTRL, FIFO_STATUS, I2C_PARAM_FIFO_DEPTH, INTR, RDATA, STATUS,
    TARGET_ID, TIMING0, TIMING1, TIMING2, TIMING3, TIMING4, TXDATA,
};
use core::cell::Cell;
use kernel::hil;
use kernel::hil::i2c;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;

pub use crate::registers::i2c_regs::I2cRegisters;

#[derive(PartialEq)]
enum I2CSlaveSignals {
    /// ABYTE contains ordinary data byte as received
    None = 0x00,
    /// ABYTE contains the 8-bit I2C address (R/W in lsb)
    Start = 0x01,
    /// ABYTE contains junk
    Stop = 0x02,
    /// ABYTE contains junk, START with address will follow
    Restart = 0x03,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum I2CSlavePendingOps {
    WriteRecvPending,
    ReadSendPending,
    Idle,
}

impl I2CSlaveSignals {
    fn from_u32(value: u32) -> Option<I2CSlaveSignals> {
        match value {
            0x00 => Some(I2CSlaveSignals::None),
            0x01 => Some(I2CSlaveSignals::Start),
            0x02 => Some(I2CSlaveSignals::Stop),
            0x03 => Some(I2CSlaveSignals::Restart),
            _ => None,
        }
    }
}

pub struct I2c<'a> {
    registers: StaticRef<I2cRegisters>,
    clock_period_nanos: u32,

    master_client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,
    slave_client: OptionalCell<&'a dyn hil::i2c::I2CHwSlaveClient>,
    slave_client_addr: Cell<u8>,

    // Set when calling the write_read operation
    // This specifies the address of the read operation
    // after the write operation. Set to 0 for single read/write operations.
    slave_read_address: Cell<u8>,

    buffer: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,
    write_index: Cell<usize>,

    read_len: Cell<usize>,
    read_index: Cell<usize>,

    // Target Mode buffer, let's keep master/target different to avoid ambiguity
    slave_write_buffer: TakeCell<'static, [u8]>,
    slave_read_buffer: TakeCell<'static, [u8]>,
    slave_write_len: Cell<usize>,
    slave_write_nxt_ofst: Cell<usize>,

    slave_read_len: Cell<usize>,
    slave_read_ofst: Cell<usize>,
    slave_pending_ops: Cell<I2CSlavePendingOps>,
}

impl<'a> hil::i2c::I2CMasterSlave<'a> for I2c<'a> {}

impl<'a> I2c<'_> {
    pub fn new(base: StaticRef<I2cRegisters>, clock_period_nanos: u32) -> I2c<'a> {
        assert_ne!(clock_period_nanos, 0);
        I2c {
            registers: base,
            clock_period_nanos,
            master_client: OptionalCell::empty(),
            slave_client: OptionalCell::empty(),
            slave_client_addr: Cell::new(0),
            slave_read_address: Cell::new(0),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            slave_write_buffer: TakeCell::empty(),
            slave_read_buffer: TakeCell::empty(),
            slave_write_len: Cell::new(0),
            slave_write_nxt_ofst: Cell::new(0),
            slave_read_len: Cell::new(0),
            slave_read_ofst: Cell::new(0),
            slave_pending_ops: Cell::new(I2CSlavePendingOps::Idle),
        }
    }

    fn is_enabled(&self) -> bool {
        self.registers.ctrl.is_set(CTRL::ENABLEHOST)
            || self.registers.ctrl.is_set(CTRL::ENABLETARGET)
    }

    fn is_host_enabled(&self) -> bool {
        self.registers.ctrl.is_set(CTRL::ENABLEHOST)
    }

    fn is_target_enabled(&self) -> bool {
        self.registers.ctrl.is_set(CTRL::ENABLETARGET)
    }

    // This function assumes the first byte it reads is a DATA/STOP/RESTART byte
    // Must only be called from a context where a START signal was handled or if the caller knows
    // that the first byte read is a non-START signal byte during a WRITE_RECV operation.
    fn slave_write_recv(&self) {
        assert_eq!(
            self.slave_pending_ops.get(),
            I2CSlavePendingOps::WriteRecvPending
        );
        let regs = self.registers;
        self.slave_client
            .map(|client| match self.slave_write_buffer.take() {
                None => {
                    // Since the kernel does write_receive()->enable()->listen(),
                    // we should always have a buffer available after interrupts are enabled.
                    // we should not need to write_expected() here. It will be handled by an ACQ_FULL
                    unreachable!("i2c-target: Lost slave write buffer");
                }
                Some(write_recv) => {
                    let buff_size = write_recv.len();
                    let mut offset = self.slave_write_nxt_ofst.get();
                    while offset < buff_size && regs.fifo_status.read(FIFO_STATUS::ACQLVL) > 0 {
                        // We have space in our buffer and there's more data to read
                        let acqdata = regs.acqdata.extract();
                        match I2CSlaveSignals::from_u32(acqdata.read(ACQDATA::SIGNAL)).unwrap() {
                            I2CSlaveSignals::None => {
                                // Useful data from host
                                write_recv[offset] = acqdata.read(ACQDATA::ABYTE) as u8;
                                offset += 1;
                            }
                            I2CSlaveSignals::Start => {
                                unreachable!("i2c-target: Unexpected START signal mid transfer");
                            }
                            _ => {
                                // STOP OR RESTART, the preceding write from host has terminated
                                client.command_complete(
                                    write_recv,
                                    offset + 1, // offset is zero indexed, this is length
                                    hil::i2c::SlaveTransmissionType::Write,
                                );
                                self.slave_reset_internal_state();
                                if I2CSlaveSignals::from_u32(acqdata.read(ACQDATA::SIGNAL)).unwrap()
                                    == I2CSlaveSignals::Stop
                                {
                                    // Reset FIFOS, mainly ACQFIFO, since we hit a STOP
                                    self.fifo_reset();
                                }
                                // This transaction is done
                                return;
                            }
                        }
                    }

                    if regs.fifo_status.read(FIFO_STATUS::ACQLVL) > 0
                        && (offset >= buff_size || offset + 1 >= buff_size)
                    {
                        // Max capacity
                        offset = buff_size;
                        // We didn't see a STOP/RESTART, and there's still more data to fetch than
                        // we can read right now!
                        client.command_complete(
                            write_recv,
                            offset,
                            hil::i2c::SlaveTransmissionType::Write,
                        );
                        // Request a new buffer with more space to fetch the next segment
                        client.write_expected();
                        // Expect a new buffer, so we can start over, but don't clear the operational
                        // state yet. i.e don't use slave_op_finish_cleanup() here as we aren't fully done,
                        // with this transaction yet.
                        self.slave_write_nxt_ofst.set(0);
                        return;
                    } else if offset < buff_size {
                        // We drained the ACQFIFO, found no STOP/RESTART, and still have more space in the
                        // internal buffer, expect a CMD_COMPLETE/ACQFULL interrupt when host finishes.
                        self.slave_write_nxt_ofst.set(offset);
                        self.slave_write_buffer.replace(write_recv);
                        return;
                    }
                    // One of the above should've occurred
                    unreachable!("i2c-target: Unexpected I2C-Target operational state");
                }
            });
    }

    fn slave_reset_internal_state(&self) {
        // An operation has finished, reset internal state
        self.slave_pending_ops.set(I2CSlavePendingOps::Idle);
        // We can reset both R/W params because we only allow one op at a time
        self.slave_read_ofst.set(0);
        self.slave_read_len.set(0);
        self.slave_write_len.set(0);
        self.slave_write_nxt_ofst.set(0);
        // Assert we returned all buffers
        assert_eq!(self.slave_read_buffer.take(), None);
        assert_eq!(self.slave_write_buffer.take(), None);
    }
    // This function must be called from a COMMAND_COMPLETE context, that is,
    // the ACQFIFO is expected to contain a STOP/RESTART for the preceding read operation from
    // the host.
    fn slave_read_cmd_complete(&self) {
        // We got here because:
        // 1. START->SLAVE_ADDR->SUB_ADDR->START(RESTART)
        // 2. CASE1->SLAVE_ADDR->HOST_READ->STOP.
        assert_eq!(
            self.slave_pending_ops.get(),
            I2CSlavePendingOps::ReadSendPending
        );

        let regs = self.registers;
        let mut restart_matched = false;
        // Pop ACQFIFO until we clear the STOP/RESTART for this ReadSendPending
        while regs.fifo_status.read(FIFO_STATUS::ACQLVL) > 0 {
            let acqdata = I2CSlaveSignals::from_u32(regs.acqdata.read(ACQDATA::SIGNAL))
                .expect("i2c-target: Unexpected Signal");
            match acqdata {
                I2CSlaveSignals::Stop => {
                    if acqdata == I2CSlaveSignals::Stop {
                        // Reset ACQFIFO, since this is a STOP
                        regs.fifo_ctrl.modify(FIFO_CTRL::ACQRST::SET);
                    }
                    self.slave_client.map(|client| {
                        let the_sauce = self
                            .slave_read_buffer
                            .take()
                            .expect("i2c-target: Slave read buffer not found");
                        client.command_complete(
                            the_sauce,
                            self.slave_read_ofst.get() + 1,
                            hil::i2c::SlaveTransmissionType::Read,
                        );
                    });
                    self.slave_reset_internal_state();
                    return;
                }
                // We were called from a command complete context, there should not be a start
                // until after the next STOP/RESTART.
                I2CSlaveSignals::Start => {
                    if !restart_matched {
                        // On a RESTART, a START with address will follow
                        unreachable!("i2c-target: Unexpected START signal")
                    }
                    // TODO: Assert here next byte is START+ADDR
                }
                I2CSlaveSignals::Restart => {
                    // It is possible that the host has already read all required bytes (if we loaded the fifo) from
                    // TXFIFO by the time this ISR is handled. CASE 1 and 2 have occurred.
                    // So we can keep going until the subsequent STOP is found.
                    // TODO: This^ needs to be verified, can this be racy??
                    restart_matched = true
                }
                _ => {} // Ignore any misc bytes
            }
        }
        if !restart_matched {
            // We did't match a RESTART or a STOP.
            unreachable!("i2c-target: STOP/RESTART byte not found");
        }
        // TODO: Is it possible that we miss the next CMD_COMPLETE IRQ for the subsequent STOP, incase,
        // we took too long to process the above?
    }

    fn slave_handle_partial_write_recv(&self) {
        assert_eq!(
            self.slave_pending_ops.get(),
            I2CSlavePendingOps::WriteRecvPending
        );

        let regs = self.registers;
        let first_acqdata = regs.acqdata.extract();
        // Check the first byte popped:
        match I2CSlaveSignals::from_u32(first_acqdata.read(ACQDATA::SIGNAL))
            .expect("i2c-target: Unexpected Signal")
        {
            I2CSlaveSignals::None => {
                // We were doing a write receive that is terminated now, host sent STOP/RESTART
                // 1. Capture the data byte we just read
                // We are mid transfer, something went wrong if offset = 0
                assert!(self.slave_write_nxt_ofst.get() != 0);
                self.slave_write_recv_load_byte(
                    u8::try_from(first_acqdata.read(ACQDATA::ABYTE))
                        .expect("i2c-target: Data overflow"),
                )
                .expect("i2c-target: invalid byte offset");
                // 2. Handle the rest (if any) bytes and alert client
                self.slave_write_recv();
            }
            I2CSlaveSignals::Start => {
                // Check message is for us
                // TODO: change this when multi-slave addresses are supported
                assert_eq!(
                    first_acqdata.read(ACQDATA::ABYTE) >> 1,
                    self.slave_client_addr.get() as u32,
                );
                // (R/W bit = 0) -> Write Transaction
                if !(first_acqdata.read(ACQDATA::ABYTE) & 0x1 == 0) {
                    unreachable!("i2c-target: Expected to start a write from the host");
                }
                self.slave_write_recv();
            }
            I2CSlaveSignals::Stop | I2CSlaveSignals::Restart => {
                // Preceding op finished.
                // STOP OR RESTART, the preceding write from host has terminated
                if I2CSlaveSignals::from_u32(first_acqdata.read(ACQDATA::SIGNAL)).unwrap()
                    == I2CSlaveSignals::Stop
                {
                    // Reset ACQFIFO, since we hit a STOP
                    regs.fifo_ctrl.modify(FIFO_CTRL::ACQRST::SET);
                }
                self.slave_client.map(|client| {
                    client.command_complete(
                        self.slave_write_buffer
                            .take()
                            .expect("i2c-target: Slave write buffer lost"),
                        self.slave_write_nxt_ofst.get(),
                        hil::i2c::SlaveTransmissionType::Write,
                    );
                    self.slave_reset_internal_state();
                    if I2CSlaveSignals::from_u32(first_acqdata.read(ACQDATA::SIGNAL)).unwrap()
                        == I2CSlaveSignals::Stop
                    {
                        // Reset FIFOS, mainly ACQFIFO, since we hit a STOP
                        self.fifo_reset();
                    }
                });
            }
        }
    }

    // Loads @byte into the current offset of the write buffer
    // this is a helper function.
    fn slave_write_recv_load_byte(&self, byte: u8) -> Result<(), ()> {
        let buffer = self
            .slave_write_buffer
            .take()
            .expect("i2c-target: Slave write buffer lost");
        let offset = self.slave_write_nxt_ofst.get();
        if offset >= buffer.len() {
            return Err(());
        }
        buffer[offset] = byte;
        // Set buffer back
        self.slave_write_buffer.replace(buffer);
        self.slave_write_nxt_ofst.set(offset + 1);
        Ok(())
    }

    fn slave_handle_cmd_complete(&self) {
        let regs = self.registers;
        // In target mode, CMD_COMPLETE is raised if the external host issues a STOP or repeated START
        // (in either case, the preceding transaction is terminated).
        // "If the transaction is a write operation (R/W bit = 0), the target proceeds
        // to read bytes from the bus and insert them into ACQ FIFO until the host
        // terminates the transaction by sending a STOP or a repeated START signal.
        // A STOP or repeated START indicator is inserted into ACQ FIFO as the next entry
        // following the last byte received, in which case other bits may be junk."
        let first_acqdata = regs.acqdata.extract();
        // Check the first byte popped:
        match I2CSlaveSignals::from_u32(first_acqdata.read(ACQDATA::SIGNAL))
            .expect("i2c-target: Unexpected Signal")
        {
            I2CSlaveSignals::None => {
                match self.slave_pending_ops.get() {
                    I2CSlavePendingOps::ReadSendPending => {
                        // ReadSendPending and host sent STOP/RESTART
                        self.slave_read_cmd_complete();
                    }
                    I2CSlavePendingOps::WriteRecvPending => {
                        // We were doing a write receive that is terminated now, host sent STOP/RESTART
                        // 1. Capture the data byte we just read
                        // We are mid transfer, something went wrong if offset = 0
                        assert!(self.slave_write_nxt_ofst.get() != 0);
                        self.slave_write_recv_load_byte(
                            u8::try_from(first_acqdata.read(ACQDATA::ABYTE))
                                .expect("i2c-target: Data overflow"),
                        )
                        .expect("i2c-target: invalid byte offset");
                        // 2. Handle the rest (if any) bytes and alert client
                        self.slave_write_recv();
                    }
                    _ => {
                        unreachable!("i2c-target: Unexpected I2C-Target internal operational state")
                    }
                }
            }
            I2CSlaveSignals::Start => {
                // Check message is for us
                // TODO: change this when multi-slave addresses are supported
                assert_eq!(
                    first_acqdata.read(ACQDATA::ABYTE) >> 1,
                    self.slave_client_addr.get() as u32,
                );
                // (R/W bit = 0) -> Write Transaction
                if first_acqdata.read(ACQDATA::ABYTE) & 0x1 == 0 {
                    // WRITE Transaction (Finished)
                    // NOTE: ACQFIFO is 64B, and there's START & STOP in the FIFO we can capture
                    // the entire transaction since the Kernel buffer exceeds FIFO depth (64).
                    match self.slave_pending_ops.get() {
                        I2CSlavePendingOps::WriteRecvPending => {
                            self.slave_write_recv();
                        }
                        _ => unreachable!(
                            "i2c-target: Mismatched HW and SW operational state, expected WriteRecvPending"
                        ),
                    }
                } else {
                    // (R/W bit = 1) -> Read Transaction
                    match self.slave_pending_ops.get() {
                        I2CSlavePendingOps::ReadSendPending => {
                            // READ Transaction (Started), and *may* have finished
                            self.slave_read_cmd_complete();
                        }
                        _ => unreachable!(
                            "i2c-target: Mismatched HW and SW operational state, expected ReadSendPending"
                        ),
                    }
                }
            }
            I2CSlaveSignals::Stop | I2CSlaveSignals::Restart => {
                // Preceding op finished. We *shouldn't* reach this,
                // the first byte read should not be a STOP/RESTART, likely means bad state from
                // a previous transfer.
                unreachable!("i2c-target: Unexpected first byte");
            }
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irqs = regs.intr_state.extract();
        if self.is_host_enabled() {
            // Handle Host/Master Mode Interrupts
            // Clear all host interrupts
            regs.intr_state.modify(
                INTR::FMT_THRESHOLD::SET
                    + INTR::RX_THRESHOLD::SET
                    + INTR::FMT_OVERFLOW::SET
                    + INTR::RX_OVERFLOW::SET
                    + INTR::NAK::SET
                    + INTR::SCL_INTERFERENCE::SET
                    + INTR::SDA_INTERFERENCE::SET
                    + INTR::STRETCH_TIMEOUT::SET
                    + INTR::SDA_UNSTABLE::SET
                    + INTR::CMD_COMPLETE::SET,
            );

            if irqs.is_set(INTR::FMT_THRESHOLD) {
                // FMT Watermark
                if self.slave_read_address.get() != 0 {
                    self.write_read_data();
                } else {
                    self.write_data();
                }
            }

            if irqs.is_set(INTR::RX_THRESHOLD) {
                // RX Watermark
                self.read_data();
            }
        } else if self.is_target_enabled() {
            // Handle Target/Slave Mode Interrupts
            // Clear all target interrupts (these are the only rw1c irqs)
            regs.intr_state.modify(
                INTR::TX_OVERFLOW::SET
                    + INTR::UNEXP_STOP::SET
                    + INTR::HOST_TIMEOUT::SET
                    + INTR::CMD_COMPLETE::SET,
            );
            // TODO: TEST: DISABLE INTERRUPTS FOR NOW?
            if irqs.is_set(INTR::CMD_COMPLETE) {
                // CMD_COMPLETE is asserted, in the beginning of a repeated START or at the end of a STOP.
                // TODO: Add a return to following functions so we can check if the OP was finished, or if it's
                //       RESTART pending.
                self.slave_handle_cmd_complete();
                // return; if OP DONE
            }

            if irqs.is_set(INTR::TX_STRETCH) {
                // TX_STRETCH is asserted whenever target intends to *transmit* data but cannot.
                // Raised if the target is stretching clocks for a read command (if host supports it).
                // This maybe on either of the following conditions:
                // 1. If there is no data available to be sent back (TX FIFO empty case), the target
                // stretches the clock until data is made available by software.
                // 2. If there is more than 1 entry in the ACQ FIFO.
                //    - Having more than 1 entry in the ACQ FIFO suggests there is potentially an unhandled condition
                //      (STOP / RESTART) or an unhandled command (START) that requires software intervention before
                //      the read can proceed.
                match self.slave_pending_ops.get() {
                    I2CSlavePendingOps::ReadSendPending => {
                        // Ensure Case 2 didn't happen
                        // Note: We *shouldn't* hit a Case 2 situation
                        assert!(regs.fifo_status.read(FIFO_STATUS::ACQLVL) <= 1);
                        // Ensure Case 1 did happen
                        assert!(regs.fifo_status.read(FIFO_STATUS::TXLVL) == 0);
                        // We are mid read, and the host is expecting more data.
                        let len = self.slave_read_len.get();
                        let the_sauce = self
                            .slave_read_buffer
                            .take()
                            .expect("i2c-target: slave read buffer lost");
                        if len - self.slave_read_ofst.get() > 1 {
                            // We need more data, because we sent all of the current buffer.
                            // but have not received a STOP/RESTART.
                            self.slave_client.map(|client| {
                                client.command_complete(
                                    the_sauce,
                                    self.slave_read_ofst.get() + 1,
                                    hil::i2c::SlaveTransmissionType::Read,
                                );
                                client.read_expected();
                                self.slave_reset_internal_state();
                            });
                        } else {
                            // Unload as much as we can
                            let offset = self.slave_read_ofst.get();
                            for i in offset..len {
                                if regs.fifo_status.read(FIFO_STATUS::TXLVL) >= I2C_PARAM_FIFO_DEPTH
                                {
                                    // TXFIFO full
                                    self.slave_read_ofst.set(i);
                                    break;
                                }
                                regs.txdata.write(TXDATA::TXDATA.val(the_sauce[i] as u32));
                                self.slave_read_ofst.set(i);
                            }
                            self.slave_read_buffer.replace(the_sauce);
                            // Rest is interrupt based
                        }
                    }
                    _ => unreachable!("i2c-target: Unexpected TX_STRETCH interrupt"),
                }
            }

            if irqs.is_set(INTR::ACQ_FULL) {
                // If the host tries to write a data byte into the ACQ FIFO when there is
                // no available space, the clock is also stretched after the ACK bit.
                // The ACQ_FULL interrupt is generated to alert software to such a situation
                // TODO: The docs don't say explicitly that this generates a TX_STRETCH, make sure it doesn't
                match self.slave_pending_ops.get() {
                    I2CSlavePendingOps::WriteRecvPending => {
                        // 1. The most likely case is we are picking up from a partially complete write receive.
                        // 2. It could also just be a massive write from the host from scratch, that exceeds the
                        // ACQFIFO depth. So it's still a partial transfer.
                        self.slave_handle_partial_write_recv();
                    }
                    _ => unreachable!("i2c-target: Unexpected ACQ_FULL interrupt"),
                }
            }

            if irqs.is_set(INTR::TX_OVERFLOW) {
                // We check the TXFIFO_LVL when loading, so we shouldn't trigger an overflow.
                unreachable!("i2c-target: Unexpected TX_OVERFLOW");
            }

            if irqs.is_set(INTR::UNEXP_STOP) {
                // STOP is received without a preceding NACK during an external host read
                // This interrupt just means that a STOP was unexpectedly observed during a host read.
                // It is not necessarily harmful, but software can be made aware just in case.
                todo!("i2c-target: Host UNEXP_STOP");
            }

            if irqs.is_set(INTR::HOST_TIMEOUT) {
                //  Host stopped sending the clock during an ongoing transaction.
                //  TODO: Returns any buffers to client and reset self
                todo!("i2c-target: Unexpected HOST_TIMEOUT");
            }
        } else {
            unreachable!("i2c-target: Unexpected I2C interrupt!")
        }
    }

    fn timing_parameter_init(&self, clock_period_nanos: u32) {
        let regs = self.registers;
        // Timing values based on opentitan/sw/device/lib/testing/i2c_testutils.c
        // Setup the timing variables for Fast I2C
        regs.timing0.modify(
            TIMING0::THIGH.val(div_up(600, clock_period_nanos))
                + TIMING0::TLOW.val(div_up(1300, clock_period_nanos)),
        );
        regs.timing1.modify(
            TIMING1::T_F.val(div_up(110, clock_period_nanos))
                + TIMING1::T_R.val(div_up(400, clock_period_nanos)),
        );
        regs.timing2.modify(
            TIMING2::THD_STA.val(div_up(600, clock_period_nanos))
                + TIMING2::TSU_STA.val(div_up(600, clock_period_nanos)),
        );
        regs.timing3.modify(
            TIMING3::THD_DAT.val(1) + TIMING3::TSU_DAT.val(div_up(100, clock_period_nanos)),
        );
        regs.timing4.modify(
            TIMING4::T_BUF.val(div_up(1300, clock_period_nanos))
                + TIMING4::TSU_STO.val(div_up(600, clock_period_nanos)),
        );
    }

    fn fifo_reset(&self) {
        let regs = self.registers;

        regs.fifo_ctrl.modify(
            FIFO_CTRL::RXRST::SET
                + FIFO_CTRL::TXRST::SET
                + FIFO_CTRL::FMTRST::SET
                + FIFO_CTRL::ACQRST::SET,
        );
        //Make sure the FIFOs are actually reset
        assert_eq!(regs.fifo_status.read(FIFO_STATUS::ACQLVL), 0);
        assert_eq!(regs.fifo_status.read(FIFO_STATUS::RXLVL), 0);
        assert_eq!(regs.fifo_status.read(FIFO_STATUS::TXLVL), 0);
        assert_eq!(regs.fifo_status.read(FIFO_STATUS::FMTLVL), 0);
    }

    fn read_data(&self) {
        let regs = self.registers;
        let mut data_popped = self.read_index.get();
        let len = self.read_len.get();

        self.buffer.map(|buf| {
            for i in self.read_index.get()..len {
                if regs.status.is_set(STATUS::RXEMPTY) {
                    // The RX buffer is empty
                    data_popped = i;
                    break;
                }
                // Read the data
                buf[i] = regs.rdata.read(RDATA::RDATA) as u8;
                data_popped = i;
            }

            if data_popped == len {
                // Finished call the callback
                self.master_client.map(|client| {
                    client.command_complete(self.buffer.take().unwrap(), Ok(()));
                });
            } else {
                self.read_index.set(data_popped + 1);

                // Update the FIFO depth
                if len - data_popped > 8 {
                    regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL8);
                } else if len - data_popped > 4 {
                    regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL4);
                } else {
                    regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL1);
                }
            }
        });
    }

    fn write_data(&self) {
        let regs = self.registers;
        let mut data_pushed = self.write_index.get();
        let len = self.write_len.get();

        self.buffer.map(|buf| {
            for i in self.write_index.get()..(len - 1) {
                if regs.status.read(STATUS::FMTFULL) != 0 {
                    // The FMT buffer is full
                    data_pushed = i;
                    break;
                }
                // Send the data
                regs.fdata
                    .write(FDATA::FBYTE.val(*buf.get(i).unwrap_or(&0) as u32));
                data_pushed = i;
            }

            // Check if we can send the last byte
            if regs.status.read(STATUS::FMTFULL) == 0 && data_pushed == (len - 1) {
                // Send the last byte with the stop signal
                regs.fdata
                    .write(FDATA::FBYTE.val(*buf.get(len).unwrap_or(&0) as u32) + FDATA::STOP::SET);

                data_pushed = len;
            }

            if data_pushed == len {
                // Finished call the callback
                self.master_client.map(|client| {
                    client.command_complete(self.buffer.take().unwrap(), Ok(()));
                });
            } else {
                self.write_index.set(data_pushed + 1);

                // Update the FIFO depth
                if len - data_pushed > 8 {
                    regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL8);
                } else if len - data_pushed > 4 {
                    regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL4);
                } else {
                    regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL1);
                }
            }
        });
    }

    fn write_read_data(&self) {
        let regs = self.registers;
        let mut data_pushed = self.write_index.get();
        let len = self.write_len.get();

        self.buffer.map(|buf| {
            let start_index = data_pushed;
            for i in start_index..(len - 1) {
                if regs.status.read(STATUS::FMTFULL) != 0 {
                    // The FMT buffer is full
                    data_pushed = i;
                    break;
                }
                // Send the data
                regs.fdata
                    .write(FDATA::FBYTE.val(*buf.get(i).unwrap_or(&0) as u32));
                data_pushed = i;
            }

            // Check if we can send the last byte
            if regs.status.read(STATUS::FMTFULL) == 0 && data_pushed == (len - 1) {
                // Send the last byte with the stop signal
                regs.fdata
                    .write(FDATA::FBYTE.val(*buf.get(len).unwrap_or(&0) as u32) + FDATA::STOP::SET);

                data_pushed = len;
            }

            if data_pushed == len {
                // Finished writing. Read the data as well.
                // Set the LSB to signal a read
                let read_addr = self.slave_read_address.get() | 1;

                // Set the start condition and the address
                regs.fdata
                    .write(FDATA::START::SET + FDATA::FBYTE.val(read_addr as u32));

                self.read_data();
            } else {
                self.write_index.set(data_pushed + 1);

                // Update the FIFO depth
                if len - data_pushed > 8 {
                    regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL8);
                } else if len - data_pushed > 4 {
                    regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL4);
                } else {
                    regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL1);
                }
            }
        });
    }
}

//TODO: Keep this until div_ceil() is stable
pub fn div_up(a: u32, b: u32) -> u32 {
    assert_ne!(b, 0);
    (a + (b - 1)) / b
}

impl<'a> hil::i2c::I2CMaster<'a> for I2c<'a> {
    fn set_master_client(&self, master_client: &'a dyn i2c::I2CHwMasterClient) {
        self.master_client.set(master_client);
    }

    fn enable(&self) {
        let regs = self.registers;

        if self.is_enabled() {
            // Simultaneous operation of running host/target is not supported
            // Also if we are currently enabled, disable first!
            return;
        }

        self.timing_parameter_init(self.clock_period_nanos);
        self.fifo_reset();

        // Enable all host interrupts
        regs.intr_enable.modify(
            INTR::FMT_THRESHOLD::SET
                + INTR::RX_THRESHOLD::SET
                + INTR::FMT_OVERFLOW::SET
                + INTR::RX_OVERFLOW::SET
                + INTR::NAK::SET
                + INTR::SCL_INTERFERENCE::SET
                + INTR::SDA_INTERFERENCE::SET
                + INTR::STRETCH_TIMEOUT::SET
                + INTR::SDA_UNSTABLE::SET
                + INTR::CMD_COMPLETE::SET,
        );

        // Enable I2C Host
        regs.ctrl.modify(CTRL::ENABLEHOST::SET);
    }

    fn disable(&self) {
        let regs = self.registers;

        // Disable all host interrupts
        regs.intr_enable.modify(
            INTR::FMT_THRESHOLD::SET
                + INTR::RX_THRESHOLD::SET
                + INTR::FMT_OVERFLOW::SET
                + INTR::RX_OVERFLOW::SET
                + INTR::NAK::SET
                + INTR::SCL_INTERFERENCE::SET
                + INTR::SDA_INTERFERENCE::SET
                + INTR::STRETCH_TIMEOUT::SET
                + INTR::SDA_UNSTABLE::SET
                + INTR::CMD_COMPLETE::SET,
        );

        regs.ctrl.modify(CTRL::ENABLEHOST::CLEAR);
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        // Set the FIFO depth and reset the FIFO
        if write_len > 8 {
            regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL8);
        } else if write_len > 4 {
            regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL4);
        } else {
            regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL1);
        }

        if read_len > 8 {
            regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL8);
        } else if read_len > 4 {
            regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL4);
        } else {
            regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL1);
        }

        self.fifo_reset();

        // Zero out the LSB to signal a write
        let write_addr = addr & !1;

        // Set the start condition and the address
        regs.fdata
            .write(FDATA::START::SET + FDATA::FBYTE.val(write_addr as u32));

        // Save all the data and offsets we still need to send and receive
        self.slave_read_address.set(addr);
        self.buffer.replace(data);
        self.write_len.set(write_len);
        self.read_len.set(read_len);
        self.write_index.set(0);
        self.read_index.set(0);

        self.write_read_data();

        Ok(())
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        // Set the FIFO depth and reset the FIFO
        if len > 8 {
            regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL8);
        } else if len > 4 {
            regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL4);
        } else {
            regs.fifo_ctrl.modify(FIFO_CTRL::FMTILVL::FMTLVL1);
        }

        self.fifo_reset();

        // Zero out the LSB to signal a write
        let write_addr = addr & !1;

        // Set the start condition and the address
        regs.fdata
            .write(FDATA::START::SET + FDATA::FBYTE.val(write_addr as u32));

        // Save all the data and offsets we still need to send
        self.slave_read_address.set(0);
        self.buffer.replace(data);
        self.write_len.set(len);
        self.write_index.set(0);

        self.write_data();

        Ok(())
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        // Set the FIFO depth and reset the FIFO
        if len > 8 {
            regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL8);
        } else if len > 4 {
            regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL4);
        } else {
            regs.fifo_ctrl.modify(FIFO_CTRL::RXILVL::RXLVL1);
        }

        self.fifo_reset();

        // Set the LSB to signal a read
        let read_addr = addr | 1;

        // Set the start condition and the address
        regs.fdata
            .write(FDATA::START::SET + FDATA::FBYTE.val(read_addr as u32));

        // Save all the data and offsets we still need to read
        self.slave_read_address.set(0);
        self.buffer.replace(buffer);
        self.read_len.set(len);
        self.read_index.set(0);

        self.read_data();

        Ok(())
    }
}

impl<'a> hil::i2c::I2CSlave<'a> for I2c<'a> {
    fn set_slave_client(&self, slave_client: &'a dyn I2CHwSlaveClient) {
        self.slave_client.set(slave_client);
    }

    fn enable(&self) {
        let regs = self.registers;

        if self.is_enabled() {
            // Simultaneous operation of running host/target is not supported
            // Also if we are currently enabled, disable first!
            return;
        }
        // Setup timing (i2c-fast)
        self.timing_parameter_init(self.clock_period_nanos);

        // Enable all target interrupts
        regs.intr_enable.modify(
            INTR::TX_STRETCH::SET
                + INTR::TX_OVERFLOW::SET
                + INTR::ACQ_FULL::SET
                + INTR::UNEXP_STOP::SET
                + INTR::HOST_TIMEOUT::SET
                + INTR::CMD_COMPLETE::SET,
        );

        // Enable I2C Target
        regs.ctrl.modify(CTRL::ENABLETARGET::SET);
        // Clear out any leftover junk
        // TODO: Assert for dev debug, we don't want to reset fifos if an op pending
        assert_eq!(self.slave_pending_ops.get(), I2CSlavePendingOps::Idle);
        self.fifo_reset();
    }

    fn disable(&self) {
        let regs = self.registers;

        // Disable all target interrupts
        regs.intr_enable.modify(
            INTR::TX_STRETCH::CLEAR
                + INTR::TX_OVERFLOW::CLEAR
                + INTR::ACQ_FULL::CLEAR
                + INTR::UNEXP_STOP::CLEAR
                + INTR::HOST_TIMEOUT::CLEAR
                + INTR::CMD_COMPLETE::SET,
        );

        regs.ctrl.modify(CTRL::ENABLETARGET::CLEAR);
    }

    fn set_address(&self, addr: u8) -> Result<(), hil::i2c::Error> {
        let regs = self.registers;
        // Address is 7-bit LSB
        if addr > 127 {
            //TODO: add new err type to HIL
            return Err(hil::i2c::Error::NotSupported);
        }
        self.slave_client_addr.set(addr);
        // Received Address mask must equal the programmed address to activate I2C Device.
        // If (address & !mask) != 0, this will not match any addresses.
        // Don't listen by default with (0x7f & !0)
        // Note: HW supports 2-target addresses, we only use one for now.
        regs.target_id
            .write(TARGET_ID::ADDRESS0.val(0x7f) + TARGET_ID::MASK0.val(0));
        Ok(())
    }

    fn write_receive(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let mut transaction_pending = false;
        match self.slave_pending_ops.get() {
            I2CSlavePendingOps::Idle => {}
            I2CSlavePendingOps::WriteRecvPending => {
                // We got here from a write_expected(), meaning there's more data to be read,
                // now into the `new` buffer.
                transaction_pending = true;
            }
            // TODO: Kernel does not error handle these returns
            _ => return Err((hil::i2c::Error::Busy, data)),
        }

        if max_len > data.len() {
            return Err((hil::i2c::Error::NotSupported, data));
        }
        self.slave_reset_internal_state();
        // Capture buffer
        self.slave_write_buffer.replace(data);
        self.slave_write_len.set(max_len);
        // Set pending op
        self.slave_pending_ops
            .set(I2CSlavePendingOps::WriteRecvPending);

        if transaction_pending {
            self.slave_write_recv();
        }
        // Wait for the kernel to get things going with enable() & listen()
        // Rest is handled on an IRQ basis
        Ok(())
    }

    fn read_send(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        match self.slave_pending_ops.get() {
            I2CSlavePendingOps::Idle => {}
            // TODO: Kernel does not error handle these returns
            _ => return Err((hil::i2c::Error::Busy, data)),
        }

        if max_len > data.len() {
            return Err((hil::i2c::Error::NotSupported, data));
        }

        let regs = self.registers;

        // Load as much data as we can to TXDATA now, in aticipation of a read.
        // Also make sure that theres enough space in ACQFIFO to receive the STOP/NACK
        if regs.fifo_status.read(FIFO_STATUS::ACQLVL) + 2 >= I2C_PARAM_FIFO_DEPTH {
            // Check there's enough space in the ACQFIFO for the expected NACK/STOP
            // TODO: Reset ACQFIFO? or brush under the rug?
            todo!("ACQFIFO cannot receive the next NACK/STOP");
        }

        self.slave_reset_internal_state();

        for i in 0..max_len {
            if regs.fifo_status.read(FIFO_STATUS::TXLVL) >= I2C_PARAM_FIFO_DEPTH {
                // TXFIFO full
                self.slave_read_ofst.set(i);
                break;
            }
            regs.txdata.write(TXDATA::TXDATA.val(data[i] as u32));
            self.slave_read_ofst.set(i);
        }
        // Capture buffer
        self.slave_read_buffer.replace(data);
        self.slave_read_len.set(max_len);
        // Set pending op
        self.slave_pending_ops
            .set(I2CSlavePendingOps::ReadSendPending);
        // Rest is handled on an IRQ basis
        Ok(())
    }

    fn listen(&self) {
        let regs = self.registers;
        // With masks set to all ones (0x7F), the target device will respond
        // to either of the two assigned unique addresses and no other.
        regs.target_id.write(
            TARGET_ID::ADDRESS0.val(u32::from(self.slave_client_addr.get()))
                + TARGET_ID::MASK0.val(0x7f),
        );
    }
}
