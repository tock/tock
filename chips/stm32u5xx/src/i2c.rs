// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use crate::dma::{self, ChannelId, Dma, DmaPeripheral};
use core::cell::Cell;
use core::fmt::{self};
use cortexm33::dma_fence::CortexMDmaFence;
use kernel::hil::i2c::{self, Error, I2CHwMasterClient, I2CMaster};
use kernel::utilities::{
    StaticRef,
    cells::{MapCell, OptionalCell, TakeCell},
    dma_slice::DmaSubSliceMut,
    leasable_buffer::SubSliceMut,
    registers::{
        ReadOnly, ReadWrite,
        interfaces::{ReadWriteable, Readable, Writeable},
        register_bitfields, register_structs,
    },
};

register_structs! {
    pub I2cRegisters {
        /// Control register 1
        (0x000 => pub cr1: ReadWrite<u32, CR1::Register>),
        /// Control register 2
        (0x004 => pub cr2: ReadWrite<u32, CR2::Register>),
        /// Own address 1 register
        (0x008 => pub oar1: ReadWrite<u32, OAR1::Register>),
        /// Own address 2 register
        (0x00C => pub oar2: ReadWrite<u32, OAR2::Register>),
        /// Timing Register
        (0x010 => pub timingr: ReadWrite<u32, TIMINGR::Register>),
        /// Timeout register
        (0x014 => pub timeoutr: ReadWrite<u32, TIMEOUTR::Register>),
        /// Interrupt and status register
        (0x018 => pub isr: ReadOnly<u32, ISR::Register>),
        /// Interrupt clear register
        (0x01C => pub icr: ReadWrite<u32, ICR::Register>),
        /// PEC (Packet error checking) register
        (0x020 => pub pecr: ReadOnly<u32, PECR::Register>),
        /// Receive data register
        (0x024 => pub rxdr: ReadOnly<u32, RXDR::Register>),
        /// Transmit data register
        (0x028 => pub txdr: ReadWrite<u32, TXDR::Register>),
        /// Autonomous mode control register
        (0x02C => pub autocr: ReadWrite<u32, AUTOCR::Register>),
        (0x030 => @END),
    }
}

// Currently Unused
pub const I2C1_BASE: StaticRef<I2cRegisters> =
    unsafe { StaticRef::new(0x50005400 as *const I2cRegisters) };

register_bitfields![u32,
    pub CR1 [
        /// Peripheral enable
        PE         OFFSET(0)   NUMBITS(1) [],
        /// TX Interrupt enable
        TXIE       OFFSET(1)   NUMBITS(1) [],
        /// RX Interrupt enable
        RXIE       OFFSET(2)   NUMBITS(1) [],
        /// Address match interrupt enable
        ADDRIE     OFFSET(3)   NUMBITS(1) [],
        /// NACK received interrupt enable
        NACKIE     OFFSET(4)   NUMBITS(1) [],
        /// STOP detection interrupt enable
        STOPIE     OFFSET(5)   NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE       OFFSET(6)   NUMBITS(1) [],
        /// Error interrupts enable
        ERRIE      OFFSET(7)   NUMBITS(1) [],
        /// Digital noise filter
        DNF        OFFSET(8)   NUMBITS(4) [],
        /// Analog noise filter OFF
        ANFOFF     OFFSET(12)  NUMBITS(1) [],

        /// DMA transmission requests enable
        TXDMAEN    OFFSET(14)  NUMBITS(1) [],
        /// DMA reception requests enable
        RXDMAEN    OFFSET(15)  NUMBITS(1) [],
        /// Target byte control
        SBC        OFFSET(16)  NUMBITS(1) [],
        /// Clock stretching disable
        NOSTRETCH  OFFSET(17)  NUMBITS(1) [],
        /// Wake-up from Stop mode enable
        WUPEN      OFFSET(18)  NUMBITS(1) [],
        /// General call enable
        GCEN       OFFSET(19)  NUMBITS(1) [],
        /// SMBus host address enable
        SMBHEN     OFFSET(20)  NUMBITS(1) [],
        /// SMBus device default address enable
        SMBDEN     OFFSET(21)  NUMBITS(1) [],
        /// SMBus alert enable
        ALERTEN    OFFSET(22)  NUMBITS(1) [],
        /// Packer error checking (PEC) enable
        PECEN      OFFSET(23)  NUMBITS(1) [],
        /// Fast-mode Plus (Fm+) drive enable
        FMP        OFFSET(24)  NUMBITS(1) [],

        /// Address match flag (ADDR) automatic clear
        ADDRACLR   OFFSET(30)  NUMBITS(1) [],
        /// STOP detection flag (STOPF) automatic clear
        STOPFACLR  OFFSET(31)  NUMBITS(1) [],
    ],
    pub CR2 [
        /// Target address (in controller mode)
        ///     Condition: In 7-bit addressing mode (ADD10 = 0):
        ///         SADD[7:1] represents .Bits SADD[9], SADD[8] and SADD[0] are don't care.
        ///     Condition: In 10-bit addressing mode (ADD10 = 1):
        ///         SADD[9:0] must be written with the 10-bit target address to be sent.
        SADD       OFFSET(0)   NUMBITS(10) [],

        /// Transfer direction (in controller mode)
        ///     0: Controller requests a write transfer
        ///     1: Controller requests a read transfer
        RD_WRN     OFFSET(10)  NUMBITS(1) [],

        /// 10-bit addressing mode (in controller mode)
        ///     0: Controller operates in 7-bit  addressing mode.
        ///     1: Controller operates in 10-bit addressing mode.
        ADD10      OFFSET(11)  NUMBITS(1) [],

        /// 10-bit address header only read direction (in controller mode)
        HEAD10R    OFFSET(12)  NUMBITS(1) [],
        /// START condition generation
        START      OFFSET(13)  NUMBITS(1) [],
        /// STOP condition generation
        STOP       OFFSET(14)  NUMBITS(1) [],

        /// NACK generation (in target mode)
        ///     0: an ACK is sent after current received byte
        ///     1: a NACK is sent after current received byte
        NACK       OFFSET(15)  NUMBITS(1) [],
        /// Number of bytes
        NBYTES     OFFSET(16)  NUMBITS(8) [],

        /// Reload mode
        ///    0: The transfer is completed after the NBYTES data transfer (STOP/RESTART follows).
        ///    1: The tranfer is not completed after the NBYTES data transfer (NBYTES is reloaded).
        ///       TCR flag is set when NBYTES is written, stretching SCL low.
        RELOAD     OFFSET(24)  NUMBITS(1) [],

        /// Automatic end mode (in controller mode)
        ///     0: software end mode: TC flag is set when NBYTES data
        ///     are transferred, streching SCL
        ///        low.
        ///     1: Automatic end mode: a STOP condition is automatically sent
        ///     when NBYTES data are transferred
        AUTOEND    OFFSET(25)  NUMBITS(1) [],
        /// Packet error checking (PEC) byte
        PECBYTE    OFFSET(26)  NUMBITS(1) [],
    ],
    pub OAR1 [
        /// Interface own target address
        OA1        OFFSET(0)   NUMBITS(10) [],
        /// OA1 10-bit mode
        OA1MODE    OFFSET(10)  NUMBITS(1) [],
        /// OA1 enable
        OA1EN      OFFSET(15)  NUMBITS(1) [],
    ],
    pub OAR2 [
        /// Interface address
        OA2        OFFSET(1)   NUMBITS(7) [],
        /// OA2 masks
        OA2MSK     OFFSET(8)   NUMBITS(3) [],
        /// OA2 enable
        OA2EN      OFFSET(15)  NUMBITS(1) [],
    ],
    pub TIMINGR [
        /// SCL low period (in controller mode)
        SCLL       OFFSET(0)   NUMBITS(8) [],
        /// SCL high period (in controller mode)
        SCLH       OFFSET(8)   NUMBITS(8) [],
        /// Data hold time
        SDADEL     OFFSET(16)  NUMBITS(4) [],
        /// Data setup time
        SCLDEL     OFFSET(20)  NUMBITS(4) [],
        /// Timing prescaler
        PRESC      OFFSET(28)  NUMBITS(4) [],
    ],
    pub TIMEOUTR [
        /// Bus timeout A
        TIMEOUTA   OFFSET(0)   NUMBITS(12) [],
        /// Idle clock timeout detection
        TIDLE      OFFSET(12)  NUMBITS(1) [],
        /// Clock timeout enable
        TIMOUTEN   OFFSET(15)  NUMBITS(1) [],
        /// Bus timeout B
        TIMEOUTB   OFFSET(16)  NUMBITS(12) [],
        /// Extended clock timeout enable
        TEXTEN     OFFSET(31)  NUMBITS(1) [],
    ],
    pub ISR [
        /// Transmit data register empty (transmitters)
        TXE        OFFSET(0)   NUMBITS(1) [],
        /// Transmit interupt status (transmitters)
        TXIS       OFFSET(1)   NUMBITS(1) [],
        /// Receive data register not empty (receivers)
        RXNE       OFFSET(2)   NUMBITS(1) [],
        /// Address matched (in target mode)
        ADDR       OFFSET(3)   NUMBITS(1) [],
        /// NACK received flag
        NACKF      OFFSET(4)   NUMBITS(1) [],
        /// STOP detection flag
        STOPF      OFFSET(5)   NUMBITS(1) [],
        /// Transfer complete (in controller mode)
        TC         OFFSET(6)   NUMBITS(1) [],
        /// Transfer complete reload
        TCR        OFFSET(7)   NUMBITS(1) [],
        /// Bus error
        BERR       OFFSET(8)   NUMBITS(1) [],
        /// Arbitration lost
        ARLO       OFFSET(9)   NUMBITS(1) [],
        /// Overrun/underrun (in target mode)
        OVR        OFFSET(10)  NUMBITS(1) [],
        /// PEC (Packet error checking) error in reception
        PECERR     OFFSET(11)  NUMBITS(1) [],
        /// Timeout or t_LOW detection flag
        TIMEOUT    OFFSET(12)  NUMBITS(1) [],
        /// SMBus alert
        ALERT      OFFSET(13)  NUMBITS(1) [],
        /// Bus busy
        BUSY       OFFSET(15)  NUMBITS(1) [],
        /// Transfer direction (in target mode)
        DIR        OFFSET(16)  NUMBITS(1) [],
        // Address match code (in target mode)
        ADDCODE    OFFSET(17)  NUMBITS(7) [],
    ],
    pub ICR [
        /// Address matched flag clear
        ADDRCF     OFFSET(3)   NUMBITS(1) [],
        /// NACK flag clear
        NACKCF     OFFSET(4)   NUMBITS(1) [],
        /// STOP detection flag clear
        STOPCF     OFFSET(5)   NUMBITS(1) [],

        /// Bus error flag clear
        BERRCF     OFFSET(8)   NUMBITS(1) [],
        /// Arbitration lost flag clear
        ARLOCF     OFFSET(9)   NUMBITS(1) [],
        /// Overrun/underrun flag clear
        OVRCF      OFFSET(10)  NUMBITS(1) [],
        /// PEC (Packet error checking) error flag clear
        PECCF      OFFSET(11)  NUMBITS(1) [],
        /// Timeout detection flag clear
        TIMEOUTCF  OFFSET(12)  NUMBITS(1) [],
        /// Alert flag clear
        ALERTCF    OFFSET(13)  NUMBITS(1) [],
    ],
    pub PECR[
        /// PEC (Packet error checking) register
        PEC        OFFSET(0)   NUMBITS(8) [],
    ],
    pub RXDR [
        /// 8-bit receive data
        RXDATA     OFFSET(0)   NUMBITS(8) [],
    ],
    pub TXDR [
        /// 8-bit transmit data
        TXDATA     OFFSET(0)   NUMBITS(8) [],
    ],
    pub AUTOCR [
        /// DMA request enable on Transfer Complete event
        TCDMAEN    OFFSET(6)   NUMBITS(1) [],
        /// DMA request enable on Transfer Complete Reload event
        TCRDMAEN   OFFSET(7)   NUMBITS(1) [],
        /// Trigger selection
        TRIGSEL    OFFSET(16)  NUMBITS(4) [],
        /// Trigger polarity
        TRIGPOL    OFFSET(20)  NUMBITS(1) [],
        /// Trigger enable
        TRIGEN     OFFSET(21)  NUMBITS(1) [],
    ],
];

pub enum I2cSpeed {
    Speed100k,
    Speed400k,
    Speed1M,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum I2cStatus {
    Idle,
    Writing,
    WritingReading,
    Reading,
}

impl fmt::Display for I2cStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            I2cStatus::Idle => write!(f, "IDLE"),
            I2cStatus::Writing => write!(f, "WRITING"),
            I2cStatus::Reading => write!(f, "READING"),
            I2cStatus::WritingReading => write!(f, "WRITING_READING"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum I2cDirection {
    To,
    From,
}

impl fmt::Display for I2cDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            I2cDirection::To => write!(f, "TO"),
            I2cDirection::From => write!(f, "FROM"),
        }
    }
}

/// I2C driver implementation with DMA for the STM32U5 series.
/// Currently, it is a controller-only driver.
pub struct I2c<'a> {
    pub registers: StaticRef<I2cRegisters>,

    // DMA reference, buffer, and TX/RX channels
    dma: OptionalCell<&'a Dma>,
    dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    dma_channel_tx: OptionalCell<ChannelId>,
    dma_channel_rx: OptionalCell<ChannelId>,

    // hil::i2c passes a buf, and either a write_len, a read_len, or both
    // here we save them locally
    buf: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,
    read_len: Cell<usize>,

    // Master/(and hopefully in the future)Slave Client
    master_client: OptionalCell<&'a dyn I2CHwMasterClient>,

    // State machine current status
    status: Cell<I2cStatus>,

    // I2C protocol variables
    slave_address: Cell<usize>,
    position: Cell<usize>,
    direction: Cell<I2cDirection>,

    // Error handling adjacent
    addr_ack: Cell<bool>,
    error: OptionalCell<Error>,
}

impl<'a> I2c<'a> {
    pub fn new(base: StaticRef<I2cRegisters>) -> Self {
        Self {
            registers: base,
            dma: OptionalCell::empty(),

            dma_channel_tx: OptionalCell::empty(),
            dma_channel_rx: OptionalCell::empty(),

            dma_buf: MapCell::empty(),

            buf: TakeCell::empty(),
            write_len: Cell::new(0),
            read_len: Cell::new(0),
            position: Cell::new(0),

            master_client: OptionalCell::empty(),
            slave_address: Cell::new(0),
            direction: Cell::new(I2cDirection::To),

            status: Cell::new(I2cStatus::Idle),
            addr_ack: Cell::new(false),
            error: OptionalCell::empty(),
        }
    }

    pub fn set_dma(i2c: &'static Self, dma: &'a Dma, tx_channel: ChannelId, rx_channel: ChannelId) {
        i2c.dma.set(dma);

        i2c.dma_channel_tx.set(tx_channel);
        i2c.dma_channel_rx.set(rx_channel);

        dma.set_client(tx_channel, i2c);
        dma.set_client(rx_channel, i2c);
    }

    pub fn set_speed(&self, speed: I2cSpeed) {
        self.disable();

        // The following values for the TIMINGR register
        // have been found using the STM32CubeMX tool,
        // as per the STM32U5 documentation
        // (65.4.10 I2C_TIMINGR register configuration examples or page 2720)
        //
        // These values are for the PCLK1 configuration (present in ./rcc.rs)
        match speed {
            I2cSpeed::Speed100k => {
                self.registers.timingr.set(0x0000_0E14);
            }
            I2cSpeed::Speed400k => {
                self.registers.timingr.set(0x0000_0004);
            }
            I2cSpeed::Speed1M => {
                self.registers.timingr.set(0x0000_0000);
            }
        }

        self.enable();
    }

    pub fn enable(&self) {
        // Fast-mode Plus drive enable
        // Even if the Board integrator has not chosen to use Fm+,
        // It's a good default.
        self.registers.cr1.modify(CR1::FMP::SET);

        // Turning AUTOEND completely off: all end operations are handled in software
        self.registers.cr2.modify(CR2::AUTOEND::CLEAR);

        // Allowing several interrupts
        self.registers.cr1.modify(
            CR1::ERRIE::SET
                + CR1::STOPIE::SET
                + CR1::NACKIE::SET
                + CR1::ADDRIE::SET
                + CR1::TCIE::SET,
        );

        // Setting the peripheral enable
        self.registers.cr1.modify(CR1::PE::SET);
    }

    pub fn disable(&self) {
        // Clearing the peripheral enable
        self.registers.cr1.modify(CR1::PE::CLEAR);
    }

    fn reset(&self) {
        // Resetting local variables
        self.write_len.set(0);
        self.read_len.set(0);
        self.position.set(0);

        self.slave_address.set(0);
        self.direction.set(I2cDirection::To);
        self.status.set(I2cStatus::Idle);
        self.addr_ack.set(false);
        self.error.clear();

        // The documentation (Chapter 65.4.6 or page 2698) states
        // that for an effective reset of the peripheral, the prodedure is:
        //      Disable the peripheral
        //      Read the CR1::PE bit
        //      Enable the peripheral
        self.disable();
        self.registers.cr1.get();
        self.enable();
    }

    /// (More information about NBYTES and RELOAD can be found
    ///  on page 2713 of the STM32U5-series Reference Manual)
    ///
    /// The I2C peripheral features a bytes-counter
    /// (for which the final value is set in NBYTES),
    /// which counts the number of TXIS/RXNE events.
    ///
    /// Before starting a transmission/reception, NBYTES must be set.
    /// But if you wish to transmit/receive more than 255 bytes,
    /// NBYTES must be set to 0xFF (255) and RELOAD to 1.
    ///
    /// For every "frame" of 255 bytes, a decision must be made:
    ///    RELOAD=1 NBYTES=255  if diff > 255 bytes
    ///    RELOAD=1 NBYTES=diff if diff < 255 bytes
    ///   (where diff is the remaining number of bytes to be sent)
    pub fn update_nbytes(&self) {
        let saved_len = match self.direction.get() {
            I2cDirection::To => &self.write_len,
            I2cDirection::From => &self.read_len,
        };

        // To assess whether this is the last chunk or not
        let diff = saved_len.get() - self.position.get();

        if diff < 255 {
            // Last chunk of bytes to be written

            // Make sure that reload is set to zero
            self.registers.cr2.modify(CR2::RELOAD::CLEAR);
            // And then write the value of that computation to NBYTES
            self.registers.cr2.modify(CR2::NBYTES.val(diff as u32));
        } else {
            // Intermediary chunk of bytes to be written

            // Set reload to 1
            self.registers.cr2.modify(CR2::RELOAD::SET);
            // Increment position with 255
            self.position.update(|pos| pos + 255);
            // Write 255 to NBYTES
            self.registers.cr2.modify(CR2::NBYTES.val(255));
        }
    }

    pub fn handle_interrupt(&self) {
        // TCR (Transfer Complete Reload)
        // is set when in RELOAD = 1
        // meaning there is still data to transfer
        if self.registers.isr.is_set(ISR::TCR) {
            // Update NBYTES will compute the next value of NBYTES
            // and set the RELOAD bit when it's the last chunk of 255 bytes to be sent

            // TCR is automatically cleared
            // when NBYTES is written with a non-zero value
            self.update_nbytes();
        }

        // TC (Transfer Complete)
        // is set when the transer has fully finished, no reloading.
        // In this case, we need to check if we should issue:
        //      START: if we're in a WriteRead operation and moving from Write to Read (ReSTART)
        //      STOP:  if we're completely done with whatever was on the bus
        if self.registers.isr.is_set(ISR::TC) {
            // If the TC flag is raised in a WritingReading operation,
            // And it had been in the "Writing" stage of it
            // We need to send a (Re)START

            // TC is also automatically cleared
            // when either START or STOP is set.
            if self.status.get() == I2cStatus::WritingReading
                && self.direction.get() == I2cDirection::To
            {
                let Some(buf) = self.buf.take() else {
                    // This is a very unlikely negative path: the only possibility of this arising is a
                    // freak incident
                    kernel::debug!("Freak incident: resetting device");
                    self.reset();
                    return;
                };

                let _ =
                    self.start_transfer(self.slave_address.get() as u8, buf, I2cDirection::From);

                self.registers.cr2.modify(CR2::START::SET);
            } else {
                self.registers.cr2.modify(CR2::STOP::SET);
            }
        }

        // Checking the ADDR flag ensures that the slave device has responsed to it's address
        // And if it hasn't, then the driver will respond with an Error::AddressNak
        if self.registers.isr.is_set(ISR::ADDR) {
            self.addr_ack.set(true);
            self.registers.icr.modify(ICR::ADDRCF::SET);
        }

        if self.registers.isr.is_set(ISR::NACKF) {
            // If the address had been acknowledged, then it's a data problem
            if self.addr_ack.get() {
                self.error.set(Error::DataNak);
            } else {
                self.error.set(Error::AddressNak);
            }

            // Clear the NACK flag, automatically triggering a STOP flag
            // (as per the STM32U5xx Reference Manual, Chapter 65.4.9 or page 2713)
            self.registers.icr.modify(ICR::NACKCF::SET);
        }

        if self.registers.isr.is_set(ISR::STOPF) {
            // Clearing the STOPF bit
            self.registers.icr.modify(ICR::STOPCF::SET);

            let err_opt = self.error.take();

            // If it's a successful RX, handle_dma_interrupt will upcall
            // with the finished transaction
            if err_opt.is_none() && self.direction.get() == I2cDirection::From {
                return;
            }

            // If there was an error, the transaction aborted mid-flight
            // We must stop DMA to pull the buffer back into self.buf
            // and disable the DMA interaction with the I2C peripheral
            if err_opt.is_some() {
                self.stop_dma();
            }

            // We now definitely need a buffer for the upcall
            let Some(buf) = self.buf.take() else {
                // This is a very unlikely negative path: the only possibility of this arising is a
                // freak incident
                kernel::debug!("Freak incident: resetting device");
                self.reset();
                return;
            };

            // Compute the final result and
            // update the internal state if it was successful
            let result = match err_opt {
                Some(e) => Err(e),
                None => {
                    self.status.set(I2cStatus::Idle);
                    Ok(())
                }
            };

            // Send it back to the consumer of the driver
            self.master_client.map(|client| {
                client.command_complete(buf, result);
            });
        }
    }

    /// The I2C peripheral has a special error interrupt
    /// and some of the ISR bits map 1-to-1 to hil::i2c::Error
    pub fn handle_error(&self) {
        self.stop_dma();

        let Some(buf) = self.buf.take() else {
            // This is a very unlikely negative path: the only possibility of this arising is a
            // freak incident
            kernel::debug!("Freak incident: resetting device");
            self.reset();
            return;
        };

        if self.registers.isr.is_set(ISR::ARLO) {
            self.error.set(Error::ArbitrationLost);
            self.registers.icr.modify(ICR::ARLOCF::SET);
        }

        if self.registers.isr.is_set(ISR::BERR) {
            self.error.set(Error::Busy);
            self.registers.icr.modify(ICR::BERRCF::SET);
        }

        if self.registers.isr.is_set(ISR::OVR) {
            self.error.set(Error::Overrun);
            self.registers.icr.modify(ICR::OVRCF::SET);
        }

        if let Some(e) = self.error.take() {
            self.master_client.map(|client| {
                client.command_complete(buf, Err(e));
            });
        }
    }

    pub fn handle_dma_interrupt(&self, channel: ChannelId) {
        // Stopping DMA means returning back the buffer,
        // and disabling the DMA interaction with the I2C peripheral
        self.stop_dma();

        // If the current action was reading, then this is the safe point
        // where we can send the command_complete to the client
        if self.direction.get() == I2cDirection::From {
            let Some(buf) = self.buf.take() else {
                // This is a very unlikely negative path: the only possibility of this arising is a
                // freak incident
                kernel::debug!("Freak incident: resetting device");
                self.reset();
                return;
            };

            self.master_client.map(|client| {
                client.command_complete(buf, Ok(()));
            });

            self.status.set(I2cStatus::Idle);
        }

        // We clear the interrupt for the  passed down channel
        self.dma.map(|dma| {
            dma.clear_interrupt(channel);
        });
    }

    pub fn start_dma(
        &self,
        buf: &'static mut [u8],
        buf_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        // Making sure we can access DMA
        let Some(dma) = self.dma.get() else {
            return Err((i2c::Error::Busy, buf));
        };

        // Based on the current direction, we will select either of the DMA channels
        let (dma_chan, dma_peripheral) = match self.direction.get() {
            I2cDirection::To => {
                self.registers.cr1.modify(CR1::TXDMAEN::SET);
                (&self.dma_channel_tx, DmaPeripheral::I2c1Tx)
            }
            I2cDirection::From => {
                self.registers.cr1.modify(CR1::RXDMAEN::SET);
                (&self.dma_channel_rx, DmaPeripheral::I2c1Rx)
            }
        };

        // Fail fast if channel is not available
        let Some(ch) = dma_chan.get() else {
            return Err((Error::Busy, buf));
        };

        //  Starting the DmaSubSlice that we can then pass to DMA
        let mut subslice = SubSliceMut::new(buf);
        subslice.slice(0..buf_len);

        let fence = unsafe { CortexMDmaFence::new() };

        let dma_slice = DmaSubSliceMut::new_static(subslice, fence);

        let ptr = dma_slice.as_mut_ptr() as u32;
        let len = dma_slice.len() as u32;

        // Local dma_buf cell
        self.dma_buf.replace(dma_slice);

        // The actual dma start
        dma.setup(ch, dma_peripheral, ptr, len);
        Ok(())
    }

    pub fn stop_dma(&self) {
        // Disabling the I2C peripheral's DMA integration
        self.registers
            .cr1
            .modify(CR1::TXDMAEN::CLEAR + CR1::RXDMAEN::CLEAR);

        // Check that dma_buf does exist
        let Some(dma_buf) = self.dma_buf.take() else {
            kernel::debug!("Freak incident: resetting device");
            return;
        };

        // And unpack it if it does
        let f = unsafe { CortexMDmaFence::new() };
        let mut buf = unsafe { dma_buf.take(f) };
        buf.reset();
        self.buf.replace(buf.take());
    }

    pub fn start_transfer(
        &self,
        addr: u8,
        data: &'static mut [u8],
        direction: I2cDirection,
    ) -> Result<(), (Error, &'static mut [u8])> {
        // First of all, we will set direction
        self.direction.set(direction);
        let saved_len = match direction {
            I2cDirection::To => &self.write_len,
            I2cDirection::From => &self.read_len,
        };

        // Setting the transaction level struct variables
        self.slave_address.set(addr as usize);
        self.position.set(0);
        self.addr_ack.set(false);

        // 7-bit address space
        self.registers.cr2.modify(CR2::ADD10::CLEAR);

        // Giving the slave address to the peripheral
        self.registers
            .cr2
            .modify(CR2::SADD.val((self.slave_address.get() << 1) as u32));

        // Direction bit set based on the passed direction
        self.registers.cr2.modify(match direction {
            I2cDirection::To => CR2::RD_WRN::CLEAR,
            I2cDirection::From => CR2::RD_WRN::SET,
        });

        // Setting the first NBYTES and the reload flag
        self.update_nbytes();

        // Starting the DMA transfer
        self.start_dma(data, saved_len.get())
    }
}

impl<'a> I2CMaster<'a> for I2c<'a> {
    fn set_master_client(&self, client: &'a dyn I2CHwMasterClient) {
        self.master_client.replace(client);
    }

    fn enable(&self) {
        self.enable();
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
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.status.get() == I2cStatus::Idle {
            self.reset();
            self.status.set(I2cStatus::WritingReading);

            self.write_len.set(write_len);
            self.read_len.set(read_len);

            self.start_transfer(addr, data, I2cDirection::To)?;

            self.registers.cr2.modify(CR2::START::SET);

            Ok(())
        } else {
            Err((Error::Busy, data))
        }
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.status.get() == I2cStatus::Idle {
            self.reset();
            self.status.set(I2cStatus::Writing);

            self.write_len.set(len);

            self.start_transfer(addr, data, I2cDirection::To)?;

            self.registers.cr2.modify(CR2::START::SET);

            Ok(())
        } else {
            Err((Error::Busy, data))
        }
    }

    fn read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.status.get() == I2cStatus::Idle {
            self.reset();
            self.status.set(I2cStatus::Reading);

            self.read_len.set(len);

            self.start_transfer(addr, data, I2cDirection::From)?;

            self.registers.cr2.modify(CR2::START::SET);

            Ok(())
        } else {
            Err((Error::Busy, data))
        }
    }
}

impl dma::DmaClient for I2c<'_> {
    fn transfer_done(&self, channel: ChannelId) {
        self.handle_dma_interrupt(channel);
    }
}
