// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! I2C Master Driver

use crate::registers::i2c_regs::{
    CTRL, FDATA, FIFO_CTRL, INTR, RDATA, STATUS, TIMING0, TIMING1, TIMING2, TIMING3, TIMING4,
};
use core::cell::Cell;
use kernel::hil;
use kernel::hil::i2c;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;

pub use crate::registers::i2c_regs::I2cRegisters;

pub struct I2c<'a> {
    registers: StaticRef<I2cRegisters>,
    clock_period_nanos: u32,

    master_client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,

    // Set when calling the write_read operation
    // This specifies the address of the read operation
    // after the write operation. Set to 0 for single read/write operations.
    slave_read_address: Cell<u8>,

    buffer: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,
    write_index: Cell<usize>,

    read_len: Cell<usize>,
    read_index: Cell<usize>,
}

impl<'a> I2c<'_> {
    pub fn new(base: StaticRef<I2cRegisters>, clock_period_nanos: u32) -> I2c<'a> {
        I2c {
            registers: base,
            clock_period_nanos,
            master_client: OptionalCell::empty(),
            slave_read_address: Cell::new(0),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irqs = regs.intr_state.extract();

        // Clear all interrupts
        regs.intr_state.modify(
            INTR::FMT_THRESHOLD::SET
                + INTR::RX_THRESHOLD::SET
                + INTR::FMT_OVERFLOW::SET
                + INTR::RX_OVERFLOW::SET
                + INTR::NAK::SET
                + INTR::SCL_INTERFERENCE::SET
                + INTR::SDA_INTERFERENCE::SET
                + INTR::STRETCH_TIMEOUT::SET
                + INTR::SDA_UNSTABLE::SET,
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
    }

    fn timing_parameter_init(&self, clock_period_nanos: u32) {
        let regs = self.registers;

        // Setup the timing variables for Fast I2C
        regs.timing0.modify(
            TIMING0::THIGH.val(600 / clock_period_nanos)
                + TIMING0::TLOW.val(1300 / clock_period_nanos),
        );
        regs.timing1
            .modify(TIMING1::T_F.val(167) + TIMING1::T_R.val(40));
        regs.timing2.modify(
            TIMING2::THD_STA.val(600 / clock_period_nanos)
                + TIMING2::TSU_STA.val(600 / clock_period_nanos),
        );
        regs.timing3
            .modify(TIMING3::THD_DAT.val(100 / clock_period_nanos) + TIMING3::TSU_DAT.val(0));
        regs.timing4.modify(
            TIMING4::T_BUF.val(600 / clock_period_nanos)
                + TIMING4::TSU_STO.val(1300 / clock_period_nanos),
        );
    }

    fn fifo_reset(&self) {
        let regs = self.registers;

        regs.fifo_ctrl
            .modify(FIFO_CTRL::RXRST::SET + FIFO_CTRL::FMTRST::SET);
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

impl<'a> hil::i2c::I2CMaster<'a> for I2c<'a> {
    fn set_master_client(&self, master_client: &'a dyn i2c::I2CHwMasterClient) {
        self.master_client.set(master_client);
    }

    fn enable(&self) {
        let regs = self.registers;

        self.timing_parameter_init(self.clock_period_nanos);
        self.fifo_reset();

        // Enable all interrupts
        regs.intr_enable.modify(
            INTR::FMT_THRESHOLD::SET
                + INTR::RX_THRESHOLD::SET
                + INTR::FMT_OVERFLOW::SET
                + INTR::RX_OVERFLOW::SET
                + INTR::NAK::SET
                + INTR::SCL_INTERFERENCE::SET
                + INTR::SDA_INTERFERENCE::SET
                + INTR::STRETCH_TIMEOUT::SET
                + INTR::SDA_UNSTABLE::SET,
        );

        // Enable I2C Host
        regs.ctrl.modify(CTRL::ENABLEHOST::SET);
    }

    fn disable(&self) {
        let regs = self.registers;

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
