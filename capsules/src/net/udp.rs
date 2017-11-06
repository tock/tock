//! Implements a basic UDP layer to exist between the 'application' layer and
//! the 6lowpan layer. Eventually, this design will have to be modified once
//! an actual IP layer exists to handle multiplexing of packets by address
//! and protocol. Such a layer does not exist yet, and so for now this layer
//! will effectively serve to take in raw data, construct a UDP datagram
//! and construct an IP packet into which this UDP datagram will be
//! inserted. 
//! TODO: This layer also receives IP packets from the 6lowpan layer,
//! checks if the packet carries a UDP datagram, and passes the data 
//! contained in the UDP datagram up to the application layer if this is the 
//! case. 
//!
//! This file also includes functions which are used 
//! internally to enable the primary interface functions. Finally, this file
//! adds a structure which can be used to assign the UDP header fields and to 
//! construct the IPv6 pseudoheader neccessary for calculating the cksum.
//! An example usage of this capsule can be found in the udp_dummy.rs file in
//! tock/boards/imix/src.

//  Note that the receive portion of this capsule, and the associated test file,
//  has not been implemented yet - in its current form, this test simply formats
//  a single UDP message using this capsule and transmits it, where it can be
//  viewed using a packet sniffer

//  Author: Hudson Ayers, hayers@stanford.edu

use net::lowpan_fragment::{FragState, TxState};
use net::ieee802154::MacAddress;
use kernel::hil::time;
use net::ip::{IP6Header, IPAddr, ip6_nh};
use net::lowpan;

// Define a struct for the UDP Header so that a section of a buffer can
// be cast as this header (unsafe code!) much in the same manner as is 
// currently done for IPv6 packets using the IP6_Header struct.

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct UDPHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub cksum: u16,
}

impl Default for UDPHeader {
    fn default() -> UDPHeader {
        UDPHeader {
            src_port: 0,
            dst_port: 0,
            len: 0,
            cksum: 0,
        }
    }
}

impl UDPHeader {
    pub fn new() -> UDPHeader {
        UDPHeader::default()
    }
    pub fn get_src_port(&self) -> u16 {
        return u16::from_be(self.src_port);
    }
    pub fn set_src_port(&mut self, src_p: u16) {
        self.src_port = src_p.to_be();
    }
    pub fn get_dst_port(&self) -> u16 {
        return u16::from_be(self.dst_port);
    }
    pub fn set_dst_port(&mut self, dst_p: u16) {
        self.dst_port = dst_p.to_be();
    }
    pub fn get_len(&self) -> u16 {
        return u16::from_be(self.len);
    }
    pub fn set_len(&mut self, l: u16) {
        self.len = l.to_be();
    }
    pub fn get_cksum(&self) -> u16 {
        return self.cksum; //Note that this always returns the checksum in network byte order
    }
    pub fn set_cksum(&mut self, ck: u16) {
        self.cksum = ck; //Assumed that the checksum passed is already in network byte order
    }
}

// Struct for constructing the pseudoheader used when calculating the cksum
// of a udp packet sent over IPv6
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IPv6Pseudoheader {
    pub src_addr: IPAddr,
    pub dst_addr: IPAddr,
    pub udp_len: u32,
    pub padded_next_header: u32, //will always be 17
}

impl Default for IPv6Pseudoheader {
    fn default() -> IPv6Pseudoheader {
        IPv6Pseudoheader {
            src_addr: IPAddr::new(),
            dst_addr: IPAddr::new(),
            udp_len: 0,
            padded_next_header: 17 as u32, //17 is UDP protocol value
        }
    }
}

impl IPv6Pseudoheader {
    pub fn new() -> IPv6Pseudoheader {
        IPv6Pseudoheader::default()
    }
}

// Define some commong sixlowpan enums. These are copied from lowpan_frag_test.
// Paul says that SAC and DAC are only needed for the lowpan frag testing, which
// I don't entirely understand bc it seems users would still need to classify
// the different possible compression types any time they are choosing how to
// send 6lowpan packets. Pending a discussion with him on this, I am choosing
// to modify his prepare_ipv6_packet code as little as possible, so all of these
// enums are still present.

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum TF {
    Inline = 0b00,
    Traffic = 0b01,
    Flow = 0b10,
    TrafficFlow = 0b11,
}

#[derive(Copy,Clone,Debug)]
pub enum SAC {
    Inline,
    LLP64,
    LLP16,
    LLPIID,
    Unspecified,
    Ctx64,
    Ctx16,
    CtxIID,
}

#[derive(Copy,Clone,Debug)]
pub enum DAC {
    Inline,
    LLP64,
    LLP16,
    LLPIID,
    Ctx64,
    Ctx16,
    CtxIID,
    McastInline,
    Mcast48,
    Mcast32,
    Mcast8,
    McastCtx,
}


// Function that computes the UDP checksum of a UDP packet
pub fn compute_udp_checksum(pseudo_header: IPv6Pseudoheader, src_port: u16, dst_port: u16, ip6_packet_len: usize, ip6_hdr_size: usize,
                            ip6_packet: &'static [u8]) -> u16 {

    //This checksum is calculated according to some of the recommendations found in RFC 1071.
    let mut sum: u32 = 0;
    {
        //First, iterate through src/dst address and add them to the sum
        let mut i = 0;
        while i <= 14 { //Need to check this math against openThread device
            let msb_src: u16 = ((pseudo_header.src_addr.0[i]) as u16) << 8;
            let lsb_src: u16 = pseudo_header.src_addr.0[i+1] as u16;
            let temp_src: u16 = msb_src + lsb_src;
            sum += temp_src as u32;


            let msb_dst: u16 = ((pseudo_header.dst_addr.0[i]) as u16) << 8;
            let lsb_dst: u16 = pseudo_header.dst_addr.0[i+1] as u16;
            let temp_dst: u16 = msb_dst + lsb_dst;
            sum += temp_dst as u32;

            i += 2; //Iterate two bytes at a time bc 16 bit checksum
        }
        debug!("Checksum is currently: {}", sum);
    }
    sum += pseudo_header.udp_len;
    //Finally, add UDP next header
    sum += pseudo_header.padded_next_header;

    //Next, add the UDP header elements to the sum
    sum += src_port as u32;
    sum += dst_port as u32;
    sum += pseudo_header.udp_len; 
    //Now just need to iterate thru data and add it to the sum
    {
        let mut i = 0;
        while i < (ip6_packet_len - ip6_hdr_size - 8) {
            let msb_dat: u16 = ((ip6_packet[ip6_hdr_size + 8 + i]) as u16) << 8;
            let lsb_dat: u16 = ip6_packet[ip6_hdr_size + 8 + i +1] as u16;
            let temp_dat: u16 = msb_dat + lsb_dat;
            sum += temp_dat as u32;

            i += 2; //Iterate two bytes at a time bc 16 bit checksum
        }
        debug!("Checksum is currently: {}", sum);
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
    
    (sum as u16).to_be() //Convert result to u16 in network byte order

}



// Function that simply calls on lowpan_fragments transmit_packet() function
pub fn send_udp_packet<'a, A: time::Alarm>(frag_state: &'a FragState<'a, A>,
                       tx_state: &'a TxState<'a>,
                       src_mac_addr: MacAddress,
                       dst_mac_addr: MacAddress,
                       ip6_packet: &'static mut [u8],
                       ip6_packet_len: usize
                       ) {

    let ret_code = frag_state.transmit_packet(src_mac_addr,
                                              dst_mac_addr,
                                              ip6_packet,
                                              ip6_packet_len,
                                              None,
                                              tx_state,
                                              true,
                                              true);

    debug!("Ret code: {:?}", ret_code);
}

// Function copied from lowpan_frag_test to create an ipv6 packet - credit to Paul
// Modifications were made so that it taken in additional fields and now creates a UDP 
// datagram inside an IPv6 packet which is ready to be sent by the 6lowpan layer
pub fn udp_ipv6_prepare_packet(tf: TF, hop_limit: u8, sac: SAC, dac: DAC, 
                               ip6_packet: &'static mut [u8], ip6_packet_len: usize, 
                               ip6_hdr_size: usize, src_addr: IPAddr, dst_addr: IPAddr, 
                               src_mac_addr: MacAddress, dst_mac_addr: MacAddress, 
                               mlp: [u8; 8], src_port: u16, dst_port: u16, 
                               ip6_header: &mut IP6Header, udp_header: &mut UDPHeader) {
    
    //First step of preparing UDP Packet is setting the headers:

    udp_header.set_src_port(src_port);
    udp_header.set_dst_port(dst_port);
    udp_header.set_len((ip6_packet_len - ip6_hdr_size) as u16);
    udp_header.set_cksum((0 as u16).to_be()); //Init to 0 then replace with actual

    let mut pseudo_header = IPv6Pseudoheader::new();
    pseudo_header.src_addr = src_addr;
    pseudo_header.dst_addr = dst_addr;
    pseudo_header.udp_len = (ip6_packet_len - ip6_hdr_size) as u32;

    let cksum = compute_udp_checksum(pseudo_header, src_port, dst_port, ip6_packet_len, ip6_hdr_size, ip6_packet);


    debug!("Checksum of packet (host order): {}", u16::from_be(cksum));
    udp_header.set_cksum(cksum);

//Now, time to construct the IP Header and send to the 6lowpan layer

    {
        *ip6_header = IP6Header::new();
//Currently, setting the IPv6 payload length and the UDP length seems to have no effect. I beleive this is due to some UDP header compression which is occuring, but not sure where this can be found.
        ip6_header.set_payload_len((ip6_packet_len - ip6_hdr_size) as u16);//Should I be including a check that ip6_packet_len <= ip6_packet.len()?

        if tf != TF::TrafficFlow {
            ip6_header.set_ecn(0b01);
        }
        if (tf as u8) & (TF::Traffic as u8) != 0 {
            ip6_header.set_dscp(0b000000);
        } else {
            ip6_header.set_dscp(0b101010);
        }

        if (tf as u8) & (TF::Flow as u8) != 0 {
            ip6_header.set_flow_label(0);
        } else {
            ip6_header.set_flow_label(0xABCDE);
        }

        ip6_header.set_next_header(ip6_nh::UDP);//Hudson Edit

        ip6_header.set_hop_limit(hop_limit);

        match sac {
            SAC::Inline => {
                ip6_header.src_addr = src_addr;
            }
            SAC::LLP64 => {
                // LLP::xxxx:xxxx:xxxx:xxxx
                ip6_header.src_addr.set_unicast_link_local();
                ip6_header.src_addr.0[8..16].copy_from_slice(&src_addr.0[8..16]);
            }
            SAC::LLP16 => {
                // LLP::ff:fe00:xxxx
                ip6_header.src_addr.set_unicast_link_local();
                // Distinct from compute_iid because the U/L bit is not flipped
                ip6_header.src_addr.0[11] = 0xff;
                ip6_header.src_addr.0[12] = 0xfe;
                ip6_header.src_addr.0[14..16].copy_from_slice(&src_addr.0[14..16]);
            }
            SAC::LLPIID => {
                // LLP::IID
                ip6_header.src_addr.set_unicast_link_local();
                ip6_header.src_addr.0[8..16].copy_from_slice(&lowpan::compute_iid(&src_mac_addr));
            }
            SAC::Unspecified => {}
            SAC::Ctx64 => {
                // MLP::xxxx:xxxx:xxxx:xxxx
                ip6_header.src_addr.set_prefix(&mlp, 64);
                ip6_header.src_addr.0[8..16].copy_from_slice(&src_addr.0[8..16]);
            }
            SAC::Ctx16 => {
                // MLP::ff:fe00:xxxx
                ip6_header.src_addr.set_prefix(&mlp, 64);
                // Distinct from compute_iid because the U/L bit is not flipped
                ip6_header.src_addr.0[11] = 0xff;
                ip6_header.src_addr.0[12] = 0xfe;
                ip6_header.src_addr.0[14..16].copy_from_slice(&src_addr.0[14..16]);
            }
            SAC::CtxIID => {
                // MLP::IID
                ip6_header.src_addr.set_prefix(&mlp, 64);
                ip6_header.src_addr.0[8..16].copy_from_slice(&lowpan::compute_iid(&src_mac_addr));
            }
        }

        match dac {
            DAC::Inline => {
                ip6_header.dst_addr = dst_addr;
            }
            DAC::LLP64 => {
                // LLP::xxxx:xxxx:xxxx:xxxx
                ip6_header.dst_addr.set_unicast_link_local();
                ip6_header.dst_addr.0[8..16].copy_from_slice(&dst_addr.0[8..16]);
            }
            DAC::LLP16 => {
                // LLP::ff:fe00:xxxx
                ip6_header.dst_addr.set_unicast_link_local();
                // Distinct from compute_iid because the U/L bit is not flipped
                ip6_header.dst_addr.0[11] = 0xff;
                ip6_header.dst_addr.0[12] = 0xfe;
                ip6_header.dst_addr.0[14..16].copy_from_slice(&src_addr.0[14..16]);
            }
            DAC::LLPIID => {
                // LLP::IID
                ip6_header.dst_addr.set_unicast_link_local();
                ip6_header.dst_addr.0[8..16].copy_from_slice(&lowpan::compute_iid(&dst_mac_addr));
            }
            DAC::Ctx64 => {
                // MLP::xxxx:xxxx:xxxx:xxxx
                ip6_header.dst_addr.set_prefix(&mlp, 64);
                ip6_header.dst_addr.0[8..16].copy_from_slice(&src_addr.0[8..16]);
            }
            DAC::Ctx16 => {
                // MLP::ff:fe00:xxxx
                ip6_header.dst_addr.set_prefix(&mlp, 64);
                // Distinct from compute_iid because the U/L bit is not flipped
                ip6_header.dst_addr.0[11] = 0xff;
                ip6_header.dst_addr.0[12] = 0xfe;
                ip6_header.dst_addr.0[14..16].copy_from_slice(&src_addr.0[14..16]);
            }
            DAC::CtxIID => {
                // MLP::IID
                ip6_header.dst_addr.set_prefix(&mlp, 64);
                ip6_header.dst_addr.0[8..16].copy_from_slice(&lowpan::compute_iid(&dst_mac_addr));
            }
            DAC::McastInline => {
                // first byte is ff, that's all we know
                ip6_header.dst_addr = dst_addr;
                ip6_header.dst_addr.0[0] = 0xff;
            }
            DAC::Mcast48 => {
                // ffXX::00XX:XXXX:XXXX
                ip6_header.dst_addr.0[0] = 0xff;
                ip6_header.dst_addr.0[1] = dst_addr.0[1];
                ip6_header.dst_addr.0[11..16].copy_from_slice(&dst_addr.0[11..16]);
            }
            DAC::Mcast32 => {
                // ffXX::00XX:XXXX
                ip6_header.dst_addr.0[0] = 0xff;
                ip6_header.dst_addr.0[1] = dst_addr.0[1];
                ip6_header.dst_addr.0[13..16].copy_from_slice(&dst_addr.0[13..16]);
            }
            DAC::Mcast8 => {
                // ff02::00XX
                ip6_header.dst_addr.0[0] = 0xff;
                ip6_header.dst_addr.0[1] = dst_addr.0[1];
                ip6_header.dst_addr.0[15] = dst_addr.0[15];
            }
            DAC::McastCtx => {
                // ffXX:XX + plen + pfx64 + XXXX:XXXX
                ip6_header.dst_addr.0[0] = 0xff;
                ip6_header.dst_addr.0[1] = dst_addr.0[1];
                ip6_header.dst_addr.0[2] = dst_addr.0[2];
                ip6_header.dst_addr.0[3] = 64 as u8;
                ip6_header.dst_addr.0[4..12].copy_from_slice(&mlp);
                ip6_header.dst_addr.0[12..16].copy_from_slice(&dst_addr.0[12..16]);
            }
        }
    }
    debug!("Packet with tf={:?} hl={} sac={:?} dac={:?}",
           tf,
           hop_limit,
           sac,
           dac);
}

