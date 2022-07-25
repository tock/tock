//! OTBN Control

use core::cell::Cell;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a> {
    /// This callback is called when a operation is computed.
    /// On error or success `output` will contain a reference to the original
    /// data supplied to `run()`.
    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8]);
}

register_structs! {
    pub OtbnRegisters {
        (0x00 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x04 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x08 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x0C => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => cmd: ReadWrite<u32, CMD::Register>),
        (0x14 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x18 => status: ReadOnly<u32, STATUS::Register>),
        (0x1C => err_bits: ReadOnly<u32, ERR_BITS::Register>),
        (0x20 => fatal_alert_cause: ReadOnly<u32, FATAL_ALERT_CAUSE::Register>),
        (0x24 => insn_cnt: ReadWrite<u32>),
        (0x28 => load_checksum: ReadWrite<u32>),
        (0x2C => _reserved0),
        (0x4000 => imem: [ReadWrite<u32>; 1024]),
        (0x5000 => _reserved1),
        (0x8000 => dmem: [ReadWrite<u32>; 1024]),
        (0x9000 => @END),
    }
}

register_bitfields![u32,
    INTR [
        DONE OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL OFFSET(0) NUMBITS(1) [],
        RECOV OFFSET(1) NUMBITS(1) [],
    ],
    CMD [
        CMD OFFSET(0) NUMBITS(8) [
            EXECUTE = 0xD8,
            SEC_WIPE_DMEM = 0xC3,
            SEC_WIPE_IMEM = 0x1E,
        ],
    ],
    CTRL [
        SOFTWARE_ERRS_FATAL OFFSET(0) NUMBITS(1) [],
    ],
    STATUS [
        STATUS OFFSET(0) NUMBITS(8) [
            IDLE = 0x00,
            BUSY_EXECUTE = 0x01,
            BUSY_SEC_WIPE_DMEM = 0x02,
            BUSY_SEC_WIPE_IMEM = 0x03,
            LOCKED = 0xFF,
        ],
    ],
    ERR_BITS [
        BAD_DATA_ADDR OFFSET(0) NUMBITS(1) [],
        BAD_INSN_ADDR OFFSET(1) NUMBITS(1) [],
        CALL_STACK OFFSET(2) NUMBITS(1) [],
        ILLEGAL_INSN OFFSET(3) NUMBITS(1) [],
        LOOP_BIT OFFSET(4) NUMBITS(1) [],
        IMEM_INTG_VIOLATION OFFSET(16) NUMBITS(1) [],
        DMEM_INTG_VIOLATION OFFSET(17) NUMBITS(1) [],
        REG_INTG_VIOLATION OFFSET(18) NUMBITS(1) [],
        BUS_INTG_VIOLATION OFFSET(19) NUMBITS(1) [],
        ILLEGAL_BUS_ACCESS OFFSET(20) NUMBITS(1) [],
        LIFECYCLE_ESCALATION OFFSET(21) NUMBITS(1) [],
        FATAL_SOFTWARE OFFSET(22) NUMBITS(1) [],
    ],
    FATAL_ALERT_CAUSE [
        IMEM_INTG_VIOLATION OFFSET(0) NUMBITS(1) [],
        DMEM_INTG_VIOLATION OFFSET(1) NUMBITS(1) [],
        REG_INTG_VIOLATION OFFSET(2) NUMBITS(1) [],
        BUS_INTG_VIOLATION OFFSET(3) NUMBITS(1) [],
        ILLEGAL_BUS_ACCESS OFFSET(4) NUMBITS(1) [],
        LIFECYCLE_ESCALATION OFFSET(5) NUMBITS(1) [],
        FATAL_SOFTWARE OFFSET(6) NUMBITS(1) [],
    ],
];

pub struct Otbn<'a> {
    registers: StaticRef<OtbnRegisters>,
    client: OptionalCell<&'a dyn Client<'a>>,

    out_buffer: TakeCell<'static, [u8]>,

    copy_address: Cell<usize>,
}

impl<'a> Otbn<'a> {
    pub fn new(base: StaticRef<OtbnRegisters>) -> Self {
        Otbn {
            registers: base,
            client: OptionalCell::empty(),
            out_buffer: TakeCell::empty(),
            copy_address: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        self.registers.intr_enable.set(0x00);
        self.registers.intr_state.set(0xFFFF_FFFF);

        // Check if there is an error
        if self.registers.err_bits.get() > 0 {
            self.client.map(|client| {
                self.out_buffer.take().map(|buf| {
                    client.op_done(Err(ErrorCode::FAIL), buf);
                })
            });
            return;
        }

        if self.registers.status.matches_all(STATUS::STATUS::IDLE) {
            let out_buf = self.out_buffer.take().unwrap();

            for i in 0..(out_buf.len() / 4) {
                let idx = i * 4;
                let d = self.registers.dmem[self.copy_address.get() / 4 + i]
                    .get()
                    .to_ne_bytes();

                out_buf[idx + 0] = d[0];
                out_buf[idx + 1] = d[1];
                out_buf[idx + 2] = d[2];
                out_buf[idx + 3] = d[3];
            }

            self.client.map(|client| {
                client.op_done(Ok(()), out_buf);
            });
        }
    }

    /// Set the client instance which will receive
    pub fn set_client(&'a self, client: &'a dyn Client<'a>) {
        self.client.set(client);
    }

    /// Load the acceleration binary data into the accelerator.
    /// This data will be accelerator specific and could be an
    /// elf file which will be run or could be binary settings used to
    /// configure the accelerator.
    /// This function can be called multiple times if multiple binary blobs
    /// are required.
    /// On error the return value will contain a return code and the original data
    pub fn load_binary(&self, input: &[u8]) -> Result<(), ErrorCode> {
        if !self.registers.status.matches_all(STATUS::STATUS::IDLE) {
            // OTBN is performing an operation, we can't make any changes
            return Err(ErrorCode::BUSY);
        }

        for i in 0..(input.len() / 4) {
            let idx = i * 4;

            let mut d = (input[idx + 0] as u32) << 0;
            d |= (input[idx + 1] as u32) << 8;
            d |= (input[idx + 2] as u32) << 16;
            d |= (input[idx + 3] as u32) << 24;

            self.registers.imem[i].set(d);
        }

        Ok(())
    }

    /// Load the data into the accelerator
    /// This function can be called multiple times if multiple loads
    /// are required.
    /// On error the return value will contain a return code and the original data
    /// The `data` buffer should be in little endian
    pub fn load_data(&self, address: usize, data: &[u8]) -> Result<(), ErrorCode> {
        if !self.registers.status.matches_all(STATUS::STATUS::IDLE) {
            // OTBN is performing an operation, we can't make any changes
            return Err(ErrorCode::BUSY);
        }

        for i in 0..(data.len() / 4) {
            let idx = i * 4;

            let mut d = (data[idx + 0] as u32) << 0;
            d |= (data[idx + 1] as u32) << 8;
            d |= (data[idx + 2] as u32) << 16;
            d |= (data[idx + 3] as u32) << 24;

            self.registers.dmem[(address / 4) + i].set(d);
        }

        Ok(())
    }

    /// Run the acceleration operation.
    /// This doesn't return any data, instead the client needs to have
    /// set a `op_done` handler to determine when this is complete.
    ///
    /// The data returned via `op_done()` will be starting at `address` and of
    /// the full length of `output`.
    ///
    /// On error the return value will contain a return code and the original data
    /// If there is data from the `load_binary()` command asyncrously waiting to
    /// be written it will be written before the operation starts.
    pub fn run(
        &self,
        address: usize,
        output: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if !self.registers.status.matches_all(STATUS::STATUS::IDLE) {
            // OTBN is performing an operation
            return Err((ErrorCode::BUSY, output));
        }

        self.registers.ctrl.modify(CTRL::SOFTWARE_ERRS_FATAL::CLEAR);

        // Clear and enable interrupts
        self.registers.intr_state.modify(INTR::DONE::SET);
        self.registers.intr_enable.modify(INTR::DONE::SET);

        self.out_buffer.replace(output);
        self.copy_address.set(address);

        self.registers.cmd.modify(CMD::CMD::EXECUTE);

        Ok(())
    }

    /// Clear the keys and any other sensitive data.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear those.
    pub fn clear_data(&self) {
        self.registers.cmd.write(CMD::CMD::SEC_WIPE_DMEM);
        self.registers.cmd.write(CMD::CMD::SEC_WIPE_IMEM);
    }
}
