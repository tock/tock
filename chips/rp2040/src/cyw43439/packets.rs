// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.
// Copyright (c) Embassy project contributors

use crate::impl_bytes;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CountryInfo {
    pub country_abbrev: [u8; 4],
    pub rev: i32,
    pub country_code: [u8; 4],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PassphraseInfo {
    pub len: u16,
    pub flags: u16,
    pub passphrase: [u8; 64],
}
impl_bytes!(PassphraseInfo);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SaePassphraseInfo {
    pub len: u16,
    pub passphrase: [u8; 128],
}
impl_bytes!(SaePassphraseInfo);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(super) struct SdpcmHeader {
    pub len: u16,
    pub len_inv: u16,
    /// Rx/Tx sequence number
    pub sequence: u8,
    ///  4 MSB Channel number, 4 LSB arbitrary flag
    pub channel_and_flags: u8,
    /// Length of next data frame, reserved for Tx
    pub next_length: u8,
    /// Data offset
    pub header_length: u8,
    /// Flow control bits, reserved for Tx
    pub wireless_flow_control: u8,
    /// Maximum Sequence number allowed by firmware for Tx
    pub bus_data_credit: u8,
    /// Reserved
    pub reserved: [u8; 2],
}
impl_bytes!(SdpcmHeader);

impl SdpcmHeader {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        let packet_len = packet.len();
        if packet_len < Self::SIZE {
            return None;
        }
        let (sdpcm_header, sdpcm_packet) = packet.split_at_mut(Self::SIZE);
        let sdpcm_header = Self::from_bytes_mut(sdpcm_header.try_into().unwrap());

        if sdpcm_header.len != !sdpcm_header.len_inv {
            return None;
        }

        if sdpcm_header.len as usize != packet_len {
            return None;
        }

        let sdpcm_packet = &mut sdpcm_packet[(sdpcm_header.header_length as usize - Self::SIZE)..];
        Some((sdpcm_header, sdpcm_packet))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub(super) struct CdcHeader {
    pub cmd: u32,
    pub len: u32,
    pub flags: u16,
    pub id: u16,
    pub status: u32,
}
impl_bytes!(CdcHeader);

impl CdcHeader {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        if packet.len() < Self::SIZE {
            return None;
        }

        let (cdc_header, payload) = packet.split_at_mut(Self::SIZE);
        let cdc_header = Self::from_bytes_mut(cdc_header.try_into().unwrap());

        let payload = &mut payload[..cdc_header.len as usize];
        Some((cdc_header, payload))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub(super) struct DownloadHeader {
    pub flag: u16, //
    pub dload_type: u16,
    pub len: u32,
    pub crc: u32,
}
impl_bytes!(DownloadHeader);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BdcHeader {
    pub flags: u8,
    /// 802.1d Priority (low 3 bits)
    pub priority: u8,
    pub flags2: u8,
    /// Offset from end of BDC header to packet data, in 4-uint8_t words. Leaves room for optional headers.
    pub data_offset: u8,
}
impl_bytes!(BdcHeader);

impl BdcHeader {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        if packet.len() < Self::SIZE {
            return None;
        }

        let (bdc_header, bdc_packet) = packet.split_at_mut(Self::SIZE);
        let bdc_header = Self::from_bytes_mut(bdc_header.try_into().unwrap());

        let packet_start = 4 * bdc_header.data_offset as usize;

        let bdc_packet = bdc_packet.get_mut(packet_start..)?;

        Some((bdc_header, bdc_packet))
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed(2))]
pub struct EventPacket {
    pub eth: EthernetHeader,
    pub hdr: EventHeader,
    pub msg: EventMessage,
}
impl_bytes!(EventPacket);

impl EventPacket {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        if packet.len() < Self::SIZE {
            return None;
        }

        let (event_header, event_packet) = packet.split_at_mut(Self::SIZE);
        let event_header = Self::from_bytes_mut(event_header.try_into().unwrap());
        event_header.byteswap();

        let event_packet = event_packet.get_mut(..event_header.msg.datalen as usize)?;

        Some((event_header, event_packet))
    }

    pub fn byteswap(&mut self) {
        self.eth.byteswap();
        self.hdr.byteswap();
        self.msg.byteswap();
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EthernetHeader {
    pub destination_mac: [u8; 6],
    pub source_mac: [u8; 6],
    pub ether_type: u16,
}

impl EthernetHeader {
    /// Swap endianness.
    pub fn byteswap(&mut self) {
        self.ether_type = self.ether_type.to_be();
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EventHeader {
    pub subtype: u16,
    pub length: u16,
    pub version: u8,
    pub oui: [u8; 3],
    pub user_subtype: u16,
}

impl EventHeader {
    pub fn byteswap(&mut self) {
        self.subtype = self.subtype.to_be();
        self.length = self.length.to_be();
        self.user_subtype = self.user_subtype.to_be();
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct EventMessage {
    /// version
    pub version: u16,
    /// see flags below
    pub flags: u16,
    /// Message (see below)
    pub event_type: u32,
    /// Status code (see below)
    pub status: u32,
    /// Reason code (if applicable)
    pub reason: u32,
    /// WLC_E_AUTH
    pub auth_type: u32,
    /// data buf
    pub datalen: u32,
    /// Station address (if applicable)
    pub addr: [u8; 6],
    /// name of the incoming packet interface
    pub ifname: [u8; 16],
    /// destination OS i/f index
    pub ifidx: u8,
    /// source bsscfg index
    pub bsscfgidx: u8,
}
impl_bytes!(EventMessage);

impl EventMessage {
    pub fn byteswap(&mut self) {
        self.version = self.version.to_be();
        self.flags = self.flags.to_be();
        self.event_type = self.event_type.to_be();
        self.status = self.status.to_be();
        self.reason = self.reason.to_be();
        self.auth_type = self.auth_type.to_be();
        self.datalen = self.datalen.to_be();
    }
}

/// Parameters for a wifi scan
#[derive(Clone, Copy, Default)]
#[repr(C)]
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
    pub channel_list: [u16; 1],
}

#[macro_export]
macro_rules! impl_bytes {
    ($t:ident) => {
        impl $t {
            /// Bytes consumed by this type.
            pub const SIZE: usize = core::mem::size_of::<Self>();

            /// Create from byte array.
            #[allow(unused)]
            pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> &Self {
                let alignment = core::mem::align_of::<Self>();
                assert_eq!(
                    bytes.as_ptr().align_offset(alignment),
                    0,
                    "{} is not aligned",
                    core::any::type_name::<Self>()
                );
                unsafe { core::mem::transmute(bytes) }
            }

            /// Create from mutable byte array.
            #[allow(unused)]
            pub fn from_bytes_mut(bytes: &mut [u8; Self::SIZE]) -> &mut Self {
                let alignment = core::mem::align_of::<Self>();
                assert_eq!(
                    bytes.as_ptr().align_offset(alignment),
                    0,
                    "{} is not aligned",
                    core::any::type_name::<Self>()
                );

                unsafe { core::mem::transmute(bytes) }
            }
        }
    };
}

#[repr(C)]
pub(crate) struct EventMask {
    pub(crate) iface: u32,
    pub(crate) events: [u8; 24],
}

impl EventMask {
    pub(crate) fn unset(&mut self, evt: Event) {
        let evt = evt as u8 as usize;
        self.events[evt / 8] &= !(1 << (evt % 8));
    }
}

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Event {
    Unknown = 0xFF,
    /// indicates status of set SSID
    SET_SSID = 0,
    /// differentiates join IBSS from found (START) IBSS
    JOIN = 1,
    /// STA founded an IBSS or AP started a BSS
    START = 2,
    /// 802.11 AUTH request
    AUTH = 3,
    /// 802.11 AUTH indication
    AUTH_IND = 4,
    /// 802.11 DEAUTH request
    DEAUTH = 5,
    /// 802.11 DEAUTH indication
    DEAUTH_IND = 6,
    /// 802.11 ASSOC request
    ASSOC = 7,
    /// 802.11 ASSOC indication
    ASSOC_IND = 8,
    /// 802.11 REASSOC request
    REASSOC = 9,
    /// 802.11 REASSOC indication
    REASSOC_IND = 10,
    /// 802.11 DISASSOC request
    DISASSOC = 11,
    /// 802.11 DISASSOC indication
    DISASSOC_IND = 12,
    /// 802.11h Quiet period started
    QUIET_START = 13,
    /// 802.11h Quiet period ended
    QUIET_END = 14,
    /// BEACONS received/lost indication
    BEACON_RX = 15,
    /// generic link indication
    LINK = 16,
    /// TKIP MIC error occurred
    MIC_ERROR = 17,
    /// NDIS style link indication
    NDIS_LINK = 18,
    /// roam attempt occurred: indicate status & reason
    ROAM = 19,
    /// change in dot11FailedCount (txfail)
    TXFAIL = 20,
    /// WPA2 pmkid cache indication
    PMKID_CACHE = 21,
    /// current AP's TSF value went backward
    RETROGRADE_TSF = 22,
    /// AP was pruned from join list for reason
    PRUNE = 23,
    /// report AutoAuth table entry match for join attempt
    AUTOAUTH = 24,
    /// Event encapsulating an EAPOL message
    EAPOL_MSG = 25,
    /// Scan results are ready or scan was aborted
    SCAN_COMPLETE = 26,
    /// indicate to host addts fail/success
    ADDTS_IND = 27,
    /// indicate to host delts fail/success
    DELTS_IND = 28,
    /// indicate to host of beacon transmit
    BCNSENT_IND = 29,
    /// Send the received beacon up to the host
    BCNRX_MSG = 30,
    /// indicate to host loss of beacon
    BCNLOST_MSG = 31,
    /// before attempting to roam
    ROAM_PREP = 32,
    /// PFN network found event
    PFN_NET_FOUND = 33,
    /// PFN network lost event
    PFN_NET_LOST = 34,
    RESET_COMPLETE = 35,
    JOIN_START = 36,
    ROAM_START = 37,
    ASSOC_START = 38,
    IBSS_ASSOC = 39,
    RADIO = 40,
    /// PSM microcode watchdog fired
    PSM_WATCHDOG = 41,
    /// CCX association start
    CCX_ASSOC_START = 42,
    /// CCX association abort
    CCX_ASSOC_ABORT = 43,
    /// probe request received
    PROBREQ_MSG = 44,
    SCAN_CONFIRM_IND = 45,
    /// WPA Handshake
    PSK_SUP = 46,
    COUNTRY_CODE_CHANGED = 47,
    /// WMMAC excedded medium time
    EXCEEDED_MEDIUM_TIME = 48,
    /// WEP ICV error occurred
    ICV_ERROR = 49,
    /// Unsupported unicast encrypted frame
    UNICAST_DECODE_ERROR = 50,
    /// Unsupported multicast encrypted frame
    MULTICAST_DECODE_ERROR = 51,
    TRACE = 52,
    /// BT-AMP HCI event
    BTA_HCI_EVENT = 53,
    /// I/F change (for wlan host notification)
    IF = 54,
    /// P2P Discovery listen state expires
    P2P_DISC_LISTEN_COMPLETE = 55,
    /// indicate RSSI change based on configured levels
    RSSI = 56,
    /// PFN best network batching event
    PFN_BEST_BATCHING = 57,
    EXTLOG_MSG = 58,
    /// Action frame reception
    ACTION_FRAME = 59,
    /// Action frame Tx complete
    ACTION_FRAME_COMPLETE = 60,
    /// assoc request received
    PRE_ASSOC_IND = 61,
    /// re-assoc request received
    PRE_REASSOC_IND = 62,
    /// channel adopted (xxx: obsoleted)
    CHANNEL_ADOPTED = 63,
    /// AP started
    AP_STARTED = 64,
    /// AP stopped due to DFS
    DFS_AP_STOP = 65,
    /// AP resumed due to DFS
    DFS_AP_RESUME = 66,
    /// WAI stations event
    WAI_STA_EVENT = 67,
    /// event encapsulating an WAI message
    WAI_MSG = 68,
    /// escan result event
    ESCAN_RESULT = 69,
    /// action frame off channel complete
    ACTION_FRAME_OFF_CHAN_COMPLETE = 70,
    /// probe response received
    PROBRESP_MSG = 71,
    /// P2P Probe request received
    P2P_PROBREQ_MSG = 72,
    DCS_REQUEST = 73,
    /// credits for D11 FIFOs. [AC0,AC1,AC2,AC3,BC_MC,ATIM]
    FIFO_CREDIT_MAP = 74,
    /// Received action frame event WITH wl_event_rx_frame_data_t header
    ACTION_FRAME_RX = 75,
    /// Wake Event timer fired, used for wake WLAN test mode
    WAKE_EVENT = 76,
    /// Radio measurement complete
    RM_COMPLETE = 77,
    /// Synchronize TSF with the host
    HTSFSYNC = 78,
    /// request an overlay IOCTL/iovar from the host
    OVERLAY_REQ = 79,
    CSA_COMPLETE_IND = 80,
    /// excess PM Wake Event to inform host
    EXCESS_PM_WAKE_EVENT = 81,
    /// no PFN networks around
    PFN_SCAN_NONE = 82,
    /// last found PFN network gets lost
    PFN_SCAN_ALLGONE = 83,
    GTK_PLUMBED = 84,
    /// 802.11 ASSOC indication for NDIS only
    ASSOC_IND_NDIS = 85,
    /// 802.11 REASSOC indication for NDIS only
    REASSOC_IND_NDIS = 86,
    ASSOC_REQ_IE = 87,
    ASSOC_RESP_IE = 88,
    /// association recreated on resume
    ASSOC_RECREATED = 89,
    /// rx action frame event for NDIS only
    ACTION_FRAME_RX_NDIS = 90,
    /// authentication request received
    AUTH_REQ = 91,
    /// fast assoc recreation failed
    SPEEDY_RECREATE_FAIL = 93,
    /// port-specific event and payload (e.g. NDIS)
    NATIVE = 94,
    /// event for tx pkt delay suddently jump
    PKTDELAY_IND = 95,
    /// AWDL AW period starts
    AWDL_AW = 96,
    /// AWDL Master/Slave/NE master role event
    AWDL_ROLE = 97,
    /// Generic AWDL event
    AWDL_EVENT = 98,
    /// NIC AF txstatus
    NIC_AF_TXS = 99,
    /// NAN event
    NAN = 100,
    BEACON_FRAME_RX = 101,
    /// desired service found
    SERVICE_FOUND = 102,
    /// GAS fragment received
    GAS_FRAGMENT_RX = 103,
    /// GAS sessions all complete
    GAS_COMPLETE = 104,
    /// New device found by p2p offload
    P2PO_ADD_DEVICE = 105,
    /// device has been removed by p2p offload
    P2PO_DEL_DEVICE = 106,
    /// WNM event to notify STA enter sleep mode
    WNM_STA_SLEEP = 107,
    /// Indication of MAC tx failures (exhaustion of 802.11 retries) exceeding threshold(s)
    TXFAIL_THRESH = 108,
    /// Proximity Detection event
    PROXD = 109,
    /// AWDL RX Probe response
    AWDL_RX_PRB_RESP = 111,
    /// AWDL RX Action Frames
    AWDL_RX_ACT_FRAME = 112,
    /// AWDL Wowl nulls
    AWDL_WOWL_NULLPKT = 113,
    /// AWDL Phycal status
    AWDL_PHYCAL_STATUS = 114,
    /// AWDL OOB AF status
    AWDL_OOB_AF_STATUS = 115,
    /// Interleaved Scan status
    AWDL_SCAN_STATUS = 116,
    /// AWDL AW Start
    AWDL_AW_START = 117,
    /// AWDL AW End
    AWDL_AW_END = 118,
    /// AWDL AW Extensions
    AWDL_AW_EXT = 119,
    AWDL_PEER_CACHE_CONTROL = 120,
    CSA_START_IND = 121,
    CSA_DONE_IND = 122,
    CSA_FAILURE_IND = 123,
    /// CCA based channel quality report
    CCA_CHAN_QUAL = 124,
    /// to report change in BSSID while roaming
    BSSID = 125,
    /// tx error indication
    TX_STAT_ERROR = 126,
    /// credit check for BCMC supported
    BCMC_CREDIT_SUPPORT = 127,
    /// psta primary interface indication
    PSTA_PRIMARY_INTF_IND = 128,
    /// Handover Request Initiated
    BT_WIFI_HANDOVER_REQ = 130,
    /// Southpaw TxInhibit notification
    SPW_TXINHIBIT = 131,
    /// FBT Authentication Request Indication
    FBT_AUTH_REQ_IND = 132,
    /// Enhancement addition for RSSI
    RSSI_LQM = 133,
    /// Full probe/beacon (IEs etc) results
    PFN_GSCAN_FULL_RESULT = 134,
    /// Significant change in rssi of bssids being tracked
    PFN_SWC = 135,
    /// a STA been authroized for traffic
    AUTHORIZED = 136,
    /// probe req with wl_event_rx_frame_data_t header
    PROBREQ_MSG_RX = 137,
    /// PFN completed scan of network list
    PFN_SCAN_COMPLETE = 138,
    /// RMC Event
    RMC_EVENT = 139,
    /// DPSTA interface indication
    DPSTA_INTF_IND = 140,
    /// RRM Event
    RRM = 141,
    /// ULP entry event
    ULP = 146,
    /// TCP Keep Alive Offload Event
    TKO = 151,
    /// authentication request received
    EXT_AUTH_REQ = 187,
    /// authentication request received
    EXT_AUTH_FRAME_RX = 188,
    /// mgmt frame Tx complete
    MGMT_FRAME_TXSTATUS = 189,
    /// highest val + 1 for range checking
    LAST = 190,
}}
/// Wifi Scan Results Header, followed by `bss_count` `BssInfo`
#[derive(Clone, Copy)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed(2))]
pub struct ScanResults {
    pub buflen: u32,
    pub version: u32,
    pub sync_id: u16,
    pub bss_count: u16,
}
impl_bytes!(ScanResults);

impl ScanResults {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut ScanResults, &mut [u8])> {
        if packet.len() < core::mem::size_of::<Self>() {
            return None;
        }

        let (scan_results, bssinfo) = packet.split_at_mut(core::mem::size_of::<Self>());
        let scan_results = ScanResults::from_bytes_mut(scan_results.try_into().unwrap());

        if scan_results.bss_count > 0 && bssinfo.len() < BssInfo::SIZE {
            return None;
        }

        Some((scan_results, bssinfo))
    }
}

#[derive(Clone, Copy)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed(2))]
#[non_exhaustive]
pub struct BssInfo {
    /// Version.
    pub version: u32,
    /// Length.
    pub length: u32,
    /// BSSID.
    pub bssid: [u8; 6],
    /// Beacon period.
    pub beacon_period: u16,
    /// Capability.
    pub capability: u16,
    /// SSID length.
    pub ssid_len: u8,
    /// SSID.
    pub ssid: [u8; 32],
    reserved1: [u8; 1],
    /// Number of rates in the rates field.
    pub rateset_count: u32,
    /// Rates in 500kpbs units.
    pub rates: [u8; 16],
    /// Channel specification.
    pub chanspec: u16,
    /// Announcement traffic indication message.
    pub atim_window: u16,
    /// Delivery traffic indication message.
    pub dtim_period: u8,
    reserved2: [u8; 1],
    /// Receive signal strength (in dbM).
    pub rssi: i16,
    /// Received noise (in dbM).
    pub phy_noise: i8,
    /// 802.11n capability.
    pub n_cap: u8,
    reserved3: [u8; 2],
    /// 802.11n BSS capabilities.
    pub nbss_cap: u32,
    /// 802.11n control channel number.
    pub ctl_ch: u8,
    reserved4: [u8; 3],
    reserved32: [u32; 1],
    /// Flags.
    pub flags: u8,
    /// VHT capability.
    pub vht_cap: u8,
    reserved5: [u8; 2],
    /// 802.11n BSS required MCS.
    pub basic_mcs: [u8; 16],
    /// Information Elements (IE) offset.
    pub ie_offset: u16,
    /// Length of Information Elements (IE) in bytes.
    pub ie_length: u32,
    /// Average signal-to-noise (SNR) ratio during frame reception.
    pub snr: i16,
    // there will be more stuff here
}
impl_bytes!(BssInfo);

impl BssInfo {
    pub(crate) fn parse(packet: &mut [u8]) -> Option<&mut Self> {
        if packet.len() < BssInfo::SIZE {
            return None;
        }

        Some(BssInfo::from_bytes_mut(
            packet[..BssInfo::SIZE].as_mut().try_into().unwrap(),
        ))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SsidInfoWithIndex {
    pub index: u32,
    pub ssid_info: SsidInfo,
}
impl_bytes!(SsidInfoWithIndex);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SsidInfo {
    pub len: u32,
    pub ssid: [u8; 32],
}
impl_bytes!(SsidInfo);
