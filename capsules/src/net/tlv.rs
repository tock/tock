//! Implements Thread 1.1.1 Specification Type-Length-Value (TLV) formats
//!
//! Acronym definitions:
//!     MLE = Mesh Link Establishment
//!     SED = Sleepy End Device


// DEBUG NOTES:
// - .to_be() may not have been called on values wider than one byte
// - See 4.5.25 Active Operational Dataset TLV and 4.5.26 Pending Operational Dataset TLV
//    -- are Active and Pending Timestamp TLVs, respectively, required to be sent as well
//    if either of the dataset tlvs are sent?

// TODO:
// - Decode functions that return buffer slice do not currently reorder bytes from network order
//    - FAILS: Tentative soln: Copy decoded buffer back into input buffer...
//    - Tentative soln: require user to allocate separate buffer to store decoded bytes


use core::mem;
use net::stream::{decode_u8, decode_u16, decode_u32, decode_bytes_be};
use net::stream::{encode_u8, encode_u16, encode_u32, encode_bytes_be};
use net::stream::SResult;

pub enum Tlv<'a> {
    SourceAddress(u16),
    Mode(u8),
    Timeout(u32),
    Challenge([u8; 8]),
    Response([u8; 8]),
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
    */
    ActiveOperationalDataset(&'a [u8]),
    PendingOperationalDataset(&'a [u8]), 
    /*
    TODO: Not required to implement MLE for SED
    ThreadDiscovery
    */
}

// Type and length fields of TLV are each one byte.
static TL_WIDTH: usize = 2;

impl<'a> Tlv<'a> {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        match *self {
            Tlv::SourceAddress(ref mac_address) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, mac_address.to_be());
                stream_done!(offset)
            }
            Tlv::Mode(ref mode) => {
                let value_width = mem::size_of::<u8>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u8, *mode);
                stream_done!(offset)
            }
            Tlv::Timeout(ref max_transmit_interval) => {
                let value_width = mem::size_of::<u32>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u32, max_transmit_interval.to_be());
                stream_done!(offset)
            }
            Tlv::Challenge(ref byte_str) => {
                let value_width = byte_str.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, byte_str);
                stream_done!(offset)
            }
            Tlv::Response(ref byte_str) => {
                let value_width = byte_str.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, byte_str);
                stream_done!(offset)
            }
            Tlv::LinkLayerFrameCounter(ref frame_counter) => {
                let value_width = mem::size_of::<u32>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u32, frame_counter.to_be());
                stream_done!(offset)
            }
            Tlv::MleFrameCounter(ref frame_counter) => {
                let value_width = mem::size_of::<u32>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u32, frame_counter.to_be());
                stream_done!(offset)
            }
            Tlv::Address16(ref mac_address) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, mac_address.to_be());
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
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u32, partition_id.to_be());
                offset = enc_consume!(buf, offset; encode_u8, weighting);
                offset = enc_consume!(buf, offset; encode_u8, data_version);
                offset = enc_consume!(buf, offset; encode_u8, stable_data_version);
                offset = enc_consume!(buf, offset; encode_u8, leader_router_id);
                stream_done!(offset)
            }
            Tlv::NetworkData(ref network_data_tlvs) => {
                let value_width = network_data_tlvs.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, network_data_tlvs);
                stream_done!(offset)
            }
            Tlv::TlvRequest(ref tlv_codes) => {
                let value_width = tlv_codes.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, tlv_codes);
                stream_done!(offset)
            }
            Tlv::ScanMask(ref scan_mask) => {
                let value_width = mem::size_of::<u8>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u8, *scan_mask);
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
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u8, parent_priority);
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
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u8, *link_margin);
                stream_done!(offset)
            }
            Tlv::Status(ref status) => {
                let value_width = mem::size_of::<u8>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u8, *status);
                stream_done!(offset)
            }
            Tlv::Version(ref version) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, *version);
                stream_done!(offset)
            }
            Tlv::ActiveOperationalDataset(ref network_mgmt_tlvs) => {
                let value_width = network_mgmt_tlvs.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, network_mgmt_tlvs);
                stream_done!(offset)
            }
            Tlv::PendingOperationalDataset(ref network_mgmt_tlvs) => {
                let value_width = network_mgmt_tlvs.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, network_mgmt_tlvs);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        buf[0] = TlvType::from(self) as u8;
        buf[1] = value_width as u8;
        stream_done!(2)
    }

    pub fn decode(buf: &mut [u8]) -> SResult<Option<Tlv>> {
        let (offset, tlv_type) = dec_try!(buf; decode_u8);
        let tlv_type = TlvType::from(tlv_type);
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            TlvType::SourceAddress => {
                let (offset, mac_address) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, Some(Tlv::SourceAddress(mac_address)))
            }
            TlvType::Mode => {
                let (offset, mode) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset, Some(Tlv::Mode(mode)))
            }
            TlvType::Timeout => {
                let (offset, max_transmit_interval) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, Some(Tlv::Timeout(max_transmit_interval)))
            }
            TlvType::Challenge => {
                let mut byte_str = [0u8; 8]; // Byte string max length 8 bytes.
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut byte_str);
                stream_done!(offset, Some(Tlv::Challenge(byte_str)))
            }
            TlvType::Response => {
                let mut byte_str = [0u8; 8]; // Byte string max length 8 bytes.
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut byte_str);
                stream_done!(offset, Some(Tlv::Response(byte_str)))
            }
            TlvType::LinkLayerFrameCounter => {
                let (offset, frame_counter) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, Some(Tlv::LinkLayerFrameCounter(frame_counter)))
            }
            TlvType::MleFrameCounter => {
                let (offset, frame_counter) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset, Some(Tlv::MleFrameCounter(frame_counter)))
            }
            TlvType::Address16 => {
                let (offset, mac_address) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, Some(Tlv::Address16(mac_address)))
            }
            TlvType::LeaderData => {
                let (offset, partition_id) = dec_try!(buf, offset; decode_u32);
                let (offset, weighting) = dec_try!(buf, offset; decode_u8);
                let (offset, data_version) = dec_try!(buf, offset; decode_u8);
                let (offset, stable_data_version) = dec_try!(buf, offset; decode_u8);
                let (offset, leader_router_id) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset, Some(Tlv::LeaderData {
                                                partition_id: partition_id,
                                                weighting: weighting,
                                                data_version: data_version,
                                                stable_data_version: stable_data_version,
                                                leader_router_id: leader_router_id,
                                            }))
            }
            TlvType::NetworkData => {
                stream_done!(0, Some(Tlv::NetworkData(&buf[offset..offset + length as usize])))
            }
            TlvType::TlvRequest => {
                stream_done!(0, None)
            }
            TlvType::ScanMask => {
                stream_done!(0, None)
            }
            TlvType::Connectivity => {
                stream_done!(0, None)
            }
            TlvType::LinkMargin => {
                stream_done!(0, None)
            }
            TlvType::Status => {
                stream_done!(0, None)
            }
            TlvType::Version => {
                stream_done!(0, None)
            }
            TlvType::ActiveOperationalDataset => {
                stream_done!(0, None)
            }
            TlvType::PendingOperationalDataset => {
                stream_done!(0, None)
            }
            TlvType::NotPresent => {
                stream_done!(offset, None)
            }
        }
    }
}

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
    NotPresent
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
            _ => TlvType::NotPresent
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

#[repr(u8)]
pub enum LinkMode {
    ReceiverOnWhenIdle = 0b00001000,
    SecureDataRequests = 0b00000100,
    FullThreadDevice = 0b00000010,
    FullNetworkDataRequired = 0b00000001,
}

#[repr(u8)]
pub enum MulticastResponder {
    Router = 0b10000000,
    EndDevice = 0b01000000,
}

// Used in Connectivity TLV
pub enum ParentPriority {
    High = 0b01000000,
    Medium = 0b00000000,
    Low = 0b11000000, 
    // Reserved = 0b10000000
}

pub enum NetworkDataTlv<'a> {
    Prefix {
        domain_id: u8,
        prefix_length_bits: u8,
        prefix: &'a [u8],
        sub_tlvs: &'a [u8],
    },
    CommissioningData { com_length: u8, com_data: &'a [u8] },
    Service {
        thread_enterprise_number: bool,
        s_id: u8,
        s_enterprise_number: u32,
        s_service_data_length: u8,
        s_service_data: &'a [u8],
        sub_tlvs: &'a [u8],
    },
}

impl<'a> NetworkDataTlv<'a> {
    pub fn encode(&self, buf: &mut [u8], stable: bool) -> SResult {
        match *self {
            NetworkDataTlv::Prefix { domain_id, prefix_length_bits, prefix, sub_tlvs } => {
                let value_width = mem::size_of::<u8>() + mem::size_of::<u8>() + prefix.len() +
                                  sub_tlvs.len();
                self.encode_tl(buf, value_width, stable);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u8, domain_id);
                offset = enc_consume!(buf, offset; encode_u8, prefix_length_bits);
                offset = enc_consume!(buf, offset; encode_bytes_be, prefix);
                offset = enc_consume!(buf, offset; encode_bytes_be, sub_tlvs);
                stream_done!(offset)
            }
            NetworkDataTlv::CommissioningData { com_length, com_data } => {
                let value_width = com_length as usize;
                self.encode_tl(buf, value_width, stable);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, com_data);
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
                self.encode_tl(buf, value_width, stable);
                let t_bit: u8 = if thread_enterprise_number {
                    1u8 << 7
                } else {
                    0
                };
                let first_byte: u8 = t_bit | (0b1111 & s_id);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u8, first_byte);
                offset = enc_consume!(buf, offset; encode_u32, s_enterprise_number.to_be());
                offset = enc_consume!(buf, offset; encode_u8, s_service_data_length);
                offset = enc_consume!(buf, offset; encode_bytes_be, s_service_data);
                offset = enc_consume!(buf, offset; encode_bytes_be, sub_tlvs);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize, stable: bool) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        let stable_bit = if stable { 1u8 } else { 0u8 };
        buf[0] = (NetworkDataTlvType::from(self) as u8) << 1 | stable_bit;
        buf[1] = value_width as u8;
        stream_done!(2)
    }

    pub fn decode(buf: &[u8]) -> SResult<Option<(NetworkDataTlv, bool)>> {
        let (offset, tlv_type_field) = dec_try!(buf; decode_u8);
        let tlv_type_raw = tlv_type_field >> 1;
        let tlv_type = NetworkDataTlvType::from(tlv_type_raw);
        let stable = (tlv_type_field & 1u8) > 0;
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            NetworkDataTlvType::Prefix => {
                let (offset, domain_id) = dec_try!(buf, offset; decode_u8);
                let (offset, prefix_length_bits) = dec_try!(buf, offset; decode_u8);
                let prefix_length_bytes = (prefix_length_bits / 8) as usize +
                                          if prefix_length_bits % 8 == 0 { 0 } else { 1 };
                stream_done!(offset,
                             Some((NetworkDataTlv::Prefix {
                                       domain_id: domain_id,
                                       prefix_length_bits: prefix_length_bits,
                                       prefix: &buf[offset..offset + prefix_length_bytes],
                                       sub_tlvs: &buf[offset + prefix_length_bytes..
                                                  offset + length as usize],
                                   },
                                   stable)))
            }
            NetworkDataTlvType::CommissioningData => {
                let (offset, com_length) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             Some((NetworkDataTlv::CommissioningData {
                                       com_length: com_length,
                                       com_data: &buf[offset..offset + length as usize],
                                   },
                                   stable)))
            }
            NetworkDataTlvType::Service => {
                let (offset, first_byte) = dec_try!(buf, offset; decode_u8);
                let thread_enterprise_number = (first_byte >> 7) > 0;
                let s_id = first_byte & 0b1111;
                let (offset, s_enterprise_number) = dec_try!(buf, offset; decode_u32);
                let (offset, s_service_data_length) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             Some((NetworkDataTlv::Service {
                                       thread_enterprise_number: thread_enterprise_number,
                                       s_id: s_id,
                                       s_enterprise_number: s_enterprise_number,
                                       s_service_data_length: s_service_data_length,
                                       s_service_data: &buf[offset..
                                                        offset +
                                                        s_service_data_length as usize],
                                       sub_tlvs: &buf[offset + s_service_data_length as usize..
                                                  offset + length as usize],
                                   },
                                   stable)))
            }
            NetworkDataTlvType::NotPresent => stream_done!(offset, None),
        }
    }
}

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
    pub fn encode(&self, buf: &mut [u8], stable: bool) -> SResult {
        match *self {
            PrefixSubTlv::HasRoute(ref r_border_router_16s) => {
                let value_width = r_border_router_16s.len();
                self.encode_tl(buf, value_width, stable);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, r_border_router_16s);
                stream_done!(offset)
            }
            PrefixSubTlv::BorderRouter(ref p_border_router_16s) => {
                let value_width = p_border_router_16s.len();
                self.encode_tl(buf, value_width, stable);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, p_border_router_16s);
                stream_done!(offset)
            }
            PrefixSubTlv::SixLoWpanId { context_id_compress, context_id, context_length } => {
                let value_width = mem::size_of::<u8>() + mem::size_of::<u8>();
                self.encode_tl(buf, value_width, stable);
                let compress_bit = if context_id_compress { 1u8 } else { 0u8 };
                let first_byte = (compress_bit << 4) | (context_id & 0b1111);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u8, first_byte);
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
        stream_done!(2)
    }

    // Returns PrefixSubTlv and true if stabl, false otherwise.
    pub fn decode(buf: &[u8]) -> SResult<Option<(PrefixSubTlv, bool)>> {
        let (offset, tlv_type_field) = dec_try!(buf; decode_u8);
        let tlv_type_raw = tlv_type_field >> 1;
        let tlv_type = PrefixSubTlvType::from(tlv_type_raw);
        let stable = (tlv_type_field & 1u8) > 0;
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            PrefixSubTlvType::HasRoute => {
                stream_done!(offset + buf[offset..].len(),
                             Some((PrefixSubTlv::HasRoute(&buf[offset..offset + length as usize]),
                                   stable)))
            }
            PrefixSubTlvType::BorderRouter => {
                stream_done!(offset + buf[offset..].len(),
                             Some((PrefixSubTlv::BorderRouter(&buf[offset..
                                                               offset + length as usize]),
                                   stable)))
            }
            PrefixSubTlvType::SixLoWpanId => {
                let (offset, first_byte) = dec_try!(buf, offset; decode_u8);
                let context_id_compress = (first_byte & 0b10000) > 0;
                let context_id = first_byte & 0b1111;
                let (offset, context_length) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             Some((PrefixSubTlv::SixLoWpanId {
                                       context_id_compress: context_id_compress,
                                       context_id: context_id,
                                       context_length: context_length,
                                   },
                                   stable)))
            }
            PrefixSubTlvType::NotPresent => stream_done!(offset, None),
        }
    }
}

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

pub struct HasRouteTlvValue {
    r_border_router_16: u16,
    r_preference: u8,
}

impl HasRouteTlvValue {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        stream_len_cond!(buf, 3); // Each Has Route TLV value is 24 bits wide.
        let mut offset = enc_consume!(buf, 0; encode_u16, self.r_border_router_16.to_be());
        let last_byte = ((self.r_preference & 0b11) as u8) << 6;
        offset = enc_consume!(buf, offset; encode_u8, last_byte);
        stream_done!(offset)
    }

    pub fn decode(buf: &[u8]) -> SResult<HasRouteTlvValue> {
        let (offset, r_border_router_16) = dec_try!(buf; decode_u16);
        let (offset, r_preference) = dec_try!(buf, offset; decode_u8);
        stream_done!(offset + buf[offset..].len(),
                     HasRouteTlvValue {
                         r_border_router_16: r_border_router_16,
                         r_preference: r_preference,
                     })
    }
}

pub struct BorderRouterTlvValue {
    // See 15.8.3
    p_border_router_16: u16,
    p_bits: u16,
}

// See 5.18.3
#[repr(u16)]
pub enum BorderRouterTlvValueBit {
    Prf = 0b1100000000000000,
    P = 0b0010000000000000,
    S = 0b0001000000000000,
    D = 0b0000100000000000,
    C = 0b0000010000000000,
    R = 0b0000001000000000,
    O = 0b0000000100000000,
    N = 0b0000000010000000,
}

impl BorderRouterTlvValue {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        stream_len_cond!(buf, 4); // Each Border Router TLV value is 32 bits wide.
        let mut offset = enc_consume!(buf, 0; encode_u16, self.p_border_router_16.to_be());
        offset = enc_consume!(buf, offset; encode_u16, self.p_bits.to_be());
        stream_done!(offset)
    }

    pub fn decode(buf: &[u8]) -> SResult<BorderRouterTlvValue> {
        let (offset, p_border_router_16) = dec_try!(buf; decode_u16);
        let (offset, p_bits) = dec_try!(buf, offset; decode_u16);
        stream_done!(offset + buf[offset..].len(),
                     BorderRouterTlvValue {
                         p_border_router_16: p_border_router_16,
                         p_bits: p_bits,
                     })
    }
}

pub enum ServiceSubTlv<'a> {
    Server {
        s_server_16: u16,
        s_server_data: &'a [u8],
    },
}

impl<'a> ServiceSubTlv<'a> {
    pub fn encode(&self, buf: &mut [u8], stable: bool) -> SResult {
        match *self {
            ServiceSubTlv::Server { s_server_16, s_server_data } => {
                let value_width = mem::size_of::<u16>() + s_server_data.len();
                self.encode_tl(buf, value_width, stable);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u16, s_server_16.to_be());
                offset = enc_consume!(buf, offset; encode_bytes_be, s_server_data);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize, stable: bool) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        let stable_bit = if stable { 1u8 } else { 0u8 };
        buf[0] = (ServiceSubTlvType::from(self) as u8) << 1 | stable_bit;
        buf[1] = value_width as u8;
        stream_done!(2)
    }

    pub fn decode(buf: &[u8]) -> SResult<Option<(ServiceSubTlv, bool)>> {
        let (offset, tlv_type_field) = dec_try!(buf; decode_u8);
        let tlv_type_raw = tlv_type_field >> 1;
        let tlv_type = ServiceSubTlvType::from(tlv_type_raw);
        let stable = (tlv_type_field & 1u8) > 0;
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            ServiceSubTlvType::Server => {
                let (offset, s_server_16) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset + buf[offset..].len(),
                             Some((ServiceSubTlv::Server {
                                       s_server_16: s_server_16,
                                       s_server_data: &buf[offset..offset + length as usize],
                                   },
                                   stable)))
            }
            ServiceSubTlvType::NotPresent => stream_done!(offset, None),
        }
    }
}

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

impl<'a, 'b> From<&'a ServiceSubTlv<'b>> for ServiceSubTlvType {
    fn from(service_sub_tlv: &'a ServiceSubTlv<'b>) -> Self {
        match *service_sub_tlv {
            ServiceSubTlv::Server { .. } => ServiceSubTlvType::Server,
        }
    }
}

pub enum NetworkManagementTlv<'a> {
    Channel { channel_page: u8, channel: u16 },
    PanId(u16),
    ExtendedPanId([u8; 8]),
    NetworkName(&'a [u8]),
    Pskc(&'a [u8]),
    NetworkMasterKey([u8; 16]),
    NetworkKeySequenceCounter([u8; 4]),
    NetworkMeshLocalPrefix([u8; 8]),
    SteeringData(&'a [u8]),
    BorderAgentLocator(u16),
    CommissionerId(&'a [u8]),
    CommissionerSessionId(u16),
    SecurityPolicy { rotation_time: u16, policy_bits: u8 },
    ActiveTimestamp {
        timestamp_seconds: [u8; 3],
        timestamp_ticks: u16,
        u_bit: bool,
    },
    CommissionerUdpPort(u16),
    PendingTimestamp {
        timestamp_seconds: [u8; 3],
        timestamp_ticks: u16,
        u_bit: bool,
    },
    DelayTimer(u32),
    ChannelMask(&'a [u8]),
}

impl<'a> NetworkManagementTlv<'a> {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        match *self {
            NetworkManagementTlv::Channel { channel_page, channel } => {
                // `channel_page` should be 0 (See 8.10.1.1.1)
                // `channel` should be 11-26 (See 8.10.1.1.2)
                let value_width = mem::size_of::<u8>() + mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u8, channel_page);
                offset = enc_consume!(buf, offset; encode_u16, channel.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::PanId(ref pan_id) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, pan_id.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::ExtendedPanId(ref extended_pan_id) => {
                let value_width = extended_pan_id.len(); // Extended PAN ID length 8 bytes.
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, extended_pan_id);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkName(ref network_name) => {
                stream_cond!(network_name.len() <= 16); // Network name max length 16 bytes.
                let value_width = network_name.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, network_name);
                stream_done!(offset)
            }
            NetworkManagementTlv::Pskc(ref pskc) => {
                stream_cond!(pskc.len() <= 16); // PSKc max length 16 bytes.
                let value_width = pskc.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, pskc);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkMasterKey(ref network_key) => {
                let value_width = network_key.len(); // Master key length 128 bits = 16 bytes.
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, network_key);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkKeySequenceCounter(ref counter) => {
                let value_width = counter.len(); // Counter length 4 bytes.
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, counter);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkMeshLocalPrefix(ref prefix) => {
                let value_width = prefix.len(); // Mesh-Local Prefix length 8 bytes.
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, prefix);
                stream_done!(offset)
            }
            NetworkManagementTlv::SteeringData(ref bloom_filter) => {
                stream_cond!(bloom_filter.len() <= 16); // Bloom filter max length 16 bytes.
                let value_width = bloom_filter.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, bloom_filter);
                stream_done!(offset)
            }
            NetworkManagementTlv::BorderAgentLocator(ref rloc_16) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, rloc_16.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::CommissionerId(ref commissioner_id) => {
                stream_cond!(commissioner_id.len() <= 64); // Commissioner ID max length 64 bytes.
                let value_width = commissioner_id.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, commissioner_id);
                stream_done!(offset)
            }
            NetworkManagementTlv::CommissionerSessionId(ref session_id) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, session_id.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::SecurityPolicy { rotation_time, policy_bits } => {
                let value_width = mem::size_of::<u16>() + mem::size_of::<u8>();
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_u16, rotation_time.to_be());
                offset = enc_consume!(buf, offset; encode_u8, policy_bits);
                stream_done!(offset)
            }
            NetworkManagementTlv::ActiveTimestamp { timestamp_seconds, timestamp_ticks, u_bit } => {
                let value_width =
                    timestamp_seconds.len() // Timestamp seconds is a 48-bit Unix time value.
                                + mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, &timestamp_seconds);
                let u_bit_val = if u_bit { 1u16 } else { 0u16 };
                let end_bytes = (timestamp_ticks << 1) | u_bit_val;
                offset = enc_consume!(buf, offset; encode_u16, end_bytes.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::CommissionerUdpPort(ref udp_port) => {
                let value_width = mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u16, udp_port.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::PendingTimestamp { timestamp_seconds,
                                                     timestamp_ticks,
                                                     u_bit } => {
                // Timestamp seconds is a 48-bit Unix time value.
                let value_width = timestamp_seconds.len() + mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, &timestamp_seconds);
                let u_bit_val = if u_bit { 1u16 } else { 0u16 };
                let end_bytes = (timestamp_ticks << 1) | u_bit_val;
                offset = enc_consume!(buf, offset; encode_u16, end_bytes.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::DelayTimer(ref time_remaining) => {
                let value_width = mem::size_of::<u32>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u32, time_remaining.to_be());
                stream_done!(offset)
            }
            NetworkManagementTlv::ChannelMask(ref entries) => {
                let value_width = entries.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, entries);
                stream_done!(offset)
            }
        }
    }

    fn encode_tl(&self, buf: &mut [u8], value_width: usize) -> SResult {
        stream_len_cond!(buf, TL_WIDTH + value_width);
        buf[0] = NetworkManagementTlvType::from(self) as u8;
        buf[1] = value_width as u8;
        stream_done!(2)
    }

    pub fn decode(buf: &[u8]) -> SResult<Option<NetworkManagementTlv>> {
        let (offset, tlv_type_raw) = dec_try!(buf; decode_u8);
        let tlv_type = NetworkManagementTlvType::from(tlv_type_raw);
        let (offset, length) = dec_try!(buf, offset; decode_u8);
        match tlv_type {
            NetworkManagementTlvType::Channel => {
                let (offset, channel_page) = dec_try!(buf, offset; decode_u8);
                let (offset, channel) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             Some(NetworkManagementTlv::Channel {
                                 channel_page: channel_page,
                                 channel: channel,
                             }))
            }
            NetworkManagementTlvType::PanId => {
                let (offset, pan_id) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset, Some(NetworkManagementTlv::PanId(pan_id)))
            }
            NetworkManagementTlvType::ExtendedPanId => {
                let mut extended_pan_id = [0u8; 8]; // Extended PAN ID length 8 bytes.
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut extended_pan_id);
                stream_done!(offset,
                             Some(NetworkManagementTlv::ExtendedPanId(extended_pan_id)))
            }
            NetworkManagementTlvType::NetworkName => {
                // Network name max length 16 bytes.
                stream_done!(offset + buf[offset..].len(),
                             Some(NetworkManagementTlv::NetworkName(&buf[offset..
                                                                     offset +
                                                                     length as usize])))
            }
            NetworkManagementTlvType::Pskc => {
                // PSKc max length 16 bytes.
                stream_done!(offset + buf[offset..].len(),
                             Some(NetworkManagementTlv::Pskc(&buf[offset..
                                                              offset + length as usize])))
            }
            NetworkManagementTlvType::NetworkMasterKey => {
                let mut network_key = [0u8; 16]; // Master key length 128 bits = 16 bytes.
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut network_key);
                stream_done!(offset,
                             Some(NetworkManagementTlv::NetworkMasterKey(network_key)))
            }
            NetworkManagementTlvType::NetworkKeySequenceCounter => {
                let mut counter = [0u8; 4]; // Counter length 4 bytes.
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut counter);
                stream_done!(offset,
                             Some(NetworkManagementTlv::NetworkKeySequenceCounter(counter)))
            }
            NetworkManagementTlvType::NetworkMeshLocalPrefix => {
                let mut prefix = [0u8; 8]; // Mesh-Local Prefix length 8 bytes.
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut prefix);
                stream_done!(offset,
                             Some(NetworkManagementTlv::NetworkMeshLocalPrefix(prefix)))
            }
            NetworkManagementTlvType::SteeringData => {
                // Bloom filter max length 16 bytes.
                stream_done!(offset + buf[offset..].len(),
                             Some(NetworkManagementTlv::SteeringData(&buf[offset..
                                                                      offset +
                                                                      length as usize])))
            }
            NetworkManagementTlvType::BorderAgentLocator => {
                let (offset, rloc_16) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             Some(NetworkManagementTlv::BorderAgentLocator(rloc_16)))
            }
            NetworkManagementTlvType::CommissionerId => {
                // Commissioner ID max length 64 bytes.
                stream_done!(offset + buf[offset..].len(),
                             Some(NetworkManagementTlv::CommissionerId(&buf[offset..
                                                                        offset +
                                                                        length as usize])))
            }
            NetworkManagementTlvType::CommissionerSessionId => {
                let (offset, session_id) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             Some(NetworkManagementTlv::CommissionerSessionId(session_id)))
            }
            NetworkManagementTlvType::SecurityPolicy => {
                let (offset, rotation_time) = dec_try!(buf, offset; decode_u16);
                let (offset, policy_bits) = dec_try!(buf, offset; decode_u8);
                stream_done!(offset,
                             Some(NetworkManagementTlv::SecurityPolicy {
                                 rotation_time: rotation_time,
                                 policy_bits: policy_bits,
                             }))
            }
            NetworkManagementTlvType::ActiveTimestamp => {
                // Timestamp seconds is a 48-bit Unix time value.
                let mut timestamp_seconds = [0u8; 3];
                let offset = dec_consume!(buf, offset; decode_bytes_be, &mut timestamp_seconds);
                let (offset, timestamp_ticks) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             Some(NetworkManagementTlv::ActiveTimestamp {
                                 timestamp_seconds: timestamp_seconds,
                                 timestamp_ticks: timestamp_ticks >> 1,
                                 u_bit: (timestamp_ticks | 1u16) > 0,
                             }))
            }
            NetworkManagementTlvType::CommissionerUdpPort => {
                let (offset, udp_port) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             Some(NetworkManagementTlv::CommissionerUdpPort(udp_port)))
            }
            NetworkManagementTlvType::PendingTimestamp => {
                // Timestamp seconds is a 48-bit Unix time value.
                let mut timestamp_seconds = [0u8; 3];
                let offset = dec_consume!(buf; decode_bytes_be, &mut timestamp_seconds);
                let (offset, timestamp_ticks) = dec_try!(buf, offset; decode_u16);
                stream_done!(offset,
                             Some(NetworkManagementTlv::PendingTimestamp {
                                 timestamp_seconds: timestamp_seconds,
                                 timestamp_ticks: timestamp_ticks >> 1,
                                 u_bit: (timestamp_ticks | 1u16) > 0,
                             }))
            }
            NetworkManagementTlvType::DelayTimer => {
                let (offset, time_remaining) = dec_try!(buf, offset; decode_u32);
                stream_done!(offset,
                             Some(NetworkManagementTlv::DelayTimer(time_remaining)))
            }
            NetworkManagementTlvType::ChannelMask => {
                stream_done!(offset + buf[offset..].len(),
                             Some(NetworkManagementTlv::ChannelMask(&buf[offset..
                                                                     offset +
                                                                     length as usize])))
            }
            NetworkManagementTlvType::NotPresent => stream_done!(offset, None),
        }
    }
}

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

// See 8.10.1.15
#[repr(u8)]
pub enum SecurityPolicy {
    O = 0b10000000,
    N = 0b01000000,
    R = 0b00100000,
    C = 0b00010000,
    B = 0b00001000,
}

pub struct ChannelMaskEntry<'a> {
    channel_page: u8,
    mask_length: u8,
    channel_mask: &'a [u8],
}

impl<'a> ChannelMaskEntry<'a> {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        let mut offset = enc_consume!(buf, 0; encode_u8, self.channel_page);
        offset = enc_consume!(buf, offset; encode_u8, self.mask_length);
        offset = enc_consume!(buf, offset; encode_bytes_be, self.channel_mask);
        stream_done!(offset)
    }

    pub fn decode(buf: &[u8]) -> SResult<ChannelMaskEntry> {
        let (offset, channel_page) = dec_try!(buf; decode_u8);
        let (offset, mask_length) = dec_try!(buf, offset; decode_u8);
        stream_done!(offset + buf[offset..].len(),
                     ChannelMaskEntry {
                         channel_page: channel_page,
                         mask_length: mask_length,
                         channel_mask: &buf[offset..],
                     })
    }
}
