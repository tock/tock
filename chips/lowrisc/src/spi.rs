//! Serial Peripheral Interface (SPI) Driver

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

register_structs! {
    pub SpiDeviceRegisters {
        (0x000 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x004 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x008 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x00C => control: ReadWrite<u32, CONTROL::Register>),
        (0x010 => cfg: ReadWrite<u32, CFG::Register>),
        (0x014 => fifo_level: ReadWrite<u32, FIFO_LEVEL::Register>),
        (0x018 => async_fifo_level: ReadOnly<u32, ASYNC_FIFO_LEVEL::Register>),
        (0x01C => status: ReadOnly<u32, STATUS::Register>),
        (0x020 => rxf_ptr: ReadWrite<u32, RXF_PTR::Register>),
        (0x024 => txf_ptr: ReadWrite<u32, TXF_PTR::Register>),
        (0x028 => rxf_addr: ReadWrite<u32, RXF_ADDR::Register>),
        (0x02C => txf_addr: ReadWrite<u32, TXF_ADDR::Register>),
        (0x030 => _reserved0),
        (0x800 => buffer: [ReadWrite<u32>; 512]),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    INTR [
        RXF OFFSET(0) NUMBITS(1) [],
        RXLVL OFFSET(1) NUMBITS(1) [],
        TXLVL OFFSET(2) NUMBITS(1) [],
        RXERR OFFSET(3) NUMBITS(1) [],
        RXOVERFLOW OFFSET(4) NUMBITS(1) [],
        TXUNDERFLOW OFFSET(5) NUMBITS(1) []
    ],
    CONTROL [
        ABORT OFFSET(0) NUMBITS(1) [],
        MODE OFFSET(4) NUMBITS(2) [],
        RST_TXFIFO OFFSET(16) NUMBITS(1) [],
        RST_RXFIFO OFFSET(17) NUMBITS(2) []
    ],
    CFG [
        CPOL OFFSET(0) NUMBITS(1) [],
        CPHA OFFSET(1) NUMBITS(1) [],
        TX_ORDER OFFSET(2) NUMBITS(1) [],
        RX_ORDER OFFSET(3) NUMBITS(1) [],
        TIMER_V OFFSET(8) NUMBITS(8) []
    ],
    FIFO_LEVEL [
        RXLVL OFFSET(0) NUMBITS(16) [],
        TXLVL OFFSET(16) NUMBITS(16) []
    ],
    ASYNC_FIFO_LEVEL [
        RXLVL OFFSET(0) NUMBITS(8) [],
        TXLVL OFFSET(16) NUMBITS(8) []
    ],
    STATUS [
        RXF_FULL OFFSET(0) NUMBITS(1) [],
        RXF_EMPTY OFFSET(1) NUMBITS(1) [],
        TXF_FULL OFFSET(2) NUMBITS(1) [],
        TXF_EMPTY OFFSET(3) NUMBITS(1) [],
        ABORT_DONE OFFSET(4) NUMBITS(1) [],
        CSB OFFSET(5) NUMBITS(1) []
    ],
    RXF_PTR [
        RPTR OFFSET(0) NUMBITS(16) [],
        WPTR OFFSET(16) NUMBITS(16) []
    ],
    TXF_PTR [
        RPTR OFFSET(0) NUMBITS(16) [],
        WPTR OFFSET(16) NUMBITS(16) []
    ],
    RXF_ADDR [
        BASE OFFSET(0) NUMBITS(16) [],
        LIMIT OFFSET(16) NUMBITS(16) []
    ],
    TXF_ADDR [
        BASE OFFSET(0) NUMBITS(16) [],
        LIMIT OFFSET(16) NUMBITS(16) []
    ]
];

pub struct SpiDevice {
    registers: StaticRef<SpiDeviceRegisters>,

    client: OptionalCell<&'static dyn hil::spi::SpiSlaveClient>,
}

impl SpiDevice {
    pub const fn new(base: StaticRef<SpiDeviceRegisters>) -> Self {
        SpiDevice {
            registers: base,
            client: OptionalCell::empty(),
        }
    }
}

impl hil::spi::SpiSlave for SpiDevice {
    fn init(&self) {
        unimplemented!();
    }

    fn has_client(&self) -> bool {
        unimplemented!();
    }

    fn set_client(&self, client: Option<&'static dyn hil::spi::SpiSlaveClient>) {
        if client.is_some() {
            self.client.set(client.unwrap());
        } else {
            self.client.take();
        }
    }

    fn set_write_byte(&self, _write_byte: u8) {
        unimplemented!();
    }

    fn read_write_bytes(
        &self,
        _write_buffer: Option<&'static mut [u8]>,
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> ReturnCode {
        unimplemented!();
    }

    fn set_clock(&self, _polarity: hil::spi::ClockPolarity) {
        unimplemented!();
    }

    fn get_clock(&self) -> hil::spi::ClockPolarity {
        unimplemented!();
    }

    fn set_phase(&self, _phase: hil::spi::ClockPhase) {
        unimplemented!();
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        unimplemented!();
    }
}

impl hil::spi::SpiSlaveDevice for SpiDevice {
    fn configure(&self, _cpol: hil::spi::ClockPolarity, _cpal: hil::spi::ClockPhase) {
        unimplemented!();
    }

    fn read_write_bytes(
        &self,
        _write_buffer: Option<&'static mut [u8]>,
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> ReturnCode {
        unimplemented!();
    }

    fn set_polarity(&self, cpol: hil::spi::ClockPolarity) {
        match cpol {
            hil::spi::ClockPolarity::IdleLow => {
                self.registers.cfg.modify(CFG::CPOL::CLEAR);
            }
            hil::spi::ClockPolarity::IdleHigh => {
                self.registers.cfg.modify(CFG::CPOL::SET);
            }
        }
    }
    fn get_polarity(&self) -> hil::spi::ClockPolarity {
        if self.registers.cfg.read(CFG::CPOL) == 1 {
            hil::spi::ClockPolarity::IdleHigh
        } else {
            hil::spi::ClockPolarity::IdleLow
        }
    }
    fn set_phase(&self, cpal: hil::spi::ClockPhase) {
        match cpal {
            hil::spi::ClockPhase::SampleTrailing => {
                self.registers.cfg.modify(CFG::CPHA::CLEAR);
            }
            hil::spi::ClockPhase::SampleLeading => {
                self.registers.cfg.modify(CFG::CPHA::SET);
            }
        }
    }
    fn get_phase(&self) -> hil::spi::ClockPhase {
        if self.registers.cfg.read(CFG::CPHA) == 1 {
            hil::spi::ClockPhase::SampleLeading
        } else {
            hil::spi::ClockPhase::SampleTrailing
        }
    }
}
