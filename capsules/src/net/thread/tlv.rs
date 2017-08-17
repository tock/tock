//! Implements Type-Length-Value (TLV) encoding and decoding as outlined
//! in the Thread 1.1.1 Specification. TLVs are used to serialize
//! information exchanged during mesh link establishment (MLE). MLE is
//! covered in Chapter 4.
//!
//! MLE messages consist of a command type and a series of TLV parameters.
//!
//! This module, as it stands, implements the minimum subset of TLVs
//! required to support MLE for attaching a Sleepy End Device (SED) to a
//! Thread network.
//!
//! MLE for network attaching comprises a four-step handshake that works
//! as follows:
//!     1. A child device multicasts a Parent Request MLE command.
//!     2. Each potential parent device on the network unicasts a Parent
//!        Response MLE command.
//!     3. The child device selects a parent based on a hierarchy of
//!        connectivity metrics and unicasts a Child ID Request MLE
//!        command.
//!     4. The selected parent unicasts a Child ID Response MLE command.
//!
//! A TLV is comprised of three parts:
//!     1. Type   - A one-byte TLV type number.
//!     2. Length - A one-byte number representing the length of the TLV
//!                 value in bytes.
//!     3. Value  - The TLV value.
//!
//! For some TLVs, the TLV type number is shifted left by one to leave the
//! least significant bit to denote whether information in the TLV value
//! is stable. Stable network data is data that is expected to be stable
//! over weeks or months (Section 5.14).
//!
//! TLVs can be nested within a TLV value. Some types of Network Data
//! TLVs, for example, contain sub-TLVs inside of their TLV value.
//!
//! To simplify variable-length TLV value decoding in Rust, TLV values are
//! assumed to have a maximum length of 128 bytes. This assumption is made
//! only when a variable-length value must be decoded from network byte
//! order before it can be interpreted correctly. Excluded from this case
//! are variable-length values that contain data that must later be
//! decoded by the caller before being interpreted (for example,
//! sub-TLVs). Such a value is instead returned as a slice of the original
//! buffer passed to the decode function.
//!
//!
//! Author: Mateo Garcia
//!         mateog@stanford.edu


// TODO: Move the MLE explanation above to the MLE module, when it is created.


// NOTES FOR DEBUGGING:
// - .to_be() may not have been called on values wider than one byte
// - encode_bytes_be may have been used instead of encode_bytes
// - decode_bytes_be may have been used instead of decode_bytes
// - See 4.5.25 Active Operational Dataset TLV and 4.5.26 Pending Operational Dataset TLV
//    - Are Active and Pending Timestamp TLVs, respectively, required to be sent as well
//      if either of the dataset tlvs are sent?


use core::mem;
use net::stream::{decode_u8, decode_u16, decode_u32, decode_bytes_be};
use net::stream::{encode_u8, encode_u16, encode_u32, encode_bytes, encode_bytes_be};
use net::stream::SResult;

const TL_WIDTH: usize = 2; // Type and length fields of TLV are each one byte.
const MAX_VALUE_FIELD_LENGTH: usize = 128; // Assume a TLV value will be no longer than 128 bytes.

/// Type-Length-Value structure.
pub enum Tlv<'a> {
    SourceAddress(u16),
    Mode(u8),
    Timeout(u32),
    Challenge([u8; 8]), // Byte string max length 8 bytes.
    Response([u8; 8]), // Byte string max length 8 bytes.
    LinkLayerFrameCounter(u32),
    // LinkQuality,                  // TLV type Not used in Thread
    // NetworkParameter,             // TLV type Not used in Thread
    MleFrameCounter(u32),
    /*
    TODO: Not required to implement MLE for SED
    Route64,
    */
    Address16(u16),
    LeaderData {
        partition_id: u32,
        weighting: u8,
        data_version: u8,
        stable_data_version: u8,
        leader_router_id: u8,
    },
    NetworkData(&'a [u8]),
    TlvRequest(&'a [u8]),
    ScanMask(u8),
    Connectivity {
        parent_priority: u8,
        link_quality_3: u8,
        link_quality_2: u8,
        link_quality_1: u8,
        leader_cost: u8,
        id_sequence: u8,
        active_routers: u8,
        sed_buffer_size: Option<u16>,
        sed_datagram_count: Option<u8>,
    },
    LinkMargin(u8),
    Status(u8),
    Version(u16),
    /*
    TODO: Not required to implement MLE for SED
    AddressRegistration
    AddressRegistration
    Channel
    PanId
    ActiveTimestamp
    PendingTimestamp
    ThreadDiscovery
    */
    ActiveOperationalDataset(&'a [u8]),
    PendingOperationalDataset(&'a [u8]),
}

impl<'a> Tlv<'a> {
    /// Serializes TLV data in `buf` into the format specific to the TLV
    /// type.
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        match *self {
            Tlv::SourceAddress(ref mac_address) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, mac_address.to_be());
                stream_done!(offset)
            }
            Tlv::Mode(ref mode) => {
                let value_width = mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u8, *mode);
                stream_done!(offset)
            }
            Tlv::Timeout(ref max_transmit_interval) => {
                let value_width = mem::size_of::<u32>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u32, max_transmit_interval.to_be());
                stream_done!(offset)
            }
            Tlv::Challenge(ref byte_str) => {
                let value_width = byte_str.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, byte_str);
                stream_done!(offset)
            }
            Tlv::Response(ref byte_str) => {
                let value_width = byte_str.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, byte_str);
                stream_done!(offset)
            }
            Tlv::LinkLayerFrameCounter(ref frame_counter) => {
                let value_width = mem::size_of::<u32>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u32, frame_counter.to_be());
                stream_done!(offset)
            }
            Tlv::MleFrameCounter(ref frame_counter) => {
                let value_width = mem::size_of::<u32>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u32, frame_counter.to_be());
                stream_done!(offset)
            }
            Tlv::Address16(ref mac_address) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, mac_address.to_be());
                stream_done!(offset)
            }
            Tlv::LeaderData { partition_id,
                              weighting,
                              data_version,
                              stable_data_version,
                              leader_router_id } => {
                let value_width =
                    mem::size_of::<u32>() + mem::size_of::<u8>() + mem::size_of::<u8>() +
                    mem::size_of::<u8>() + mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u32, partition_id.to_be());
                offset = enc_consume!(buf, offset; encode_u8, weighting);
                offset = enc_consume!(buf, offset; encode_u8, data_version);
                offset = enc_consume!(buf, offset; encode_u8, stable_data_version);
                offset = enc_consume!(buf, offset; encode_u8, leader_router_id);
                stream_done!(offset)
            }
            Tlv::NetworkData(ref network_data_tlvs) => {
                let value_width = network_data_tlvs.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes, network_data_tlvs);
                stream_done!(offset)
            }
            Tlv::TlvRequest(ref tlv_codes) => {
                let value_width = tlv_codes.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes, tlv_codes);
                stream_done!(offset)
            }
            Tlv::ScanMask(ref scan_mask) => {
                let value_width = mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u8, *scan_mask);
                stream_done!(offset)
            }
            Tlv::Connectivity { parent_priority,
                                link_quality_3,
                                link_quality_2,
                                link_quality_1,
                                leader_cost,
                                id_sequence,
                                active_routers,
                                sed_buffer_size,
                                sed_datagram_count } => {
                let base_width =
                    mem::size_of::<u8>() + mem::size_of::<u8>() + mem::size_of::<u8>() +
                    mem::size_of::<u8>() + mem::size_of::<u8>() +
                    mem::size_of::<u8>() + mem::size_of::<u8>();
                let sed_buf_size_width = match sed_buffer_size {
                    None => 0,
                    Some(_) => mem::size_of::<u16>(),
                };
                let sed_datagram_cnt_width = match sed_datagram_count {
                    None => 0,
                    Some(_) => mem::size_of::<u8>(),
                };
                let value_width = base_width + sed_buf_size_width + sed_datagram_cnt_width;
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u8, parent_priority);
                offset = enc_consume!(buf, offset; encode_u8, link_quality_3);
                offset = enc_consume!(buf, offset; encode_u8, link_quality_2);
                offset = enc_consume!(buf, offset; encode_u8, link_quality_1);
                offset = enc_consume!(buf, offset; encode_u8, leader_cost);
                offset = enc_consume!(buf, offset; encode_u8, id_sequence);
                offset = enc_consume!(buf, offset; encode_u8, active_routers);
                if let Some(ref buf_size) = sed_buffer_size {
                    offset = enc_consume!(buf, offset; encode_u16, buf_size.to_be());
                }
                if let Some(ref datagram_cnt) = sed_datagram_count {
                    offset = enc_consume!(buf, offset; encode_u8, *datagram_cnt);
                }
                stream_done!(offset)
            }
            Tlv::LinkMargin(ref link_margin) => {
                let value_width = mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u8, *link_margin);
                stream_done!(offset)
            }
            Tlv::Status(ref status) => {
                let value_width = mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u8, *status);
                stream_done!(offset)
            }
            Tlv::Version(ref version) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, *version);
                stream_done!(offset)
            }
            Tlv::ActiveOperationalDataset(ref network_mgmt_tlvs) => {
                let value_width = network_mgmt_tlvs.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes, network_mgmt_tlvs);
                stream_done!(offset)
            }
            Tlv::PendingOperationalDataset(ref network_mgmt_tlvs) => {
                let value_width = network_mgmt_tlvs.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes, network_mgmt_tlvs);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        buf[0] = TlvType::from(self) as u8;
        buf[1] = value_width as u8;
        stream_done!(TL_WIDTH)
    }

    /// Deserializes TLV data from `buf` into the TLV variant specific to
    /// the TLV type.
    /// `SResult::Error` is returned if the type field does not match any
    /// implemented TLV type.
    pub fn decode(buf: &[u8]) -> SResult<Tlv> {
        let (offset, tlv_type) = dec_try!(buf; decode_u8);
        let tlv_type = TlvType::from(tlv_type);
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            TlvType::SourceAddress => {
                let (offset, mac_address) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, Tlv::SourceAddress(mac_address))
            }
            TlvType::Mode => {
                let (offset, mode) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset, Tlv::Mode(mode))
            }
            TlvType::Timeout => {
                let (offset, max_transmit_interval) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, Tlv::Timeout(max_transmit_interval))
            }
            TlvType::Challenge => {
                let mut byte_str = [0u8; 8];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut byte_str);
                stream_done!(offset, Tlv::Challenge(byte_str))
            }
            TlvType::Response => {
                let mut byte_str = [0u8; 8];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut byte_str);
                stream_done!(offset, Tlv::Response(byte_str))
            }
            TlvType::LinkLayerFrameCounter => {
                let (offset, frame_counter) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, Tlv::LinkLayerFrameCounter(frame_counter))
            }
            TlvType::MleFrameCounter => {
                let (offset, frame_counter) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, Tlv::MleFrameCounter(frame_counter))
            }
            TlvType::Address16 => {
                let (offset, mac_address) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, Tlv::Address16(mac_address))
            }
            TlvType::LeaderData => {
                let (offset, partition_id) = dec_try!(buf, offset; decode_u32);
                let (offset, weighting) = dec_try!(buf, offset; decode_u8);
                let (offset, data_version) = dec_try!(buf, offset; decode_u8);
                let (offset, stable_data_version) = dec_try!(buf, offset; decode_u8);
                let (offset, leader_router_id) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             Tlv::LeaderData {
                                 partition_id: partition_id,
                                 weighting: weighting,
                                 data_version: data_version,
                                 stable_data_version: stable_data_version,
                                 leader_router_id: leader_router_id,
                             })
            }
            TlvType::NetworkData => {
                stream_done!(offset + length as usize,
                             Tlv::NetworkData(&buf[offset..offset + length as usize]))
            }
            TlvType::TlvRequest => {
                stream_done!(offset + length as usize,
                             Tlv::TlvRequest(&buf[offset..offset + length as usize]))
            }
            TlvType::ScanMask => {
                let (offset, scan_mask) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset, Tlv::ScanMask(scan_mask))
            }
            TlvType::Connectivity => {
                let (offset, parent_priority) = dec_try!(buf, offset; decode_u8);
                let (offset, link_quality_3) = dec_try!(buf, offset; decode_u8);
                let (offset, link_quality_2) = dec_try!(buf, offset; decode_u8);
                let (offset, link_quality_1) = dec_try!(buf, offset; decode_u8);
                let (offset, leader_cost) = dec_try!(buf, offset; decode_u8);
                let (offset, id_sequence) = dec_try!(buf, offset; decode_u8);
                let (offset, active_routers) = dec_try!(buf, offset; decode_u8);
                let mut offset = offset;
                let mut sed_buffer_size = None;
                if offset + mem::size_of::<u16>() < length as usize {
                    let (new_offset, sed_buffer_size_raw) = dec_try!(buf, offset; decode_u16);
                    offset = new_offset;
                    sed_buffer_size = Some(sed_buffer_size_raw);
                }
                let mut sed_datagram_count = None;
                if offset + mem::size_of::<u8>() < length as usize {
                    let (new_offset, sed_datagram_count_raw) = dec_try!(buf, offset; decode_u8);
                    offset = new_offset;
                    sed_datagram_count = Some(sed_datagram_count_raw);
                }
                stream_done!(offset,
                             Tlv::Connectivity {
                                 parent_priority: parent_priority,
                                 link_quality_3: link_quality_3,
                                 link_quality_2: link_quality_2,
                                 link_quality_1: link_quality_1,
                                 leader_cost: leader_cost,
                                 id_sequence: id_sequence,
                                 active_routers: active_routers,
                                 sed_buffer_size: sed_buffer_size,
                                 sed_datagram_count: sed_datagram_count,
                             })
            }
            TlvType::LinkMargin => {
                let (offset, link_margin) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset, Tlv::LinkMargin(link_margin))
            }
            TlvType::Status => {
                let (offset, status) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset, Tlv::Status(status))
            }
            TlvType::Version => {
                let (offset, version) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, Tlv::Version(version))
            }
            TlvType::ActiveOperationalDataset => {
                stream_done!(offset + length as usize,
                             Tlv::ActiveOperationalDataset(&buf[offset..offset + length as usize]))
            }
            TlvType::PendingOperationalDataset => {
                stream_done!(offset + length as usize,
                             Tlv::PendingOperationalDataset(&buf[offset..offset + length as usize]))
            }
            TlvType::NotPresent => stream_err!(),
        }
    }
}

/// Value encoded in the type field of a Type-Length-Value (TLV)
/// structure.
#[repr(u8)]
pub enum TlvType {
    SourceAddress = 0,
    Mode = 1,
    Timeout = 2,
    Challenge = 3,
    Response = 4,
    LinkLayerFrameCounter = 5,
    // LinkQuality = 6,         // TLV type not used in Thread
    // NetworkParameter = 7,    // TLV type not used in Thread
    MleFrameCounter = 8,
    /*
    TODO: Not required to implement MLE for SED
    Route64 = 9,
    */
    Address16 = 10,
    LeaderData = 11,
    NetworkData = 12,
    TlvRequest = 13,
    ScanMask = 14,
    Connectivity = 15,
    LinkMargin = 16,
    Status = 17,
    Version = 18,
    /*
    TODO: Not required to implement MLE for SED
    AddressRegistration = 19,
    Channel = 20,
    PanId = 21,
    ActiveTimestamp = 22,
    PendingTimestamp = 23,
    */
    ActiveOperationalDataset = 24,
    PendingOperationalDataset = 25,
    /*
    TODO: Not required to implement MLE for SED
    ThreadDiscovery = 26,
    */
    NotPresent,
}

impl From<u8> for TlvType {
    fn from(tlv_type: u8) -> Self {
        match tlv_type {
            0 => TlvType::SourceAddress,
            1 => TlvType::Mode,
            2 => TlvType::Timeout,
            3 => TlvType::Challenge,
            4 => TlvType::Response,
            5 => TlvType::LinkLayerFrameCounter,
            8 => TlvType::MleFrameCounter,
            10 => TlvType::Address16,
            11 => TlvType::LeaderData,
            12 => TlvType::NetworkData,
            13 => TlvType::TlvRequest,
            14 => TlvType::ScanMask,
            15 => TlvType::Connectivity,
            16 => TlvType::LinkMargin,
            17 => TlvType::Status,
            18 => TlvType::Version,
            24 => TlvType::ActiveOperationalDataset,
            25 => TlvType::PendingOperationalDataset,
            _ => TlvType::NotPresent,
        }
    }
}

impl<'a, 'b> From<&'a Tlv<'b>> for TlvType {
    fn from(tlv: &'a Tlv<'b>) -> Self {
        match *tlv {
            Tlv::SourceAddress(_) => TlvType::SourceAddress,
            Tlv::Mode(_) => TlvType::Mode,
            Tlv::Timeout(_) => TlvType::Timeout,
            Tlv::Challenge(_) => TlvType::Challenge,
            Tlv::Response(_) => TlvType::Response,
            Tlv::LinkLayerFrameCounter(_) => TlvType::LinkLayerFrameCounter,
            Tlv::MleFrameCounter(_) => TlvType::MleFrameCounter,
            Tlv::Address16(_) => TlvType::Address16,
            Tlv::LeaderData { .. } => TlvType::LeaderData,
            Tlv::NetworkData(_) => TlvType::NetworkData,
            Tlv::TlvRequest(_) => TlvType::TlvRequest,
            Tlv::ScanMask(_) => TlvType::ScanMask,
            Tlv::Connectivity { .. } => TlvType::Connectivity,
            Tlv::LinkMargin(_) => TlvType::LinkMargin,
            Tlv::Status(_) => TlvType::Status,
            Tlv::Version(_) => TlvType::Version,
            Tlv::ActiveOperationalDataset(_) => TlvType::ActiveOperationalDataset,
            Tlv::PendingOperationalDataset(_) => TlvType::PendingOperationalDataset,
        }
    }
}

/// Used in Mode TLV.
#[repr(u8)]
pub enum LinkMode {
    ReceiverOnWhenIdle = 0b0000_1000,
    SecureDataRequests = 0b0000_0100,
    FullThreadDevice = 0b0000_0010,
    FullNetworkDataRequired = 0b0000_0001,
}

/// Used in Scan Mask TLV.
#[repr(u8)]
pub enum MulticastResponder {
    Router = 0b1000_0000,
    EndDevice = 0b0100_0000,
}

/// Used in Connectivity TLV.
pub enum ParentPriority {
    // Reserved = 0b1000_0000
    High = 0b0100_0000,
    Medium = 0b0000_0000,
    Low = 0b1100_0000,
}

/// These TLVs are contained within the value of a Network Data TLV.
/// See Section 5.18.
pub enum NetworkDataTlv<'a> {
    Prefix {
        domain_id: u8,
        prefix_length_bits: u8,
        prefix: [u8; 3], // IPv6 prefix max length 48 bits.
        sub_tlvs: &'a [u8],
    },
    CommissioningData {
        com_length: u8,
        com_data: [u8; MAX_VALUE_FIELD_LENGTH],
    },
    Service {
        thread_enterprise_number: bool,
        // See 5.18.6.
        s_id: u8,
        s_enterprise_number: u32,
        s_service_data_length: u8,
        s_service_data: [u8; MAX_VALUE_FIELD_LENGTH],
        sub_tlvs: &'a [u8],
    },
}

impl<'a> NetworkDataTlv<'a> {
    /// Serializes TLV data in `buf` into the format specific to the
    /// Network Data TLV type.
    pub fn encode(&self, buf: &mut [u8], stable: bool) -> SResult {
        match *self {
            NetworkDataTlv::Prefix { domain_id, prefix_length_bits, prefix, sub_tlvs } => {
                let value_width = mem::size_of::<u8>() + mem::size_of::<u8>() + prefix.len() +
                                  sub_tlvs.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                offset = enc_consume!(buf, offset; encode_u8, domain_id);
                offset = enc_consume!(buf, offset; encode_u8, prefix_length_bits);
                offset = enc_consume!(buf, offset; encode_bytes_be, &prefix);
                offset = enc_consume!(buf, offset; encode_bytes, sub_tlvs);
                stream_done!(offset)
            }
            NetworkDataTlv::CommissioningData { com_length, com_data } => {
                let value_width = com_length as usize;
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                offset = enc_consume!(buf, offset; encode_bytes_be, &com_data);
                stream_done!(offset)
            }
            NetworkDataTlv::Service { thread_enterprise_number,
                                      s_id,
                                      s_enterprise_number,
                                      s_service_data_length,
                                      s_service_data,
                                      sub_tlvs } => {
                let value_width =
                    mem::size_of::<u8>() + mem::size_of::<u32>() + mem::size_of::<u8>() +
                    s_service_data.len() + sub_tlvs.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                let t_bit: u8 = if thread_enterprise_number {
                    1u8 << 7
                } else {
                    0
                };
                let first_byte: u8 = t_bit | (0b1111 & s_id);
                offset = enc_consume!(buf, offset; encode_u8, first_byte);
                offset = enc_consume!(buf, offset; encode_u32, s_enterprise_number.to_be());
                offset = enc_consume!(buf, offset; encode_u8, s_service_data_length);
                offset = enc_consume!(buf, offset; encode_bytes_be, &s_service_data);
                offset = enc_consume!(buf, offset; encode_bytes, sub_tlvs);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize, stable: bool) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        let stable_bit = if stable { 1u8 } else { 0u8 };
        buf[0] = (NetworkDataTlvType::from(self) as u8) << 1 | stable_bit;
        buf[1] = value_width as u8;
        stream_done!(TL_WIDTH)
    }

    /// Deserializes TLV data from `buf` into the Network Data TLV variant
    /// specific to the TLV type.
    /// Returns NetworkDataTlv and true if the data stable, false
    /// otherwise.
    /// `SResult::Error` is returned if the type field does not match any
    /// implemented TLV type.
    pub fn decode(buf: &[u8]) -> SResult<(NetworkDataTlv, bool)> {
        let (offset, tlv_type_field) = dec_try!(buf; decode_u8);
        let tlv_type_raw = tlv_type_field >> 1;
        let tlv_type = NetworkDataTlvType::from(tlv_type_raw);
        let stable = (tlv_type_field & 1u8) > 0;
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            NetworkDataTlvType::Prefix => {
                let (offset, domain_id) = dec_try!(buf, offset; decode_u8);
                let (offset, prefix_length_bits) = dec_try!(buf, offset; decode_u8);
                let mut prefix = [0u8; 3];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut prefix);
                stream_done!(offset + length as usize,
                             (NetworkDataTlv::Prefix {
                                  domain_id: domain_id,
                                  prefix_length_bits: prefix_length_bits,
                                  prefix: prefix,
                                  sub_tlvs: &buf[offset..offset + length as usize],
                              },
                              stable))
            }
            NetworkDataTlvType::CommissioningData => {
                let (offset, com_length) = dec_try!(buf, offset; decode_u8);
                let mut com_data = [0u8; MAX_VALUE_FIELD_LENGTH];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut com_data);
                stream_done!(offset,
                             (NetworkDataTlv::CommissioningData {
                                  com_length: com_length,
                                  com_data: com_data,
                              },
                              stable))
            }
            NetworkDataTlvType::Service => {
                let (offset, first_byte) = dec_try!(buf, offset; decode_u8);
                let thread_enterprise_number = (first_byte >> 7) > 0;
                let s_id = first_byte & 0b1111;
                let (offset, s_enterprise_number) = dec_try!(buf, offset; decode_u32);
                let (offset, s_service_data_length) = dec_try!(buf, offset; decode_u8);
                let mut s_service_data = [0u8; MAX_VALUE_FIELD_LENGTH];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut s_service_data);
                stream_done!(offset + length as usize,
                             (NetworkDataTlv::Service {
                                  thread_enterprise_number: thread_enterprise_number,
                                  s_id: s_id,
                                  s_enterprise_number: s_enterprise_number,
                                  s_service_data_length: s_service_data_length,
                                  s_service_data: s_service_data,
                                  sub_tlvs: &buf[offset..offset + length as usize],
                              },
                              stable))
            }
            NetworkDataTlvType::NotPresent => stream_err!(),
        }
    }
}

/// Value encoded in the type field of a Network Data TLV.
/// Gaps in type numbers are filled by PrefixSubTlv and ServiceSubTlv.
#[repr(u8)]
pub enum NetworkDataTlvType {
    Prefix = 1,
    CommissioningData = 4,
    Service = 5,
    NotPresent,
}

impl From<u8> for NetworkDataTlvType {
    fn from(tlv_type: u8) -> Self {
        match tlv_type {
            1 => NetworkDataTlvType::Prefix,
            4 => NetworkDataTlvType::CommissioningData,
            5 => NetworkDataTlvType::Service,
            _ => NetworkDataTlvType::NotPresent,
        }
    }
}

impl<'a, 'b> From<&'a NetworkDataTlv<'b>> for NetworkDataTlvType {
    fn from(network_data_tlv: &'a NetworkDataTlv<'b>) -> Self {
        match *network_data_tlv {
            NetworkDataTlv::Prefix { .. } => NetworkDataTlvType::Prefix,
            NetworkDataTlv::CommissioningData { .. } => NetworkDataTlvType::CommissioningData,
            NetworkDataTlv::Service { .. } => NetworkDataTlvType::Service,
        }
    }
}

/// These TLVs are contained within the value of a Prefix TLV.
pub enum PrefixSubTlv<'a> {
    HasRoute(&'a [u8]),
    BorderRouter(&'a [u8]),
    SixLoWpanId {
        context_id_compress: bool,
        context_id: u8,
        context_length: u8,
    },
}

impl<'a> PrefixSubTlv<'a> {
    /// Serializes TLV data in `buf` into the format specific to the
    /// Prefix sub-TLV type.
    pub fn encode(&self, buf: &mut [u8], stable: bool) -> SResult {
        match *self {
            PrefixSubTlv::HasRoute(ref r_border_router_16s) => {
                let value_width = r_border_router_16s.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                offset = enc_consume!(buf, offset; encode_bytes, r_border_router_16s);
                stream_done!(offset)
            }
            PrefixSubTlv::BorderRouter(ref p_border_router_16s) => {
                let value_width = p_border_router_16s.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                offset = enc_consume!(buf, offset; encode_bytes, p_border_router_16s);
                stream_done!(offset)
            }
            PrefixSubTlv::SixLoWpanId { context_id_compress, context_id, context_length } => {
                let value_width = mem::size_of::<u8>() + mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                let compress_bit = if context_id_compress { 1u8 } else { 0u8 };
                let first_byte = (compress_bit << 4) | (context_id & 0b1111);
                offset = enc_consume!(buf, offset; encode_u8, first_byte);
                offset = enc_consume!(buf, offset; encode_u8, context_length);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize, stable: bool) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        let stable_bit = if stable { 1u8 } else { 0u8 };
        buf[0] = (PrefixSubTlvType::from(self) as u8) << 1 | stable_bit;
        buf[1] = value_width as u8;
        stream_done!(TL_WIDTH)
    }

    /// Deserializes TLV data from `buf` into the Prefix sub-TLV variant
    /// specific to the TLV type.
    /// Returns PrefixSubTlv and true if the data stable, false
    /// otherwise.
    /// `SResult::Error` is returned if the type field does not match any
    /// implemented TLV type.
    pub fn decode(buf: &[u8]) -> SResult<(PrefixSubTlv, bool)> {
        let (offset, tlv_type_field) = dec_try!(buf; decode_u8);
        let tlv_type_raw = tlv_type_field >> 1;
        let tlv_type = PrefixSubTlvType::from(tlv_type_raw);
        let stable = (tlv_type_field & 1u8) > 0;
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            PrefixSubTlvType::HasRoute => {
                stream_done!(offset + length as usize,
                             (PrefixSubTlv::HasRoute(&buf[offset..offset + length as usize]),
                              stable))
            }
            PrefixSubTlvType::BorderRouter => {
                stream_done!(offset + length as usize,
                             (PrefixSubTlv::BorderRouter(&buf[offset..offset + length as usize]),
                              stable))
            }
            PrefixSubTlvType::SixLoWpanId => {
                let (offset, first_byte) = dec_try!(buf, offset; decode_u8);
                let context_id_compress = (first_byte & 0b1_0000) > 0;
                let context_id = first_byte & 0b1111;
                let (offset, context_length) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             (PrefixSubTlv::SixLoWpanId {
                                  context_id_compress: context_id_compress,
                                  context_id: context_id,
                                  context_length: context_length,
                              },
                              stable))
            }
            PrefixSubTlvType::NotPresent => stream_err!(),
        }
    }
}

/// Value encoded in the type field of a Prefix sub-TLV.
/// Gaps in type numbers are filled by NetworkDataTlv and ServiceSubTlv.
#[repr(u8)]
pub enum PrefixSubTlvType {
    HasRoute = 0,
    BorderRouter = 2,
    SixLoWpanId = 3,
    NotPresent,
}

impl From<u8> for PrefixSubTlvType {
    fn from(tlv_type: u8) -> Self {
        match tlv_type {
            0 => PrefixSubTlvType::HasRoute,
            2 => PrefixSubTlvType::BorderRouter,
            3 => PrefixSubTlvType::SixLoWpanId,
            _ => PrefixSubTlvType::NotPresent,
        }
    }
}

impl<'a, 'b> From<&'a PrefixSubTlv<'b>> for PrefixSubTlvType {
    fn from(prefix_sub_tlv: &'a PrefixSubTlv<'b>) -> Self {
        match *prefix_sub_tlv {
            PrefixSubTlv::HasRoute(_) => PrefixSubTlvType::HasRoute,
            PrefixSubTlv::BorderRouter(_) => PrefixSubTlvType::BorderRouter,
            PrefixSubTlv::SixLoWpanId { .. } => PrefixSubTlvType::SixLoWpanId,
        }
    }
}

/// Used in Has Route TLV.
pub struct HasRouteTlvValue {
    // See 5.18.1.
    r_border_router_16: u16,
    r_preference: u8,
}

impl HasRouteTlvValue {
    /// Serializes this Has Route TLV value into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        stream_len_cond!(buf, 3);
        let mut offset = enc_consume!(buf, 0; encode_u16, self.r_border_router_16.to_be());
        let last_byte = ((self.r_preference & 0b11) as u8) << 6;
        offset = enc_consume!(buf, offset; encode_u8, last_byte);
        stream_done!(offset)
    }

    /// Deserializes Has Route TLV value from `buf` and returns it.
    pub fn decode(buf: &[u8]) -> SResult<HasRouteTlvValue> {
        stream_len_cond!(buf, 3);
        let (offset, r_border_router_16) = dec_try!(buf; decode_u16);
        let (offset, last_byte) = dec_try!(buf, offset; decode_u8);
        let r_preference = last_byte >> 6;
        stream_done!(offset,
                     HasRouteTlvValue {
                         r_border_router_16: r_border_router_16,
                         r_preference: r_preference,
                     })
    }
}

/// Used in Border Router TLV.
pub struct BorderRouterTlvValue {
    // See 5.18.3.
    p_border_router_16: u16,
    p_bits: u16,
}

/// Used in Border Router TLV value.
#[repr(u16)]
pub enum BorderRouterTlvValueBit {
    // See 5.18.3 for a more detailed explanation of each.
    Prf = 0b1100_0000_0000_0000, // Preference
    P = 0b0010_0000_0000_0000, // Preferred
    S = 0b0001_0000_0000_0000, // SLAAC
    D = 0b0000_1000_0000_0000, // DHCP
    C = 0b0000_0100_0000_0000, // Configure
    R = 0b0000_0010_0000_0000, // Default
    O = 0b0000_0001_0000_0000, // On mesh
    N = 0b0000_0000_1000_0000, // NDDNS
}

impl BorderRouterTlvValue {
    /// Serializes this Border Route TLV value into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        stream_len_cond!(buf, 4); // Each Border Router TLV value is 32 bits wide.
        let mut offset = enc_consume!(buf, 0; encode_u16, self.p_border_router_16.to_be());
        offset = enc_consume!(buf, offset; encode_u16, self.p_bits.to_be());
        stream_done!(offset)
    }

    /// Deserializes Border Route TLV value from `buf` and returns it.
    pub fn decode(buf: &[u8]) -> SResult<BorderRouterTlvValue> {
        let (offset, p_border_router_16) = dec_try!(buf; decode_u16);
        let (offset, p_bits) = dec_try!(buf, offset; decode_u16);
        stream_done!(offset,
                     BorderRouterTlvValue {
                         p_border_router_16: p_border_router_16,
                         p_bits: p_bits,
                     })
    }
}

/// These TLVs are contained within the value of a Service TLV.
pub enum ServiceSubTlv {
    Server {
        // See 5.18.6.
        s_server_16: u16,
        s_server_data: [u8; MAX_VALUE_FIELD_LENGTH],
    },
}

impl<'a> ServiceSubTlv {
    /// Serializes TLV data in `buf` into the format specific to the
    /// Service sub-TLV type.
    pub fn encode(&self, buf: &mut [u8], stable: bool) -> SResult {
        match *self {
            ServiceSubTlv::Server { s_server_16, s_server_data } => {
                let value_width = mem::size_of::<u16>() + s_server_data.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width, stable);
                offset = enc_consume!(buf, offset; encode_u16, s_server_16.to_be());
                offset = enc_consume!(buf, offset; encode_bytes_be, &s_server_data);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize, stable: bool) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        let stable_bit = if stable { 1u8 } else { 0u8 };
        buf[0] = (ServiceSubTlvType::from(self) as u8) << 1 | stable_bit;
        buf[1] = value_width as u8;
        stream_done!(TL_WIDTH)
    }

    /// Deserializes TLV data from `buf` into the Service sub-TLV variant
    /// specific to the TLV type.
    /// Returns ServiceSubTlv and true if the data stable, false
    /// otherwise.
    /// `SResult::Error` is returned if the type field does not match any
    /// implemented TLV type.
    pub fn decode(buf: &[u8]) -> SResult<(ServiceSubTlv, bool)> {
        let (offset, tlv_type_field) = dec_try!(buf; decode_u8);
        let tlv_type_raw = tlv_type_field >> 1;
        let tlv_type = ServiceSubTlvType::from(tlv_type_raw);
        let stable = (tlv_type_field & 1u8) > 0;
        let (offset, _) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            ServiceSubTlvType::Server => {
                let (offset, s_server_16) = dec_try!(buf, offset; decode_u16);
                let mut s_server_data = [0u8; MAX_VALUE_FIELD_LENGTH];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut s_server_data);
                stream_done!(offset,
                             (ServiceSubTlv::Server {
                                  s_server_16: s_server_16,
                                  s_server_data: s_server_data,
                              },
                              stable))
            }
            ServiceSubTlvType::NotPresent => stream_err!(),
        }
    }
}

/// Value encoded in the type field of a Service sub-TLV.
/// Gaps in type numbers are filled by NetworkDataTlv and PrefixSubTlv.
#[repr(u8)]
pub enum ServiceSubTlvType {
    Server = 6,
    NotPresent,
}

impl From<u8> for ServiceSubTlvType {
    fn from(tlv_type: u8) -> Self {
        match tlv_type {
            6 => ServiceSubTlvType::Server,
            _ => ServiceSubTlvType::NotPresent,
        }
    }
}

impl<'a> From<&'a ServiceSubTlv> for ServiceSubTlvType {
    fn from(service_sub_tlv: &'a ServiceSubTlv) -> Self {
        match *service_sub_tlv {
            ServiceSubTlv::Server { .. } => ServiceSubTlvType::Server,
        }
    }
}

/// These TLVs are contained within the value of a Pending Operational
/// Dataset TLV or an Active Operational Dataset TLV.
/// See Section 8.10.1.
pub enum NetworkManagementTlv<'a> {
    Channel { channel_page: u8, channel: u16 },
    PanId(u16),
    ExtendedPanId([u8; 8]), // Extended PAN ID length 8 bytes.
    NetworkName([u8; 16]), // Network name max length 16 bytes.
    Pskc([u8; 16]), // PSKc max length 16 bytes.
    NetworkMasterKey([u8; 16]), // Master key length 128 bits = 16 bytes.
    NetworkKeySequenceCounter([u8; 4]), // Counter length 4 bytes.
    NetworkMeshLocalPrefix([u8; 8]), // Mesh-Local Prefix length 8 bytes.
    SteeringData([u8; 16]), // Bloom filter max length 16 bytes.
    BorderAgentLocator(u16),
    CommissionerId([u8; 64]), // Commissioner ID max length 64 bytes.
    CommissionerSessionId(u16),
    SecurityPolicy { rotation_time: u16, policy_bits: u8 },
    ActiveTimestamp {
        timestamp_seconds: [u8; 3], // Timestamp seconds is a 48-bit Unix time value.
        timestamp_ticks: u16,
        u_bit: bool,
    },
    CommissionerUdpPort(u16),
    PendingTimestamp {
        timestamp_seconds: [u8; 3], // Timestamp seconds is a 48-bit Unix time value.
        timestamp_ticks: u16,
        u_bit: bool,
    },
    DelayTimer(u32),
    ChannelMask(&'a [u8]),
}

impl<'a> NetworkManagementTlv<'a> {
    /// Serializes TLV data in `buf` into the format specific to the
    /// Network Management TLV type.
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        match *self {
            NetworkManagementTlv::Channel { channel_page, channel } => {
                // `channel_page` should be 0 (See 8.10.1.1.1)
                // `channel` should be 11-26 (See 8.10.1.1.2)
                let value_width = mem::size_of::<u8>() + mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u8, channel_page);
                offset = enc_consume!(buf, offset; encode_u16, channel.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::PanId(ref pan_id) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, pan_id.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::ExtendedPanId(ref extended_pan_id) => {
                let value_width = extended_pan_id.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, extended_pan_id);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkName(ref network_name) => {
                stream_cond!(network_name.len() <= 16);
                let value_width = network_name.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, network_name);
                stream_done!(offset)
            }
            NetworkManagementTlv::Pskc(ref pskc) => {
                stream_cond!(pskc.len() <= 16);
                let value_width = pskc.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, pskc);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkMasterKey(ref network_key) => {
                let value_width = network_key.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, network_key);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkKeySequenceCounter(ref counter) => {
                let value_width = counter.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, counter);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkMeshLocalPrefix(ref prefix) => {
                let value_width = prefix.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, prefix);
                stream_done!(offset)
            }
            NetworkManagementTlv::SteeringData(ref bloom_filter) => {
                stream_cond!(bloom_filter.len() <= 16);
                let value_width = bloom_filter.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, bloom_filter);
                stream_done!(offset)
            }
            NetworkManagementTlv::BorderAgentLocator(ref rloc_16) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, rloc_16.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::CommissionerId(ref commissioner_id) => {
                stream_cond!(commissioner_id.len() <= 64);
                let value_width = commissioner_id.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, commissioner_id);
                stream_done!(offset)
            }
            NetworkManagementTlv::CommissionerSessionId(ref session_id) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, session_id.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::SecurityPolicy { rotation_time, policy_bits } => {
                let value_width = mem::size_of::<u16>() + mem::size_of::<u8>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, rotation_time.to_be());
                offset = enc_consume!(buf, offset; encode_u8, policy_bits);
                stream_done!(offset)
            }
            NetworkManagementTlv::ActiveTimestamp { timestamp_seconds, timestamp_ticks, u_bit } => {
                let value_width = timestamp_seconds.len() + mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, &timestamp_seconds);
                let u_bit_val = if u_bit { 1u16 } else { 0u16 };
                let end_bytes = (timestamp_ticks << 1) | u_bit_val;
                offset = enc_consume!(buf, offset; encode_u16, end_bytes.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::CommissionerUdpPort(ref udp_port) => {
                let value_width = mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u16, udp_port.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::PendingTimestamp { timestamp_seconds,
                                                     timestamp_ticks,
                                                     u_bit } => {
                let value_width = timestamp_seconds.len() + mem::size_of::<u16>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes_be, &timestamp_seconds);
                let u_bit_val = if u_bit { 1u16 } else { 0u16 };
                let end_bytes = (timestamp_ticks << 1) | u_bit_val;
                offset = enc_consume!(buf, offset; encode_u16, end_bytes.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::DelayTimer(ref time_remaining) => {
                let value_width = mem::size_of::<u32>();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_u32, time_remaining.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::ChannelMask(ref entries) => {
                let value_width = entries.len();
                let mut offset = enc_consume!(buf; self; encode_tl, value_width);
                offset = enc_consume!(buf, offset; encode_bytes, entries);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        buf[0] = NetworkManagementTlvType::from(self) as u8;
        buf[1] = value_width as u8;
        stream_done!(TL_WIDTH)
    }

    /// Deserializes TLV data from `buf` into the Network Management TLV
    /// variant specific to the TLV type.
    /// Returns ServiceSubTlv and true if the data stable, false
    /// otherwise.
    /// `SResult::Error` is returned if the type field does not match any
    /// implemented TLV type.
    pub fn decode(buf: &[u8]) -> SResult<NetworkManagementTlv> {
        let (offset, tlv_type_raw) = dec_try!(buf; decode_u8);
        let tlv_type = NetworkManagementTlvType::from(tlv_type_raw);
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            NetworkManagementTlvType::Channel => {
                let (offset, channel_page) = dec_try!(buf, offset; decode_u8);
                let (offset, channel) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             NetworkManagementTlv::Channel {
                                 channel_page: channel_page,
                                 channel: channel,
                             })
            }
            NetworkManagementTlvType::PanId => {
                let (offset, pan_id) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, NetworkManagementTlv::PanId(pan_id))
            }
            NetworkManagementTlvType::ExtendedPanId => {
                let mut extended_pan_id = [0u8; 8];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut extended_pan_id);
                stream_done!(offset, NetworkManagementTlv::ExtendedPanId(extended_pan_id))
            }
            NetworkManagementTlvType::NetworkName => {
                let mut network_name = [0u8; 16];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut network_name);
                stream_done!(offset, NetworkManagementTlv::NetworkName(network_name))
            }
            NetworkManagementTlvType::Pskc => {
                let mut pskc = [0u8; 16];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut pskc);
                stream_done!(offset, NetworkManagementTlv::Pskc(pskc))
            }
            NetworkManagementTlvType::NetworkMasterKey => {
                let mut network_key = [0u8; 16];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut network_key);
                stream_done!(offset, NetworkManagementTlv::NetworkMasterKey(network_key))
            }
            NetworkManagementTlvType::NetworkKeySequenceCounter => {
                let mut counter = [0u8; 4];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut counter);
                stream_done!(offset,
                             NetworkManagementTlv::NetworkKeySequenceCounter(counter))
            }
            NetworkManagementTlvType::NetworkMeshLocalPrefix => {
                let mut prefix = [0u8; 8];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut prefix);
                stream_done!(offset, NetworkManagementTlv::NetworkMeshLocalPrefix(prefix))
            }
            NetworkManagementTlvType::SteeringData => {
                let mut bloom_filter = [0u8; 16];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut bloom_filter);
                stream_done!(offset, NetworkManagementTlv::SteeringData(bloom_filter))
            }
            NetworkManagementTlvType::BorderAgentLocator => {
                let (offset, rloc_16) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, NetworkManagementTlv::BorderAgentLocator(rloc_16))
            }
            NetworkManagementTlvType::CommissionerId => {
                let mut commissioner_id = [0u8; 64];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut commissioner_id);
                stream_done!(offset,
                             NetworkManagementTlv::CommissionerId(commissioner_id))
            }
            NetworkManagementTlvType::CommissionerSessionId => {
                let (offset, session_id) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             NetworkManagementTlv::CommissionerSessionId(session_id))
            }
            NetworkManagementTlvType::SecurityPolicy => {
                let (offset, rotation_time) = dec_try!(buf, offset; decode_u16);
                let (offset, policy_bits) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             NetworkManagementTlv::SecurityPolicy {
                                 rotation_time: rotation_time,
                                 policy_bits: policy_bits,
                             })
            }
            NetworkManagementTlvType::ActiveTimestamp => {
                let mut timestamp_seconds = [0u8; 3];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut timestamp_seconds);
                let (offset, timestamp_ticks) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             NetworkManagementTlv::ActiveTimestamp {
                                 timestamp_seconds: timestamp_seconds,
                                 timestamp_ticks: timestamp_ticks >> 1,
                                 u_bit: (timestamp_ticks | 1u16) > 0,
                             })
            }
            NetworkManagementTlvType::CommissionerUdpPort => {
                let (offset, udp_port) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, NetworkManagementTlv::CommissionerUdpPort(udp_port))
            }
            NetworkManagementTlvType::PendingTimestamp => {
                let mut timestamp_seconds = [0u8; 3];
                let offset = dec_consume!(buf; decode_bytes_be, &mut timestamp_seconds);
                let (offset, timestamp_ticks) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             NetworkManagementTlv::PendingTimestamp {
                                 timestamp_seconds: timestamp_seconds,
                                 timestamp_ticks: timestamp_ticks >> 1,
                                 u_bit: (timestamp_ticks | 1u16) > 0,
                             })
            }
            NetworkManagementTlvType::DelayTimer => {
                let (offset, time_remaining) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, NetworkManagementTlv::DelayTimer(time_remaining))
            }
            NetworkManagementTlvType::ChannelMask => {
                stream_done!(offset + length as usize,
                             NetworkManagementTlv::ChannelMask(&buf[offset..
                                                                offset + length as usize]))
            }
            NetworkManagementTlvType::NotPresent => stream_err!(),
        }
    }
}

/// Value encoded in the type field of a Network Management TLV.
#[repr(u8)]
pub enum NetworkManagementTlvType {
    Channel = 0,
    PanId = 1,
    ExtendedPanId = 2,
    NetworkName = 3,
    Pskc = 4,
    NetworkMasterKey = 5,
    NetworkKeySequenceCounter = 6,
    NetworkMeshLocalPrefix = 7,
    SteeringData = 8,
    BorderAgentLocator = 9,
    CommissionerId = 10,
    CommissionerSessionId = 11,
    SecurityPolicy = 12,
    ActiveTimestamp = 14,
    CommissionerUdpPort = 15,
    PendingTimestamp = 51,
    DelayTimer = 52,
    ChannelMask = 53,
    NotPresent,
}

impl From<u8> for NetworkManagementTlvType {
    fn from(type_num: u8) -> Self {
        match type_num {
            0 => NetworkManagementTlvType::Channel,
            1 => NetworkManagementTlvType::PanId,
            2 => NetworkManagementTlvType::ExtendedPanId,
            3 => NetworkManagementTlvType::NetworkName,
            4 => NetworkManagementTlvType::Pskc,
            5 => NetworkManagementTlvType::NetworkMasterKey,
            6 => NetworkManagementTlvType::NetworkKeySequenceCounter,
            7 => NetworkManagementTlvType::NetworkMeshLocalPrefix,
            8 => NetworkManagementTlvType::SteeringData,
            9 => NetworkManagementTlvType::BorderAgentLocator,
            10 => NetworkManagementTlvType::CommissionerId,
            11 => NetworkManagementTlvType::CommissionerSessionId,
            12 => NetworkManagementTlvType::SecurityPolicy,
            14 => NetworkManagementTlvType::ActiveTimestamp,
            15 => NetworkManagementTlvType::CommissionerUdpPort,
            51 => NetworkManagementTlvType::PendingTimestamp,
            52 => NetworkManagementTlvType::DelayTimer,
            53 => NetworkManagementTlvType::ChannelMask,
            _ => NetworkManagementTlvType::NotPresent,
        }
    }
}

impl<'a, 'b> From<&'a NetworkManagementTlv<'b>> for NetworkManagementTlvType {
    fn from(network_mgmt_tlv: &'a NetworkManagementTlv<'b>) -> Self {
        match *network_mgmt_tlv {
            NetworkManagementTlv::Channel { .. } => NetworkManagementTlvType::Channel,
            NetworkManagementTlv::PanId(_) => NetworkManagementTlvType::PanId,
            NetworkManagementTlv::ExtendedPanId(_) => NetworkManagementTlvType::ExtendedPanId,
            NetworkManagementTlv::NetworkName(_) => NetworkManagementTlvType::NetworkName,
            NetworkManagementTlv::Pskc(_) => NetworkManagementTlvType::Pskc,
            NetworkManagementTlv::NetworkMasterKey(_) => NetworkManagementTlvType::NetworkMasterKey,
            NetworkManagementTlv::NetworkKeySequenceCounter(_) => {
                NetworkManagementTlvType::NetworkKeySequenceCounter
            }
            NetworkManagementTlv::NetworkMeshLocalPrefix(_) => {
                NetworkManagementTlvType::NetworkMeshLocalPrefix
            }
            NetworkManagementTlv::SteeringData(_) => NetworkManagementTlvType::SteeringData,
            NetworkManagementTlv::BorderAgentLocator(_) => {
                NetworkManagementTlvType::BorderAgentLocator
            }
            NetworkManagementTlv::CommissionerId(_) => NetworkManagementTlvType::CommissionerId,
            NetworkManagementTlv::CommissionerSessionId(_) => {
                NetworkManagementTlvType::CommissionerSessionId
            }
            NetworkManagementTlv::SecurityPolicy { .. } => NetworkManagementTlvType::SecurityPolicy,
            NetworkManagementTlv::ActiveTimestamp { .. } => {
                NetworkManagementTlvType::ActiveTimestamp
            }
            NetworkManagementTlv::CommissionerUdpPort(_) => {
                NetworkManagementTlvType::CommissionerUdpPort
            }
            NetworkManagementTlv::PendingTimestamp { .. } => {
                NetworkManagementTlvType::PendingTimestamp
            }
            NetworkManagementTlv::DelayTimer(_) => NetworkManagementTlvType::DelayTimer,
            NetworkManagementTlv::ChannelMask(_) => NetworkManagementTlvType::ChannelMask,
        }
    }
}

/// Used in Security Policy TLV.
/// See 8.10.1.15
#[repr(u8)]
pub enum SecurityPolicy {
    O = 0b1000_0000, // Out-of-band commissioning enabled.
    N = 0b0100_0000, // Native commissioning using PSKc is allowed.
    R = 0b0010_0000, // Thread 1.x Routers are enabled.
    C = 0b0001_0000, // External commissioner authentication is allowed using PSKc.
    B = 0b0000_1000, // Thread 1.x Beacons are enabled.
}

/// Used in Channel Mask TLV.
pub struct ChannelMaskEntry {
    channel_page: u8,
    mask_length: u8,
    channel_mask: [u8; MAX_VALUE_FIELD_LENGTH],
}

impl<'a> ChannelMaskEntry {
    /// Serializes this Channel Mask Entry into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        let mut offset = enc_consume!(buf, 0; encode_u8, self.channel_page);
        offset = enc_consume!(buf, offset; encode_u8, self.mask_length);
        offset = enc_consume!(buf, offset; encode_bytes_be, &self.channel_mask);
        stream_done!(offset)
    }

    /// Deserializes Channel Mask Entry from `buf` and returns it.
    pub fn decode(buf: &[u8]) -> SResult<ChannelMaskEntry> {
        let (offset, channel_page) = dec_try!(buf; decode_u8);
        let (offset, mask_length) = dec_try!(buf, offset; decode_u8);
        let mut channel_mask = [0u8; MAX_VALUE_FIELD_LENGTH];
        let offset = dec_consume!(buf, offset; decode_bytes_be, &mut channel_mask);
        stream_done!(offset,
                     ChannelMaskEntry {
                         channel_page: channel_page,
                         mask_length: mask_length,
                         channel_mask: channel_mask,
                     })
    }
}
