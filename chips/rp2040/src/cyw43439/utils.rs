// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::slice;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::utilities::registers::{register_bitfields, FieldValue, LocalRegisterCopy};

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum ChannelType {
    Control = 0,
    Event = 1,
    Data = 2
}
}

pub(super) fn as_bytes<T: Sized>(data: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            core::ptr::from_ref::<T>(data) as *const u8,
            core::mem::size_of::<T>(),
        )
    }
}

pub(super) fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

pub(crate) const WL_SCAN_ACTION_ABORT: u16 = 0x3;
pub(crate) const WL_SCAN_ACTION_START: u16 = 0x1;

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
pub(crate) const IRQ_DATA_UNAVAILABLE: u16 = 0x0001;
pub(crate) const IRQ_F2_PACKET_AVAILABLE: u16 = 0x0020;

pub(crate) const SPI_F2_WATERMARK: u32 = 0x20;
pub(crate) const BACKPLANE_ADDRESS_MASK: u32 = 0x7FFF;
pub(crate) const BACKPLANE_WINDOW_SIZE: u32 = BACKPLANE_ADDRESS_MASK + 1;
pub(crate) const BACKPLANE_MAX_TRANSFER_SIZE: usize = 64;

pub(crate) const SDIOD_CORE_BASE_ADDRESS: u32 = 0x18002000;
pub(crate) const I_HMB_SW_MASK: u32 = 0x24;
pub(crate) const SDIO_INT_HOST_MASK: u32 = 0x000000f0;

pub(crate) const STATUS_F2_RX_READY: u32 = 0x20;

pub(crate) const ATCM_RAM_BASE_ADDRESS: u32 = 0;
pub(crate) const RAM_SIZE: u32 = 512 * 1024;

pub(crate) const CONFIG_DATA: u32 = 0x000300B1;
pub(crate) const INTR_STATUS_RESET: u32 = 0x99;
pub(crate) const INTR_ENABLE_RESET: u32 = 0xBE;

pub(crate) const REG_BACKPLANE_FUNCTION2_WATERMARK: u32 = 0x10008;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_LOW: u32 = 0x1000A;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_MID: u32 = 0x1000B;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH: u32 = 0x1000C;
pub(crate) const REG_BACKPLANE_CHIP_CLOCK_CSR: u32 = 0x1000E;
pub(crate) const REG_BACKPLANE_PULL_UP: u32 = 0x1000F;

// AMBA Interconnect bus
pub(crate) const AI_IOCTRL_OFFSET: u32 = 0x408;
pub(crate) const AI_IOCTRL_BIT_FGC: u8 = 0x0002;
pub(crate) const AI_IOCTRL_BIT_CLOCK_EN: u8 = 0x0001;

pub(crate) const AI_RESETCTRL_OFFSET: u32 = 0x800;
pub(crate) const AI_RESETCTRL_BIT_RESET: u8 = 1;

// Backplane ALP clock
pub(crate) const BACKPLANE_ALP_AVAIL_REQ: u8 = 0x08;
pub(crate) const BACKPLANE_ALP_AVAIL: u8 = 0x40;

pub(super) type Function = CYW43_CMD::FUNCTION_NUM::Value;
pub(super) type Access = CYW43_CMD::ACCESS::Value;
pub(super) type Command = CYW43_CMD::COMMAND::Value;

register_bitfields![u32,
    pub(super) CYW43_CMD [
        COMMAND OFFSET(31) NUMBITS(1) [
            Read = 0,
            Write = 1,
        ],
        ACCESS OFFSET(30) NUMBITS(1) [
            FixedAddr = 0,
            IncAddr = 1,
        ],
        FUNCTION_NUM OFFSET(28) NUMBITS(2) [
            Spi = 0b00,
            Backplane = 0b01,
            Wlan = 0b10,
        ],
        ADDRESS OFFSET(11) NUMBITS(17) [],
        LENGTH OFFSET(0) NUMBITS(11) [],
    ]
];

pub(super) struct Cyw43Cmd(LocalRegisterCopy<u32, CYW43_CMD::Register>);

impl Cyw43Cmd {
    pub(super) fn new(
        command: CYW43_CMD::COMMAND::Value,
        access: CYW43_CMD::ACCESS::Value,
        function_no: CYW43_CMD::FUNCTION_NUM::Value,
        address: u32,
        length: u32,
    ) -> Self {
        let mut local_reg = LocalRegisterCopy::<u32, CYW43_CMD::Register>::new(0u32);

        local_reg.modify(
            FieldValue::from(command)
                + access.into()
                + function_no.into()
                + CYW43_CMD::ADDRESS.val(address)
                + CYW43_CMD::LENGTH.val(length),
        );

        Self(local_reg)
    }

    pub(super) fn get(&self) -> u32 {
        self.0.get()
    }
}

pub static NVRAM: &[u8] = b"
    NVRAMRev=$Rev$\x00\
    manfid=0x2d0\x00\
    prodid=0x0727\x00\
    vendid=0x14e4\x00\
    devid=0x43e2\x00\
    boardtype=0x0887\x00\
    boardrev=0x1100\x00\
    boardnum=22\x00\
    macaddr=00:A0:50:b5:59:5e\x00\
    sromrev=11\x00\
    boardflags=0x00404001\x00\
    boardflags3=0x04000000\x00\
    xtalfreq=37400\x00\
    nocrc=1\x00\
    ag0=255\x00\
    aa2g=1\x00\
    ccode=ALL\x00\
    pa0itssit=0x20\x00\
    extpagain2g=0\x00\
    pa2ga0=-168,6649,-778\x00\
    AvVmid_c0=0x0,0xc8\x00\
    cckpwroffset0=5\x00\
    maxp2ga0=84\x00\
    txpwrbckof=6\x00\
    cckbw202gpo=0\x00\
    legofdmbw202gpo=0x66111111\x00\
    mcsbw202gpo=0x77711111\x00\
    propbw202gpo=0xdd\x00\
    ofdmdigfilttype=18\x00\
    ofdmdigfilttypebe=18\x00\
    papdmode=1\x00\
    papdvalidtest=1\x00\
    pacalidx2g=45\x00\
    papdepsoffset=-30\x00\
    papdendidx=58\x00\
    ltecxmux=0\x00\
    ltecxpadnum=0x0102\x00\
    ltecxfnsel=0x44\x00\
    ltecxgcigpio=0x01\x00\
    il0macaddr=00:90:4c:c5:12:38\x00\
    wl0id=0x431b\x00\
    deadman_to=0xffffffff\x00\
    muxenab=0x100\x00\
    spurconfig=0x3\x00\
    glitch_based_crsmin=1\x00\
    btc_mode=1\x00\
    \x00";
