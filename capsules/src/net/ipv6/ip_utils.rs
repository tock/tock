//! This file implements various utilities used by the different components
//! of the IP stack. Note that this file also contains the definition for the
//! [IPAddr](struct.IPAddr.html] struct and associated helper functions.

use net::icmpv6::icmpv6::{ICMP6Header, ICMP6HeaderOptions};
use net::ipv6::ipv6::IP6Header;
use net::udp::udp::UDPHeader;

#[derive(Copy, Clone, PartialEq)]
pub enum MacAddr {
    ShortAddr(u16),
    LongAddr([u8; 8]),
}

pub mod ip6_nh {
    pub const HOP_OPTS: u8 = 0;
    pub const TCP: u8 = 6;
    pub const UDP: u8 = 17;
    pub const IP6: u8 = 41;
    pub const ROUTING: u8 = 43;
    pub const FRAGMENT: u8 = 44;
    pub const ICMP: u8 = 58;
    pub const NO_NEXT: u8 = 59;
    pub const DST_OPTS: u8 = 60;
    pub const MOBILITY: u8 = 135;
}

#[derive(Copy, Clone, Debug)]
pub struct IPAddr(pub [u8; 16]);

impl IPAddr {
    pub fn new() -> IPAddr {
        // Defaults to the unspecified address
        IPAddr([0; 16])
    }

    pub fn is_unspecified(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn is_unicast_link_local(&self) -> bool {
        self.0[0] == 0xfe
            && (self.0[1] & 0xc0) == 0x80
            && (self.0[1] & 0x3f) == 0
            && self.0[2..8].iter().all(|&b| b == 0)
    }

    pub fn set_unicast_link_local(&mut self) {
        self.0[0] = 0xfe;
        self.0[1] = 0x80;
        for i in 2..8 {
            self.0[i] = 0;
        }
    }

    // Panics if prefix slice does not contain enough bits
    pub fn set_prefix(&mut self, prefix: &[u8], prefix_len: u8) {
        let full_bytes = (prefix_len / 8) as usize;
        let remaining = (prefix_len & 0x7) as usize;
        let bytes = full_bytes + (if remaining != 0 { 1 } else { 0 });
        assert!(bytes <= prefix.len() && bytes <= 16);

        self.0[0..full_bytes].copy_from_slice(&prefix[0..full_bytes]);
        if remaining != 0 {
            let mask = (0xff as u8) << (8 - remaining);
            self.0[full_bytes] &= !mask;
            self.0[full_bytes] |= mask & prefix[full_bytes];
        }
    }

    pub fn is_multicast(&self) -> bool {
        self.0[0] == 0xff
    }
}

pub fn compute_udp_checksum(
    ip6_header: &IP6Header,
    udp_header: &UDPHeader,
    udp_length: u16,
    payload: &[u8],
) -> u16 {
    //This checksum is calculated according to some of the recommendations found in RFC 1071.

    let src_port = udp_header.src_port;
    let dst_port = udp_header.dst_port;
    let mut sum: u32 = 0;
    {
        //First, iterate through src/dst address and add them to the sum
        let mut i = 0;
        while i <= 14 {
            let msb_src: u16 = ((ip6_header.src_addr.0[i]) as u16) << 8;
            let lsb_src: u16 = ip6_header.src_addr.0[i + 1] as u16;
            let temp_src: u16 = msb_src + lsb_src;
            sum += temp_src as u32;

            let msb_dst: u16 = ((ip6_header.dst_addr.0[i]) as u16) << 8;
            let lsb_dst: u16 = ip6_header.dst_addr.0[i + 1] as u16;
            let temp_dst: u16 = msb_dst + lsb_dst;
            sum += temp_dst as u32;

            i += 2; //Iterate two bytes at a time bc 16 bit checksum
        }
    }
    sum += udp_header.len as u32;
    //Finally, add UDP next header
    sum += 17; //was "padded next header"

    //return sum as u16;
    //Next, add the UDP header elements to the sum
    sum += src_port as u32;
    sum += dst_port as u32;
    sum += udp_header.len as u32;
    //Now just need to iterate thru data and add it to the sum
    {
        let mut i: usize = 0;
        while i < ((udp_length - 8) as usize) {
            let msb_dat: u16 = ((payload[i]) as u16) << 8;
            let lsb_dat: u16 = payload[i + 1] as u16;
            let temp_dat: u16 = msb_dat + lsb_dat;
            sum += temp_dat as u32;

            i += 2; //Iterate two bytes at a time bc 16 bit checksum
        }
        //debug!("Checksum is currently: {:?}", sum);
    }
    //now all 16 bit addition has occurred

    while sum > 65535 {
        let sum_high: u32 = sum >> 16; //upper 16 bits of sum
        let sum_low: u32 = sum & 65535; //lower 16 bits of sum
        sum = sum_high + sum_low;
    }

    //Finally, flip all bits
    sum = !sum;
    sum = sum & 65535; //Remove upper 16 bits (which should be FFFF after flip)
    (sum as u16) //Return result as u16 in host byte order
}

pub fn compute_icmp_checksum(
    ipv6_header: &IP6Header,
    icmp_header: &ICMP6Header,
    payload: &[u8],
) -> u16 {
    let mut sum: u32 = 0;

    // add ipv6 pseudo-header
    sum += compute_ipv6_ph_sum(ipv6_header);

    // add type and code
    let msb = (icmp_header.get_type_as_int() as u32) << 8;
    let lsb = icmp_header.get_code() as u32;
    sum += msb + lsb;

    // add options
    match icmp_header.get_options() {
        ICMP6HeaderOptions::Type1 { unused } | ICMP6HeaderOptions::Type3 { unused } => {
            sum += unused >> 16; // upper 16 bits
            sum += unused & 0xffff; // lower 16 bits
        }
        ICMP6HeaderOptions::Type128 { id, seqno } | ICMP6HeaderOptions::Type129 { id, seqno } => {
            sum += id as u32;
            sum += seqno as u32;
        }
    }

    // add icmp payload
    let payload_len = icmp_header.get_len() - icmp_header.get_hdr_size() as u16;
    sum += compute_sum(payload, payload_len);

    // carry overflow
    while sum > 0xffff {
        let sum_upper = sum >> 16;
        let sum_lower = sum & 0xffff;
        sum += sum_upper + sum_lower;
    }

    sum = !sum;
    sum = sum & 0xffff;

    sum as u16
}

pub fn compute_ipv6_ph_sum(ip6_header: &IP6Header) -> u32 {
    let mut sum: u32 = 0;

    // sum over src/dest addresses
    let mut i = 0;
    while i < 16 {
        let msb_src = (ip6_header.src_addr.0[i] as u32) << 8;
        let lsb_src = ip6_header.src_addr.0[i + 1] as u32;
        sum += msb_src + lsb_src;

        let msb_dst = (ip6_header.dst_addr.0[i] as u32) << 8;
        let lsb_dst = ip6_header.dst_addr.0[i + 1] as u32;
        sum += msb_dst + lsb_dst;

        i += 2;
    }

    sum += ip6_header.payload_len as u32;
    sum += ip6_header.next_header as u32;

    sum
}

pub fn compute_sum(buf: &[u8], len: u16) -> u32 {
    let mut sum: u32 = 0;

    let mut i: usize = 0;
    while i < (len as usize) {
        let msb = (buf[i] as u32) << 8;
        let lsb = buf[i + 1] as u32;
        sum += msb + lsb;
        i += 2;
    }

    sum
}
