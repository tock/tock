use core::cell::Cell;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::hil::i2c::{self, Error, I2CHwMasterClient, I2CMaster};
use kernel::ClockInterface;

use crate::rcc;

pub enum I2CSpeed {
    Speed100k,
    Speed400k,
    Speed1M,
}

/// Serial peripheral interface
#[repr(C)]
struct I2CRegisters {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// own address register 1
    oar1: ReadWrite<u32, OAR1::Register>,
    /// own address register 2
    oar2: ReadWrite<u32, OAR2::Register>,
    /// timing register
    timingr: ReadWrite<u32, TIMINGR::Register>,
    /// timeout register
    timeout: ReadWrite<u32, TIMEOUT::Register>,
    /// interrupt and status register
    isr: ReadWrite<u32, ISR::Register>,
    /// interrupt clear register
    icr: ReadWrite<u32, ICR::Register>,
    /// PEC register
    pecr: ReadWrite<u32, PECR::Register>,
    /// receive data register
    rxdr: ReadWrite<u32, RXDR::Register>,
    /// transmit data register
    txdr: ReadWrite<u32, TXDR::Register>,
}

register_bitfields![u32,
    CR1 [
        /// PEC enable
        PCEN OFFSET(23) NUMBITS(1) [],
        /// SMBus alert enable
        ALERTEN OFFSET(22) NUMBITS(1) [],
        /// SMBus Device Default address enable
        SMBDEN OFFSET(21) NUMBITS(1) [],
        /// SMBus Host address enable
        SMBHEN OFFSET(20) NUMBITS(1) [],
        /// General call enable
        GCEN OFFSET(19) NUMBITS(1) [],
        /// Wakeup from Stop mode enable
        WUPEN OFFSET(18) NUMBITS(1) [],
        /// Clock stretching disable
        NOSTRETCH OFFSET(17) NUMBITS(1) [],
        /// Slave byte control
        SBC OFFSET(16) NUMBITS(1) [],
        /// DMA reception requests enable
        RXDMAEN OFFSET(15) NUMBITS(1) [],
        /// DMA transmission requests enable
        TXDMAEN OFFSET(14) NUMBITS(1) [],
        /// Analog noise filter OFF
        ANOFF OFFSET(12) NUMBITS(1) [],
        /// Digital noise filter
        DNF OFFSET(8) NUMBITS(4) [],
        /// Error interrupts enable
        ERRIE OFFSET(7) NUMBITS(1) [],
        /// Transfer Complete interrupt enable
        TCIE OFFSET(6) NUMBITS(1) [],
        /// STOP detection Interrupt enable
        STOPIE OFFSET(5) NUMBITS(1) [],
        /// Not acknowledge received Interrupt enable
        NACKIE OFFSET(4) NUMBITS(1) [],
        /// Address match Interrupt enable (slave only)
        ADDRIE OFFSET(3) NUMBITS(3) [],
        /// RX Interrupt enable
        RXIE OFFSET(2) NUMBITS(1) [],
        /// TX Interrupt enable
        TXIE OFFSET(1) NUMBITS(1) [],
        /// Peripheral enable
        PE OFFSET(0) NUMBITS(1) []
    ],
    CR2 [
        /// Packet error checking byte
        PECBYTE OFFSET(26) NUMBITS(1) [],
        /// Automatic end mode (master mode)
        AUTOEND OFFSET(25) NUMBITS(1) [],
        /// NBYTES reload mode
        RELOAD OFFSET(24) NUMBITS(1) [],
        /// Number of bytes
        NBYTES OFFSET(16) NUMBITS(8) [],
        /// NACK generation (slave mode)
        NACK OFFSET(15) NUMBITS(1) [],
        /// Stop generation (master mode)
        STOP OFFSET(14) NUMBITS(1) [],
        /// Start generation
        START OFFSET(13) NUMBITS(1) [],
        /// 10-bit address header only read direction (master receiver mode)
        HEAD10R OFFSET(12) NUMBITS(1) [],
        /// 10-bit addressing mode (master mode)
        ADD10 OFFSET(11) NUMBITS(1) [],
        /// Transfer direction (master mode)
        RD_WRN OFFSET(10) NUMBITS(1) [],
        /// Slave address bit 9:8 (master mode)
        SADD8_9 OFFSET(8) NUMBITS(2) [],
        // Slave address bit 7:1 (master mode)
        SADD7_1 OFFSET(1) NUMBITS(7) [],
        /// Slave address bit 0 (master mode)
        SADD OFFSET(0) NUMBITS(1) []
    ],
    OAR1 [
        /// Own Address 1 enable
        OA1EN OFFSET(15) NUMBITS(1) [],
        /// Own Address 1 10-bitmode
        OA1MODE OFFSET(10) NUMBITS(1) [],
        /// Interface address
        OA1 OFFSET(0) NUMBITS(10) []
    ],
    OAR2 [
        /// Own Address 2 enable
        OA2EN OFFSET(15) NUMBITS(1) [],
        /// Own Address 2 masks
        OA2MSK OFFSET(8) NUMBITS(3) [],
        /// Interface address
        OA2 OFFSET(1) NUMBITS(7) []
    ],
    TIMINGR [
        /// Timing prescaler
        PRESC OFFSET(28) NUMBITS(4) [],
        /// Data setup time
        SCLDEL OFFSET(20) NUMBITS(4) [],
        /// Data hold time
        SDAEL OFFSET(16) NUMBITS(4) [],
        /// SCL high period (master mode)
        SCLH OFFSET(8) NUMBITS(8) [],
        /// SCL low period (master mode)
        SCLL OFFSET(0) NUMBITS(8) []
    ],
    TIMEOUT [
        /// Extended clock timeout enable
        TEXTEN OFFSET(31) NUMBITS(1) [],
        /// Bus timeout B
        TIMEOUTB OFFSET(16) NUMBITS(12) [],
        /// Clock timeout enable
        TIMOUTEN OFFSET(15) NUMBITS(1) [],
        /// Idle clock timeout detection
        TIDLE OFFSET(12) NUMBITS(1) [],
        /// Bus Timeout A
        TIMEOUTA OFFSET(0) NUMBITS(12) []
    ],
    ISR [
        /// Address match code (slavemode)
        ADDCODE OFFSET(17) NUMBITS(7) [],
        /// Transfer direction (slave mode)
        DIR OFFSET(16) NUMBITS(1) [],
        /// Bus busy
        BUSY OFFSET(15) NUMBITS(1) [],
        /// SMBus alert
        ALERT OFFSET(13) NUMBITS(1) [],
        /// Timeout or tLOW detection flag
        TIMEOUT OFFSET(12) NUMBITS(1) [],
        /// Bus error
        PECERR OFFSET(11) NUMBITS(1) [],
        /// Overrun/Underrun (slave mode)
        OVR OFFSET(10) NUMBITS(1) [],
        /// Arbitration lost
        ARLO OFFSET(9) NUMBITS(1) [],
        /// Bus error
        BERR OFFSET(8) NUMBITS(1) [],
        /// Transfer Complete Reload
        TCR OFFSET(7) NUMBITS(1) [],
        /// Transfer Complete (master mode)
        TC OFFSET(6) NUMBITS(1) [],
        /// Stop detection flag
        STOPF OFFSET(5) NUMBITS(1) [],
        /// Not Acknowledge received flag
        NACKF OFFSET(4) NUMBITS(1) [],
        /// Address matched (slave mode)
        ADDR OFFSET(3) NUMBITS(1) [],
        /// Receive data register not empty (receivers)
        RXNE OFFSET(2) NUMBITS(1) [],
        /// Transmit interrupt status (transmitters)
        TXIS OFFSET(1) NUMBITS(1) [],
        /// Transmit data register empty (transmitters)
        TXE OFFSET(0) NUMBITS(1) []
    ],
    ICR [
        /// Alert flag clear
        ALERTCF OFFSET(13) NUMBITS(1) [],
        /// Timeout detection flag clear
        TIMOUTCF OFFSET(12) NUMBITS(1) [],
        /// PEC Error flag clear
        PECCF OFFSET(11) NUMBITS(1) [],
        /// Overrun/Underrun flag clear
        OVRCF OFFSET(10) NUMBITS(1) [],
        /// Arbitration Lost flag clear
        ARLOCF OFFSET(9) NUMBITS(1) [],
        /// Bus error flag clear
        BERRCF OFFSET(8) NUMBITS(1) [],
        /// Stop detection flag clear
        STOPCF OFFSET(5) NUMBITS(1) [],
        /// Not Acknowledge flag clear
        NACKCF OFFSET(4) NUMBITS(1) [],
        /// Address matched flag clear
        ADDRCF OFFSET(3) NUMBITS(1) []
    ],
    PECR [
        /// Packet error checking register
        PEC OFFSET(0) NUMBITS(8) []
    ],
    RXDR [
        /// 8-bit receive data
        RXDATA OFFSET(0) NUMBITS(8) []
    ],
    TXDR [
        /// 8-bit transmit data
        TXDATA OFFSET(0) NUMBITS(8) []
    ]
];

const I2C1_BASE: StaticRef<I2CRegisters> =
    unsafe { StaticRef::new(0x4000_5400 as *const I2CRegisters) };

// const I2C2_BASE: StaticRef<I2CRegisters> =
// 	unsafe { StaticRef::new(0x4000_5800 as *const I2CRegisters) };

pub struct I2C<'a> {
    registers: StaticRef<I2CRegisters>,
    clock: I2CClock,

    // I2C slave support not yet implemented
    master_client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,

    buffer: TakeCell<'static, [u8]>,
    tx_position: Cell<u8>,
    rx_position: Cell<u8>,
    tx_len: Cell<u8>,
    rx_len: Cell<u8>,

    slave_address: Cell<u8>,

    status: Cell<I2CStatus>,
    // transfers: Cell<u8>
}

#[derive(Copy, Clone, PartialEq)]
enum I2CStatus {
    Idle,
    Writing,
    WritingReading,
    Reading,
}

pub static mut I2C1: I2C = I2C::new(
    I2C1_BASE,
    I2CClock(rcc::PeripheralClock::APB1(rcc::PCLK1::I2C1)),
);

impl I2C<'a> {
    const fn new(base_addr: StaticRef<I2CRegisters>, clock: I2CClock) -> I2C<'a> {
        I2C {
            registers: base_addr,
            clock,

            master_client: OptionalCell::empty(),

            slave_address: Cell::new(0),

            buffer: TakeCell::empty(),
            tx_position: Cell::new(0),
            rx_position: Cell::new(0),

            tx_len: Cell::new(0),
            rx_len: Cell::new(0),

            status: Cell::new(I2CStatus::Idle),
        }
    }

    pub fn set_speed(&self, speed: I2CSpeed, system_clock_in_mhz: usize) {
        // debug!("stm32f3 i2c set_speed");
        self.disable();
        match speed {
            I2CSpeed::Speed100k => {
                let prescaler = system_clock_in_mhz / 4 - 1;
                self.registers.timingr.modify(
                    TIMINGR::PRESC.val(prescaler as u32)
                        + TIMINGR::SCLL.val(19)
                        + TIMINGR::SCLH.val(15)
                        + TIMINGR::SDAEL.val(2)
                        + TIMINGR::SCLDEL.val(4),
                );
            }
            I2CSpeed::Speed400k => {
                let prescaler = system_clock_in_mhz / 8 - 1;
                self.registers.timingr.modify(
                    TIMINGR::PRESC.val(prescaler as u32)
                        + TIMINGR::SCLL.val(9)
                        + TIMINGR::SCLH.val(3)
                        + TIMINGR::SDAEL.val(3)
                        + TIMINGR::SCLDEL.val(3),
                );
            }
            I2CSpeed::Speed1M => {
                panic!("i2c speed 1MHz not implemented");
            }
        }
        self.enable();
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        // debug!("stm32f3 i2c enable clock");
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn handle_event(&self) {
        // debug!("stm32f3 i2c event");
        if self.registers.isr.is_set(ISR::TXIS) {
            // send the next byte
            if self.buffer.is_some() && self.tx_position.get() < self.tx_len.get() {
                self.buffer.map(|buf| {
                    let byte = buf[self.tx_position.get() as usize];
                    // debug!("sending byte {}", byte);
                    self.registers.txdr.write(TXDR::TXDATA.val(byte as u32));
                    self.tx_position.set(self.tx_position.get() + 1);
                });
            } else {
                // TODO disable TXIE
                // debug!("i2c error, attempting to transmit more bytes than available in the buffer");
            }
        }

        while self.registers.isr.is_set(ISR::RXNE) {
            // send the next byte
            let byte = self.registers.rxdr.read(RXDR::RXDATA) as u8;
            if self.buffer.is_some() && self.rx_position.get() < self.rx_len.get() {
                self.buffer.map(|buf| {
                    // debug!("read byte {}", byte);
                    buf[self.rx_position.get() as usize] = byte;
                    self.rx_position.set(self.rx_position.get() + 1);
                });
            } else {
                // TODO disable RXIE
                // debug!("i2c drop byte");
            }
        }

        if self.registers.isr.is_set(ISR::TC) {
            match self.status.get() {
                I2CStatus::Writing | I2CStatus::WritingReading => {
                    // debug!(
                    //     "WriteRead transfer partial complete {}, {}",
                    //     self.tx_position.get(),
                    //     self.rx_position.get()
                    // );
                    if self.tx_position.get() < self.tx_len.get() {
                        self.master_client.map(|client| {
                            self.buffer
                                .take()
                                .map(|buf| client.command_complete(buf, Error::DataNak))
                        });
                        self.registers.cr2.modify(CR2::STOP::SET);
                        self.stop();
                    } else {
                        // debug!(
                        //     "Write transfer complete {}, {}",
                        //     self.tx_position.get(),
                        //     self.rx_position.get()
                        // );
                        if self.status.get() == I2CStatus::Writing {
                            self.master_client.map(|client| {
                                self.buffer
                                    .take()
                                    .map(|buf| client.command_complete(buf, Error::CommandComplete))
                            });
                            self.registers.cr2.modify(CR2::STOP::SET);
                            self.stop();
                        } else {
                            self.status.set(I2CStatus::Reading);
                            self.start_read();
                        }
                    }
                }
                I2CStatus::Reading => {
                    // debug!(
                    //     "Read transfer complete {}, {}",
                    //     self.tx_position.get(),
                    //     self.rx_position.get()
                    // );
                    let error = if self.rx_position.get() == self.rx_len.get() {
                        Error::CommandComplete
                    } else {
                        Error::DataNak
                    };
                    self.master_client.map(|client| {
                        self.buffer
                            .take()
                            .map(|buf| client.command_complete(buf, error))
                    });
                    self.registers.cr2.modify(CR2::STOP::SET);
                    self.stop();
                }
                _ => panic!("i2c should noy be here"),
            }
        }

        // if self.registers.isr.is_set(ISR::STOPF) {
        //     // debug!("i2c transfer stop");
        //     self.registers.icr.modify(ICR::STOPCF::SET);
        //     match self.status.get() {
        //         I2CStatus::Writing => {
        //             debug!("i2c writing only");
        //             let error = if self.tx_position.get() == self.tx_len.get() {
        //                 Error::CommandComplete
        //             } else {
        //                 Error::DataNak
        //             };
        //             self.master_client.map(|client| {
        //                 self.buffer
        //                     .take()
        //                     .map(|buf| client.command_complete(buf, error))
        //             });
        //             self.stop();
        //             self.status.set(I2CStatus::Idle);
        //         }
        //         I2CStatus::Reading => {
        //             debug!(
        //                 "Read transfer complete {}, {}",
        //                 self.tx_position.get(),
        //                 self.rx_position.get()
        //             );
        //             let error = if self.rx_position.get() == self.rx_len.get() {
        //                 Error::CommandComplete
        //             } else {
        //                 Error::DataNak
        //             };
        //             self.master_client.map(|client| {
        //                 self.buffer
        //                     .take()
        //                     .map(|buf| client.command_complete(buf, error))
        //             });
        //             self.registers.cr2.modify(CR2::STOP::SET);
        //             self.stop();
        //         }
        //         _ => panic!("i2c should not arrive here"),
        //     }
        // }

        if self.registers.isr.is_set(ISR::NACKF) {
            // abort transfer due to NACK
            // debug!("i2c not ack");
            self.registers.icr.modify(ICR::NACKCF::SET);
            self.master_client.map(|client| {
                self.buffer
                    .take()
                    .map(|buf| client.command_complete(buf, Error::AddressNak))
            });
            self.registers.cr2.modify(CR2::STOP::SET);
            self.stop();
        }
    }

    pub fn handle_error(&self) {
        // debug!("stm32f3 i2c error");
    }

    fn reset(&self) {
        self.disable();
        self.enable();
    }

    fn start_write(&self) {
        // debug!(
        //     "stm32f3 i2c is idle write addr {} len {}",
        //     self.slave_address.get(),
        //     self.tx_len.get()
        // );
        self.tx_position.set(0);
        self.registers
            .cr2
            .modify(CR2::NBYTES.val(self.tx_len.get() as u32));
        self.registers
            .cr2
            .modify(CR2::SADD7_1.val(self.slave_address.get() as u32));
        self.registers.cr2.modify(CR2::RD_WRN::CLEAR);
        self.registers.cr1.modify(
            CR1::TXIE::SET + CR1::ERRIE::SET + CR1::NACKIE::SET + CR1::TCIE::SET, // + CR1::STOPIE::SET, // + CR1::RXIE::SET,
        );
        // self.registers.cr1.modify(CR1::TXIE::SET);
        // self.registers.cr1.modify(CR1::NACKIE::SET);
        // self.registers.cr1.modify(CR1::ERRIE::SET);
        self.registers.cr2.modify(CR2::START::SET);
    }

    fn stop(&self) {
        self.registers.cr1.modify(
            CR1::TXIE::CLEAR
                + CR1::ERRIE::CLEAR
                + CR1::NACKIE::CLEAR
                + CR1::TCIE::CLEAR
                + CR1::STOPIE::CLEAR
                + CR1::RXIE::CLEAR,
        );
        self.status.set(I2CStatus::Idle);
    }

    fn start_read(&self) {
        // debug!(
        //     "stm32f3 i2c is idle read addr {} len {}",
        //     self.slave_address.get(),
        //     self.rx_len.get()
        // );
        self.rx_position.set(0);
        self.registers
            .cr2
            .modify(CR2::NBYTES.val(self.rx_len.get() as u32));
        self.registers
            .cr2
            .modify(CR2::SADD7_1.val(self.slave_address.get() as u32));
        self.registers.cr2.modify(CR2::AUTOEND::CLEAR);
        self.registers.cr2.modify(CR2::RD_WRN::SET);
        self.registers
            .cr1
            .modify(CR1::ERRIE::SET + CR1::NACKIE::SET + CR1::TCIE::SET + CR1::RXIE::SET);
        // self.registers.cr1.modify(CR1::TXIE::SET);
        // self.registers.cr1.modify(CR1::NACKIE::SET);
        // self.registers.cr1.modify(CR1::ERRIE::SET);
        self.registers.cr2.modify(CR2::START::SET);
    }
}

impl i2c::I2CMaster for I2C<'a> {
    fn set_master_client(&self, master_client: &'static dyn I2CHwMasterClient) {
        self.master_client.replace(master_client);
    }
    fn enable(&self) {
        // debug!("stm32f3 i2c enable");
        self.registers.cr1.modify(CR1::PE::SET);
    }
    fn disable(&self) {
        // debug!("stm32f3 i2c disable");
        self.registers.cr1.modify(CR1::PE::CLEAR);
    }
    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        // debug!("stm32f3 i2c write_read {}", addr);
        if self.status.get() == I2CStatus::Idle {
            self.reset();
            self.status.set(I2CStatus::WritingReading);
            self.slave_address.set(addr);
            self.buffer.replace(data);
            self.tx_len.set(write_len);
            self.rx_len.set(read_len);
            self.registers.cr2.modify(CR2::AUTOEND::CLEAR);
            self.start_write();
        }
    }
    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        // debug!("stm32f3 i2c write {}", addr);
        if self.status.get() == I2CStatus::Idle {
            self.reset();
            self.status.set(I2CStatus::Writing);
            self.slave_address.set(addr);
            self.buffer.replace(data);
            self.tx_len.set(len);
            self.registers.cr2.modify(CR2::AUTOEND::CLEAR);
            self.start_write();
        }
    }
    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        // debug!("stm32f3 i2c read");
        if self.status.get() == I2CStatus::Idle {
            self.reset();
            self.status.set(I2CStatus::Reading);
            self.slave_address.set(addr);
            self.buffer.replace(buffer);
            self.rx_len.set(len);
            self.registers.cr2.modify(CR2::AUTOEND::CLEAR);
            self.start_read();
        }
    }
}

struct I2CClock(rcc::PeripheralClock);

impl ClockInterface for I2CClock {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
