//! Implementation of I2C for nRF51.
//!
//! This module supports nRF51's two I2C master (`TWIM`) peripherals.

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{FieldValue, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::i2c;
use {nrf5x, nrf5x::gpio, nrf5x::pinmux::Pinmux};

/// An I2C master device.
///
/// A `TWIM` instance wraps a `registers::TWIM` together with
/// additional data necessary to implement an asynchronous interface.
pub struct TWIM {
    registers: StaticRef<TwimRegisters>,
    client: OptionalCell<&'static i2c::I2CHwMasterClient>,
    tx_len: Cell<u8>,
    rx_len: Cell<u8>,
    pos: Cell<usize>,
    buf: TakeCell<'static, [u8]>,
}

impl TWIM {
    const fn new(instance: usize) -> TWIM {
        TWIM {
            registers: TWIM_BASE[instance],
            client: OptionalCell::empty(),
            tx_len: Cell::new(0),
            rx_len: Cell::new(0),
            pos: Cell::new(0),
            buf: TakeCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static i2c::I2CHwMasterClient) {
        debug_assert!(self.client.is_none());
        self.client.set(client);
    }

    /// Configures an already constructed `TWIM`.
    pub fn configure(&self, scl: Pinmux, sda: Pinmux) {
        let scl_idx: u32 = scl.into();
        let sda_idx: u32 = sda.into();

        // configure pins as inputs with drive strength S0D1
        unsafe {
            let sdapin = &nrf5x::gpio::PORT[sda_idx as usize];
            sdapin.write_config(gpio::PinConfig::DRIVE::S0D1);
            let sclpin = &nrf5x::gpio::PORT[scl_idx as usize];
            sclpin.write_config(gpio::PinConfig::DRIVE::S0D1);
        }

        let regs = &*self.registers;
        regs.psel_scl.set(scl_idx);
        regs.psel_sda.set(sda_idx);
    }

    /// Sets the I2C bus speed to one of three possible values
    /// enumerated in `Speed`.
    pub fn set_speed(&self, speed: FieldValue<u32, Frequency::Register>) {
        let regs = &*self.registers;
        regs.frequency.write(speed);
    }

    /// Enables hardware TWIM peripheral.
    pub fn enable(&self) {
        let regs = &*self.registers;
        regs.enable.write(Twim::ENABLE::ON);
    }

    /// Disables hardware TWIM peripheral.
    pub fn disable(&self) {
        let regs = &*self.registers;
        regs.enable.write(Twim::ENABLE::OFF);
    }

    fn start_read(&self) {
        let regs = &*self.registers;
        if self.rx_len.get() == 1 {
            regs.shorts.write(Shorts::BB_STOP::SET);
        } else {
            regs.shorts.write(Shorts::BB_SUSPEND::SET);
        }
        self.tx_len.set(0);
        self.pos.set(0);
        let regs = regs;
        regs.intenset.write(InterruptEnable::RXREADY::SET);
        regs.intenset.write(InterruptEnable::ERROR::SET);
        // start the transfer
        regs.tasks_startrx.write(Task::ENABLE::SET);
    }

    fn reply(&self, result: i2c::Error) {
        self.client.map(|client| {
            self.buf.take().map(|buf| {
                client.command_complete(buf, result);
            });
        });
    }

    /// The SPI0_TWI0 and SPI1_TWI1 interrupts are dispatched to the
    /// correct handler by the service_pending_interrupts() routine in
    /// chip.rs based on which peripheral is enabled.
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        if regs.events_rxdreceived.get() == 1 {
            regs.events_rxdreceived.set(0);
            let pos = self.pos.get();
            self.pos.set(pos + 1);
            if self.pos.get() < self.rx_len.get() as usize {
                let v = regs.rxd.read(Data::DATA);
                self.buf.map(|buf| buf[pos] = v as u8);
                if pos == self.rx_len.get() as usize - 2 {
                    regs.shorts.write(Shorts::BB_STOP::SET);
                };
                regs.tasks_resume.write(Task::ENABLE::SET);
            } else {
                regs.shorts.set(0);
                let v = regs.rxd.read(Data::DATA);
                self.buf.map(|buf| buf[pos] = v as u8);
                regs.intenclr.write(InterruptEnable::RXREADY::SET);
                regs.intenclr.write(InterruptEnable::ERROR::SET);
                self.reply(i2c::Error::CommandComplete);
                self.pos.set(0);
            }
        };
        if regs.events_txdsent.get() == 1 {
            regs.events_txdsent.set(0);
            let pos = self.pos.get() + 1;
            self.pos.set(pos);
            if pos < self.tx_len.get() as usize {
                self.buf.map(|buf| regs.txd.set(buf[pos].into()));
            } else {
                if self.rx_len.get() > 0 {
                    regs.intenclr.write(InterruptEnable::TXSENT::SET);
                    self.start_read();
                } else {
                    regs.tasks_stop.write(Task::ENABLE::SET);
                    regs.intenclr.write(InterruptEnable::TXSENT::SET);
                    regs.intenclr.write(InterruptEnable::ERROR::SET);
                    self.reply(i2c::Error::CommandComplete);
                }
            }
        };
        if regs.events_error.get() == 1 {
            regs.events_error.set(0);
            let errorsrc = if regs.errorsrc.is_set(ErrorSrc::OVERRUN) {
                i2c::Error::Overrun
            } else if regs.errorsrc.is_set(ErrorSrc::ADDRESSNACK) {
                i2c::Error::AddressNak
            } else if regs.errorsrc.is_set(ErrorSrc::DATANACK) {
                i2c::Error::DataNak
            } else {
                i2c::Error::CommandComplete
            };
            regs.errorsrc.set(0);
            self.reply(errorsrc);
        }
    }

    pub fn is_enabled(&self) -> bool {
        let regs = &*self.registers;
        regs.enable.get() == 5
    }
}

impl i2c::I2CMaster for TWIM {
    fn enable(&self) {
        self.enable();
    }

    fn disable(&self) {
        self.disable();
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        let regs = &*self.registers;
        let buffer_len = cmp::min(data.len(), 255);
        self.tx_len.set(cmp::min(write_len, buffer_len as u8));
        self.rx_len.set(cmp::min(read_len, buffer_len as u8));
        regs.intenset.write(InterruptEnable::TXSENT::SET);
        regs.intenset.write(InterruptEnable::ERROR::SET);
        // start the transfer
        self.pos.set(0);
        regs.address.write(Address::ADDRESS.val(addr.into()));
        regs.tasks_resume.write(Task::ENABLE::SET);
        regs.tasks_starttx.write(Task::ENABLE::SET);
        regs.txd.set(data[0].into());
        self.buf.replace(data);
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        self.write_read(addr, data, len, 0);
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        let regs = &*self.registers;
        regs.address.set(addr as u32);
        let buffer_len = cmp::min(buffer.len(), 255);
        self.rx_len.set(cmp::min(len, buffer_len as u8));
        regs.tasks_resume.write(Task::ENABLE::SET);
        self.start_read();
        self.buf.replace(buffer);
    }
}

impl i2c::I2CSlave for TWIM {
    fn enable(&self) {
        panic!("nRF51 does not support I2C slave mode");
    }
    fn disable(&self) {
        panic!("nRF51 does not support I2C slave mode");
    }
    fn set_address(&self, _addr: u8) {
        panic!("nRF51 does not support I2C slave mode");
    }
    fn write_receive(&self, _data: &'static mut [u8], _max_len: u8) {
        panic!("nRF51 does not support I2C slave mode");
    }
    fn read_send(&self, _data: &'static mut [u8], _max_len: u8) {
        panic!("nRF51 does not support I2C slave mode");
    }
    fn listen(&self) {
        panic!("nRF51 does not support I2C slave mode");
    }
}

impl i2c::I2CMasterSlave for TWIM {}

/// I2C master instance 0.
pub static mut TWIM0: TWIM = TWIM::new(0);
/// I2C master instance 1.
pub static mut TWIM1: TWIM = TWIM::new(1);

register_bitfields! [
    u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    /// Events
    Event [
        OCCURED OFFSET(0) NUMBITS(1) []
    ],
    /// Shortcuts
    Shorts [
        BB_SUSPEND OFFSET(0) NUMBITS(1) [],
        BB_STOP OFFSET(1) NUMBITS(1) []
    ],
    /// Interrupts
    InterruptEnable [
        STOPPED OFFSET(1) NUMBITS(1) [],
        RXREADY OFFSET(2) NUMBITS(1) [],
        TXSENT OFFSET(7) NUMBITS(1) [],
        ERROR OFFSET(9) NUMBITS(1) [],
        BB OFFSET(14) NUMBITS(1) []
    ],
    ErrorSrc [
        OVERRUN OFFSET(0) NUMBITS(1) [],
        ADDRESSNACK OFFSET(1) NUMBITS(1) [],
        DATANACK OFFSET(2) NUMBITS(1) []
    ],
    /// TWIM enable
    Twim [
        ENABLE OFFSET(0) NUMBITS(3) [
            ON = 5,
            OFF = 0
        ]
    ],
    Psel [
        PIN OFFSET(0) NUMBITS(32) []
    ],
    Data [
        DATA OFFSET(0) NUMBITS(8) []
    ],
    Frequency [
        FREQUENCY OFFSET(0) NUMBITS(32) [
            K100 = 0x01980000,
            K250 = 0x04000000,
            K400 = 0x06680000
        ]
    ],
    Address [
        ADDRESS OFFSET(0) NUMBITS(7) []
    ]
];

/// Uninitialized `TWIM` instances.
const TWIM_BASE: [StaticRef<TwimRegisters>; 2] = unsafe {
    [
        StaticRef::new(0x40003000 as *const TwimRegisters),
        StaticRef::new(0x40004000 as *const TwimRegisters),
    ]
};

#[repr(C)]
struct TwimRegisters {
    /// Start TWI receive sequence
    ///
    /// addr = base + 0x000
    tasks_startrx: WriteOnly<u32, Task::Register>,
    _reserved_0: [u32; 1],
    /// Start TWI transmit sequence
    ///
    /// addr = base + 0x008
    tasks_starttx: WriteOnly<u32, Task::Register>,
    _reserved_1: [u32; 2],
    /// Stop TWI transaction_ Must be issued while the TWI master is not suspended_
    ///
    /// addr = base + 0x014
    tasks_stop: WriteOnly<u32, Task::Register>,
    _reserved_2: [u32; 1],
    /// Suspend TWI transaction
    ///
    /// addr = base + 0x01C
    tasks_suspend: WriteOnly<u32, Task::Register>,
    /// Resume TWI transaction
    ///
    /// addr = base + 0x020
    tasks_resume: WriteOnly<u32, Task::Register>,
    _reserved_3: [u32; 56],
    /// TWI stopped
    ///
    /// addr = base + 0x104
    events_stopped: ReadWrite<u32, Event::Register>,
    /// TWI RXD byte received
    ///
    /// addr = base + 0x108
    events_rxdreceived: ReadWrite<u32, Event::Register>,
    _reserved_4: [u32; 4],
    /// TWI TXD byte sent
    ///
    /// addr = base + 0x11c
    events_txdsent: ReadWrite<u32, Event::Register>,
    _reserved_5: [u32; 1],
    /// TWI error
    ///
    /// addr = base + 0x124
    events_error: ReadWrite<u32, Event::Register>,
    _reserved_6: [u32; 4],
    /// TWI byte boundary
    ///
    /// addr = base + 0x138
    events_bb: ReadWrite<u32, Event::Register>,
    _reserved_7: [u32; 49],
    /// Shortcut register
    ///
    /// addr = base + 0x200
    shorts: ReadWrite<u32, Shorts::Register>,
    _reserved_8: [u32; 63],
    /// Enable or disable interrupt
    ///
    /// addr = base + 0x300
    inten: ReadOnly<u32, InterruptEnable::Register>,
    /// Enable interrupt
    ///
    /// addr = base + 0x304
    intenset: WriteOnly<u32, InterruptEnable::Register>,
    /// Disable interrupt
    ///
    /// addr = base + 0x308
    intenclr: WriteOnly<u32, InterruptEnable::Register>,
    _reserved_9: [u32; 110],
    /// Error source
    ///
    /// addr = base + 0x4C4
    errorsrc: ReadWrite<u32, ErrorSrc::Register>,
    _reserved_10: [u32; 14],
    /// Enable TWIM
    ///
    /// addr = base + 0x500
    enable: ReadWrite<u32, Twim::Register>,
    _reserved_11: [u32; 1],
    /// Pin select for SCL signal
    ///
    /// addr = base + 0x508
    psel_scl: ReadWrite<u32, Psel::Register>,
    /// Pin select for SDA signal
    ///
    /// addr = base + 0x50C
    psel_sda: ReadWrite<u32, Psel::Register>,
    _reserved_12: [u32; 2],
    /// RXD register
    ///
    /// addr = base + 0x518
    rxd: ReadOnly<u32, Data::Register>,
    /// TXD register
    ///
    /// addr = base + 0x51C
    txd: ReadWrite<u32, Data::Register>,
    _reserved_13: [u32; 1],
    /// TWI frequency
    ///
    /// addr = base + 0x524
    frequency: ReadWrite<u32, Frequency::Register>,
    _reserved_14: [u32; 24],
    /// Address used in the TWI transfer
    ///
    /// addr = base + 0x588
    address: ReadWrite<u32, Address::Register>,
}
