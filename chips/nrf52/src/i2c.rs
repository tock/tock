// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of I2C for nRF52 using EasyDMA.
//!
//! This module supports nRF52's two I2C master (`TWI`) peripherals,
//! and the I2C slave (`TWIS`).

use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use nrf5x::pinmux::Pinmux;

/// Uninitialized `TWI` instances.
const INSTANCES: [StaticRef<TwiRegisters>; 2] = unsafe {
    [
        StaticRef::new(0x40003000 as *const TwiRegisters),
        StaticRef::new(0x40004000 as *const TwiRegisters),
    ]
};

/// An I2C master device.
///
/// A `TWI` instance wraps a `registers::TWI` together with
/// additional data necessary to implement an asynchronous interface.
pub struct TWI {
    registers: StaticRef<TwiRegisters>,
    client: OptionalCell<&'static dyn hil::i2c::I2CHwMasterClient>,
    slave_client: OptionalCell<&'static dyn hil::i2c::I2CHwSlaveClient>,
    buf: TakeCell<'static, [u8]>,
    slave_read_buf: TakeCell<'static, [u8]>,
}

/// I2C bus speed.
#[repr(u32)]
pub enum Speed {
    K100 = 0x01980000,
    K250 = 0x04000000,
    K400 = 0x06400000,
}

impl TWI {
    fn new(registers: StaticRef<TwiRegisters>) -> Self {
        Self {
            registers,
            client: OptionalCell::empty(),
            slave_client: OptionalCell::empty(),
            buf: TakeCell::empty(),
            slave_read_buf: TakeCell::empty(),
        }
    }

    pub fn new_twi0() -> Self {
        TWI::new(INSTANCES[0])
    }

    pub fn new_twi1() -> Self {
        TWI::new(INSTANCES[1])
    }

    /// Configures an already constructed `TWI`.
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
    fn enable_master(&self) {
        self.registers.enable.write(ENABLE::ENABLE::EnableMaster);
    }

    /// Enables hardware TWIS peripheral.
    fn enable_slave(&self) {
        self.registers.enable.write(ENABLE::ENABLE::EnableSlave);
    }

    /// Disables hardware TWIM/TWIS peripheral.
    fn disable(&self) {
        self.registers.enable.write(ENABLE::ENABLE::Disable);
    }

    pub fn handle_interrupt(&self) {
        if self.is_master_enabled() {
            if self.registers.events_stopped.is_set(EVENT::EVENT) {
                self.registers.events_stopped.write(EVENT::EVENT::CLEAR);

                self.client.map(|client| match self.buf.take() {
                    None => (),
                    Some(buf) => {
                        client.command_complete(buf, Ok(()));
                    }
                });
            }

            if self.registers.events_error.is_set(EVENT::EVENT) {
                self.registers.events_error.write(EVENT::EVENT::CLEAR);
                let errorsrc = self.registers.errorsrc_master.extract();
                self.registers
                    .errorsrc_master
                    .write(ERRORSRC::ANACK::ErrorDidNotOccur + ERRORSRC::DNACK::ErrorDidNotOccur);
                self.client.map(|client| match self.buf.take() {
                    None => (),
                    Some(buf) => {
                        let status = if errorsrc.is_set(ERRORSRC::ANACK) {
                            Err(hil::i2c::Error::AddressNak)
                        } else if errorsrc.is_set(ERRORSRC::DNACK) {
                            Err(hil::i2c::Error::DataNak)
                        } else {
                            Ok(())
                        };
                        client.command_complete(buf, status);
                    }
                });
            }
        } else {
            self.registers.events_stopped.write(EVENT::EVENT::CLEAR);

            // If RX started and we don't have a buffer then report
            // read_expected()
            if self.registers.events_rxstarted.is_set(EVENT::EVENT) {
                self.registers.events_rxstarted.write(EVENT::EVENT::CLEAR);
                self.slave_client
                    .map(|client| match self.slave_read_buf.take() {
                        None => {
                            client.write_expected();
                        }
                        Some(_buf) => {}
                    });
            }

            if self.registers.events_write.is_set(EVENT::EVENT) {
                self.registers.events_write.write(EVENT::EVENT::CLEAR);
                let length = self.registers.rxd_amount.read(AMOUNT::AMOUNT) as usize;
                self.slave_client.map(|client| match self.buf.take() {
                    None => (),
                    Some(buf) => {
                        client.command_complete(
                            buf,
                            length,
                            hil::i2c::SlaveTransmissionType::Write,
                        );
                    }
                });
            }

            if self.registers.events_read.is_set(EVENT::EVENT) {
                self.registers.events_read.write(EVENT::EVENT::CLEAR);
                let length = self.registers.txd_amount.read(AMOUNT::AMOUNT) as usize;
                self.slave_client
                    .map(|client| match self.slave_read_buf.take() {
                        None => (),
                        Some(buf) => {
                            client.command_complete(
                                buf,
                                length,
                                hil::i2c::SlaveTransmissionType::Read,
                            );
                        }
                    });
            }
        }

        // We can blindly clear the following events since we're not using them.
        self.registers.events_suspended.write(EVENT::EVENT::CLEAR);
        self.registers.events_rxstarted.write(EVENT::EVENT::CLEAR);
        self.registers.events_lastrx.write(EVENT::EVENT::CLEAR);
        self.registers.events_lasttx.write(EVENT::EVENT::CLEAR);
    }

    pub fn is_enabled(&self) -> bool {
        self.is_master_enabled() || self.is_slave_enabled()
    }

    fn is_master_enabled(&self) -> bool {
        self.registers
            .enable
            .matches_all(ENABLE::ENABLE::EnableMaster)
    }

    fn is_slave_enabled(&self) -> bool {
        self.registers
            .enable
            .matches_all(ENABLE::ENABLE::EnableSlave)
    }
}

impl hil::i2c::I2CMaster for TWI {
    fn set_master_client(&self, client: &'static dyn hil::i2c::I2CHwMasterClient) {
        self.client.set(client);
    }

    fn enable(&self) {
        self.enable_master();
    }

    fn disable(&self) {
        self.disable();
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.registers
            .address_0
            .write(ADDRESS::ADDRESS.val(addr as u32));
        self.registers.txd_ptr.set(data.as_mut_ptr() as u32);
        self.registers
            .txd_maxcnt
            .write(MAXCNT::MAXCNT.val(write_len as u32));
        self.registers.rxd_ptr.set(data.as_mut_ptr() as u32);
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
        Ok(())
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.registers
            .address_0
            .write(ADDRESS::ADDRESS.val(addr as u32));
        self.registers.txd_ptr.set(data.as_mut_ptr() as u32);
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
        Ok(())
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.registers
            .address_0
            .write(ADDRESS::ADDRESS.val(addr as u32));
        self.registers.rxd_ptr.set(buffer.as_mut_ptr() as u32);
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
        Ok(())
    }
}

impl hil::i2c::I2CSlave for TWI {
    fn set_slave_client(&self, client: &'static dyn hil::i2c::I2CHwSlaveClient) {
        self.slave_client.set(client);
    }

    fn enable(&self) {
        self.enable_slave();
    }

    fn disable(&self) {
        self.disable();
    }

    fn set_address(&self, addr: u8) -> Result<(), hil::i2c::Error> {
        self.registers
            .address_0
            .write(ADDRESS::ADDRESS.val(addr as u32));
        self.registers.config.modify(CONFIG::ADDRESS0::Enable);
        Ok(())
    }

    fn write_receive(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.registers.rxd_ptr.set(data.as_mut_ptr() as u32);
        self.registers
            .rxd_maxcnt
            .write(MAXCNT::MAXCNT.val(max_len as u32));

        self.registers
            .intenset
            .modify(INTE::STOPPED::Enable + INTE::ERROR::Enable);

        self.buf.replace(data);

        self.registers.tasks_preparerx.write(TASK::TASK::SET);

        Ok(())
    }

    fn read_send(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        self.registers.txd_ptr.set(data.as_mut_ptr() as u32);
        self.registers
            .txd_maxcnt
            .write(MAXCNT::MAXCNT.val(max_len as u32));

        self.registers
            .intenset
            .modify(INTE::STOPPED::Enable + INTE::ERROR::Enable + INTE::READ::Enable);

        self.slave_read_buf.replace(data);

        self.registers.tasks_preparetx.write(TASK::TASK::SET);

        Ok(())
    }

    fn listen(&self) {
        self.registers.tasks_preparerx.write(TASK::TASK::SET);
    }
}

// The SPI0_TWI0 and SPI1_TWI1 interrupts are dispatched to the
// correct handler by the service_pending_interrupts() routine in
// chip.rs based on which peripheral is enabled.

register_structs! {
    pub TwiRegisters {
        /// Start TWI receive sequence
        (0x00 => tasks_startrx: WriteOnly<u32, TASK::Register>),
        (0x04 => _reserved0),
        /// Start TWI transmit sequence
        (0x08 => tasks_starttx: WriteOnly<u32, TASK::Register>),
        (0x0C => _reserved1),
        /// Stop TWI transaction
        (0x14 => tasks_stop: WriteOnly<u32, TASK::Register>),
        (0x18 => _reserved2),
        /// Suspend TWI transaction
        (0x1C => tasks_suspend: WriteOnly<u32, TASK::Register>),
        /// Resume TWI transaction
        (0x20 => tasks_resume: WriteOnly<u32, TASK::Register>),
        (0x24 => _reserved3),
        (0x30 => tasks_preparerx: WriteOnly<u32, TASK::Register>),
        (0x34 => tasks_preparetx: WriteOnly<u32, TASK::Register>),
        (0x38 => _reserved4),
        /// TWI stopped
        (0x104 => events_stopped: ReadWrite<u32, EVENT::Register>),
        (0x108 => _reserved5),
        /// TWI error
        (0x124 => events_error: ReadWrite<u32, EVENT::Register>),
        (0x128 => _reserved6),
        /// Last byte has been sent out after the SUSPEND task has been issued, TWI
        /// traffic is now suspended.
        (0x148 => events_suspended: ReadWrite<u32, EVENT::Register>),
        /// Receive sequence started
        (0x14C => events_rxstarted: ReadWrite<u32, EVENT::Register>),
        /// Transmit sequence started
        (0x150 => events_txstarted: ReadWrite<u32, EVENT::Register>),
        (0x154 => _reserved7),
        /// Byte boundary, starting to receive the last byte
        (0x15C => events_lastrx: ReadWrite<u32, EVENT::Register>),
        /// Byte boundary, starting to transmit the last byte
        (0x160 => events_lasttx: ReadWrite<u32, EVENT::Register>),
        (0x164 => events_write: ReadWrite<u32, EVENT::Register>),
        (0x168 => events_read: ReadWrite<u32, EVENT::Register>),
        (0x16C => _reserved8),
        /// Shortcut register
        (0x200 => shorts: ReadWrite<u32, SHORTS::Register>),
        (0x204 => _reserved9),
        /// Enable or disable interrupt
        (0x300 => inten: ReadWrite<u32, INTE::Register>),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, INTE::Register>),
        /// Disable interrupt
        (0x308 => intenclr: ReadWrite<u32, INTE::Register>),
        (0x30C => _reserved10),
        /// Error source
        (0x4C4 => errorsrc_master: ReadWrite<u32, ERRORSRC::Register>),
        (0x4C8 => _reserved11),
        (0x4D0 => errorsrc_slave: ReadWrite<u32, ERRORSRC::Register>),
        (0x4D4 => match_reg: ReadWrite<u32>),
        (0x4D8 => _reserved12),
        /// Enable TWI
        (0x500 => enable: ReadWrite<u32, ENABLE::Register>),
        (0x504 => _reserved13),
        /// Pin select for SCL signal
        (0x508 => psel_scl: VolatileCell<Pinmux>),
        /// Pin select for SDA signal
        (0x50C => psel_sda: VolatileCell<Pinmux>),
        (0x510 => _reserved_14),
        /// TWI frequency
        (0x524 => frequency: ReadWrite<u32>),
        (0x528 => _reserved15),
        /// Data pointer
        (0x534 => rxd_ptr: ReadWrite<u32>),
        /// Maximum number of bytes in receive buffer
        (0x538 => rxd_maxcnt: ReadWrite<u32, MAXCNT::Register>),
        /// Number of bytes transferred in the last transaction
        (0x53C => rxd_amount: ReadWrite<u32, AMOUNT::Register>),
        /// EasyDMA list type
        (0x540 => rxd_list: ReadWrite<u32>),
        /// Data pointer
        (0x544 => txd_ptr: ReadWrite<u32>),
        /// Maximum number of bytes in transmit buffer
        (0x548 => txd_maxcnt: ReadWrite<u32, MAXCNT::Register>),
        /// Number of bytes transferred in the last transaction
        (0x54C => txd_amount: ReadWrite<u32, AMOUNT::Register>),
        /// EasyDMA list type
        (0x550 => txd_list: ReadWrite<u32>),
        (0x554 => _reserved_16),
        /// Address used in the TWI transfer
        (0x588 => address_0: ReadWrite<u32, ADDRESS::Register>),
        (0x58C => address_1: ReadWrite<u32, ADDRESS::Register>),
        (0x590 => _reserved_17),
        (0x594 => config: ReadWrite<u32, CONFIG::Register>),
        (0x598 => _reserved_18),
        (0x5C0 => orc: ReadWrite<u32>),
        (0x5C4 => @END),
    }
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
        ],
        WRITE OFFSET(25) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        READ OFFSET(26) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
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
        /// Enable or disable TWI
        ENABLE OFFSET(0) NUMBITS(4) [
            Disable = 0,
            EnableMaster = 6,
            EnableSlave = 9,
        ]
    ],
    MAXCNT [
        /// Maximum number of bytes in buffer
        MAXCNT OFFSET(0) NUMBITS(16)
    ],
    AMOUNT [
        AMOUNT OFFSET(0) NUMBITS(7),
    ],
    ADDRESS [
        /// Address used in the TWI transfer
        ADDRESS OFFSET(0) NUMBITS(7)
    ],
    CONFIG [
        /// Address used in the TWI transfer
        ADDRESS0 OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        ADDRESS1 OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ]
    ],
];
