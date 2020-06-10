//! I2C Master Driver

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::i2c;

register_structs! {
    pub I2cRegisters {
        (0x00 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x04 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x08 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x0C => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x10 => status: ReadOnly<u32, STATUS::Register>),
        (0x14 => rdata: ReadOnly<u32, RDATA::Register>),
        (0x18 => fdata: WriteOnly<u32, FDATA::Register>),
        (0x1C => fifo_ctrl: ReadWrite<u32, FIFO_CTRL::Register>),
        (0x20 => fifo_status: ReadOnly<u32, FIFO_STATUS::Register>),
        (0x24 => ovrd: ReadWrite<u32, OVRD::Register>),
        (0x28 => val: ReadOnly<u32, VAL::Register>),
        (0x2C => timing0: ReadWrite<u32, TIMING0::Register>),
        (0x30 => timing1: ReadWrite<u32, TIMING1::Register>),
        (0x34 => timing2: ReadWrite<u32, TIMING2::Register>),
        (0x38 => timing3: ReadWrite<u32, TIMING3::Register>),
        (0x3C => timing4: ReadWrite<u32, TIMING4::Register>),
        (0x40 => timeout_ctrl: ReadWrite<u32, TIMEOUT_CTRL::Register>),
        (0x44 => @END),
    }
}

register_bitfields![u32,
    INTR [
        FMT_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        FMT_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        NAK OFFSET(4) NUMBITS(1) [],
        SCL_INTERFERENCE OFFSET(5) NUMBITS(1) [],
        SDA_INTERFERENCE OFFSET(6) NUMBITS(1) [],
        STRETCH_TIMEOUT OFFSET(7) NUMBITS(1) [],
        SDA_UNSTABLE OFFSET(8) NUMBITS(1) []
    ],
    CTRL [
        ENABLEHOST OFFSET(0) NUMBITS(1) []
    ],
    STATUS [
        FMTFULL OFFSET(0) NUMBITS(1) [],
        RXFULL OFFSET(1) NUMBITS(1) [],
        FMTEMPTY OFFSET(2) NUMBITS(1) [],
        HOSTIDLE OFFSET(3) NUMBITS(1) [],
        TARGETIDLE OFFSET(4) NUMBITS(1) [],
        RXEMPTY OFFSET(5) NUMBITS(1) []
    ],
    RDATA [
        RDATA OFFSET(0) NUMBITS(8) []
    ],
    FDATA [
        FBYTE OFFSET(0) NUMBITS(8) [],
        START OFFSET(8) NUMBITS(1) [],
        STOP OFFSET(9) NUMBITS(1) [],
        READ OFFSET(10) NUMBITS(1) [],
        RCONT OFFSET(11) NUMBITS(1) [],
        NAKOK OFFSET(12) NUMBITS(1) []
    ],
    FIFO_CTRL [
        RXRST OFFSET(0) NUMBITS(1) [],
        FMTRST OFFSET(1) NUMBITS(1) [],
        RXILVL OFFSET(2) NUMBITS(3) [
            RXLVL1 = 0,
            RXLVL4 = 1,
            RXLVL8 = 2,
            RXLVL16 = 3,
            RXLVL30 = 4
        ],
        FMTILVL OFFSET(5) NUMBITS(3) [
            FMTLVL1 = 0,
            FMTLVL4 = 1,
            FMTLVL8 = 2,
            FMTLVL16 = 3,
            FMTLVL30 = 4
        ]
    ],
    FIFO_STATUS [
        FMTLVL OFFSET(0) NUMBITS(6) [],
        RXLVL OFFSET(16) NUMBITS(6) []
    ],
    OVRD [
        TXOVRDEN OFFSET(0) NUMBITS(1) [],
        SCLVAL OFFSET(1) NUMBITS(1) [],
        SDAVAL OFFSET(2) NUMBITS(1) []
    ],
    VAL [
        SCL_RX OFFSET(0) NUMBITS(1) [],
        SDA_RX OFFSET(1) NUMBITS(1) []
    ],
    TIMING0 [
        THIGH OFFSET(0) NUMBITS(16) [],
        TLOW OFFSET(16) NUMBITS(16) []
    ],
    TIMING1 [
        T_R OFFSET(0) NUMBITS(16) [],
        T_F OFFSET(16) NUMBITS(16) []
    ],
    TIMING2 [
        TSU_STA OFFSET(0) NUMBITS(16) [],
        THD_STA OFFSET(16) NUMBITS(16) []
    ],
    TIMING3 [
        TSU_DAT OFFSET(0) NUMBITS(16) [],
        THD_DAT OFFSET(16) NUMBITS(16) []
    ],
    TIMING4 [
        TSU_STO OFFSET(0) NUMBITS(16) [],
        T_BUF OFFSET(16) NUMBITS(16) []
    ],
    TIMEOUT_CTRL [
        VAL OFFSET(0) NUMBITS(31) [],
        EN OFFSET(31) NUMBITS(1) []
    ]
];

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
    pub const fn new(base: StaticRef<I2cRegisters>, clock_period_nanos: u32) -> I2c<'a> {
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
            INTR::FMT_WATERMARK::SET
                + INTR::RX_WATERMARK::SET
                + INTR::FMT_OVERFLOW::SET
                + INTR::RX_OVERFLOW::SET
                + INTR::NAK::SET
                + INTR::SCL_INTERFERENCE::SET
                + INTR::SDA_INTERFERENCE::SET
                + INTR::STRETCH_TIMEOUT::SET
                + INTR::SDA_UNSTABLE::SET,
        );

        if irqs.is_set(INTR::FMT_WATERMARK) {
            // FMT Watermark
            if self.slave_read_address.get() != 0 {
                self.write_read_data();
            } else {
                self.write_data();
            }
        }

        if irqs.is_set(INTR::RX_WATERMARK) {
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
            for i in data_popped..len {
                if regs.status.is_set(STATUS::RXEMPTY) {
                    // The RX buffer is empty
                    data_popped = i;
                    break;
                }
                // Read the data
                buf[i as usize] = regs.rdata.read(RDATA::RDATA) as u8;
                data_popped = i;
            }

            if data_popped == len {
                // Finished call the callback
                self.master_client.map(|client| {
                    client.command_complete(
                        self.buffer.take().unwrap(),
                        hil::i2c::Error::CommandComplete,
                    );
                });
            } else {
                self.read_index.set(data_popped as usize + 1);

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
            for i in data_pushed..(len - 1) {
                if regs.status.read(STATUS::FMTFULL) != 0 {
                    // The FMT buffer is full
                    data_pushed = i;
                    break;
                }
                // Send the data
                regs.fdata.write(FDATA::FBYTE.val(buf[i as usize] as u32));
                data_pushed = i;
            }

            // Check if we can send the last byte
            if regs.status.read(STATUS::FMTFULL) == 0 && data_pushed == (len - 1) {
                // Send the last byte with the stop signal
                regs.fdata
                    .write(FDATA::FBYTE.val(buf[len as usize] as u32) + FDATA::STOP::SET);

                data_pushed = len;
            }

            if data_pushed == len {
                // Finished call the callback
                self.master_client.map(|client| {
                    client.command_complete(
                        self.buffer.take().unwrap(),
                        hil::i2c::Error::CommandComplete,
                    );
                });
            } else {
                self.write_index.set(data_pushed as usize + 1);

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
            for i in data_pushed..(len - 1) {
                if regs.status.read(STATUS::FMTFULL) != 0 {
                    // The FMT buffer is full
                    data_pushed = i;
                    break;
                }
                // Send the data
                regs.fdata.write(FDATA::FBYTE.val(buf[i as usize] as u32));
                data_pushed = i;
            }

            // Check if we can send the last byte
            if regs.status.read(STATUS::FMTFULL) == 0 && data_pushed == (len - 1) {
                // Send the last byte with the stop signal
                regs.fdata
                    .write(FDATA::FBYTE.val(buf[len as usize] as u32) + FDATA::STOP::SET);

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
                self.write_index.set(data_pushed as usize + 1);

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

impl<'a> hil::i2c::I2CMaster for I2c<'a> {
    fn set_master_client(&self, master_client: &'a dyn i2c::I2CHwMasterClient) {
        self.master_client.set(master_client);
    }

    fn enable(&self) {
        let regs = self.registers;

        self.timing_parameter_init(self.clock_period_nanos);
        self.fifo_reset();

        // Enable all interrupts
        regs.intr_enable.modify(
            INTR::FMT_WATERMARK::SET
                + INTR::RX_WATERMARK::SET
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

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
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
        self.write_len.set(write_len as usize);
        self.read_len.set(read_len as usize);
        self.write_index.set(0);
        self.read_index.set(0);

        self.write_read_data();
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
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
        self.write_len.set(len as usize);
        self.write_index.set(0);

        self.write_data();
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
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
        self.read_len.set(len as usize);
        self.read_index.set(0);

        self.read_data();
    }
}
