//! I2C driver, cc26x2 family

use core::cmp;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::i2c;

/// A wrapper module for interal register types.
///
/// The module allows us to hide construction of these internal types since arbitrarily creating
/// them can have safety consequences.
mod regs {
    use kernel::common::registers::{ReadOnly, WriteOnly};
    /// Models the `mctrl` and `mstat` registers, which occupy the same address, but have completely
    /// different meanings for the same bits.
    ///
    /// When written, it is `mctrl` and it configures the I2C controller operation.
    /// When read, it is `mstat` and indicates the state of the I2C controller
    /// (CC13x2, CC26x2 SimpleLink Wireless MCU Technical Reference Manual pg. 1777)
    ///
    /// ## Safety
    ///
    /// Since this allows the client to access the same 32-bits using different types, it's important
    /// that this type is only instantiated to occupy the memory of the control and status registers.
    pub union ControlStatReg {
        /// The control register modality
        ctrl: WriteOnly<u32, super::Control::Register>,
        /// The status register modality
        stat: ReadOnly<u32, super::Status::Register>,
    }

    // This implements access to the union fields as methods, since access to untagged union fields is
    // unsafe (for good reason in general). In this case, though, it's actually representing how
    // memory accesses work assuming `ControlStatReg` is only instanitated to model the
    // combined control/status register.
    impl ControlStatReg {
        /// Returns the control register modality
        pub fn ctrl(&self) -> &WriteOnly<u32, super::Control::Register> {
            unsafe { &self.ctrl }
        }

        /// Returns the status register modality
        pub fn stat(&self) -> &ReadOnly<u32, super::Status::Register> {
            unsafe { &self.stat }
        }
    }
}

use self::regs::ControlStatReg;

#[repr(C)]
struct I2CMasterRegisters {
    /// Master slave address
    msa: ReadWrite<u32, Address::Register>,
    mstat_ctrl: ControlStatReg,
    mdr: ReadWrite<u8>,
    _reserved: [u8; 3],
    mtpr: ReadWrite<u32, TimerPeriod::Register>,
    mimr: ReadWrite<u32, Interrupt::Register>,
    mris: ReadOnly<u32, Interrupt::Register>,
    mmis: ReadOnly<u32, Interrupt::Register>,
    micr: WriteOnly<u32, Interrupt::Register>,
    mcr: ReadWrite<u32, Configuration::Register>,
}

register_bitfields![
    u32,
    Address [
        RS OFFSET(0) NUMBITS(1) [
            Transmit = 0,
            Receive = 1
        ],
        SA OFFSET(1) NUMBITS(7) []
    ],
    Status [
        BUSY OFFSET(0) NUMBITS(1) [],
        ERR OFFSET(1) NUMBITS(1) [],
        ADRACK_N OFFSET(2) NUMBITS(1) [],
        DATACK_N OFFSET(3) NUMBITS(1) [],
        ARBLST OFFSET(4) NUMBITS(1) [],
        IDLE OFFSET(5) NUMBITS(1) [],
        BUSBSY OFFSET(6) NUMBITS(1) []
    ],
    Control [
        RUN OFFSET(0) NUMBITS(1) [],
        START OFFSET(1) NUMBITS(1) [],
        STOP OFFSET(2) NUMBITS(1) [],
        ACK OFFSET(3) NUMBITS(1) []
    ],
    TimerPeriod [
        TPR OFFSET(0) NUMBITS(7) [],
        WRITE OFFSET(7) NUMBITS(1) [
            Valid = 0,
            Ignore = 1
        ]
    ],
    Interrupt [
        IM OFFSET(0) NUMBITS(1) []
    ],
    Configuration [
        LPBK OFFSET(0) NUMBITS(1) [],
        MFE OFFSET(4) NUMBITS(1) [],
        SFE OFFSET(5) NUMBITS(1) []
    ]
];

struct Transfer {
    mode: TransferMode,
    buf: &'static mut [u8],
    index: usize,
    len: usize,
}

#[derive(Copy, Clone)]
enum TransferMode {
    Transmit,
    Receive,
    TransmitThenReceive(usize),
}

const I2C0REGISTERS: StaticRef<I2CMasterRegisters> =
    unsafe { StaticRef::new(0x4000_2800 as *const _) };

pub static mut I2C0: I2CMaster = I2CMaster::new(I2C0REGISTERS);

pub struct I2CMaster<'a> {
    registers: StaticRef<I2CMasterRegisters>,
    client: OptionalCell<&'a i2c::I2CHwMasterClient>,
    transfer: MapCell<Transfer>,
}

impl<'a> I2CMaster<'a> {
    const fn new(registers: StaticRef<I2CMasterRegisters>) -> I2CMaster<'a> {
        I2CMaster {
            registers: registers,
            client: OptionalCell::empty(),
            transfer: MapCell::empty(),
        }
    }

    pub fn set_client(&'a self, client: &'a i2c::I2CHwMasterClient) {
        self.client.set(client)
    }

    /// Initiate writing a single byte. An interrupt becomes pending upon completion of the write.
    ///
    /// * `byte`  - the byte to send
    /// * `first` - whether this is the first byte in a transfer (i.e. whether to include a "START"
    ///             condition)
    /// * `last`  - whether this is the last byte in a transfer (i.e. whether to include a "STOP"
    ///             condition)
    fn write_byte(&self, byte: u8, first: bool, last: bool) {
        self.registers.mdr.set(byte);
        self.registers.mstat_ctrl.ctrl().write(
            Control::RUN.val(1) + Control::START.val(first as u32) + Control::STOP.val(last as u32),
        );
    }

    /// Initiate reading a single byte. An interrupt becomes pending when the byte is available in
    /// the MDR register.
    ///
    /// * `first` - whether this is the first byte in a transfer (i.e. whether to include a "START"
    ///             condition)
    /// * `last`  - whether this is the last byte in a transfer (i.e. whether to include a "STOP"
    ///             condition)
    fn read_byte(&self, first: bool, last: bool) {
        self.registers.mstat_ctrl.ctrl().write(
            Control::RUN.val(1)
                + Control::RUN.val(1)
                + Control::START.val(first as u32)
                + Control::STOP.val(last as u32),
        );
    }

    pub fn handle_interrupt(&self) {
        self.registers.micr.write(Interrupt::IM::SET);
        if let Some(mut transfer) = self.transfer.take() {
            let status = self.registers.mstat_ctrl.stat();

            if status.is_set(Status::ADRACK_N) {
                self.client.map(move |client| {
                    client.command_complete(transfer.buf, i2c::Error::AddressNak);
                });
                return;
            } else if status.is_set(Status::DATACK_N) {
                self.client.map(move |client| {
                    client.command_complete(transfer.buf, i2c::Error::DataNak);
                });
                return;
            } else if status.is_set(Status::ARBLST) {
                self.client.map(move |client| {
                    client.command_complete(transfer.buf, i2c::Error::ArbitrationLost);
                });
                return;
            }

            match transfer.mode {
                TransferMode::Transmit => {
                    transfer.index += 1;
                    if transfer.len > transfer.index {
                        self.write_byte(
                            transfer.buf[transfer.index],
                            false,
                            transfer.len == transfer.index + 1,
                        );
                        self.transfer.put(transfer);
                    } else {
                        self.client.map(move |client| {
                            client.command_complete(transfer.buf, i2c::Error::CommandComplete)
                        });
                    }
                }
                TransferMode::Receive => {
                    transfer.buf[transfer.index] = self.registers.mdr.get();
                    transfer.index += 1;
                    if transfer.len > transfer.index {
                        transfer.index += 1;
                        self.read_byte(false, transfer.len == transfer.index + 1);
                        self.transfer.put(transfer);
                    } else {
                        self.client.map(move |client| {
                            client.command_complete(transfer.buf, i2c::Error::CommandComplete)
                        });
                    }
                }
                TransferMode::TransmitThenReceive(read_len) => {
                    transfer.index += 1;
                    if transfer.len > transfer.index {
                        self.write_byte(
                            transfer.buf[transfer.index],
                            false,
                            transfer.len == transfer.index + 1,
                        );
                        self.transfer.put(transfer);
                    } else {
                        transfer.index = 0;
                        transfer.len = cmp::min(read_len, transfer.buf.len());
                        transfer.mode = TransferMode::Receive;
                        self.registers.msa.modify(Address::RS::Receive);
                        self.read_byte(true, transfer.len == transfer.index + 1);
                        self.transfer.put(transfer);
                    }
                }
            }
        }
    }
}

const MCU_CLOCK: u32 = 48_000_000;

impl<'a> i2c::I2CMaster for I2CMaster<'a> {
    fn enable(&self) {
        let tpr = ((MCU_CLOCK + (2 * 10 * 100_000) - 1) / (2 * 10 * 100_000)) - 1;
        self.registers.mcr.write(Configuration::MFE::SET);
        self.registers
            .mtpr
            .write(TimerPeriod::WRITE::Valid + TimerPeriod::TPR.val(tpr));
        self.registers.mimr.write(Interrupt::IM::SET);
    }

    fn disable(&self) {
        self.registers.mcr.modify(Configuration::MFE.val(0))
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        self.registers
            .msa
            .write(Address::RS::Transmit + Address::SA.val(addr as u32));
        let len = cmp::min(write_len as usize, data.len());
        if len > 0 {
            self.write_byte(data[0], true, len == 1);
            self.transfer.put(Transfer {
                mode: TransferMode::TransmitThenReceive(read_len as usize),
                buf: data,
                index: 0,
                len: len,
            });
        }
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        self.registers
            .msa
            .write(Address::RS::Transmit + Address::SA.val(addr as u32));
        let len = cmp::min(len as usize, data.len());
        if len > 0 {
            self.write_byte(data[0], true, len == 1);
            self.transfer.put(Transfer {
                mode: TransferMode::Transmit,
                buf: data,
                index: 0,
                len: len,
            });
        }
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        self.registers
            .msa
            .write(Address::RS::Receive + Address::SA.val(addr as u32));
        let len = cmp::min(len as usize, buffer.len());
        if len > 0 {
            self.read_byte(true, len == 1);
            self.transfer.put(Transfer {
                mode: TransferMode::Receive,
                buf: buffer,
                index: 0,
                len: len,
            });
        }
    }
}
