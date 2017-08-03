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


use core::mem;
#[macro_use]
use net::stream::{encode_u8, encode_u16, encode_u32, encode_bytes, encode_bytes_be};
use net::stream::SResult;

pub enum Tlv<'a> {
    SourceAddress(u16),
    Mode(u8),
    Timeout(u32),
    Challenge(&'a [u8]),
    Response(&'a [u8]),
    LinkLayerFrameCounter(u32),
    // LinkQuality,                  // TLV type Not used in Thread
    // NetworkParameter,             // TLV type Not used in Thread
    MleFrameCounter(u32),
    // Route64,                      // TODO: Not required to implement MLE for SED
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
    // AddressRegistration          // TODO: Not required to implement MLE for SED
    // AddressRegistration          // TODO: Not required to implement MLE for SED
    // Channel                      // TODO: Not required to implement MLE for SED
    // PanId                        // TODO: Not required to implement MLE for SED
    // ActiveTimestamp              // TODO: Not required to implement MLE for SED
    // PendingTimestamp             // TODO: Not required to implement MLE for SED
    ActiveOperationalDataset(&'a [u8]),
    PendingOperationalDataset(&'a [u8]), 
    // ThreadDiscovery              // TODO: Not required to implement MLE for Sleepy End Device
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
            Tlv::LinkLayerFrameCounter(ref frame_count) => {
                let value_width = mem::size_of::<u32>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u32, frame_count.to_be());
                stream_done!(offset)
            }
            Tlv::MleFrameCounter(ref frame_count) => {
                let value_width = mem::size_of::<u32>();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_u32, frame_count.to_be());
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

    /*
    pub fn decode(buf: &[u8], type: TlvType) -> SResult<Tlv> {
        match type {
            // TOOD:
            // - finish Tlv
            // - write decode
        }
    }
    */
}

#[repr(u8)]
pub enum TlvType {
    SourceAddress = 0,
    Mode = 1,
    Timeout = 2,
    Challenge = 3,
    Response = 4,
    LinkLayerFrameCounter = 5,
    // LinkQuality            = 6,  // TLV type not used in Thread
    // NetworkParameter       = 7,  // TLV type not used in Thread
    MleFrameCounter = 8,
    // Route64                   = 9,  // TODO: Not required to implement MLE for SED
    Address16 = 10,
    LeaderData = 11,
    NetworkData = 12,
    TlvRequest = 13,
    ScanMask = 14,
    Connectivity = 15,
    LinkMargin = 16,
    Status = 17,
    Version = 18,
    // AddressRegistration       = 19, // TODO: Not required to implement MLE for SED
    // Channel                   = 20, // TODO: Not required to implement MLE for SED
    // PanId                     = 21, // TODO: Not required to implement MLE for SED
    // ActiveTimestamp           = 22, // TODO: Not required to implement MLE for SED
    // PendingTimestamp          = 23, // TODO: Not required to implement MLE for SED
    ActiveOperationalDataset = 24,
    PendingOperationalDataset = 25, 
    // ThreadDiscovery           = 26, // TODO: Not required to implement MLE for Sleepy End Device
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
            // Tlv::LinkQuality               => TlvType::LinkQuality,         // TLV type not used in Thread
            // Tlv::NetworkParameter          => TlvType::NetworkParameter,    // TLV type not used in Thread
            Tlv::MleFrameCounter(_) => TlvType::MleFrameCounter,
            // Tlv::Route64                   => TlvType::Route64,             // TODO: Not required to implement MLE for SED
            Tlv::Address16(_) => TlvType::Address16,
            Tlv::LeaderData { .. } => TlvType::LeaderData,
            Tlv::NetworkData(_) => TlvType::NetworkData,
            Tlv::TlvRequest(_) => TlvType::TlvRequest,
            Tlv::ScanMask(_) => TlvType::ScanMask,
            Tlv::Connectivity { .. } => TlvType::Connectivity,
            Tlv::LinkMargin(_) => TlvType::LinkMargin,
            Tlv::Status(_) => TlvType::Status,
            Tlv::Version(_) => TlvType::Version,
            // Tlv::AddressRegistration       => TlvType::AddressRegistration, // TODO: Not required to implement MLE for SED
            // Tlv::Channel                   => TlvType::Channel,             // TODO: Not required to implement MLE for SED
            // Tlv::PanId                     => TlvType::PanId,               // TODO: Not required to implement MLE for SED
            // Tlv::ActiveTimestamp           => TlvType::ActiveTimestamp,     // TODO: Not required to implement MLE for SED
            // Tlv::PendingTimestamp          => TlvType::PendingTimestamp,    // TODO: Not required to implement MLE for SED
            Tlv::ActiveOperationalDataset(_) => TlvType::ActiveOperationalDataset, 
            Tlv::PendingOperationalDataset(_) => TlvType::PendingOperationalDataset,
            // Tlv::ThreadDiscovery           => TlvType::ThreadDiscovery,     // TODO: Not required to implement MLE for SED
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
    // Reserved = 0b10000000,
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
                let first_byte: u8 = (thread_enterprise_number as u8) | (0b00001111 & s_id);
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
}

#[repr(u8)]
pub enum NetworkDataTlvType {
    Prefix = 1,
    CommissioningData = 4,
    Service = 5,
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
}

#[repr(u8)]
pub enum PrefixSubTlvType {
    HasRoute = 0,
    BorderRouter = 2,
    SixLoWpanId = 3,
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
}

pub struct BorderRouterTlvValue {
    // See 15.8.3
    p_border_router_16: u16,
    p_preference: u8,
    p_preferred: u8,
    p_slaac: u8,
    p_dhcp: u8,
    p_configure: u8,
    p_default: u8,
    p_on_mesh: u8,
    p_nd_dns: u8,
}

impl BorderRouterTlvValue {
    pub fn encode(&self, buf: &mut [u8]) -> SResult {
        stream_len_cond!(buf, 4); // Each Border Router TLV value is 32 bits wide.
        let mut offset = enc_consume!(buf, 0; encode_u16, self.p_border_router_16.to_be());
        let mut bits: u16 = ((self.p_preference & 0b11) as u16) << 14;
        bits = bits | ((self.p_preferred & 0b1) as u16) << 13;
        bits = bits | ((self.p_slaac & 0b1) as u16) << 12;
        bits = bits | ((self.p_dhcp & 0b1) as u16) << 11;
        bits = bits | ((self.p_configure & 0b1) as u16) << 10;
        bits = bits | ((self.p_default & 0b1) as u16) << 9;
        bits = bits | ((self.p_on_mesh & 0b1) as u16) << 8;
        bits = bits | ((self.p_nd_dns & 0b1) as u16) << 7;
        offset = enc_consume!(buf, offset; encode_u16, bits);
        stream_done!(offset)
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
}

#[repr(u8)]
pub enum ServiceSubTlvType {
    Server = 6,
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
    ExtendedPanId(&'a [u8]),
    NetworkName(&'a [u8]),
    Pskc(&'a [u8]),
    NetworkMasterKey(&'a [u8]),
    NetworkKeySequenceCounter(&'a [u8]),
    NetworkMeshLocalPrefix(&'a [u8]),
    SteeringData(&'a [u8]),
    BorderAgentLocator(u16),
    CommissionerId(&'a [u8]),
    CommissionerSessionId(u16),
    SecurityPolicy { rotation_time: u16, policy_bits: u8 },
    ActiveTimestamp { timestamp_seconds: &'a [u8], timestamp_ticks: u16, u_bit: bool },
    /*
    CommissionerUdpPort,
    PendingTimestamp,
    DelayTimer,
    ChannelMask,
    */
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
                stream_cond!(extended_pan_id.len() == 8); // Extended PAN ID length 8 bytes
                let value_width = extended_pan_id.len();
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
                stream_cond!(network_key.len() == 16); // 128 bits
                let value_width = network_key.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, network_key);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkKeySequenceCounter(ref counter) => {
                stream_cond!(counter.len() == 4); // Counter length 4 bytes.
                let value_width = counter.len();
                self.encode_tl(buf, value_width);
                let offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, counter);
                stream_done!(offset)
            }
            NetworkManagementTlv::NetworkMeshLocalPrefix(ref prefix) => {
                stream_cond!(prefix.len() == 8); // Mesh-Local Prefix length 8 bytes.
                let value_width = prefix.len();
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
                stream_cond!(timestamp_seconds.len() == 3);    // Timestamp seconds is a 48-bit Unix time value.
                let value_width = timestamp_seconds.len() + mem::size_of::<u16>();
                self.encode_tl(buf, value_width);
                let mut offset = enc_consume!(buf, TL_WIDTH; encode_bytes_be, timestamp_seconds);
                let u_bit_val = if u_bit { 1u16 } else { 0u16 };
                let end_bytes = (timestamp_ticks << 1) | u_bit_val;
                offset = enc_consume!(buf, offset; encode_u16, end_bytes.to_be());
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
            NetworkManagementTlv::ActiveTimestamp { .. } => NetworkManagementTlvType::ActiveTimestamp,
            /*
            NetworkManagementTlv::CommissionerUdpPort => {
                NetworkManagementTlvType::CommissionerUdpPort
            }
            NetworkManagementTlv::PendingTimestamp => NetworkManagementTlvType::PendingTimestamp,
            NetworkManagementTlv::DelayTimer => NetworkManagementTlvType::DelayTimer,
            NetworkManagementTlv::ChannelMask => NetworkManagementTlvType::ChannelMask,
            */
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
        stream_len_cond!(buf, 3); // Each Has Route TLV value is 24 bits wide.
        let mut offset = enc_consume!(buf, 0; encode_u8, self.channel_page);
        offset = enc_consume!(buf, offset; encode_u8, self.mask_length);
        offset = enc_consume!(buf, offset; encode_bytes_be, self.channel_mask);
        stream_done!(offset)
    }
}
