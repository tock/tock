// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub DmaChannelRegisters {
        /// Channel x linked-list base address register (Relative 0x00)
        (0x000 => pub lBAR: ReadWrite<u32>),
        /// Channel x flag clear register (Relative 0x04)
        (0x004 => _reserved0: [u32; 2]),
        (0x00C => pub fCR: ReadWrite<u32>),
        /// Channel x status register (Relative 0x08)
        (0x010 => pub sR: ReadOnly<u32>),
        /// Channel x control register (Relative 0x0C)
        (0x014 => pub cR: ReadWrite<u32>),
        (0x018 => _reserved1: [u32; 10]),
        /// Channel x transfer register 1 (Relative 0x40)
        (0x040 => pub tR1: ReadWrite<u32>),
        /// Channel x transfer register 2 (Relative 0x44)
        (0x044 => pub tR2: ReadWrite<u32>),
        /// Channel x block register 1 (Relative 0x48)
        (0x048 => pub bR1: ReadWrite<u32>),
        /// Channel x source address register (Relative 0x4C)
        (0x04C => pub sAR: ReadWrite<u32>),
        /// Channel x destination address register (Relative 0x50)
        (0x050 => pub dAR: ReadWrite<u32>),
        (0x054 => _reserved2: [u32; 10]),
        /// Channel x linked-list address register (Relative 0x7C)
        (0x07C => pub lLR: ReadWrite<u32>),
        (0x080 => @END),
    }
}

register_structs! {
    pub DmaRegisters {
        /// GPDMA secure configuration register (0x00)
        (0x000 => pub seccfgr: ReadWrite<u32>),
        /// GPDMA privileged configuration register (0x04)
        (0x004 => pub privcfgr: ReadWrite<u32>),
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

pub struct Dma<'a> {
    registers: StaticRef<DmaRegisters>,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> Dma<'a> {
    pub const fn new(base: StaticRef<DmaRegisters>) -> Self {
        Self {
            registers: base,
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn setup_usart1_tx(&self, channel: usize, buffer_addr: u32, length: u32) {
        if channel >= 16 { return; }
        
        // 1. Mark channel as Secure AND Privileged
        let sec = self.registers.seccfgr.get();
        self.registers.seccfgr.set(sec | (1 << channel));
        let priv_reg = self.registers.privcfgr.get();
        self.registers.privcfgr.set(priv_reg | (1 << channel));

        let ch = &self.registers.channels[channel];

        // 2. Ensure channel is disabled
        ch.cR.set(0);

        // 3. Clear all flags
        ch.fCR.set(0x0000FFFF);

        // 4. Configure Transfer Register 1 (TR1)
        // SINC (bit 3) = 1
        // SAP (bit 14) = 0 (Port 0)
        // DAP (bit 30) = 0 (Port 0 - Safer for U545)
        ch.tR1.set(1 << 3);

        // 5. Configure Transfer Register 2 (TR2)
        // REQSEL = 25 (USART1_TX on U545), DREQ = 1 (Destination request)
        ch.tR2.set(25 | (1 << 10));

        // 6. Set Addresses
        ch.sAR.set(buffer_addr);
        ch.dAR.set(0x50013828); // USART1_TDR Secure Address

        // 7. Set Block Register 1 (BR1)
        ch.bR1.set(length & 0xFFFF);

        // 8. Enable Transfer Complete Interrupt (bit 8) and Start (bit 0)
        ch.cR.set((1 << 8) | 1); 
    }

    pub fn setup_usart1_rx(&self, channel: usize, buffer_addr: u32, length: u32) {
        if channel >= 16 { return; }
        
        // Mark channel as Secure AND Privileged
        let sec = self.registers.seccfgr.get();
        self.registers.seccfgr.set(sec | (1 << channel));
        let priv_reg = self.registers.privcfgr.get();
        self.registers.privcfgr.set(priv_reg | (1 << channel));

        let ch = &self.registers.channels[channel];

        ch.cR.set(0);
        ch.fCR.set(0x0000FFFF);

        // Configure TR1 (Security + Direction)
        // DINC (19), SSEC (15), DSEC (31)1
        ch.tR1.set((1 << 19) | (1 << 15) | (1 << 31));

        // Configure TR2 (Trigger Source) - REQSEL = 24
        ch.tR2.set(24);

        // 6. Set Addresses
        ch.sAR.set(0x50013824);
        ch.dAR.set(buffer_addr);

        // 7. Set Block Register 1 (BR1)
        ch.bR1.set(length & 0xFFFF);

        // 8. Enable
        ch.cR.set((1 << 8) | 1); 
    }

    pub fn clear_interrupt(&self, channel: usize) {
        if channel >= 16 { return; }
        let ch = &self.registers.channels[channel];
        ch.fCR.set(0x0000FFFF);
    }
}
