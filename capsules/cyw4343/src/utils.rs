// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

// SPI registers
pub(crate) const REG_BUS_CTRL: u32 = 0x0;
pub(crate) const REG_BUS_INTERRUPT: u32 = 0x04; // 16 bits - Interrupt status
pub(crate) const REG_BUS_INTERRUPT_ENABLE: u32 = 0x06; // 16 bits - Interrupt mask
pub(crate) const REG_BUS_STATUS: u32 = 0x8;
pub(crate) const REG_BUS_TEST_RO: u32 = 0x14;
pub(crate) const REG_BUS_TEST_RW: u32 = 0x18;
pub(crate) const STATUS_F2_PKT_AVAILABLE: u32 = 0x00000100;
pub(crate) const STATUS_F2_PKT_LEN_MASK: u32 = 0x000FFE00;
pub(crate) const STATUS_F2_PKT_LEN_SHIFT: u32 = 9;
pub(crate) const IRQ_F2_PACKET_AVAILABLE: u32 = 0x0020;

pub(crate) const SPI_F2_WATERMARK: u32 = 0x20;
pub(crate) const BACKPLANE_ADDRESS_MASK: u32 = 0x7FFF;
pub(crate) const BACKPLANE_WINDOW_SIZE: u32 = BACKPLANE_ADDRESS_MASK + 1;

pub(crate) const SDIOD_CORE_BASE_ADDRESS: u32 = 0x18002000;
pub(crate) const I_HMB_SW_MASK: u32 = 0x24;
pub(crate) const SDIO_INT_HOST_MASK: u32 = 0x000000f0;

pub(crate) const STATUS_F2_RX_READY: u32 = 0x20;

pub(crate) const RAM_BASE_ADDR: u32 = 0;
pub(crate) const RAM_SIZE: u32 = 512 * 1024;

pub(crate) const CONFIG_DATA: u32 = 0x000304B1;

pub(crate) const INTR_STATUS_RESET: u32 = 0x99;
pub(crate) const INTR_ENABLE_RESET: u32 = 0xBE;

pub(crate) const REG_BACKPLANE_FUNCTION2_WATERMARK: u32 = 0x10008;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_LOW: u32 = 0x1000A;

pub(crate) const REG_BACKPLANE_CHIP_CLOCK_CSR: u32 = 0x1000E;
pub(crate) const REG_BACKPLANE_PULL_UP: u32 = 0x1000F;

// AMBA Interconnect bus
pub(crate) const AI_IOCTRL_OFFSET: u32 = 0x408;
pub(crate) const AI_IOCTRL_BIT_FGC: u32 = 0x0002;
pub(crate) const AI_IOCTRL_BIT_CLOCK_EN: u32 = 0x0001;

pub(crate) const AI_RESETCTRL_OFFSET: u32 = 0x0800;
pub(crate) const AI_RESETCTRL_BIT_RESET: u32 = 0x01;

// Backplane ALP clock
pub(crate) const BACKPLANE_ALP_AVAIL_REQ: u8 = 0x08;
pub(crate) const BACKPLANE_ALP_AVAIL: u32 = 0x40;

pub(crate) const SOCRAM_CORE_BASE_ADDR: u32 = 0x18104000;
pub(crate) const WLAN_ARM_CORE_BASE_ADDR: u32 = 0x18103000;

pub(crate) const WL_SCAN_ACTION_ABORT: u16 = 0x3;
pub(crate) const WL_SCAN_ACTION_START: u16 = 0x1;

// Other consts
pub(crate) const BDC_PADDING_SIZE: usize = 2;
pub(crate) const BDC_VERSION: u8 = 2;
pub(crate) const BDC_VERSION_SHIFT: u8 = 4;

pub(crate) const WSEC_AES: u32 = 0x4;
pub(crate) const SCANTYPE_PASSIVE: u8 = 1;

/// Broadcom Ethertype for identifying event packets
pub(crate) const ETHER_TYPE_BRCM: u16 = 0x886c;

/// Broadcom OUI (Organizationally Unique Identifier): Used in the proprietary(221) IE (Information Element) in all Broadcom devices
pub(crate) const BRCM_OUI: [u8; 3] = [0x00, 0x10, 0x18];

/// Event subtype (vendor specific)
pub(crate) const EVT_SUBTYPE: u16 = 32769;

pub(crate) const CLM_CHUNK_SIZE: usize = 1024;
pub(crate) const CLM_DOWNLOAD_FLAG_HANDLER_VER: u16 = 0x1000;
pub(crate) const CLM_DOWNLOAD_FLAG_BEGIN: u16 = 0x2;
pub(crate) const CLM_DOWNLOAD_FLAG_END: u16 = 0x4;
pub(crate) const CLM_DOWNLOAD_TYPE: u16 = 0x2;

pub(crate) const MAX_SPI_BP_CHUNK_SIZE: usize = 64;
pub(crate) const NVRAM_END: u32 = RAM_BASE_ADDR + RAM_SIZE - 4;

#[macro_export]
macro_rules! backplane_window_bits {
    ($addr:expr) => {
        ($addr & !$crate::utils::BACKPLANE_ADDRESS_MASK) >> 8
    };
}

#[macro_export]
macro_rules! reset_and_restore_bufs {
    ($self: ident, $($buf:ident),*) => {{
        $($buf.reset();)*
        $($self.$buf.set($buf);)*
    }}
}
