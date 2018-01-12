//! Registers of the SAM4L's USB controller

#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use usbc::data::{Mode, Speed};

// Base address of USBC registers.  See "7.1 Product Mapping"
const USBC_BASE: u32 = 0x400A5000;

registers![USBC_BASE, {
    0x0000 => { "Device General Control Register", UDCON, "RW" },
    0x0004 => { "Device Global Interrupt Register", UDINT, "R" },

    0x0008 => { "Device Global Interrupt Clear Register", UDINTCLR, "W" },
    0x000C => { "Device Global Interrupt Set Register", UDINTSET, "W" },
    0x0010 => { "Device Global Interrupt Enable Register", UDINTE, "R" },
    0x0014 => { "Device Global Interrupt Enable Clear Register", UDINTECLR, "W" },
    0x0018 => { "Device Global Interrupt Enable Set Register", UDINTESET, "W" },
    0x001C => { "Endpoint Enable/Reset Register", UERST, "RW" },
    0x0020 => { "Device Frame Number Register", UDFNUM, "R" },

    0x0100 => { "DEBUG UECFG0", UECFG0, "RW" },
    0x01C0 => { "DEBUG UECON0", UECON0, "R" },
    0x01F0 => { "DEBUG UECON0SET", UECON0SET, "W" },

    0x0100 => { "Endpoint n Configuration Register", UECFGn, "RW", 8 },
    0x0130 => { "Endpoint n Status Register", UESTAn, "R", 8 },
    0x0160 => { "Endpoint n Status Clear Register", UESTAnCLR, "W", 8 },
    0x0190 => { "Endpoint n Status Set Register", UESTAnSET, "W", 8 },
    0x01C0 => { "Endpoint n Control Register", UECONn, "R", 8 },
    0x01F0 => { "Endpoint n Control Set Register", UECONnSET, "W", 8 },
    0x0220 => { "Endpoint n Control Clear Register", UECONnCLR, "W", 8 },

    0x0400 => { "Host General Control Register", UHCON, "RW" },
    0x0404 => { "Host Global Interrupt Register", UHINT, "R" },
    0x0408 => { "Host Global Interrupt Clear Register", UHINTCLR, "W" },
    0x040C => { "Host Global Interrupt Set Register", UHINTSET, "W" },
    0x0410 => { "Host Global Interrupt Enable Register", UHINTE, "R" },
    0x0414 => { "Host Global Interrupt Enable Clear Register", UHINTECLR, "W" },
    0x0418 => { "Host Global Interrupt Enable Set Register", UHINTESET, "W" },
    0x041C => { "Pipe Enable/Reset Register", UPRST, "RW" },
    0x0420 => { "Host Frame Number Register", UHFNUM, "RW" },
    0x0424 => { "Host Start Of Frame Control Register", UHSOFC, "RW" },

    0x0500 => { "Pipe n Configuration Register", UPCFGn, "RW", 8 },
    0x0530 => { "Pipe n Status Register", UPSTAn, "R", 8 },
    0x0560 => { "Pipe n Status Clear Register", UPSTAnCLR, "W", 8 },
    0x0590 => { "Pipe n Status Set Register", UPSTAnSET, "W", 8 },
    0x05C0 => { "Pipe n Control Register", UPCONn, "R", 8 },
    0x05F0 => { "Pipe n Control Set Register", UPCONnSET, "W", 8 },
    0x0620 => { "Pipe n Control Clear Register", UPCONnCLR, "W", 8 },
    0x0650 => { "Pipe n IN Request Register", UPINRQn, "RW", 8 },

    0x0800 => { "General Control Register", USBCON, "RW" },
    0x0804 => { "General Status Register", USBSTA, "R" },
    0x0808 => { "General Status Clear Register", USBSTACLR, "W" },
    0x080C => { "General Status Set Register", USBSTASET, "W" },
    0x0818 => { "IP Version Register", UVERS, "R" },
    0x081C => { "IP Features Register", UFEATURES, "R" },
    0x0820 => { "IP PB Address Size Register", UADDRSIZE, "R" },
    0x0824 => { "IP Name Register 1", UNAME1, "R" },
    0x0828 => { "IP Name Register 2", UNAME2, "R" },
    0x082C => { "USB Finite State Machine Status Register", USBFSM, "R" },
    0x0830 => { "USB Descriptor address", UDESC, "RW" }
}];

// work around rustfmt a bit here, which likes to remove the semicolons and
// then fail syntax parsing....
// these macros will be replaced soon with the new bitfield interface, so we'll
// be okay with this being a bit silly-looking short term
#[rustfmt_skip]
bitfield![USBCON, USBCON_UIMOD, "RW", Mode, 25, 1]; // sheet says bit 25, but maybe it's 24?
#[rustfmt_skip]
bitfield![USBCON, USBCON_USBE, "RW", bool, 15, 1];
#[rustfmt_skip]
bitfield![USBCON, USBCON_FRZCLK, "RW", bool, 14, 1];

#[rustfmt_skip]
bitfield![UDCON, UDCON_DETACH, "RW", bool, 8, 1];
#[rustfmt_skip]
bitfield![UDCON, UDCON_LS, "RW", Speed, 12, 1];
#[rustfmt_skip]
bitfield![UDCON, UDCON_UADD, "RW", u8, 0, 0b1111111];
#[rustfmt_skip]
bitfield![UDCON, UDCON_ADDEN, "RW", bool, 7, 1];

#[rustfmt_skip]
bitfield![USBSTA, USBSTA_CLKUSABLE, "R", bool, 14, 1];

// Bitfields for UDINT, UDINTCLR, UDINTESET
pub const UDINT_SUSP: u32 = 1 << 0;
pub const UDINT_SOF: u32 = 1 << 2;
pub const UDINT_EORST: u32 = 1 << 3;
pub const UDINT_WAKEUP: u32 = 1 << 4;
pub const UDINT_EORSM: u32 = 1 << 5;
pub const UDINT_UPRSM: u32 = 1 << 6;

// Bitfields for UECONnSET, UESTAn, UESTAnCLR
pub const TXIN: u32 = 1 << 0;
pub const RXOUT: u32 = 1 << 1;
pub const RXSTP: u32 = 1 << 2;
pub const ERRORF: u32 = 1 << 2;
pub const NAKOUT: u32 = 1 << 3;
pub const NAKIN: u32 = 1 << 4;
pub const STALLED: u32 = 1 << 6;
pub const CRCERR: u32 = 1 << 6;
pub const RAMACERR: u32 = 1 << 11;
pub const STALLRQ: u32 = 1 << 19;

// Bitfields for UESTAn
pub const CTRLDIR: u32 = 1 << 17;
