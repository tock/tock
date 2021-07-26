//! OTBN Control

use core::cell::Cell;
use kernel::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a, const T: usize> {
    /// This callback is called when the binary data has been loaded
    /// On error or success `input` will contain a reference to the original
    /// data supplied to `load_binary()`.
    fn binary_load_done(&'a self, result: Result<(), ErrorCode>, input: &'static mut [u8]);

    /// This callback is called when a operation is computed.
    /// On error or success `output` will contain a reference to the original
    /// data supplied to `run()`.
    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8; T]);
}

register_structs! {
    pub OtbnRegisters {
        (0x00 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x04 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x08 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x0C => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => cmd: ReadWrite<u32, CMD::Register>),
        (0x14 => status: ReadOnly<u32, STATUS::Register>),
        (0x18 => err_bits: ReadOnly<u32, ERR_BITS::Register>),
        (0x1C => start_addr: ReadWrite<u32, START_ADDR::Register>),
        (0x20 => fatal_alert_cause: ReadOnly<u32, FATAL_ALERT_CAUSE::Register>),
        (0x24 => _reserved0),
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
        START OFFSET(0) NUMBITS(1) [],
    ],
    STATUS [
        BUSY OFFSET(0) NUMBITS(1) [],
    ],
    ERR_BITS [
        BAD_DATA_ADDR OFFSET(0) NUMBITS(1) [],
        BAD_INSN_ADDR OFFSET(1) NUMBITS(1) [],
        CALL_STACK OFFSET(2) NUMBITS(1) [],
        ILLEGAL_INSN OFFSET(3) NUMBITS(1) [],
        LOOP_BIT OFFSET(4) NUMBITS(1) [],
        FATAL_IMEM OFFSET(5) NUMBITS(1) [],
        FATAL_DMEM OFFSET(6) NUMBITS(1) [],
        FATAL_REG OFFSET(7) NUMBITS(1) [],
    ],
    START_ADDR [
        START_ADDR OFFSET(0) NUMBITS(32) [],
    ],
    FATAL_ALERT_CAUSE [
        BUS_INTEGRITY_ERROR OFFSET(0) NUMBITS(1) [],
        IMEM_ERROR OFFSET(1) NUMBITS(1) [],
        DMEM_ERROR OFFSET(2) NUMBITS(1) [],
        REG_ERROR OFFSET(3) NUMBITS(1) [],
    ],
];

pub struct Otbn<'a> {
    registers: StaticRef<OtbnRegisters>,
    client: OptionalCell<&'a dyn Client<'a, 1024>>,

    in_buffer: Cell<Option<LeasableBuffer<'static, u8>>>,
    out_buffer: TakeCell<'static, [u8; 1024]>,

    add_data_deferred_call: Cell<bool>,
    deferred_caller: &'static DynamicDeferredCall,
    deferred_handle: OptionalCell<DeferredCallHandle>,
}

impl<'a> Otbn<'a> {
    pub const fn new(
        base: StaticRef<OtbnRegisters>,
        deferred_caller: &'static DynamicDeferredCall,
    ) -> Self {
        Otbn {
            registers: base,
            client: OptionalCell::empty(),
            in_buffer: Cell::new(None),
            out_buffer: TakeCell::empty(),

            add_data_deferred_call: Cell::new(false),
            deferred_caller,
            deferred_handle: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        // Check if there is an error
        if self.registers.err_bits.get() > 0 {
            self.client.map(|client| {
                self.out_buffer.take().map(|buf| {
                    client.op_done(Err(ErrorCode::FAIL), buf);
                })
            });
            return;
        }

        if !self.registers.status.is_set(STATUS::BUSY) {
            self.client.map(|client| {
                client.op_done(Ok(()), self.out_buffer.take().unwrap());
            });
        }

        self.registers.intr_enable.set(0x00);
        self.registers.intr_state.set(0xFFFF_FFFF);
    }

    pub fn initialise(&self, deferred_call_handle: DeferredCallHandle) {
        self.deferred_handle.set(deferred_call_handle);
    }

    /// Set the client instance which will receive
    pub fn set_client(&'a self, client: &'a dyn Client<'a, 1024>) {
        self.client.set(client);
    }

    /// Load the acceleration binary data into the accelerator.
    /// This data will be accelerator specific and could be an
    /// elf file which will be run or could be binary settings used to
    /// configure the accelerator.
    /// This function can be called multiple times if multiple binary blobs
    /// are required.
    /// There is no guarantee the data has been written until the `binary_load_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    pub fn load_binary(
        &self,
        input: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.registers.status.is_set(STATUS::BUSY) {
            // OTBN is performing an operation, we can't make any changes
            return Err((ErrorCode::BUSY, input.take()));
        }

        for i in 0..(input.len() / 4) {
            let idx = i * 4;

            let mut d = (input[idx + 0] as u32) << 0;
            d |= (input[idx + 1] as u32) << 8;
            d |= (input[idx + 2] as u32) << 16;
            d |= (input[idx + 3] as u32) << 24;

            self.registers.imem[i].set(d);
        }

        self.in_buffer.set(Some(input));

        // Schedule a deferred call as there are no interrupts to monitor
        // the binary loading.
        self.add_data_deferred_call.set(true);
        self.deferred_handle
            .map(|handle| self.deferred_caller.set(*handle));

        Ok(())
    }

    /// Set the OTBN properties
    /// key values:
    ///  `0` -> Start Address, set the start address
    pub fn set_property(&self, key: usize, value: usize) -> Result<(), ErrorCode> {
        if self.registers.status.is_set(STATUS::BUSY) {
            // OTBN is performing and operation
            return Err(ErrorCode::BUSY);
        }

        match key {
            0 => {
                self.registers.start_addr.set(value as u32);
                Ok(())
            }
            _ => Err(ErrorCode::NOSUPPORT),
        }
    }

    /// Run the acceleration operation.
    /// This doesn't return any data, instead the client needs to have
    /// set a `op_done` handler to determine when this is complete.
    /// On error the return value will contain a return code and the original data
    /// If there is data from the `load_binary()` command asyncrously waiting to
    /// be written it will be written before the operation starts.
    pub fn run(
        &'a self,
        output: &'static mut [u8; 1024],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 1024])> {
        if self.registers.status.is_set(STATUS::BUSY) {
            // OTBN is performing and operation
            return Err((ErrorCode::BUSY, output));
        }

        // Clear and enable interrupts
        self.registers.intr_state.modify(INTR::DONE::SET);
        self.registers.intr_enable.modify(INTR::DONE::SET);

        self.out_buffer.replace(output);

        self.registers.cmd.modify(CMD::START::SET);

        Ok(())
    }

    /// Clear the keys and any other sensitive data.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear those.
    pub fn clear_data(&self) {}
}

impl<'a> DynamicDeferredCallClient for Otbn<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        if self.add_data_deferred_call.get() {
            self.add_data_deferred_call.set(false);

            self.client.map(|client| {
                self.in_buffer.take().map(|buffer| {
                    client.binary_load_done(Ok(()), buffer.take());
                });
            });
        }
    }
}
