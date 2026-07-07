// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use core::cell::Cell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, Field, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

/// Base address for USART1 in Secure Alias mode.
const USART1_BASE_ADDR: u32 = 0x5001_3800;
/// USART1 Receive Data Register (RDR) address.
const USART1_RDR: u32 = USART1_BASE_ADDR + 0x24;
/// USART1 Transmit Data Register (TDR) address.
const USART1_TDR: u32 = USART1_BASE_ADDR + 0x28;

/// Base address for SPI1 in Secure Alias mode.
const SPI1_BASE_ADDR: u32 = 0x5001_3000;
const SPI1_RXDR: u32 = SPI1_BASE_ADDR + 0x030;
const SPI1_TXDR: u32 = SPI1_BASE_ADDR + 0x020;

/// GPDMA Request Selection IDs (REQSEL)
/// Found in the GPDMA request multiplexer table of the STM32U5 reference manual.
const GPDMA_REQ_USART1_RX: u32 = 24;
const GPDMA_REQ_USART1_TX: u32 = 25;
const GPDMA_REQ_SPI1_RX: u32 = 6;
const GPDMA_REQ_SPI1_TX: u32 = 7;

register_bitfields! [
    u32,
    pub DmaChannelTR1 [
        /// Destination security
        DSEC OFFSET(31) NUMBITS(1) [],
        /// Destination allocated port
        DAP OFFSET(30) NUMBITS(1) [],
        /// Destination increment
        DINC OFFSET(19) NUMBITS(1) [],
        /// Source security
        SSEC OFFSET(15) NUMBITS(1) [],
        /// Source allocated port
        SAP OFFSET(14) NUMBITS(1) [],
        /// Source increment
        SINC OFFSET(3) NUMBITS(1) [],
    ],
    pub DmaChannelTR2 [
        /// Destination request
        DREQ OFFSET(10) NUMBITS(1) [],
        /// Request selection
        REQSEL OFFSET(0) NUMBITS(7) [],
    ],
    pub DmaChannelBR1 [
        /// Block number of data bytes to transfer
        BNDT OFFSET(0) NUMBITS(16) []
    ],
    pub DmaChannelSAR [
        /// Source address
        SAR OFFSET(0) NUMBITS(32) []
    ],
    pub DmaChannelDAR [
        /// Destination address
        DAR OFFSET(0) NUMBITS(32) []
    ],
    pub DmaChannelCR [
        /// Transfer complete interrupt enable
        TCIE OFFSET(8) NUMBITS(1) [],
        /// Enable
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub DmaChannelFCR [
        /// Completed suspension flag clear
        SUSPF OFFSET(13) NUMBITS(1) [],
        /// User setting error flag clear
        USEF OFFSET(12) NUMBITS(1) [],
        /// Update link error flag clear
        ULEF OFFSET(11) NUMBITS(1) [],
        /// Data transfer error flag clear
        DTEF OFFSET(10) NUMBITS(1) [],
        /// Half transfer flag clear
        HTF OFFSET(9) NUMBITS(1) [],
        /// Transfer complete flag clear
        TCF OFFSET(8) NUMBITS(1) [],
    ],
    pub DmaChannelEnable [
        CH0  OFFSET(0)  NUMBITS(1) [],
        CH1  OFFSET(1)  NUMBITS(1) [],
        CH2  OFFSET(2)  NUMBITS(1) [],
        CH3  OFFSET(3)  NUMBITS(1) [],
        CH4  OFFSET(4)  NUMBITS(1) [],
        CH5  OFFSET(5)  NUMBITS(1) [],
        CH6  OFFSET(6)  NUMBITS(1) [],
        CH7  OFFSET(7)  NUMBITS(1) [],
        CH8  OFFSET(8)  NUMBITS(1) [],
        CH9  OFFSET(9)  NUMBITS(1) [],
        CH10 OFFSET(10) NUMBITS(1) [],
        CH11 OFFSET(11) NUMBITS(1) [],
        CH12 OFFSET(12) NUMBITS(1) [],
        CH13 OFFSET(13) NUMBITS(1) [],
        CH14 OFFSET(14) NUMBITS(1) [],
        CH15 OFFSET(15) NUMBITS(1) [],
    ]
];

const CH_FIELDS: [Field<u32, DmaChannelEnable::Register>; 16] = [
    DmaChannelEnable::CH0,
    DmaChannelEnable::CH1,
    DmaChannelEnable::CH2,
    DmaChannelEnable::CH3,
    DmaChannelEnable::CH4,
    DmaChannelEnable::CH5,
    DmaChannelEnable::CH6,
    DmaChannelEnable::CH7,
    DmaChannelEnable::CH8,
    DmaChannelEnable::CH9,
    DmaChannelEnable::CH10,
    DmaChannelEnable::CH11,
    DmaChannelEnable::CH12,
    DmaChannelEnable::CH13,
    DmaChannelEnable::CH14,
    DmaChannelEnable::CH15,
];

register_structs! {
    pub DmaChannelRegisters {
        /// Channel x linked-list base address register (Relative 0x00)
        (0x000 => pub l_bar: ReadWrite<u32>),
        /// Channel x flag clear register (Relative 0x04)
        (0x004 => _reserved0: [u32; 2]),
        (0x00C => pub f_cr: ReadWrite<u32, DmaChannelFCR::Register>),
        /// Channel x status register (Relative 0x08)
        (0x010 => pub s_r: ReadOnly<u32>),
        /// Channel x control register (Relative 0x0C)
        (0x014 => pub c_r: ReadWrite<u32, DmaChannelCR::Register>),
        (0x018 => _reserved1: [u32; 10]),
        /// Channel x transfer register 1 (Relative 0x40)
        (0x040 => pub t_r1: ReadWrite<u32, DmaChannelTR1::Register>),
        /// Channel x transfer register 2 (Relative 0x44)
        (0x044 => pub t_r2: ReadWrite<u32, DmaChannelTR2::Register>),
        /// Channel x block register 1 (Relative 0x48)
        (0x048 => pub b_r1: ReadWrite<u32, DmaChannelBR1::Register>),
        /// Channel x source address register (Relative 0x4C)
        (0x04C => pub s_ar: ReadWrite<u32, DmaChannelSAR::Register>),
        /// Channel x destination address register (Relative 0x50)
        (0x050 => pub d_ar: ReadWrite<u32, DmaChannelDAR::Register>),
        (0x054 => _reserved2: [u32; 10]),
        /// Channel x linked-list address register (Relative 0x7C)
        (0x07C => pub l_lr: ReadWrite<u32>),
        (0x080 => @END),
    }
}

register_structs! {
    pub DmaRegisters {
        /// GPDMA secure configuration register (0x00)
        (0x000 => pub seccfgr: ReadWrite<u32, DmaChannelEnable::Register>),
        /// GPDMA privileged configuration register (0x04)
        (0x004 => pub privcfgr: ReadWrite<u32, DmaChannelEnable::Register>),
        (0x008 => _reserved0: [u32; 1]),
        /// Masked interrupt status register (0x0C)
        (0x00C => pub misr: ReadOnly<u32>),
        (0x010 => pub smisr: ReadOnly<u32>),
        (0x014 => _reserved1: [u32; 15]),
        /// Channels starting at 0x50. Each channel is 0x80 bytes long.
        (0x050 => pub channels: [DmaChannelRegisters; 16]),
        (0x850 => @END),
    }
}

/// Base address for DMA1 in Secure Alias mode.
pub const DMA1_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x50020000 as *const DmaRegisters) };

pub trait DmaClient {
    fn transfer_done(&self, channel: ChannelId);
}

#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ChannelId {
    Channel00 = 0,
    Channel01 = 1,
    Channel02 = 2,
    Channel03 = 3,
    Channel04 = 4,
    Channel05 = 5,
    Channel06 = 6,
    Channel07 = 7,
    Channel08 = 8,
    Channel09 = 9,
    Channel10 = 10,
    Channel11 = 11,
    Channel12 = 12,
    Channel13 = 13,
    Channel14 = 14,
    Channel15 = 15,
}

impl From<ChannelId> for usize {
    fn from(val: ChannelId) -> usize {
        val as usize
    }
}

pub enum DmaDirection {
    MemoryToPeripheral,
    PeripheralToMemory,
}

#[derive(Copy, Clone)]
pub enum DmaPeripheral {
    Usart1Tx,
    Usart1Rx,
    Spi1Tx,
    Spi1Rx,
}

impl DmaPeripheral {
    fn get_params(&self) -> (u32, u32, DmaDirection) {
        match self {
            DmaPeripheral::Usart1Tx => (
                USART1_TDR,
                GPDMA_REQ_USART1_TX,
                DmaDirection::MemoryToPeripheral,
            ),
            DmaPeripheral::Usart1Rx => (
                USART1_RDR,
                GPDMA_REQ_USART1_RX,
                DmaDirection::PeripheralToMemory,
            ),
            DmaPeripheral::Spi1Tx => (
                SPI1_TXDR,
                GPDMA_REQ_SPI1_TX,
                DmaDirection::MemoryToPeripheral,
            ),
            DmaPeripheral::Spi1Rx => (
                SPI1_RXDR,
                GPDMA_REQ_SPI1_RX,
                DmaDirection::PeripheralToMemory,
            ),
        }
    }
}

pub struct ChannelDma {
    pub channel: ChannelId,
    pub in_use: Cell<bool>,
    pub client: OptionalCell<&'static dyn DmaClient>,
}

impl ChannelDma {
    pub const fn new(id: ChannelId) -> Self {
        Self {
            channel: id,
            in_use: Cell::new(false),
            client: OptionalCell::empty(),
        }
    }
}

pub struct Dma {
    pub registers: StaticRef<DmaRegisters>,
    channels: [ChannelDma; 16],
}

impl Dma {
    pub const fn new(base: StaticRef<DmaRegisters>) -> Self {
        Self {
            registers: base,
            channels: [
                ChannelDma::new(ChannelId::Channel00),
                ChannelDma::new(ChannelId::Channel01),
                ChannelDma::new(ChannelId::Channel02),
                ChannelDma::new(ChannelId::Channel03),
                ChannelDma::new(ChannelId::Channel04),
                ChannelDma::new(ChannelId::Channel05),
                ChannelDma::new(ChannelId::Channel06),
                ChannelDma::new(ChannelId::Channel07),
                ChannelDma::new(ChannelId::Channel08),
                ChannelDma::new(ChannelId::Channel09),
                ChannelDma::new(ChannelId::Channel10),
                ChannelDma::new(ChannelId::Channel11),
                ChannelDma::new(ChannelId::Channel12),
                ChannelDma::new(ChannelId::Channel13),
                ChannelDma::new(ChannelId::Channel14),
                ChannelDma::new(ChannelId::Channel15),
            ],
        }
    }

    pub fn setup(
        &self,
        channel: ChannelId,
        peripheral: DmaPeripheral,
        buffer_addr: u32,
        length: u32,
    ) {
        let channel_id: usize = channel.into();
        let (periph_addr, reqsel, direction) = peripheral.get_params();

        // 1. Mark channel as Secure AND Privileged
        self.registers.seccfgr.modify(CH_FIELDS[channel_id].val(1));
        self.registers.privcfgr.modify(CH_FIELDS[channel_id].val(1));

        let ch = &self.registers.channels[channel_id];

        // 2. Ensure channel is disabled
        ch.c_r.write(DmaChannelCR::EN::CLEAR);

        // 3. Clear all flags
        ch.f_cr.write(
            DmaChannelFCR::SUSPF::SET
                + DmaChannelFCR::USEF::SET
                + DmaChannelFCR::ULEF::SET
                + DmaChannelFCR::DTEF::SET
                + DmaChannelFCR::HTF::SET
                + DmaChannelFCR::TCF::SET,
        );

        // 4. Configure TR1, TR2 and addresses based on direction
        match direction {
            DmaDirection::MemoryToPeripheral => {
                // Source is memory (incrementing), Destination is peripheral (fixed)
                ch.t_r1.write(
                    DmaChannelTR1::SINC::SET
                        + DmaChannelTR1::SAP::CLEAR
                        + DmaChannelTR1::DAP::CLEAR,
                );
                // Source request comes from destination peripheral
                ch.t_r2
                    .write(DmaChannelTR2::REQSEL.val(reqsel) + DmaChannelTR2::DREQ::SET);
                ch.s_ar.write(DmaChannelSAR::SAR.val(buffer_addr));
                ch.d_ar.write(DmaChannelDAR::DAR.val(periph_addr));
            }
            DmaDirection::PeripheralToMemory => {
                // Destination is memory (incrementing), Source is peripheral (fixed)
                // Note: Keeping security bits as in previous RX implementation
                ch.t_r1.write(
                    DmaChannelTR1::DINC::SET + DmaChannelTR1::SSEC::SET + DmaChannelTR1::DSEC::SET,
                );
                // Source request comes from source peripheral
                ch.t_r2.write(DmaChannelTR2::REQSEL.val(reqsel));
                ch.s_ar.write(DmaChannelSAR::SAR.val(periph_addr));
                ch.d_ar.write(DmaChannelDAR::DAR.val(buffer_addr));
            }
        }

        // 5. Set Block Register 1 (BR1)
        ch.b_r1.write(DmaChannelBR1::BNDT.val(length & 0xFFFF));

        // 6. Enable Transfer Complete Interrupt and start the channel
        ch.c_r
            .write(DmaChannelCR::TCIE::SET + DmaChannelCR::EN::SET);
    }

    pub fn clear_interrupt(&self, channel: ChannelId) {
        // `channel_id` is a `usize` converted from `ChannelId`
        // which is an enum that can only take values between 0 and 15.
        let channel_id: usize = channel.into();
        let ch = &self.registers.channels[channel_id];
        ch.f_cr.write(
            DmaChannelFCR::SUSPF::SET
                + DmaChannelFCR::USEF::SET
                + DmaChannelFCR::ULEF::SET
                + DmaChannelFCR::DTEF::SET
                + DmaChannelFCR::HTF::SET
                + DmaChannelFCR::TCF::SET,
        );
    }

    pub fn request_channel(&self) -> Option<ChannelId> {
        for channel in &self.channels {
            if !channel.in_use.get() {
                channel.in_use.set(true);
                return Some(channel.channel);
            }
        }
        None
    }

    pub fn release_channel(&self, id: ChannelId) {
        // `channel_id` is a `usize` converted from `ChannelId`
        // which is an enum that can only take values between 0 and 15.
        let index: usize = id.into();
        if self.channels[index].in_use.get() {
            self.channels[index].in_use.set(false);
            self.channels[index].client.clear();
        }
    }

    pub fn set_client(&self, id: ChannelId, client: &'static dyn DmaClient) {
        // `channel_id` is a `usize` converted from `ChannelId`
        // which is an enum that can only take values between 0 and 15.
        let index: usize = id.into();
        self.channels[index].client.set(client);
    }

    pub fn handle_interrupt(&self, id: ChannelId) {
        self.clear_interrupt(id);
        // `channel_id` is a `usize` converted from `ChannelId`
        // which is an enum that can only take values between 0 and 15.
        let index: usize = id.into();
        self.channels[index].client.map(|client| {
            client.transfer_done(id);
        });
    }
}
