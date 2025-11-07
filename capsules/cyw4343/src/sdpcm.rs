// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! CYW4343x SDPCM protocol headers and packet types

use core::ffi::CStr;

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

use capsules_extra::wifi;

macro_rules! parse {
    (
        $(#[$attr_struct:meta])* $vis_struct:vis struct $name:ident { $($(#[$attr_field:meta])* $vis_field:vis $field:ident : $field_ty:tt),* $(,)? }
        ) => {
        $(#[$attr_struct])*
        $vis_struct struct $name {
            $($(#[$attr_field])* $vis_field $field : $field_ty),*,
        }
        impl $name {
            #![allow(unused)]
            pub const SIZE: usize = core::mem::size_of::<Self>();
            pub const fn into_bytes(self) -> [u8; Self::SIZE] {
                let mut __bytes = [0u8; Self::SIZE];
                let mut __len = 0;
                $(
                    parse!(@f __len, __bytes, self.$field, $field_ty);
                )*
                __bytes
            }
            pub const fn from_bytes(__bytes: &[u8]) -> Self {
                let mut __len = 0;
                $(
                    parse!(@from_f __len, __bytes, $field, $field_ty);
                )*
                Self {
                    $($field),*
                }
            }
        }
    };

    // Inner macros for copying the bytes from the buffer into a field.
    (@from_f $len: ident, $bytes:ident, $field:ident, u8) => {
        let $field = $bytes[$len];
        $len += 1;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, u16) => {
        let $field = u16::from_le_bytes([$bytes[$len], $bytes[$len + 1]]);
        $len += 2;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, u32) => {
        let $field = u32::from_le_bytes([$bytes[$len], $bytes[$len + 1], $bytes[$len + 2], $bytes[$len + 3]]);
        $len += 4;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, i32) => {
        let $field = i32::from_le_bytes([$bytes[$len], $bytes[$len + 1], $bytes[$len + 2], $bytes[$len + 3]]);
        $len += 4;
    };
    (@from_f $len: ident, $bytes:ident, $field:ident, [u8; $N:literal]) => {
        let mut $field = [0u8; $N];
        let mut __idx = 0;
        while __idx < $N {
            $field[__idx] = $bytes[$len];
            __idx += 1;
            $len += 1;
        }
    };

    // Inner macros for copying the field value to the bytes buffer.
    (@f $len:ident, $bytes:ident, $field:expr, u8) => {
        $bytes[$len] = $field;
        $len += 1;
    };
    (@f $len:ident, $bytes: ident, $field: expr, u16) => {
        let __field_le_bytes = $field.to_le_bytes();
        $bytes[$len] = __field_le_bytes[0];
        $bytes[$len + 1] = __field_le_bytes[1];
        $len += 2;
    };
    (@f $len:ident, $bytes: ident, $field: expr, i32) => {
        let __field_le_bytes = $field.to_le_bytes();
        $bytes[$len] = __field_le_bytes[0];
        $bytes[$len + 1] = __field_le_bytes[1];
        $bytes[$len + 2] = __field_le_bytes[2];
        $bytes[$len + 3] = __field_le_bytes[3];
        $len += 4;
    };
    (@f $len:ident, $bytes: ident, $field: expr, u32) => {
        let __field_le_bytes = $field.to_le_bytes();
        $bytes[$len] = __field_le_bytes[0];
        $bytes[$len + 1] = __field_le_bytes[1];
        $bytes[$len + 2] = __field_le_bytes[2];
        $bytes[$len + 3] = __field_le_bytes[3];
        $len += 4;
    };
    (@f $len:ident, $bytes:ident, $field:expr, [u8; $N:literal]) => {
        let mut __idx = 0;
        while __idx < $N {
            $bytes[$len] = $field[__idx];
            $len += 1;
            __idx += 1;
        }
    };
}

enum_from_primitive! {
    #[derive(Debug, Clone, Copy)]
    pub enum ChannelType {
        Control = 0,
        Event = 1,
        Data = 2
    }
}

parse!(
    /// SDPCM header
    #[derive(Clone, Debug)]
    pub struct SdpcmHeader {
        pub len: u16,
        pub len_inv: u16,
        pub seq: u8,
        pub flags: u8,
        pub next_len: u8,
        pub data_offset: u8,
        pub flow_ctrl: u8,
        pub data_credit: u8,
        pub reserved: u16,
    }
);

parse!(
    #[derive(Clone, Copy)]
    pub struct ScanResults {
        pub buflen: u32,
        pub version: u32,
        pub sync_id: u16,
        pub bss_count: u16,
    }
);

parse!(
    /// BDC (bulk data communication) header
    #[derive(Clone, Debug)]
    pub struct BdcHeader {
        pub flags: u8,
        pub priority: u8,
        pub flags2: u8,
        pub data_offset: u8,
    }
);

parse!(
    /// CDC (control data communication) header (for IOCTL packets)
    #[derive(Debug, Clone)]
    pub struct CdcHeader {
        pub cmd: u32,
        pub len: u32,
        pub flags: u32,
        pub status: u32,
    }
);

/// Two types of IOCTL operations: get and set
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum IoctlType {
    Get = 0,
    Set = 2,
}

/// IOCTL commands
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum IoctlCommand {
    Up = 2,
    Down = 3,
    SetInfra = 20,
    SetAuth = 22,
    SetSsid = 26,
    SetChannel = 30,
    Disassoc = 52,
    SetAntdiv = 64,
    SetGmode = 110,
    SetAp = 118,
    SetWsec = 134,
    SetBand = 142,
    SetWpaAuth = 165,
    GetVar = 262,
    SetVar = 263,
    SetWsecPmk = 268,
}

parse!(
    pub struct ScanParams {
        pub version: u32,
        pub action: u16,
        pub sync_id: u16,
        pub ssid_len: u32,
        pub ssid: [u8; 32],
        pub bssid: [u8; 6],
        pub bss_type: u8,
        pub scan_type: u8,
        pub nprobes: u32,
        pub active_time: u32,
        pub passive_time: u32,
        pub home_time: u32,
        pub channel_num: u32,
        pub channel_list: u16,
    }
);

parse!(
    pub struct PassphraseInfo {
        pub len: u16,
        pub flags: u16,
        pub buf: [u8; 64],
    }
);

impl PassphraseInfo {
    pub(super) fn from_wpa1_to_bytes(value: wifi::WpaPassphrase) -> [u8; Self::SIZE] {
        let mut passphrase_info = [0u8; PassphraseInfo::SIZE];
        passphrase_info[0..2].copy_from_slice(&(value.len as u16).to_le_bytes());
        passphrase_info[2..4].copy_from_slice(&1u16.to_le_bytes());
        passphrase_info[4..][..64].copy_from_slice(&value.buf[..64]);
        passphrase_info
    }

    pub(super) fn from_wpa3_to_bytes(value: wifi::Wpa3Passphrase) -> [u8; Self::SIZE] {
        let mut passphrase_info = [0u8; PassphraseInfo::SIZE];
        let len = u8::min(value.len, 64);
        passphrase_info[0..2].copy_from_slice(&(len as u16).to_le_bytes());
        passphrase_info[2..4].copy_from_slice(&1u16.to_le_bytes());
        passphrase_info[4..][..64].copy_from_slice(&value.buf[..64]);
        passphrase_info
    }
}

parse!(
    pub struct SaePassphraseInfo {
        pub len: u16,
        pub buf: [u8; 128],
    }
);

impl From<wifi::Wpa3Passphrase> for SaePassphraseInfo {
    fn from(value: wifi::Wpa3Passphrase) -> Self {
        Self {
            len: value.len as _,
            buf: value.buf,
        }
    }
}

enum_from_primitive!(
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Event {
        SetSsid = 0,
        Join = 1,
        Start = 2,
        EscanResult = 69,
        Radio = 40,
        If = 54,
        ProbreqMsg = 44,
        ProbreqMsgRx = 137,
        ProbrespMsg = 71,
        Roam = 19,
    }
);

parse!(
    pub struct EventMask {
        iface: u32,
        evts: [u8; 24],
    }
);

impl EventMask {
    pub const fn with_masked_evts(masked_evts: &[Event]) -> Self {
        let mut mask = Self {
            iface: 0,
            evts: [0xff; 24],
        };
        let mut idx = 0;
        while idx < masked_evts.len() {
            let evt = masked_evts[idx];
            mask.evts[evt as usize / 8] &= !(1 << (evt as usize % 8));
            idx += 1;
        }
        mask
    }
}

enum_from_primitive!(
    #[derive(Debug)]
    #[repr(u16)]
    pub enum DloadType {
        Ucode = 0b0,
        Clm = 0b1,
    }
);

parse!(
    pub struct WlDloadData {
        pub flag: u16,
        pub dload_type: u16,
        pub len: u32,
        pub crc: u32,
    }
);

parse!(
    #[derive(Debug)]
    pub struct EthernetHeader {
        pub destination_address: [u8; 6],
        pub source_address: [u8; 6],
        pub ethertype: u16,
    }
);

parse!(
    pub struct WlEscanResultHeader {
        pub buflen: u32,
        pub version: u32,
        pub sync_id: u16,
        pub bss_count: u16,
    }
);

parse!(
    #[derive(Default)]
    pub struct BssInfo {
        version: u32,
        length: u32,
        bssid: [u8; 6],
        beacon_period: u16,
        capability: u16,
        pub ssid_len: u8,
        pub ssid: [u8; 32],
        reserved1: [u8; 1],
        rateset_count: u32,
        rates: [u8; 16],
        chanspec: u16,
        atim_window: u16,
        dtim_period: u8,
        reserved2: [u8; 1],
        rssi: u16,
        phy_noise: u8,
        n_cap: u8,
        reserved3: [u8; 2],
        nbss_cap: u32,
        ctl_ch: u8,
        reserved4: [u8; 3],
        reserved32: [u8; 4],
        flags: u8,
        vht_cap: u8,
        reserved5: [u8; 2],
        basic_mcs: [u8; 16],
        ie_offset: u16,
        ie_length: u32,
        snr: u16,
    }
);

parse!(
    pub struct SsidInfo {
        pub len: u32,
        pub buf: [u8; 32],
    }
);

parse!(
    pub struct SsidInfoWithIndex {
        pub idx: u32,
        pub len: u32,
        pub buf: [u8; 32],
    }
);

parse!(
    pub struct CountryInfo {
        pub country_abbrev: [u8; 4],
        pub rev: i32,
        pub country_code: [u8; 4],
    }
);

parse!(
    #[derive(Debug)]
    pub struct EventHeader {
        pub subtype: u16,
        pub length: u16,
        pub version: u8,
        pub oui: [u8; 3],
        pub user_subtype: u16,
    }
);

parse!(
    pub struct EventMessage {
        pub version: u16,
        pub flags: u16,
        pub event_type: u32,
        pub status: u32,
        pub reason: u32,
        pub auth_type: u32,
        pub datalen: u32,
        pub addr: [u8; 6],
        pub ifname: [u8; 16],
        pub ifidx: u8,
        pub bsscfgidx: u8,
    }
);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Iovar {
    Mfp = 0,
    SaePassword = 1,
    BusTxGlom = 2,
    Apsta = 3,
    Country = 4,
    AmpduBaWsize = 5,
    AmpduMpdu = 6,
    BssCfgEventMsgs = 7,
    G2Mrate = 8,
    Bss = 9,
    GpioOut = 10,
    Escan = 11,
    BssCfgSupWpa = 12,
    BssCfgSupWpa2Eapver = 13,
    BssCfgSupWpaTmo = 14,
    BssCfgSsid = 15,
    BssCfgWsec = 16,
    BssCfgWpaAuth = 17,
    CurEthAddr = 18,
    ClmLoad = 19,
}

impl Iovar {
    pub const fn len(&self) -> usize {
        IOVARS[*self as usize].to_bytes_with_nul().len()
    }
}

impl From<Iovar> for &[u8] {
    #[inline]
    fn from(value: Iovar) -> Self {
        IOVARS[value as usize].to_bytes_with_nul()
    }
}

pub const MAX_IOVAR_LEN: usize = const {
    let mut idx = 0;
    let mut max = 0;
    while idx < IOVARS.len() {
        let len = IOVARS[idx].to_bytes_with_nul().len();
        if len > max {
            max = len;
        }
        idx += 1;
    }
    max
};

pub static IOVARS: [&CStr; 20] = [
    c"mfp",
    c"sae_password",
    c"bus:txglom",
    c"apsta",
    c"country",
    c"amdpu_ba_wsize",
    c"ampdu_mpdu",
    c"bsscfg:event_msgs",
    c"2g_mrate",
    c"bss",
    c"gpioout",
    c"escan",
    c"bsscfg:sup_wpa",
    c"bsscfg:sup_wpa2_eapver",
    c"bsscfg:sup_wpa_tmo",
    c"bsscfg:ssid",
    c"bsscfg:wsec",
    c"bsscfg:wpa_auth",
    c"cur_etheraddr",
    c"clmload",
];

#[cfg(test)]
mod tests {
    #[test]
    fn test_u8() {
        parse!(
            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            struct Data {
                field0: u8,
                field1: u8,
            }
        );
        let data = Data {
            field0: 0xaf,
            field1: 0x2c,
        };
        let bytes = data.into_bytes();
        assert_eq!(data, Data::from_bytes(&bytes));
    }

    #[test]
    fn test_u16() {
        parse!(
            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            struct Data {
                field0: u16,
                field1: u16,
                field2: u16,
            }
        );
        let data = Data {
            field0: 0xbbef,
            field1: 0xace0,
            field2: 0x0430,
        };

        let bytes = data.into_bytes();
        assert_eq!(data, Data::from_bytes(&bytes));
    }

    #[test]
    fn test_u32() {
        parse!(
            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            struct Data {
                field0: u32,
                field1: u32,
                field2: u32,
            }
        );
        let data = Data {
            field0: 0x00fe_bbef,
            field1: 0x2612_ace0,
            field2: 0x2001_0430,
        };

        let bytes = data.into_bytes();
        assert_eq!(data, Data::from_bytes(&bytes));
    }

    #[test]
    fn test_buffer() {
        parse!(
            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            struct Data {
                field0: u8,
                field1: u32,
                field2: u32,
                field3: [u8; 251],
                field4: u16,
            }
        );

        let data = Data {
            field0: 0xfe,
            field1: 0x2612_ace0,
            field2: 0x2001_0430,
            field3: [0xa0; 251],
            field4: 0x0502,
        };

        let bytes = data.into_bytes();
        assert_eq!(data, Data::from_bytes(&bytes));
    }

    #[test]
    fn test_mix() {
        parse!(
            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            struct Data {
                field0: u8,
                field1: u32,
                field2: u16,
                field3: u16,
                field4: u8,
                field5: u16,
            }
        );
        let data = Data {
            field0: 0xef,
            field1: 0x2612_ace0,
            field2: 0x043f,
            field3: 0x00fe,
            field4: 0xce,
            field5: 0x2001,
        };

        let bytes = data.into_bytes();
        assert_eq!(data, Data::from_bytes(&bytes));
    }
}
