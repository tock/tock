//! OTBN Control

use kernel::common::cells::OptionalCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil::accel;
use kernel::ErrorCode;

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
    _registers: StaticRef<OtbnRegisters>,
    client: OptionalCell<&'a dyn accel::Client<'a, 1024>>,
}

impl<'a> Otbn<'a> {
    pub const fn new(base: StaticRef<OtbnRegisters>) -> Self {
        Otbn {
            _registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        unimplemented!();
    }
}

impl<'a> accel::Accel<'a, 1024> for Otbn<'a> {
    fn set_client(&'a self, client: &'a dyn accel::Client<'a, 1024>) {
        self.client.set(client);
    }

    fn load_binary(
        &self,
        input: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        Err((ErrorCode::NOSUPPORT, input.take()))
    }

    fn set_property(&self, _key: usize, _value: usize) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn run(
        &'a self,
        output: &'static mut [u8; 1024],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 1024])> {
        Err((ErrorCode::NOSUPPORT, output))
    }

    fn clear_data(&self) {}
}
