// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use crate::dma::{ChannelId, Dma};
use core::cell::Cell;
use core::cmp;
use cortexm33::dma_fence::CortexMDmaFence;
use kernel::hil::gpio::Output;
use kernel::hil::spi::{self, ClockPhase, ClockPolarity};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

register_structs! {
    pub SpiRegisters {
        //Control register 1
        (0x000 => cr1: ReadWrite<u32, CR1::Register>),
        //Control register 2
        (0x004 => cr2: ReadWrite<u32, CR2::Register>),
        //Configuration register 1
        (0x008 => cfg1: ReadWrite<u32, CFG1::Register>),
        //Configuration register 2
        (0x00C => cfg2: ReadWrite<u32, CFG2::Register>),
        //Interrupt enable register
        (0x010 => ier: ReadWrite<u32, IER::Register>),
        //Status register
        (0x014 => sr: ReadOnly<u32, SR::Register>),
        //Interrupt/status flags clear register
        (0x018 => ifcr: WriteOnly<u32, IFCR::Register>),
        //Autonomous mode control register
        (0x01C => autocr: ReadWrite<u32, AUTOCR::Register>),
        //Transmit data register
        (0x020 => txdr: WriteOnly<u32, TXDR::Register>),
        (0x024 => _reserved0),
        //Receive data register
        (0x030 => rxdr: ReadOnly<u32, RXDR::Register>),
        (0x034 => _reserved1),
        //Polynomial register
        (0x040 => crcpoly: ReadWrite<u32, CRCPOLY::Register>),
        //Transmitter CRC register
        (0x044 => txcrc: ReadOnly<u32, TXCRC::Register>),
        //Receiver CRC register
        (0x048 => rxcrc: ReadOnly<u32, RXCRC::Register>),
        //Underrun data register
        (0x04C => udrdr: ReadWrite<u32, UDRDR::Register>),
        (0x050 => @END),
    }
}

// Base addresses for SPI1, 2 and 3 in Secure Alias mode
pub const SPI1_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x5001_3000 as *const SpiRegisters) };
// pub const SPI2_BASE: StaticRef<SpiRegisters> =
//     unsafe { StaticRef::new(0x5000_3800 as *const SpiRegisters) }; //not used
// pub const SPI3_BASE: StaticRef<SpiRegisters> =
//     unsafe { StaticRef::new(0x5600_2000 as *const SpiRegisters) }; //not used

register_bitfields![u32,
    // Control Register 1
    pub CR1 [
        /// Locking the AF configuration of associated I/Os
        /// 0: AF configuration is not locked
        /// 1: AF configuration is locked
        IOLOCK OFFSET(16) NUMBITS(1) [],

        /// CRC calculation initialization pattern control for transmitter
        /// 0: all zero pattern is applied
        /// 1: all ones pattern is applied
        TCRCINI OFFSET(15) NUMBITS(1) [],

        /// CRC calculation initialization pattern control for receiver
        /// 0: All zero pattern is applied
        /// 1: All ones pattern is applied
        RCRCINI OFFSET(14) NUMBITS(1) [],

        /// Full size (33-bit or 17-bit) CRC polynomial configuration
        /// 0: Full size (33-bit or 17-bit) CRC polynomial is not used
        /// 1: Full size (33-bit or 17-bit) CRC polynomial is used
        CRC33_17 OFFSET(13) NUMBITS(1) [],

        /// Internal slave select signal input level
        /// This bit has an effect only when the SSM bit is set. The value of this bit
        /// is forced onto the peripheral NSS input internally.
        SSI OFFSET(12) NUMBITS(1) [],

        /// Rx/Tx direction at half-duplex mode
        /// 0: SPI is receiver
        /// 1: SPI is transmitter
        HDDIR OFFSET(11) NUMBITS(1) [],

        /// Master suspend request
        /// In master mode, when this bit is set by software, the CSTART bit is reset
        /// at the end of the current frame and communication is suspended.
        CSUSP OFFSET(10) NUMBITS(1) [],

        /// Master transfer start
        /// 0: master transfer is at idle
        /// 1: master transfer is ongoing or temporary suspended by automatic suspend
        CSTART OFFSET(9) NUMBITS(1) [],

        /// Master automatic suspension in Receive mode
        /// 0: SPI flow/clock generation is continuous, regardless of overrun condition
        /// 1: SPI flow is suspended temporary on RxFIFO full condition
        MASRX OFFSET(8) NUMBITS(1) [],

        /// Serial peripheral enable
        /// 0: Serial peripheral disabled
        /// 1: Serial peripheral enabled
        SPE OFFSET(0) NUMBITS(1) []
    ],

    // Control Register 2
    pub CR2 [
        ///Number of data at current transfer
        TSIZE OFFSET(0)  NUMBITS(16) []
    ],

    // SPI configuration register 1
    pub CFG1 [
        /// Bypass of the prescaler at master baud rate clock generator
        /// 0: bypass is disabled
        /// 1: bypass is enabled
        BPASS OFFSET(31) NUMBITS(1) [],

        /// Master baud rate prescaler setting
        /// 000: SPI master clock/2
        /// 001: SPI master clock/4
        /// ...
        /// 111: SPI master clock/256
        MBR OFFSET(28) NUMBITS(3) [
            Div2 = 0b000,
            Div4 = 0b001,
            Div8 = 0b010,
            Div16 = 0b011,
            Div32 = 0b100,
            Div64 = 0b101,
            Div128 = 0b110,
            Div256 = 0b111
        ],

        /// Hardware CRC computation enable
        /// 0: CRC calculation disabled
        /// 1: CRC calculation enabled
        CRCEN OFFSET(22) NUMBITS(1) [],

        /// Length of CRC frame to be transferred and compared
        CRCSIZE OFFSET(16) NUMBITS(5) [],

        /// Tx DMA stream enable
        /// 0: Tx DMA disabled
        /// 1: Tx DMA enabled
        TXDMAEN OFFSET(15) NUMBITS(1) [],

        /// Rx DMA stream enable
        /// 0: Rx-DMA disabled
        /// 1: Rx-DMA enabled
        RXDMAEN OFFSET(14) NUMBITS(1) [],

        /// Behavior of slave transmitter at underrun condition
        /// 0: slave sends a constant pattern defined by the user in the SPI_UDRDR register
        /// 1: Slave repeats the last received data from master.
        UDRCFG OFFSET(9) NUMBITS(1) [],

        /// FIFO threshold level
        /// Defines number of data frames in a single data packet.
        FTHLV OFFSET(5) NUMBITS(4) [],

        /// Number of bits in a single SPI data frame
        DSIZE OFFSET(0) NUMBITS(5) []
    ],

    // SPI configuration register 2
    pub CFG2 [
        /// Alternate function GPIOs control
        /// This bit is taken into account when SPE = 0 only.
        /// 0: The peripheral takes no control of GPIOs while it is disabled
        /// 1: The peripheral keeps always control of all associated GPIOs
        AFCNTR OFFSET(31) NUMBITS(1) [],

        /// NSS output management in master mode
        /// 0: NSS is kept at active level until data transfer is complete, it becomes inactive with EOT flag
        /// 1: SPI data frames are interleaved with NSS nonactive pulses when MIDI[3:0] > 1
        SSOM OFFSET(30) NUMBITS(1) [],

        /// NSS output enable
        /// This bit is taken into account in master mode only
        /// 0: NSS output is disabled and the SPI can work in multimaster configuration
        /// 1: NSS output is enabled. The SPI cannot work in a multimaster environment.
        SSOE OFFSET(29) NUMBITS(1) [],

        /// NSS input/output polarity
        /// 0: low level is active for NSS signal
        /// 1: high level is active for NSS signal
        SSIOP OFFSET(28) NUMBITS(1) [],

        /// Software management of internal slave select signal input
        /// 0: Input value of the internal slave select signal is determined by the external NSS hardware pin
        /// 1: Input value of the internal slave select signal is determined by the SSI bit controlled by software
        SSM OFFSET(26) NUMBITS(1) [],

        // Clock polarity
        // 0: SCK signal is at 0 when idle
        // 1: SCK signal is at 1 when idle
        CPOL OFFSET(25) NUMBITS(1) [],

        /// Clock Phase
        /// 0: the first clock transition is the first data capture edge
        /// 1: the second clock transition is the first data capture edge
        CPHA OFFSET(24) NUMBITS(1) [],

        /// data frame format
        /// 0: MSB transmitted first
        /// 1: LSB transmitted first
        LSBFRST OFFSET(23) NUMBITS(1) [],

        /// SPI master
        /// 0: SPI slave
        /// 1: SPI master
        MASTER OFFSET(22) NUMBITS(1) [],

        /// Serial protocol
        /// 000: SPI Motorola
        /// 001: SPI TI
        SP OFFSET(19) NUMBITS(3) [
            Motorola = 0b000,
            Ti = 0b001
        ],

        /// SPI Communication Mode
        /// 00: full-duplex
        /// 01: simplex transmitter
        /// 10: simplex receiver
        /// 11: half-duplex
        COMM OFFSET(17) NUMBITS(2) [
            FullDuplex = 0b00,
            SimplexTx = 0b01,
            SimplexRx = 0b10,
            HalfDuplex = 0b11
        ],

        /// Swap functionality of MISO and MOSI pins
        /// 0: no swap
        /// 1: MOSI and MISO are swapped
        IOSWP OFFSET(15) NUMBITS(1) [],

        /// RDY signal input/output polarity
        /// 0: high level of the signal means the slave is ready for communication
        /// 1: low level of the signal means the slave is ready for communication
        RDIOP OFFSET(14) NUMBITS(1) [],

        /// RDY signal input/output management
        /// 0: RDY signal is defined internally fixed as permanently active (RDIOP setting has no effect)
        /// 1: RDY signal is overtaken from alternate function input (at master case) or output (at slave case)
        RDIOM OFFSET(13) NUMBITS(1) [],

        /// Master Inter-Data Idleness
        /// Specifies minimum time delay (expressed in SPI clock cycles periods) inserted between two
        /// consecutive data frames in master mode.
        MIDI OFFSET(4) NUMBITS(4) [],

        /// Master NSS Idleness
        /// Specifies an extra delay, expressed in number of SPI clock cycle periods, inserted
        /// additionally between active edge of NSS opening a session and the beginning of the first
        /// data frame of the session in master mode when SSOE is enabled.
        MSSI OFFSET(0) NUMBITS(4) []
    ],

    /// SPI interrupt enable register
    pub IER [
        /// Mode Fault interrupt enable
        MODFIE OFFSET(9) NUMBITS(1) [],

        /// TIFRE interrupt enable
        TIFREIE OFFSET(8) NUMBITS(1) [],

        /// CRC error interrupt enable
        CRCEIE OFFSET(7) NUMBITS(1) [],

        /// OVR interrupt enable
        OVRIE OFFSET(6) NUMBITS(1) [],

        /// UDR interrupt enable
        UDRIE OFFSET(5) NUMBITS(1) [],

        /// TXTF interrupt enable
        TXTFIE OFFSET(4) NUMBITS(1) [],

        /// EOT, SUSP and TXC interrupt enable
        EOTIE OFFSET(3) NUMBITS(1) [],

        /// DXP interrupt enabled
        DXPIE OFFSET(2) NUMBITS(1) [],

        /// TXP interrupt enabled
        TXPIE OFFSET(1) NUMBITS(1) [],

        /// RXP interrupt enabled
        RXPIE OFFSET(0) NUMBITS(1) []
    ],

    /// SPI status register
    pub SR [
        /// number of data frames remaining in current TSIZE session
        CTSIZE OFFSET(16) NUMBITS(16) [],

        /// RxFIFO word not empty
        RXWNE OFFSET(15) NUMBITS(1) [],

        /// RxFIFO packing level
        RXPLVL OFFSET(13) NUMBITS(2) [],

        /// TxFIFO transmission complete
        TXC OFFSET(12) NUMBITS(1) [],

        /// Suspension status
        SUSP OFFSET(11) NUMBITS(1) [],

        /// Mode fault
        MODF OFFSET(9) NUMBITS(1) [],

        /// TI frame format error
        TIFRE OFFSET(8) NUMBITS(1) [],

        /// CRC error
        CRCE OFFSET(7) NUMBITS(1) [],

        /// Overrun
        OVR OFFSET(6) NUMBITS(1) [],

        /// Underrun
        UDR OFFSET(5) NUMBITS(1) [],

        /// Transmission transfer filled
        TXTF OFFSET(4) NUMBITS(1) [],

        /// End of transfer
        EOT OFFSET(3) NUMBITS(1) [],

        /// Duplex packet
        DXP OFFSET(2) NUMBITS(1) [],

        /// Tx-packet space available
        TXP OFFSET(1) NUMBITS(1) [],

        /// RXP: Rx-packet available
        RXP OFFSET(0) NUMBITS(1) []
    ],

    /// SPI interrupt/status flags clear register
    pub IFCR [
        /// Suspend flag clear
        SUSPC OFFSET(11) NUMBITS(1) [],
        /// Mode fault flag clear
        MODFC OFFSET(9) NUMBITS(1) [],
        /// TI frame format error flag clear
        TIFREC OFFSET(8) NUMBITS(1) [],
        /// CRC error flag clear
        CRCEC OFFSET(7) NUMBITS(1) [],
        /// Overrun flag clear
        OVRC OFFSET(6) NUMBITS(1) [],
        /// Underrun flag clear
        UDRC OFFSET(5) NUMBITS(1) [],
        /// Transmission transfer filled flag clear
        TXTFC OFFSET(4) NUMBITS(1) [],
        /// End of transfer flag clear
        EOTC OFFSET(3) NUMBITS(1) []
    ],

    /// SPI autonomous mode control register
    pub AUTOCR [
        /// Hardware control of CSTART triggering enable
        TRIGEN OFFSET(21) NUMBITS(1) [],
        /// Trigger polarity
        TRIGPOL OFFSET(20) NUMBITS(1) [],
        /// Trigger selection
        TRIGSEL OFFSET(16) NUMBITS(4) []
    ],

    /// SPI transmit data register
    pub TXDR[
        /// Transmit data register
        TXDR OFFSET(0) NUMBITS(32) []
    ],

    /// SPI receive data register
    pub RXDR [
        // Receive data register
        RXDR OFFSET(0) NUMBITS(32) []
    ],

    /// SPI polynomial register
    pub CRCPOLY [
        /// CRC polynomial register
        CRCPOLY OFFSET(0) NUMBITS(32) []
    ],

    /// SPI transmitter CRC register
    pub TXCRC [
        /// CRC register for transmitter
        TXCRC OFFSET(0) NUMBITS(32) []
    ],

    /// SPI receiver CRC register
    pub RXCRC [
        /// CRC register for receiver
        RXCRC OFFSET(0) NUMBITS(32) []
    ],

    /// SPI underrun data register
    pub UDRDR [
        /// Data at slave underrun condition
        UDRDR OFFSET(0) NUMBITS(32) []
    ]
];

pub struct Spi<'a> {
    pub registers: StaticRef<SpiRegisters>,
    client: OptionalCell<&'a dyn spi::SpiMasterClient>,
    dma: OptionalCell<&'a Dma>,
    dma_channel_tx: Cell<Option<ChannelId>>,
    dma_channel_rx: Cell<Option<ChannelId>>,
    // tx_dma_peripheral: Cell<Option<DmaPeripheral>>,
    // rx_dma_peripheral: Cell<Option<DmaPeripheral>>,
    tx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    rx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    dma_len: Cell<usize>,
    transfers_in_progress: Cell<u8>,
    active_slave: OptionalCell<&'a crate::gpio::Pin<'a>>,
    active_after: Cell<bool>,
}

impl<'a> Spi<'a> {
    pub fn new(base: StaticRef<SpiRegisters>) -> Self {
        Self {
            registers: base,
            client: OptionalCell::empty(),
            dma: OptionalCell::empty(),
            dma_channel_tx: Cell::new(None),
            dma_channel_rx: Cell::new(None),
            // tx_dma_peripheral: Cell::new(None),
            // rx_dma_peripheral: Cell::new(None),
            tx_dma_buf: MapCell::empty(),
            rx_dma_buf: MapCell::empty(),
            dma_len: Cell::new(0),
            transfers_in_progress: Cell::new(0),
            active_slave: OptionalCell::empty(),
            active_after: Cell::new(false),
        }
    }

    /// associates a DMA controller and channels with the SPI driver.
    pub fn set_dma(
        spi: &'static Self,
        dma: &'a Dma,
        // tx_peripheral: DmaPeripheral,
        tx_channel: ChannelId,
        // rx_peripheral: DmaPeripheral,
        rx_channel: ChannelId,
    ) {
        spi.dma.set(dma);
        // spi.tx_dma_peripheral.set(Some(tx_peripheral));
        spi.dma_channel_tx.set(Some(tx_channel));
        // spi.rx_dma_peripheral.set(Some(rx_peripheral));
        spi.dma_channel_rx.set(Some(rx_channel));
        // dma.set_client(tx_channel, spi);
        // dma.set_client(rx_channel, spi);
    }

    // pub fn handle_dma_interrupt(&self, is_tx: bool) {
    //     if is_tx {
    //         self.dma.map(|dma| {
    //             if let Some(ch) = self.dma_channel_tx.get() {
    //                 dma.clear_interrupt(ch);
    //             }
    //         });
    //         self.registers.cr3.modify(CR3::DMAT::CLEAR);
    //         self.tx_deferred.set(false);
    //         if let Some(dma_slice) = self.tx_dma_buf.take() {
    //             let fence = unsafe { CortexMDmaFence::new() };
    //             let mut subslice = unsafe { dma_slice.take(fence) };
    //             subslice.reset();
    //             let buf = subslice.take();
    //             let len = self.tx_len.get();
    //             self.tx_client.map(move |client| {
    //                 client.transmitted_buffer(buf, len, Ok(()));
    //             });
    //         }
    //     } else {
    //         self.dma.map(|dma| {
    //             if let Some(ch) = self.dma_channel_rx.get() {
    //                 dma.clear_interrupt(ch);
    //             }
    //         });
    //         self.registers.cr3.modify(CR3::DMAR::CLEAR);
    //         self.rx_deferred.set(false);
    //         if let Some(dma_slice) = self.rx_dma_buf.take() {
    //             let fence = unsafe { CortexMDmaFence::new() };
    //             let mut subslice = unsafe { dma_slice.take(fence) };
    //             subslice.reset();
    //             let buf = subslice.take();
    //             let len = self.rx_len.get();
    //             self.rx_client.map(move |client| {
    //                 client.received_buffer(buf, len, Ok(()), uart::Error::None);
    //             });
    //         }
    //     }
    // }

    pub fn handle_interrupt(&self) {
        // Used only during debugging. Since we use DMA, we do not enable SPI
        // interrupts during normal operations
    }

    fn enable_tx(&self) {
        self.registers.cfg1.modify(CFG1::TXDMAEN::SET);
    }

    // fn disable_tx(&self) {
    //     self.registers.cfg1.modify(CFG1::TXDMAEN::CLEAR);
    // }

    fn enable_rx(&self) {
        self.registers.cfg1.modify(CFG1::RXDMAEN::SET);
    }

    // fn disable_rx(&self) {
    //     self.registers.cfg1.modify(CFG1::RXDMAEN::CLEAR);
    // }
}

impl<'a> spi::SpiMaster<'a> for Spi<'a> {
    type ChipSelect = &'a crate::gpio::Pin<'a>;

    fn set_phase(&self, phase: ClockPhase) -> Result<(), kernel::ErrorCode> {
        let regs = &*self.registers;

        match phase {
            ClockPhase::SampleLeading => {
                regs.cfg2.modify(CFG2::CPHA::CLEAR);
            }
            ClockPhase::SampleTrailing => {
                regs.cfg2.modify(CFG2::CPHA::SET);
            }
        }
        Ok(())
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), kernel::ErrorCode> {
        let regs = &*self.registers;

        match polarity {
            ClockPolarity::IdleLow => {
                regs.cfg2.modify(CFG2::CPOL::CLEAR);
            }
            ClockPolarity::IdleHigh => {
                regs.cfg2.modify(CFG2::CPOL::SET);
            }
        }

        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, kernel::ErrorCode> {
        let regs = &*self.registers;
        let spi_clock_freq: u32 = 4_000_000;

        let divider = spi_clock_freq / rate;

        let (mbr_val, actual_rate) = if divider <= 2 {
            (CFG1::MBR::Div2, spi_clock_freq / 2)
        } else if divider <= 4 {
            (CFG1::MBR::Div4, spi_clock_freq / 4)
        } else if divider <= 8 {
            (CFG1::MBR::Div8, spi_clock_freq / 8)
        } else if divider <= 16 {
            (CFG1::MBR::Div16, spi_clock_freq / 16)
        } else if divider <= 32 {
            (CFG1::MBR::Div32, spi_clock_freq / 32)
        } else if divider <= 64 {
            (CFG1::MBR::Div64, spi_clock_freq / 64)
        } else if divider <= 128 {
            (CFG1::MBR::Div128, spi_clock_freq / 128)
        } else {
            (CFG1::MBR::Div256, spi_clock_freq / 256)
        };

        if regs.cr1.is_set(CR1::SPE) {
            regs.cr1.modify(CR1::SPE::CLEAR);
        }

        self.registers.cfg1.modify(mbr_val);

        if regs.cr1.is_set(CR1::SPE) {
            regs.cr1.modify(CR1::SPE::SET);
        }

        Ok(actual_rate)
    }

    fn get_rate(&self) -> u32 {
        let spi_clock_freq: u32 = 4_000_000;

        match self.registers.cfg1.read(CFG1::MBR) {
            0b000 => spi_clock_freq / 2,
            0b001 => spi_clock_freq / 4,
            0b010 => spi_clock_freq / 8,
            0b011 => spi_clock_freq / 16,
            0b100 => spi_clock_freq / 32,
            0b101 => spi_clock_freq / 64,
            0b110 => spi_clock_freq / 128,
            _ => spi_clock_freq / 256,
        }
    }

    fn get_phase(&self) -> ClockPhase {
        let regs = &*self.registers;

        if !regs.cfg2.is_set(CFG2::CPHA) {
            ClockPhase::SampleLeading
        } else {
            ClockPhase::SampleTrailing
        }
    }

    fn get_polarity(&self) -> ClockPolarity {
        let regs = &*self.registers;

        if !regs.cfg2.is_set(CFG2::CPOL) {
            ClockPolarity::IdleLow
        } else {
            ClockPolarity::IdleHigh
        }
    }

    fn is_busy(&self) -> bool {
        let regs = &*self.registers;

        regs.cr1.is_set(CR1::CSTART) || !regs.sr.is_set(SR::TXC)
    }

    fn hold_low(&self) {
        self.active_after.set(true);
    }

    fn release_low(&self) {
        self.active_after.set(false);
    }

    fn init(&self) -> Result<(), kernel::ErrorCode> {
        let regs = &*self.registers;

        //need to disable spi in order to change config
        regs.cr1.modify(CR1::SPE::CLEAR);

        //set baudrate
        regs.cfg1.modify(CFG1::MBR::Div2);

        //sets spi in master mode, full-duplex, with slave-select managed by software
        regs.cfg2
            .modify(CFG2::MASTER::SET + CFG2::SSM::SET + CFG2::COMM::FullDuplex);

        regs.cr1.modify(CR1::SSI::SET);

        // clear any pending mode faults
        regs.ifcr.write(IFCR::MODFC::SET);

        //enable spi
        regs.cr1.modify(CR1::SPE::SET);
        Ok(())
    }

    fn set_client(&self, client: &'a dyn spi::SpiMasterClient) {
        self.client.set(client);
    }

    fn read_byte(&self) -> Result<u8, kernel::ErrorCode> {
        self.read_write_byte(0)
    }

    // checked with logic analyzer and works
    fn write_byte(&self, val: u8) -> Result<(), kernel::ErrorCode> {
        let regs = &*self.registers;

        // wait until the FIFO has space for at least one packet
        while !regs.sr.is_set(SR::TXP) {}

        // write byte into TXDR
        regs.txdr.write(TXDR::TXDR.val(val as u32));

        // start transfer
        regs.cr1.modify(CR1::CSTART::SET);

        //wait for transfer
        while !regs.sr.is_set(SR::TXC) {}

        Ok(())
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), kernel::ErrorCode> {
        self.active_slave.set(cs);
        Ok(())
    }

    // have to check with logic analyzer
    fn read_write_byte(&self, val: u8) -> Result<u8, kernel::ErrorCode> {
        let regs = &*self.registers;

        // set the transfer size
        regs.cr2.modify(CR2::TSIZE.val(1));

        // start the transfer
        regs.cr1.modify(CR1::CSTART::SET);

        // wait until the tx fifo actually has space then write the byte
        while !regs.sr.is_set(SR::TXP) {}
        regs.txdr.write(TXDR::TXDR.val(val as u32));

        // wait for the incoming byte to arrive in the rx fifo
        while !regs.sr.is_set(SR::RXP) {}

        // retrieve the byte
        let byte = regs.rxdr.get() as u8;

        // clear the completion flags
        regs.ifcr.write(IFCR::EOTC::SET + IFCR::TXTFC::SET);

        // wait for the transfer to finish
        while !regs.sr.is_set(SR::EOT) {}

        Ok(byte)
    }

    // have to check with logic analyzer
    fn read_write_bytes(
        &self,
        mut write_buffer: kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>,
        read_buffer: Option<kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>,
            Option<kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>>,
        ),
    > {
        let regs = &*self.registers;

        // check if there is another transaction pending
        if self.is_busy() {
            return Err((kernel::ErrorCode::BUSY, write_buffer, read_buffer));
        }

        // verify if the dma is associated with this spi instance
        if self.dma.is_none() {
            return Err((kernel::ErrorCode::OFF, write_buffer, read_buffer));
        }

        self.active_slave.map(|p| {
            p.clear();
        });

        // we default to the len of the write buf but we pick the minimum of write/ read buf
        let mut count: usize = write_buffer.len();
        read_buffer
            .as_ref()
            .map(|buf| count = cmp::min(count, buf.len()));

        self.dma_len.set(count);
        // send the transfer size to hardware
        regs.cr2.modify(CR2::TSIZE.val(count as u32));
        self.transfers_in_progress.set(0);

        // rx transfer
        read_buffer.map(|mut rx_buffer| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);

            rx_buffer.slice(0..count);
            let fence = unsafe { CortexMDmaFence::new() };
            let dma_slice = DmaSubSliceMut::new_static(rx_buffer, fence);

            let ptr = dma_slice.as_mut_ptr() as u32;
            let len = dma_slice.len() as u32;

            self.rx_dma_buf.replace(dma_slice);

            self.dma.map(move |dma| {
                if let Some(ch) = self.dma_channel_rx.get() {
                    dma.setup(ch, crate::dma::DmaPeripheral::Spi1Rx, ptr, len);
                    self.enable_rx();
                }
            });
        });

        // tx transfer
        self.transfers_in_progress
            .set(self.transfers_in_progress.get() + 1);

        write_buffer.slice(0..count);
        let fence = unsafe { CortexMDmaFence::new() };
        let dma_slice = DmaSubSliceMut::new_static(write_buffer, fence);

        let ptr = dma_slice.as_mut_ptr() as u32;
        let len = dma_slice.len() as u32;

        self.tx_dma_buf.replace(dma_slice);

        self.dma.map(move |dma| {
            if let Some(ch) = self.dma_channel_tx.get() {
                dma.setup(ch, crate::dma::DmaPeripheral::Spi1Tx, ptr, len);
                self.enable_tx();
            }
        });

        //start the transfer
        regs.cr1.modify(CR1::CSTART::SET);

        Ok(())
    }
}

// impl crate::dma::DmaClient for Spi<'_> {
//     fn transfer_done(&self, channel: ChannelId) {
//         if let Some(tx_ch) = self.dma_channel_tx.get() {
//             if channel == tx_ch {
//                 self.handle_dma_interrupt(true);
//                 return;
//             }
//         }
//         if let Some(rx_ch) = self.dma_channel_rx.get() {
//             if channel == rx_ch {
//                 self.handle_dma_interrupt(false);
//             }
//         }
//     }
// }

impl crate::dma::DmaClient for Spi<'_> {
    fn transfer_done(&self, channel: ChannelId) {
        let regs = &*self.registers;

        if let Some(tx_ch) = self.dma_channel_tx.get() {
            if channel == tx_ch {
                regs.cfg1.modify(CFG1::TXDMAEN::CLEAR);
                self.dma.map(|dma| dma.clear_interrupt(tx_ch));
                self.transfers_in_progress
                    .set(self.transfers_in_progress.get() - 1);
            }
        }

        if let Some(rx_ch) = self.dma_channel_rx.get() {
            if channel == rx_ch {
                regs.cfg1.modify(CFG1::RXDMAEN::CLEAR);
                self.dma.map(|dma| dma.clear_interrupt(rx_ch));
                self.transfers_in_progress
                    .set(self.transfers_in_progress.get() - 1);
            }
        }

        if self.transfers_in_progress.get() == 0 {
            // clear flags
            regs.ifcr
                .write(IFCR::EOTC::SET + IFCR::TXTFC::SET + IFCR::OVRC::SET);

            if !self.active_after.get() {
                self.active_slave.map(|p| {
                    p.set();
                });
            }

            let tx_buffer = self.tx_dma_buf.take().map(|dma_slice| {
                let fence = unsafe { CortexMDmaFence::new() };
                let mut subslice = unsafe { dma_slice.take(fence) };
                subslice.reset();
                subslice
            });

            let rx_buffer = self.rx_dma_buf.take().map(|dma_slice| {
                let fence = unsafe { CortexMDmaFence::new() };
                let mut subslice = unsafe { dma_slice.take(fence) };
                subslice.reset();
                subslice
            });

            let length = self.dma_len.get();
            self.dma_len.set(0);

            self.client.map(|client| {
                tx_buffer.map(|t| {
                    client.read_write_done(t, rx_buffer, Ok(length));
                });
            });
        }
    }
}
