//! Implementation of I2C for nRF52 using EasyDMA.
//!
//! This module supports nRF52's two I2C master (`TWIM`) peripherals,
//! but not I2C slave (`TWIS`).
//!
//! - Author: Jay Kickliter
//! - Author: Andrew Thompson
//! - Date: Nov 4, 2017

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use nrf5x::pinmux::Pinmux;

/// Uninitialized `TWIM` instances.
const INSTANCES: [StaticRef<TwimRegisters>; 2] = unsafe {
    [
        StaticRef::new(0x40003000 as *const TwimRegisters),
        StaticRef::new(0x40004000 as *const TwimRegisters),
    ]
};

/// An I2C master device.
///
/// A `TWIM` instance wraps a `registers::TWIM` together with
/// additional data necessary to implement an asynchronous interface.
pub struct TWIM {
    registers: StaticRef<TwimRegisters>,
    client: OptionalCell<&'static dyn hil::i2c::I2CHwMasterClient>,
    buf: TakeCell<'static, [u8]>,
}

/// I2C bus speed.
#[repr(u32)]
pub enum Speed {
    K100 = 0x01980000,
    K250 = 0x04000000,
    K400 = 0x06400000,
}

impl TWIM {
    const fn new(registers: StaticRef<TwimRegisters>) -> Self {
        Self {
            registers,
            client: OptionalCell::empty(),
            buf: TakeCell::empty(),
        }
    }

    pub const fn new_twim0() -> Self {
        TWIM::new(INSTANCES[0])
    }

    pub const fn new_twim1() -> Self {
        TWIM::new(INSTANCES[1])
    }

    pub fn set_client(&self, client: &'static dyn hil::i2c::I2CHwMasterClient) {
        debug_assert!(self.client.is_none());
        self.client.set(client);
    }

    /// Configures an already constructed `TWIM`.
    pub fn configure(&self, scl: Pinmux, sda: Pinmux) {
        self.registers.psel_scl.set(scl);
        self.registers.psel_sda.set(sda);
    }

    /// Sets the I2C bus speed to one of three possible values
    /// enumerated in `Speed`.
    pub fn set_speed(&self, speed: Speed) {
        self.registers.frequency.set(speed as u32);
    }

    /// Enables hardware TWIM peripheral.
    pub fn enable(&self) {
        self.registers.enable.write(ENABLE::ENABLE::Enable);
    }

    /// Disables hardware TWIM peripheral.
    pub fn disable(&self) {
        self.registers.enable.write(ENABLE::ENABLE::Disable);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.events_stopped.is_set(EVENT::EVENT) {
            self.registers.events_stopped.write(EVENT::EVENT::CLEAR);
            self.client.map(|client| match self.buf.take() {
                None => (),
                Some(buf) => {
                    client.command_complete(buf, hil::i2c::Error::CommandComplete);
                }
            });
        }

        if self.registers.events_error.is_set(EVENT::EVENT) {
            self.registers.events_error.write(EVENT::EVENT::CLEAR);
            let errorsrc = self.registers.errorsrc.extract();
            self.registers
                .errorsrc
                .write(ERRORSRC::ANACK::ErrorDidNotOccur + ERRORSRC::DNACK::ErrorDidNotOccur);
            self.client.map(|client| match self.buf.take() {
                None => (),
                Some(buf) => {
                    let i2c_error = if errorsrc.is_set(ERRORSRC::ANACK) {
                        hil::i2c::Error::AddressNak
                    } else if errorsrc.is_set(ERRORSRC::DNACK) {
                        hil::i2c::Error::DataNak
                    } else {
                        hil::i2c::Error::CommandComplete
                    };
                    client.command_complete(buf, i2c_error);
                }
            });
        }

        // We can blindly clear the following events since we're not using them.
        self.registers.events_suspended.write(EVENT::EVENT::CLEAR);
        self.registers.events_rxstarted.write(EVENT::EVENT::CLEAR);
        self.registers.events_lastrx.write(EVENT::EVENT::CLEAR);
        self.registers.events_lasttx.write(EVENT::EVENT::CLEAR);
    }

    pub fn is_enabled(&self) -> bool {
        self.registers.enable.matches_all(ENABLE::ENABLE::Enable)
    }
}

impl hil::i2c::I2CMaster for TWIM {
    fn set_master_client(&self, client: &'static dyn hil::i2c::I2CHwMasterClient) {
        self.set_client(client);
    }
    fn enable(&self) {
        self.enable();
    }

    fn disable(&self) {
        self.disable();
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        self.registers
            .address
            .write(ADDRESS::ADDRESS.val((addr >> 1) as u32));
        self.registers.txd_ptr.set(data.as_mut_ptr());
        self.registers
            .txd_maxcnt
            .write(MAXCNT::MAXCNT.val(write_len as u32));
        self.registers.rxd_ptr.set(data.as_mut_ptr());
        self.registers
            .rxd_maxcnt
            .write(MAXCNT::MAXCNT.val(read_len as u32));
        // Use the NRF52 shortcut register to configure the peripheral to
        // switch to RX after TX is complete, and then to switch to the STOP
        // state once RX is done. This avoids us having to juggle tasks in
        // the interrupt handler.
        self.registers
            .shorts
            .write(SHORTS::LASTTX_STARTRX::EnableShortcut + SHORTS::LASTRX_STOP::EnableShortcut);
        self.registers
            .intenset
            .write(INTE::STOPPED::Enable + INTE::ERROR::Enable);
        // start the transfer
        self.registers.tasks_starttx.write(TASK::TASK::SET);
        self.buf.replace(data);
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        self.registers
            .address
            .write(ADDRESS::ADDRESS.val((addr >> 1) as u32));
        self.registers.txd_ptr.set(data.as_mut_ptr());
        self.registers
            .txd_maxcnt
            .write(MAXCNT::MAXCNT.val(len as u32));
        // Use the NRF52 shortcut register to switch to the STOP state once
        // the TX is complete.
        self.registers
            .shorts
            .write(SHORTS::LASTTX_STOP::EnableShortcut);
        self.registers
            .intenset
            .write(INTE::STOPPED::Enable + INTE::ERROR::Enable);
        // start the transfer
        self.registers.tasks_starttx.write(TASK::TASK::SET);
        self.buf.replace(data);
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        self.registers
            .address
            .write(ADDRESS::ADDRESS.val((addr >> 1) as u32));
        self.registers.rxd_ptr.set(buffer.as_mut_ptr());
        self.registers
            .rxd_maxcnt
            .write(MAXCNT::MAXCNT.val(len as u32));
        // Use the NRF52 shortcut register to switch to the STOP state once
        // the RX is complete.
        self.registers
            .shorts
            .write(SHORTS::LASTRX_STOP::EnableShortcut);
        self.registers
            .intenset
            .write(INTE::STOPPED::Enable + INTE::ERROR::Enable);
        // start the transfer
        self.registers.tasks_startrx.write(TASK::TASK::SET);
        self.buf.replace(buffer);
    }
}

// The SPI0_TWI0 and SPI1_TWI1 interrupts are dispatched to the
// correct handler by the service_pending_interrupts() routine in
// chip.rs based on which peripheral is enabled.

#[repr(C)]
struct TwimRegisters {
    /// Start TWI receive sequence
    tasks_startrx: WriteOnly<u32, TASK::Register>,
    _reserved0: [u8; 4],
    /// Start TWI transmit sequence
    tasks_starttx: WriteOnly<u32, TASK::Register>,
    _reserved1: [u8; 8],
    /// Stop TWI transaction
    tasks_stop: WriteOnly<u32, TASK::Register>,
    _reserved2: [u8; 4],
    /// Suspend TWI transaction
    tasks_suspend: WriteOnly<u32, TASK::Register>,
    /// Resume TWI transaction
    tasks_resume: WriteOnly<u32, TASK::Register>,
    _reserved3: [u8; 224],
    /// TWI stopped
    events_stopped: ReadWrite<u32, EVENT::Register>,
    _reserved4: [u8; 28],
    /// TWI error
    events_error: ReadWrite<u32, EVENT::Register>,
    _reserved5: [u8; 32],
    /// Last byte has been sent out after the SUSPEND task has been issued, TWI
    /// traffic is now suspended.
    events_suspended: ReadWrite<u32, EVENT::Register>,
    /// Receive sequence started
    events_rxstarted: ReadWrite<u32, EVENT::Register>,
    /// Transmit sequence started
    events_txstarted: ReadWrite<u32, EVENT::Register>,
    _reserved6: [u8; 8],
    /// Byte boundary, starting to receive the last byte
    events_lastrx: ReadWrite<u32, EVENT::Register>,
    /// Byte boundary, starting to transmit the last byte
    events_lasttx: ReadWrite<u32, EVENT::Register>,
    _reserved7: [u8; 156],
    /// Shortcut register
    shorts: ReadWrite<u32, SHORTS::Register>,
    _reserved8: [u8; 252],
    /// Enable or disable interrupt
    inten: ReadWrite<u32, INTE::Register>,
    /// Enable interrupt
    intenset: ReadWrite<u32, INTE::Register>,
    /// Disable interrupt
    intenclr: ReadWrite<u32, INTE::Register>,
    _reserved9: [u8; 440],
    /// Error source
    errorsrc: ReadWrite<u32, ERRORSRC::Register>,
    _reserved10: [u8; 56],
    /// Enable TWIM
    enable: ReadWrite<u32, ENABLE::Register>,
    _reserved11: [u8; 4],
    /// Pin select for SCL signal
    psel_scl: VolatileCell<Pinmux>,
    /// Pin select for SDA signal
    psel_sda: VolatileCell<Pinmux>,
    _reserved_12: [u8; 20],
    /// TWI frequency
    frequency: ReadWrite<u32>,
    _reserved13: [u8; 12],
    /// Data pointer
    rxd_ptr: VolatileCell<*mut u8>,
    /// Maximum number of bytes in receive buffer
    rxd_maxcnt: ReadWrite<u32, MAXCNT::Register>,
    /// Number of bytes transferred in the last transaction
    rxd_amount: ReadWrite<u32>,
    /// EasyDMA list type
    rxd_list: ReadWrite<u32>,
    /// Data pointer
    txd_ptr: VolatileCell<*mut u8>,
    /// Maximum number of bytes in transmit buffer
    txd_maxcnt: ReadWrite<u32, MAXCNT::Register>,
    /// Number of bytes transferred in the last transaction
    txd_amount: ReadWrite<u32>,
    /// EasyDMA list type
    txd_list: ReadWrite<u32>,
    _reserved_14: [u8; 52],
    /// Address used in the TWI transfer
    address: ReadWrite<u32, ADDRESS::Register>,
}

register_bitfields![u32,
    SHORTS [
        /// Shortcut between EVENTS_LASTTX event and TASKS_STARTRX task
        LASTTX_STARTRX OFFSET(7) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_LASTTX event and TASKS_SUSPEND task
        LASTTX_SUSPEND OFFSET(8) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_LASTTX event and TASKS_STOP task
        LASTTX_STOP OFFSET(9) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_LASTRX event and TASKS_STARTTX task
        LASTRX_STARTTX OFFSET(10) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ],
        /// Shortcut between EVENTS_LASTRX event and TASKS_STOP task
        LASTRX_STOP OFFSET(12) NUMBITS(1) [
            /// Disable shortcut
            DisableShortcut = 0,
            /// Enable shortcut
            EnableShortcut = 1
        ]
    ],
    INTE [
        /// Enable or disable interrupt on EVENTS_STOPPED event
        STOPPED OFFSET(1) NUMBITS(1) [
            /// Disable
            Disable = 0,
            /// Enable
            Enable = 1
        ],
        /// Enable or disable interrupt on EVENTS_ERROR event
        ERROR OFFSET(9) NUMBITS(1) [
            /// Disable
            Disable = 0,
            /// Enable
            Enable = 1
        ],
        /// Enable or disable interrupt on EVENTS_RXSTARTED event
        RXSTARTED OFFSET(19) NUMBITS(1) [
            /// Disable
            Disable = 0,
            /// Enable
            Enable = 1
        ],
        /// Enable or disable interrupt on EVENTS_TXSTARTED event
        TXSTARTED OFFSET(20) NUMBITS(1) [
            /// Disable
            Disable = 0,
            /// Enable
            Enable = 1
        ],
        /// Enable or disable interrupt on EVENTS_LASTRX event
        LASTRX OFFSET(23) NUMBITS(1) [
            /// Disable
            Disable = 0,
            /// Enable
            Enable = 1
        ],
        /// Enable or disable interrupt on EVENTS_LASTTX event
        LASTTX OFFSET(24) NUMBITS(1) [
            /// Disable
            Disable = 0,
            /// Enable
            Enable = 1
        ]
    ],
    ERRORSRC [
        /// NACK received after sending the address (write '1' to clear)
        ANACK OFFSET(1) NUMBITS(1) [
            /// Error did not occur
            ErrorDidNotOccur = 0,
            /// Error occurred
            ErrorOccurred = 1
        ],
        /// NACK received after sending a data byte (write '1' to clear)
        DNACK OFFSET(2) NUMBITS(1) [
            /// Error did not occur
            ErrorDidNotOccur = 0,
            /// Error occurred
            ErrorOccurred = 1
        ]
    ],
    EVENT [
        EVENT 0
    ],
    TASK [
        TASK 0
    ],
    ENABLE [
        /// Enable or disable TWIM
        ENABLE OFFSET(0) NUMBITS(4) [
            Disable = 0,
            Enable = 6
        ]
    ],
    MAXCNT [
        /// Maximum number of bytes in buffer
        MAXCNT OFFSET(0) NUMBITS(16)
    ],
    ADDRESS [
        /// Address used in the TWI transfer
        ADDRESS OFFSET(0) NUMBITS(7)
    ]
];
