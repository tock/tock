//! Implements IEEE 802.15.4-2015 header encoding and decoding.
//! Supports the general MAC frame format, which encompasses data frames, beacon
//! frames, MAC command frames, and the like.

use crate::net::stream::SResult;
use crate::net::stream::{decode_bytes_be, decode_u16, decode_u32, decode_u8};
use crate::net::stream::{encode_bytes, encode_bytes_be, encode_u16, encode_u32, encode_u8};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MacAddress {
    Short(u16),
    Long([u8; 8]),
}

impl MacAddress {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        match *self {
            MacAddress::Short(ref short_addr) => encode_u16(buf, short_addr.to_be()),
            MacAddress::Long(ref long_addr) => encode_bytes_be(buf, long_addr),
        }
    }

    pub fn decode(buf: &[u8], mode: AddressMode) -> SResult<Option<MacAddress>> {
        match mode {
            AddressMode::NotPresent => stream_done!(0, None),
            AddressMode::Short => {
                let (off, short_addr_be) = dec_try!(buf; decode_u16);
                let short_addr = u16::from_be(short_addr_be);
                stream_done!(off, Some(MacAddress::Short(short_addr)));
            }
            AddressMode::Long => {
                let mut long_addr = [0u8; 8];
                let off = dec_consume!(buf; decode_bytes_be, &mut long_addr);
                stream_done!(off, Some(MacAddress::Long(long_addr)));
            }
        }
    }
}

pub type PanID = u16;

mod frame_control {
    pub const FRAME_TYPE_MASK: u16 = 0b111;
    pub const SECURITY_ENABLED: u16 = 1 << 3;
    pub const FRAME_PENDING: u16 = 1 << 4;
    pub const ACK_REQUESTED: u16 = 1 << 5;
    pub const PAN_ID_COMPRESSION: u16 = 1 << 6;
    pub const SEQ_SUPPRESSED: u16 = 1 << 8;
    pub const IE_PRESENT: u16 = 1 << 9;
    pub const DST_MODE_POS: usize = 10;
    pub const FRAME_VERSION_MASK: u16 = 0b11 << 12;
    pub const SRC_MODE_POS: usize = 14;
    pub const MODE_MASK: u16 = 0b11;
}

#[repr(u16)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FrameType {
    // Reserved = 0b100,
    Beacon = 0b000,
    Data = 0b001,
    Acknowledgement = 0b010,
    MACCommand = 0b011,
    Multipurpose = 0b101,
    Fragment = 0b110,
    Extended = 0b111,
}

impl FrameType {
    pub fn from_fcf(fcf: u16) -> Option<FrameType> {
        match fcf & frame_control::FRAME_TYPE_MASK {
            0b000 => Some(FrameType::Beacon),
            0b001 => Some(FrameType::Data),
            0b010 => Some(FrameType::Acknowledgement),
            0b011 => Some(FrameType::MACCommand),
            0b101 => Some(FrameType::Multipurpose),
            0b110 => Some(FrameType::Fragment),
            0b111 => Some(FrameType::Extended),
            _ => None,
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FrameVersion {
    // Reserved = 0x3000
    V2003 = 0x0000,
    V2006 = 0x1000,
    V2015 = 0x2000,
}

impl FrameVersion {
    pub fn from_fcf(fcf: u16) -> Option<FrameVersion> {
        match fcf & frame_control::FRAME_VERSION_MASK {
            0x0000 => Some(FrameVersion::V2003),
            0x1000 => Some(FrameVersion::V2006),
            0x2000 => Some(FrameVersion::V2015),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AddressMode {
    NotPresent = 0b00,
    Short = 0b10,
    Long = 0b11,
}

impl From<&'a Option<MacAddress>> for AddressMode {
    fn from(opt_addr: &'a Option<MacAddress>) -> Self {
        match *opt_addr {
            None => AddressMode::NotPresent,
            Some(addr) => match addr {
                MacAddress::Short(_) => AddressMode::Short,
                MacAddress::Long(_) => AddressMode::Long,
            },
        }
    }
}

impl AddressMode {
    pub fn from_mode(mode: u16) -> Option<AddressMode> {
        match mode {
            0b00 => Some(AddressMode::NotPresent),
            0b10 => Some(AddressMode::Short),
            0b11 => Some(AddressMode::Long),
            _ => None,
        }
    }
}

mod security_control {
    pub const SECURITY_LEVEL_MASK: u8 = 0b111;
    pub const KEY_ID_MODE_MASK: u8 = 0b11 << 3;
    pub const FRAME_COUNTER_SUPPRESSION: u8 = 1 << 5;
    pub const ASN_IN_NONCE: u8 = 1 << 6;
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SecurityLevel {
    // Reserved = 0b100,
    None = 0b000,
    Mic32 = 0b001,
    Mic64 = 0b010,
    Mic128 = 0b011,
    EncMic32 = 0b101,
    EncMic64 = 0b110,
    EncMic128 = 0b111,
}

impl SecurityLevel {
    pub fn from_scf(scf: u8) -> Option<SecurityLevel> {
        match scf & security_control::SECURITY_LEVEL_MASK {
            0b000 => Some(SecurityLevel::None),
            0b001 => Some(SecurityLevel::Mic32),
            0b010 => Some(SecurityLevel::Mic64),
            0b011 => Some(SecurityLevel::Mic128),
            0b101 => Some(SecurityLevel::EncMic32),
            0b110 => Some(SecurityLevel::EncMic64),
            0b111 => Some(SecurityLevel::EncMic128),
            _ => None,
        }
    }

    pub fn encryption_needed(&self) -> bool {
        match *self {
            SecurityLevel::EncMic32 | SecurityLevel::EncMic64 | SecurityLevel::EncMic128 => true,
            _ => false,
        }
    }

    pub fn mic_len(&self) -> usize {
        match *self {
            SecurityLevel::Mic32 | SecurityLevel::EncMic32 => 4,
            SecurityLevel::Mic64 | SecurityLevel::EncMic64 => 8,
            SecurityLevel::Mic128 | SecurityLevel::EncMic128 => 16,
            _ => 0,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyIdMode {
    Implicit = 0x00,
    Index = 0x08,
    Source4Index = 0x10,
    Source8Index = 0x18,
}

impl KeyIdMode {
    pub fn from_scf(scf: u8) -> Option<KeyIdMode> {
        match scf & security_control::KEY_ID_MODE_MASK {
            0x00 => Some(KeyIdMode::Implicit),
            0x08 => Some(KeyIdMode::Index),
            0x10 => Some(KeyIdMode::Source4Index),
            0x18 => Some(KeyIdMode::Source8Index),
            _ => panic!("Unreachable case because of mask"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyId {
    Implicit,
    Index(u8),
    Source4Index([u8; 4], u8),
    Source8Index([u8; 8], u8),
}

impl KeyId {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        let off = match *self {
            KeyId::Implicit => 0,
            KeyId::Index(index) => enc_consume!(buf; encode_u8, index),
            KeyId::Source4Index(ref src, index) => {
                let off = enc_consume!(buf; encode_bytes_be, src);
                enc_consume!(buf, off; encode_u8, index)
            }
            KeyId::Source8Index(ref src, index) => {
                let off = enc_consume!(buf; encode_bytes_be, src);
                enc_consume!(buf, off; encode_u8, index)
            }
        };
        stream_done!(off);
    }

    pub fn decode(buf: &[u8], mode: KeyIdMode) -> SResult<KeyId> {
        match mode {
            KeyIdMode::Implicit => stream_done!(0, KeyId::Implicit),
            KeyIdMode::Index => {
                let (off, index) = dec_try!(buf; decode_u8);
                stream_done!(off, KeyId::Index(index));
            }
            KeyIdMode::Source4Index => {
                let mut src = [0u8; 4];
                let off = dec_consume!(buf; decode_bytes_be, &mut src);
                let (off, index) = dec_try!(buf, off; decode_u8);
                stream_done!(off, KeyId::Source4Index(src, index));
            }
            KeyIdMode::Source8Index => {
                let mut src = [0u8; 8];
                let off = dec_consume!(buf; decode_bytes_be, &mut src);
                let (off, index) = dec_try!(buf, off; decode_u8);
                stream_done!(off, KeyId::Source8Index(src, index));
            }
        }
    }
}

impl From<&'a KeyId> for KeyIdMode {
    fn from(key_id: &'a KeyId) -> Self {
        match *key_id {
            KeyId::Implicit => KeyIdMode::Implicit,
            KeyId::Index(_) => KeyIdMode::Index,
            KeyId::Source4Index(_, _) => KeyIdMode::Source4Index,
            KeyId::Source8Index(_, _) => KeyIdMode::Source8Index,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Security {
    pub level: SecurityLevel,
    pub asn_in_nonce: bool,
    pub frame_counter: Option<u32>,
    pub key_id: KeyId,
}

impl Security {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        // The security control field is collected while encoding the other
        // fields in the auxiliary security header, and only written in the end
        stream_len_cond!(buf, 1);
        let mut off = 1;

        // Security control field
        let mut scf = self.level as u8;
        if self.asn_in_nonce {
            scf |= security_control::ASN_IN_NONCE;
        }

        // Frame counter field
        if let Some(ref frame_counter) = self.frame_counter {
            off = enc_consume!(buf, off; encode_u32, frame_counter.to_be());
        } else {
            scf |= security_control::FRAME_COUNTER_SUPPRESSION;
        }

        // Key identifier field
        scf |= KeyIdMode::from(&self.key_id) as u8;
        off = enc_consume!(buf, off; self.key_id; encode);

        // Put the security control field in front
        enc_try!(buf; encode_u8, scf);
        stream_done!(off);
    }

    pub fn decode(buf: &[u8]) -> SResult<Security> {
        // Security control field
        let (off, scf) = dec_try!(buf; decode_u8);
        let level = stream_from_option!(SecurityLevel::from_scf(scf));
        let asn_in_nonce = (scf & security_control::ASN_IN_NONCE) != 0;

        // Frame counter field
        let frame_counter_present = (scf & security_control::FRAME_COUNTER_SUPPRESSION) != 0;
        let (off, frame_counter) = if frame_counter_present {
            let (off, frame_counter_be) = dec_try!(buf, off; decode_u32);
            (off, Some(u32::from_be(frame_counter_be)))
        } else {
            (off, None)
        };

        // Key identifier field
        let key_id_mode = stream_from_option!(KeyIdMode::from_scf(scf));
        let (off, key_id) = dec_try!(buf, off; KeyId::decode, key_id_mode);

        stream_done!(
            off,
            Security {
                level: level,
                asn_in_nonce: asn_in_nonce,
                frame_counter: frame_counter,
                key_id: key_id,
            }
        );
    }
}

mod ie_control {
    // Header IE constants
    pub const HEADER_LEN_MAX: usize = (1 << 7) - 1;
    pub const HEADER_LEN_MASK: u16 = HEADER_LEN_MAX as u16;
    pub const HEADER_ID_POS: usize = 7;

    // Payload IE constants
    pub const PAYLOAD_LEN_MAX: usize = (1 << 11) - 1;
    pub const PAYLOAD_LEN_MASK: u16 = PAYLOAD_LEN_MAX as u16;
    pub const PAYLOAD_ID_MASK: u8 = 0xf; // Only 4 bits
    pub const PAYLOAD_ID_POS: usize = 11;

    pub const TYPE: u16 = 0x8000;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HeaderIE<'a> {
    Undissected { element_id: u8, content: &'a [u8] },
    Termination1,
    Termination2,
}

impl Default for HeaderIE<'a> {
    fn default() -> Self {
        HeaderIE::Termination1
    }
}

impl HeaderIE<'a> {
    pub fn is_termination(&self) -> bool {
        match *self {
            HeaderIE::Termination1 | HeaderIE::Termination2 => true,
            _ => false,
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        // Append the content field of the IE first
        let mut off = 2;
        let element_id: u8 = match *self {
            HeaderIE::Undissected {
                element_id,
                content,
            } => {
                off = enc_consume!(buf, off; encode_bytes, content);
                element_id
            }
            HeaderIE::Termination1 => 0x7e,
            HeaderIE::Termination2 => 0x7f,
        };

        // Write the two octets that begin each header IE
        let content_len = off - 2;
        stream_cond!(content_len <= ie_control::HEADER_LEN_MAX);
        let ie_ctl = ((content_len as u16) & ie_control::HEADER_LEN_MASK)
            | ((element_id as u16) << ie_control::HEADER_ID_POS);
        enc_consume!(buf; encode_u16, ie_ctl.to_be());

        stream_done!(off);
    }

    pub fn decode<'b>(buf: &'b [u8]) -> SResult<HeaderIE<'b>> {
        let (off, ie_ctl_be) = dec_try!(buf; decode_u16);
        let ie_ctl = u16::from_be(ie_ctl_be);

        // Header IEs are type 0
        stream_cond!(ie_ctl & ie_control::TYPE == 0);
        let content_len = (ie_ctl & ie_control::HEADER_LEN_MASK) as usize;
        let element_id = (ie_ctl >> ie_control::HEADER_ID_POS) as u8;

        stream_len_cond!(buf, off + content_len);
        let content = &buf[off..off + content_len];

        let ie = match element_id {
            0x7e => HeaderIE::Termination1,
            0x7f => HeaderIE::Termination2,
            element_id => HeaderIE::Undissected {
                element_id: element_id,
                content: content,
            },
        };

        stream_done!(off + content_len, ie);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PayloadIE<'a> {
    Undissected { group_id: u8, content: &'a [u8] },
    Termination,
}

impl Default for PayloadIE<'a> {
    fn default() -> Self {
        PayloadIE::Termination
    }
}

impl PayloadIE<'a> {
    pub fn is_termination(&self) -> bool {
        match *self {
            PayloadIE::Termination => true,
            _ => false,
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        // Append the content field of the IE first
        let mut off = 2;
        let group_id: u8 = match *self {
            PayloadIE::Undissected { group_id, content } => {
                off = enc_consume!(buf, off; encode_bytes, content);
                group_id
            }
            PayloadIE::Termination => 0xf,
        };

        // Write the two octets that begin each payload IE
        let content_len = off - 2;
        stream_cond!(content_len <= ie_control::PAYLOAD_LEN_MAX);
        let ie_ctl = ((content_len as u16) & ie_control::PAYLOAD_LEN_MASK)
            | ((group_id & ie_control::PAYLOAD_ID_MASK) as u16) << ie_control::PAYLOAD_ID_POS;
        enc_consume!(buf; encode_u16, ie_ctl.to_be());

        stream_done!(off);
    }

    pub fn decode<'b>(buf: &'b [u8]) -> SResult<PayloadIE<'b>> {
        let (off, ie_ctl_be) = dec_try!(buf; decode_u16);
        let ie_ctl = u16::from_be(ie_ctl_be);

        // Payload IEs are type 1
        stream_cond!(ie_ctl & ie_control::TYPE != 0);
        let content_len = (ie_ctl & ie_control::PAYLOAD_LEN_MASK) as usize;
        let element_id =
            ((ie_ctl >> ie_control::PAYLOAD_ID_POS) as u8) & ie_control::PAYLOAD_ID_MASK;

        stream_len_cond!(buf, off + content_len);
        let content = &buf[off..off + content_len];

        let ie = match element_id {
            0xf => PayloadIE::Termination,
            group_id => PayloadIE::Undissected {
                group_id: group_id,
                content: content,
            },
        };

        stream_done!(off + content_len, ie);
    }
}

pub const MAX_HEADER_IES: usize = 5;
pub const MAX_PAYLOAD_IES: usize = 5;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Header<'a> {
    pub frame_type: FrameType,
    pub frame_pending: bool,
    pub ack_requested: bool,
    pub version: FrameVersion,
    pub seq: Option<u8>,
    pub dst_pan: Option<PanID>,
    pub dst_addr: Option<MacAddress>,
    pub src_pan: Option<PanID>,
    pub src_addr: Option<MacAddress>,
    pub security: Option<Security>,
    pub header_ies: [HeaderIE<'a>; MAX_HEADER_IES],
    pub header_ies_len: usize,
    pub payload_ies: [PayloadIE<'a>; MAX_PAYLOAD_IES],
    pub payload_ies_len: usize,
}

impl Header<'a> {
    pub fn encode(&self, buf: &mut [u8], has_payload: bool) -> SResult<usize> {
        // The frame control field is collected in the course of encoding the
        // various other fields of the header and then written only at the end
        stream_len_cond!(buf, 2);
        let mut off = 2;

        // Sequence number
        if self.version != FrameVersion::V2015 {
            // The sequence number is always present before version 2015
            stream_cond!(self.seq.is_some());
        }
        let seq_suppressed = if let Some(seq) = self.seq {
            off = enc_consume!(buf, off; encode_u8, seq);
            false
        } else {
            true
        };

        // Addressing fields
        let (off, pan_id_compression) = enc_try!(buf, off; self; encode_addressing);
        let dst_mode = AddressMode::from(&self.dst_addr);
        let src_mode = AddressMode::from(&self.src_addr);

        // Auxiliary security header
        // Note: security can be enabled with security level 0 (None)
        let (mut off, security_enabled) = match self.security {
            None => (off, false),
            Some(ref security) => (enc_consume!(buf, off; security; encode), true),
        };

        // Information elements
        // IE list termination is implicit and handled by encoding/decoding
        // procedures. Hence, we ensure that there are no termination headers
        // in our lists.
        let has_header_ies = self.header_ies_len != 0;
        let has_payload_ies = self.payload_ies_len != 0;
        stream_cond!(self.header_ies_len <= MAX_HEADER_IES);
        stream_cond!(self.payload_ies_len <= MAX_PAYLOAD_IES);
        for ie in self.header_ies[..self.header_ies_len].iter() {
            stream_cond!(!ie.is_termination());
            off = enc_consume!(buf, off; ie; encode);
        }
        if has_payload_ies {
            // terminate with header termination 1
            off = enc_consume!(buf, off; HeaderIE::Termination1; encode);
        } else if has_header_ies && has_payload {
            // terminate with header termination 2
            off = enc_consume!(buf, off; HeaderIE::Termination2; encode);
        }
        // The MAC payload includes payload IEs
        let mac_payload_off = off;
        for ie in self.payload_ies[..self.payload_ies_len].iter() {
            stream_cond!(!ie.is_termination());
            off = enc_consume!(buf, off; ie; encode);
        }
        if has_payload_ies && has_payload {
            // terminate with payload termination
            off = enc_consume!(buf, off; PayloadIE::Termination; encode);
        }
        let ie_present = has_header_ies || has_payload_ies;

        // Flags that can be independently determined
        let mut fcf = self.frame_type as u16;
        if security_enabled {
            fcf |= frame_control::SECURITY_ENABLED;
        }
        if self.frame_pending {
            fcf |= frame_control::FRAME_PENDING;
        }
        if self.ack_requested {
            fcf |= frame_control::ACK_REQUESTED;
        }
        if pan_id_compression {
            fcf |= frame_control::PAN_ID_COMPRESSION;
        }
        if seq_suppressed {
            fcf |= frame_control::SEQ_SUPPRESSED;
        }
        if ie_present {
            fcf |= frame_control::IE_PRESENT;
        }
        fcf |= (dst_mode as u16) << frame_control::DST_MODE_POS;
        fcf |= self.version as u16;
        fcf |= (src_mode as u16) << frame_control::SRC_MODE_POS;

        // Put the frame control field in front
        enc_try!(buf; encode_u16, fcf.to_be());
        stream_done!(off, mac_payload_off);
    }

    pub fn encode_addressing(&self, buf: &mut [u8]) -> SResult<bool> {
        // IEEE 802.15.4: Section 7.2.1.5
        // The pan ID compression field's meaning is dependent on the version
        let mut drop_src_pan = false;
        let pan_id_compression = match self.version {
            FrameVersion::V2015 => {
                // In this mode, the only valid combinations are determined by
                // Table 7-2
                match (self.dst_addr, self.src_addr) {
                    (None, None) => {
                        stream_cond!(self.src_pan.is_none());
                        self.dst_pan.is_some()
                    }
                    (Some(_), None) => {
                        stream_cond!(self.src_pan.is_none());
                        self.dst_pan.is_none()
                    }
                    (None, Some(_)) => {
                        stream_cond!(self.dst_pan.is_none());
                        self.src_pan.is_none()
                    }
                    (Some(_), Some(_)) => {
                        // When both addresses are present, we require that both
                        // pans are provided, and we will only drop the source
                        // pan ID if it matches the destination.
                        drop_src_pan = match (self.dst_pan, self.src_pan) {
                            (Some(dst_pan), Some(src_pan)) => dst_pan == src_pan,
                            _ => stream_err!(),
                        };
                        drop_src_pan
                    }
                }
            }
            FrameVersion::V2003 | FrameVersion::V2006 => {
                // In these two modes, the source pan ID is only omitted if it
                // matches the destination pan ID. Hence, the user must always
                // provide a pan ID iff an address is provided.
                stream_cond!(self.dst_addr.is_some() == self.dst_pan.is_some());
                stream_cond!(self.src_addr.is_some() == self.src_pan.is_some());

                match (self.dst_pan, self.src_pan) {
                    (Some(dst_pan), Some(src_pan)) => {
                        drop_src_pan = dst_pan == src_pan;
                    }
                    _ => {}
                }
                drop_src_pan
            }
        };

        // The presence of the actual address fields are now the same
        let mut off = 0;
        if let Some(pan) = self.dst_pan {
            off = enc_consume!(buf, off; encode_u16, pan.to_be());
        }
        if let Some(addr) = self.dst_addr {
            off = enc_consume!(buf, off; addr; encode);
        }
        if let Some(pan) = self.src_pan {
            if !drop_src_pan {
                off = enc_consume!(buf, off; encode_u16, pan.to_be());
            }
        }
        if let Some(addr) = self.src_addr {
            off = enc_consume!(buf, off; addr; encode);
        }

        stream_done!(off, pan_id_compression);
    }

    /// Decodes an IEEE 802.15.4 MAC header from a byte slice, where the MAC
    /// header may contain slices into the given byte slice to represent
    /// undissected information elements (IE). `unsecured` controls whether or
    /// not payload IEs (which are encrypted if the frame has not yet been
    /// unsecured) can be parsed.
    pub fn decode<'b>(buf: &'b [u8], unsecured: bool) -> SResult<(Header<'b>, usize)> {
        // Frame control field
        let (off, fcf_be) = dec_try!(buf; decode_u16);
        let fcf = u16::from_be(fcf_be);

        // In order of least significant bits first
        let frame_type = stream_from_option!(FrameType::from_fcf(fcf));
        let security_enabled = (fcf & frame_control::SECURITY_ENABLED) != 0;
        let frame_pending = (fcf & frame_control::FRAME_PENDING) != 0;
        let ack_requested = (fcf & frame_control::ACK_REQUESTED) != 0;
        let pan_id_compression = (fcf & frame_control::PAN_ID_COMPRESSION) != 0;
        let seq_suppressed = (fcf & frame_control::SEQ_SUPPRESSED) != 0;
        let ie_present = (fcf & frame_control::IE_PRESENT) != 0;
        let dst_mode = {
            let mode = (fcf >> frame_control::DST_MODE_POS) & frame_control::MODE_MASK;
            stream_from_option!(AddressMode::from_mode(mode))
        };
        let version = stream_from_option!(FrameVersion::from_fcf(fcf));
        let src_mode = {
            let mode = (fcf >> frame_control::SRC_MODE_POS) & frame_control::MODE_MASK;
            stream_from_option!(AddressMode::from_mode(mode))
        };

        // Sequence number
        let (off, seq) = if !seq_suppressed {
            let (off, seq) = dec_try!(buf, off; decode_u8);
            (off, Some(seq))
        } else {
            (off, None)
        };

        // Addressing fields
        let (off, (dst_pan, dst_addr, src_pan, src_addr)) = dec_try!(buf, off;
                                                                     Self::decode_addressing,
                                                                     version,
                                                                     dst_mode,
                                                                     src_mode,
                                                                     pan_id_compression);

        // Auxiliary security header
        let (mut off, security) = if security_enabled {
            let (off, security) = dec_try!(buf, off; Security::decode);
            (off, Some(security))
        } else {
            (off, None)
        };

        // Information elements
        let mut header_ies: [HeaderIE<'b>; MAX_HEADER_IES] = Default::default();
        let mut header_ies_len = 0;
        let mut payload_ies: [PayloadIE<'b>; MAX_PAYLOAD_IES] = Default::default();
        let mut payload_ies_len = 0;

        let mut has_payload_ies = false;
        if ie_present {
            loop {
                let (next_off, ie) = dec_try!(buf, off; HeaderIE::decode);
                off = next_off;
                match ie {
                    HeaderIE::Termination1 => {
                        has_payload_ies = true;
                        break;
                    }
                    HeaderIE::Termination2 => {
                        break;
                    }
                    other_ie => {
                        stream_cond!(header_ies_len + 1 < MAX_HEADER_IES);
                        header_ies[header_ies_len] = other_ie;
                        header_ies_len += 1;
                    }
                }
            }
        }
        // The MAC payload includes the payload IEs. We can only parse them if
        // the frame is not encrypted.
        let mac_payload_off = off;
        let unencrypted = unsecured || !security_enabled;
        if has_payload_ies && unencrypted {
            loop {
                let (next_off, ie) = dec_try!(buf, off; PayloadIE::decode);
                off = next_off;
                match ie {
                    PayloadIE::Termination => {
                        break;
                    }
                    other_ie => {
                        stream_cond!(payload_ies_len + 1 < MAX_PAYLOAD_IES);
                        payload_ies[payload_ies_len] = other_ie;
                        payload_ies_len += 1;
                    }
                }
            }
        }

        stream_done!(
            off,
            (
                Header {
                    frame_type: frame_type,
                    frame_pending: frame_pending,
                    ack_requested: ack_requested,
                    version: version,
                    seq: seq,
                    dst_pan: dst_pan,
                    dst_addr: dst_addr,
                    src_pan: src_pan,
                    src_addr: src_addr,
                    security: security,
                    header_ies: header_ies,
                    header_ies_len: header_ies_len,
                    payload_ies: payload_ies,
                    payload_ies_len: payload_ies_len,
                },
                mac_payload_off
            )
        );
    }

    pub fn decode_addressing(
        buf: &[u8],
        version: FrameVersion,
        dst_mode: AddressMode,
        src_mode: AddressMode,
        pan_id_compression: bool,
    ) -> SResult<(
        Option<PanID>,
        Option<MacAddress>,
        Option<PanID>,
        Option<MacAddress>,
    )> {
        // IEEE 802.15.4: Section 7.2.1.5
        // Whether or not the addresses are included is determined by the mode
        // fields in the frame control field, but the presence of pan IDs
        // depends on the pan ID compression field and the frame version
        let mut src_pan_dropped = false;
        let dst_present = dst_mode != AddressMode::NotPresent;
        let src_present = src_mode != AddressMode::NotPresent;
        let (dst_pan_present, src_pan_present) = match version {
            FrameVersion::V2015 => {
                // Everything is determined by Table 7-2 in the spec
                match (dst_present, src_present) {
                    (false, false) => (pan_id_compression, false),
                    (true, false) => (!pan_id_compression, false),
                    (false, true) => (false, !pan_id_compression),
                    (true, true) => {
                        src_pan_dropped = pan_id_compression;
                        (true, !pan_id_compression)
                    }
                }
            }
            FrameVersion::V2003 | FrameVersion::V2006 => {
                // In these two modes, pan IDs are specified if addresses are
                // specified, except when the source pan matches the destination
                // pan, in which case the source pan is omitted and the pan ID
                // compression flag is set
                src_pan_dropped = pan_id_compression;
                (dst_present, src_present && !src_pan_dropped)
            }
        };

        let off = 0;
        let (off, dst_pan) = if dst_pan_present {
            let (off, pan_be) = dec_try!(buf, off; decode_u16);
            (off, Some(u16::from_be(pan_be)))
        } else {
            (off, None)
        };
        let (off, dst_addr) = dec_try!(buf, off; MacAddress::decode, dst_mode);
        let (off, src_pan) = if src_pan_present {
            let (off, pan_be) = dec_try!(buf, off; decode_u16);
            (off, Some(u16::from_be(pan_be)))
        } else {
            if src_pan_dropped {
                // If the src pan has been omitted as set above, then the dst
                // pan must be present, otherwise the modes and compression flag
                // were set wrong and the header is invalid
                (off, Some(stream_from_option!(dst_pan)))
            } else {
                (off, None)
            }
        };
        let (off, src_addr) = dec_try!(buf, off; MacAddress::decode, src_mode);

        stream_done!(off, (dst_pan, dst_addr, src_pan, src_addr));
    }
}
