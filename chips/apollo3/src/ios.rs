// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! IO Slave Driver (I2C and SPI)
//!
//! This file provides support for the Apollo3 IOS. The IOS is a little
//! strange in that I2C operations go straight to a local RAM area.
//!
//! The first byte of data (after the I2C address and operation byte)
//! will be interpreteted as an address offset by the hardware.
//!
//! If the offset was between 0x00 and 0x77 the data will be in the
//! RAM which we can access from 0x5000_0000 to 0x5000_0077. This
//! will generate the XCMPWR interrupt on writes from the master.
//!
//! If the offset was the following it is written to the interrupt
//! or FIFO registers. This will generate a XCMPWF on writes from the
//! master:
//!     - 0x78-7B -> IOINT Regs
//!     - 0x7C    -> FIFOCTRLO
//!     - 0x7D    -> FIFOCTRUP
//!     - 0x7F    -> FIFO (DATA)
//!
//! Unfortunately we have no way to know where the data was written.
//!
//! We currently don't support the FIFO registers. As there is no way
//! to know where the data was written we assume it was written to offset
//! 0x0F. This is the first non interrupt flag generating address.
//! This also matches the first byte of a MCTP packet.
//!
//! So, if you would like to write data to this device, the first byte of
//! data must be 0x0F.

use crate::ios::i2c::SlaveTransmissionType;
use core::cell::Cell;
use kernel::debug;
use kernel::hil::i2c::{self, Error, I2CHwSlaveClient, I2CSlave};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

const SRAM_ROBASE_OFFSET: u32 = 0x78;

const IOS_BASE: StaticRef<IosRegisters> =
    unsafe { StaticRef::new(0x5000_0100 as *const IosRegisters) };

register_structs! {
    pub IosRegisters {
        (0x000 => fifoptr: ReadWrite<u32, FIFOPTR::Register>),
        (0x004 => fifocfg: ReadWrite<u32, FIFOCFG::Register>),
        (0x008 => fifothr: ReadWrite<u32, FIFOTHR::Register>),
        (0x00C => fupd: ReadWrite<u32, FUPD::Register>),
        (0x010 => fifoctr: ReadWrite<u32, FIFOCTR::Register>),
        (0x014 => fifoinc : ReadWrite<u32, FIFOINC::Register>),
        (0x018 => cfg: ReadWrite<u32, CFG::Register>),
        (0x01C => prenc: ReadWrite<u32, PRENC::Register>),
        (0x020 => iointctl: ReadWrite<u32, IOINTCTL::Register>),
        (0x024 => genadd: ReadOnly<u32, GENADD::Register>),
        (0x028 => _reserved2),
        (0x100 => inten: ReadWrite<u32, INT::Register>),
        (0x104 => intstat: ReadWrite<u32, INT::Register>),
        (0x108 => intclr: ReadWrite<u32, INT::Register>),
        (0x10C => intset: ReadWrite<u32, INT::Register>),
        (0x110 => regaccinten: ReadWrite<u32, REGACC::Register>),
        (0x114 => regaccintstat: ReadWrite<u32, REGACC::Register>),
        (0x118 => regaccintclr: ReadWrite<u32, REGACC::Register>),
        (0x11C => regaccintset: ReadWrite<u32, REGACC::Register>),
        (0x120 => @END),
    }
}

register_bitfields![u32,
    FIFOPTR [
        FIFOPTR OFFSET(0) NUMBITS(8) [],
        FIFOSIZ OFFSET(8) NUMBITS(8) []
    ],
    FIFOCFG [
        FIFOBASE OFFSET(0) NUMBITS(5) [],
        FIFOMAX OFFSET(8) NUMBITS(6) [],
        ROBASE OFFSET(24) NUMBITS(6) [],
    ],
    FIFOTHR [
        FIFOTHR OFFSET(8) NUMBITS(8) []
    ],
    FUPD [
        FIFOUPD OFFSET(0) NUMBITS(1) [],
        IOREAD OFFSET(1) NUMBITS(1) []
    ],
    FIFOCTR [
        FIFOCTR OFFSET(0) NUMBITS(10) []
    ],
    FIFOINC [
        FIFOINC OFFSET(0) NUMBITS(10) []
    ],
    CFG [
        IFCSEL OFFSET(0) NUMBITS(1) [],
        SPOL OFFSET(1) NUMBITS(1) [],
        LSB OFFSET(2) NUMBITS(1) [],
        STARTRD OFFSET(4) NUMBITS(1) [],
        I2CADDR OFFSET(8) NUMBITS(11) [],
        IFCEN OFFSET(31) NUMBITS(1) []
    ],
    PRENC [
        PRENC OFFSET(0) NUMBITS(5) []
    ],
    IOINTCTL [
        IOINTEN OFFSET(0) NUMBITS(8) [],
        IOINT OFFSET(8) NUMBITS(8) [],
        IOINTCLR OFFSET(16) NUMBITS(1) [],
        IOINTSET OFFSET(24) NUMBITS(8) []
    ],
    GENADD [
        GADATA OFFSET(0) NUMBITS(8) [],
    ],
    INT [
        FSIZE OFFSET(0) NUMBITS(1) [],
        FOVFL OFFSET(1) NUMBITS(1) [],
        FUNDFL OFFSET(2) NUMBITS(1) [],
        FRDERR OFFSET(3) NUMBITS(1) [],
        GENAD OFFSET(4) NUMBITS(1) [],
        IOINTW OFFSET(5) NUMBITS(1) [],
        XCMPRF OFFSET(6) NUMBITS(1) [],
        XCMPRR OFFSET(7) NUMBITS(1) [],
        XCMPWF OFFSET(8) NUMBITS(1) [],
        XCMPWR OFFSET(9) NUMBITS(1) []
    ],
    REGACC [
        REGACC OFFSET(0) NUMBITS(32) []
    ]
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    None,
    I2C,
}

pub struct Ios<'a> {
    registers: StaticRef<IosRegisters>,

    i2c_slave_client: OptionalCell<&'a dyn I2CHwSlaveClient>,

    write_buf: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,
    read_buf: TakeCell<'static, [u8]>,
    read_len: Cell<usize>,
    op: Cell<Operation>,
}

impl<'a> Ios<'a> {
    pub fn new() -> Ios<'a> {
        Ios {
            registers: IOS_BASE,
            i2c_slave_client: OptionalCell::empty(),
            write_buf: TakeCell::empty(),
            write_len: Cell::new(0),
            read_buf: TakeCell::empty(),
            read_len: Cell::new(0),
            op: Cell::new(Operation::None),
        }
    }

    fn i2c_interface_enable(&self) {
        let regs = self.registers;

        regs.cfg.modify(CFG::IFCEN::SET);
        regs.cfg.modify(CFG::IFCSEL::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        let irqs = self.registers.intstat.extract();

        // Clear interrrupts
        self.registers.intclr.set(0xFFFF_FFFF);
        self.registers.regaccintclr.set(0xFFFF_FFFF);
        // Ensure interrupts remain enabled
        self.registers.inten.set(0xFFFF_FFFF);
        self.registers.regaccinten.set(0xFFFF_FFFF);

        let _offset = self.registers.fifoptr.read(FIFOPTR::FIFOPTR);

        if irqs.is_set(INT::XCMPWR) {
            // If we get here that means the I2C master has written something
            // addressed to us and the offset is in the "Direct Area", between
            // 0x00 and 0x77.
            //
            // Unfortunately we have no way to know where the data was written,
            // so we assume it starts at 0x0F.
            let len = (SRAM_ROBASE_OFFSET as usize).min(self.write_len.get());

            self.write_buf.take().map(|buf| {
                buf[0] = 0x0F;

                for i in 1..len {
                    unsafe {
                        buf[i] = *((0x5000_000F + (i as u32 - 1)) as *mut u8);
                        // Zero the data after we read it
                        *((0x5000_000F + (i as u32 - 1)) as *mut u8) = 0x00;
                    }
                }

                self.i2c_slave_client.get().map(|client| {
                    client.command_complete(buf, len, SlaveTransmissionType::Write);
                });
            });
        }

        if irqs.is_set(INT::XCMPWF) {
            // If we get here that means the I2C master has written something
            // addressed to us and the offset is in the "FIFO Area", 0x7F

            // We currently don't support the FIFO area. We have no way
            // to report errors, so let's just print something.

            debug!("Write to the FIFO area, which is not currently supported");
        }
    }
}

impl<'a> I2CSlave<'a> for Ios<'a> {
    fn set_slave_client(&self, slave_client: &'a dyn i2c::I2CHwSlaveClient) {
        self.i2c_slave_client.set(slave_client);
    }

    fn enable(&self) {
        self.op.set(Operation::I2C);

        // Eliminate the "read-only" section, so an external host can use the
        // entire "direct write" section.
        self.registers
            .fifocfg
            .modify(FIFOCFG::ROBASE.val(SRAM_ROBASE_OFFSET / 8));

        // Set the FIFO base to the maximum value, making the "direct write"
        // section as big as possible.
        self.registers
            .fifocfg
            .modify(FIFOCFG::FIFOBASE.val(SRAM_ROBASE_OFFSET / 8));

        // We don't need any RAM space, so extend the FIFO all the way to the end
        // of the LRAM.
        self.registers
            .fifocfg
            .modify(FIFOCFG::FIFOMAX.val(0x100 / 8));

        // Clear FIFOs
        self.registers.fifoctr.modify(FIFOCTR::FIFOCTR.val(0x00));
        self.registers.fifoptr.modify(FIFOPTR::FIFOSIZ.val(0x00));

        // Setup FIFO interrupt threshold
        self.registers.fifothr.modify(FIFOTHR::FIFOTHR.val(0x08));

        self.i2c_interface_enable();

        // Clear interrrupts
        self.registers.intclr.set(0xFFFF_FFFF);

        // Update the FIFO
        self.registers.fupd.modify(FUPD::FIFOUPD::SET);
        self.registers.fifoptr.modify(FIFOPTR::FIFOPTR.val(0x80));
        self.registers.fupd.modify(FUPD::FIFOUPD::CLEAR);
    }

    fn disable(&self) {
        if self.op.get() == Operation::I2C {
            self.registers.cfg.modify(CFG::IFCEN::CLEAR);

            self.op.set(Operation::None);
        }
    }

    fn set_address(&self, addr: u8) -> Result<(), Error> {
        self.registers
            .cfg
            .modify(CFG::I2CADDR.val((addr as u32) << 1));

        Ok(())
    }

    fn write_receive(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        self.write_len.set(max_len.min(data.len()));
        self.write_buf.replace(data);

        Ok(())
    }

    fn read_send(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        for (i, d) in data.iter().enumerate() {
            unsafe {
                *((0x5000_0000 + 0x7F + (i as u32)) as *mut u8) = *d;
            }
        }

        self.read_len.set(max_len.min(data.len()));
        self.read_buf.replace(data);

        Ok(())
    }

    fn listen(&self) {
        self.registers.inten.modify(
            INT::FSIZE::SET
                + INT::FOVFL::SET
                + INT::FUNDFL::SET
                + INT::FRDERR::SET
                + INT::GENAD::SET
                + INT::IOINTW::SET
                + INT::XCMPRF::SET
                + INT::XCMPRF::SET
                + INT::XCMPWF::SET
                + INT::XCMPWR::SET,
        );
    }
}
