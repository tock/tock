// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! A test Ethernet capsule
//!
//! This capsule runs a very simple network application. It handles basic ARP
//! discovery, can respond to ICMP pings, and has UDP and TCP echo servers (both
//! on port 11).

use kernel::{
    component::Component,
    hil::{
        ethernet::{EthernetAdapterDatapath, EthernetAdapterDatapathClient},
        usb::Client,
    },
    static_init,
    utilities::cells::TakeCell,
};
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use zerocopy::{
    byteorder::network_endian::U16, network_endian::U32, AsBytes, FromBytes, LayoutVerified,
    Unaligned,
};

fn checksum<const N: usize>(payload: [&[u8]; N]) -> u16 {
    let mut sum: u32 = 0;

    let mut iterator = payload.into_iter().flatten();
    loop {
        match iterator.next_chunk() {
            Ok([a, b]) => sum += *b as u32 | (*a as u32) << 8,
            Err(a) => {
                a.into_iter().for_each(|e| sum += *e as u32);
                break;
            }
        }
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    return !(sum as u16);
}

#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
struct EthernetHeader {
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    ethertype: U16,
}

#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
struct IpV4Header {
    version_ihl: u8,
    dsp_ecn: u8,
    total_len: U16,
    ident: U16,
    flags_fragment: U16,
    ttl: u8,
    protocol: u8,
    checksum: U16,
    source: [u8; 4],
    destination: [u8; 4],
}

#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
struct IcmpV4Header {
    icmp_type: u8,
    icmp_code: u8,
    checksum: U16,
    rest: [u8; 4],
}

#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
struct UDPHeader {
    source_port: U16,
    destination_port: U16,
    length: U16,
    checksum: U16,
}

#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
struct TCPHeader {
    source_port: U16,
    destination_port: U16,
    sequence_number: U32,
    ack_number: U32,
    data_offset: u8,
    flags: u8,
    window_size: U16,
    checksum: U16,
    urgent_pointer: U16,
}

#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
struct Arp {
    hw_type: U16,
    protocol_type: U16,
    hw_addr_len: u8,
    proto_addr_len: u8,
    operation: U16,
    sender_hw_addr: [u8; 6],
    sender_proto_addr: [u8; 4],
    target_hw_addr: [u8; 6],
    target_proto_addr: [u8; 4],
}

pub(crate) const SRC_ADDR: [u8; 6] = [0x06, 0x1b, 0x5f, 0x44, 0x31, 0xee];

pub(crate) struct NetworkTest {
    adapter: &'static dyn EthernetAdapterDatapath<'static>,
    buffer: TakeCell<'static, [u8]>,
    my_ipv4_addr: [u8; 4],
    my_mac_addr: [u8; 6],
}

impl NetworkTest {
    pub fn new(
        adapter: &'static dyn EthernetAdapterDatapath<'static>,
        buffer: &'static mut [u8; 1522],
        my_ipv4_addr: [u8; 4],
        my_mac_addr: [u8; 6],
    ) -> NetworkTest {
        NetworkTest {
            adapter,
            buffer: TakeCell::new(buffer),
            my_ipv4_addr,
            my_mac_addr,
        }
    }

    fn handle_arp(&self, eth_header: &EthernetHeader, eth_body: &[u8]) {
        let arp: LayoutVerified<&[u8], Arp> = LayoutVerified::new_unaligned_from_prefix(eth_body)
            .unwrap()
            .0;

        self.buffer.take().and_then(|buffer| {
            let len = {
                let (mut eth_header_resp, eth_response_body): (
                    LayoutVerified<&mut [u8], EthernetHeader>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(&mut *buffer)?;
                *eth_header_resp = EthernetHeader {
                    dst_mac: eth_header.src_mac,
                    src_mac: self.my_mac_addr,
                    ethertype: eth_header.ethertype,
                };
                let (mut arp_response, _): (LayoutVerified<&mut [u8], Arp>, _) =
                    LayoutVerified::new_unaligned_from_prefix(eth_response_body)?;
                *arp_response = Arp {
                    hw_type: arp.hw_type,
                    protocol_type: arp.protocol_type,
                    hw_addr_len: arp.hw_addr_len,
                    proto_addr_len: arp.proto_addr_len,
                    operation: 2u16.into(),
                    target_hw_addr: arp.sender_hw_addr,
                    target_proto_addr: arp.sender_proto_addr,
                    sender_hw_addr: self.my_mac_addr,
                    sender_proto_addr: self.my_ipv4_addr,
                };
                eth_header_resp.bytes().len() + arp_response.bytes().len()
            };
            if let Err((_, buffer)) = self.adapter.transmit_frame(buffer, len as u16, 0) {
                kernel::debug!("Uh oh");
                self.buffer.replace(buffer);
            }
            Some(())
        });
    }

    fn handle_ipv4(&self, eth_header: &EthernetHeader, eth_body: &[u8]) {
        let (ip, ip_body): (LayoutVerified<&[u8], IpV4Header>, &[u8]) =
            LayoutVerified::new_unaligned_from_prefix(eth_body).unwrap();

        match ip.protocol {
            0x01 => {
                // ICMP
                self.handle_icmpv4(eth_header, &ip, ip_body);
            }
            0x06 => {
                // TCP
                self.handle_tcpv4(eth_header, &ip, ip_body);
            }
            0x11 => {
                // UDP
                self.handle_udpv4(eth_header, &ip, ip_body);
            }
            a => kernel::debug!("Unknown IPv4 packet {:#x}", a),
        }
    }

    fn send_icmpv4(
        &self,
        dst_mac_address: [u8; 6],
        dest_ip_address: [u8; 4],
        icmp_type: u8,
        icmp_code: u8,
        icmp_rest: [u8; 4],
        body: &[u8],
    ) -> Option<usize> {
        self.buffer.take().and_then(|buffer| {
            let len = {
                let (mut eth_header_resp, eth_response_body): (
                    LayoutVerified<&mut [u8], EthernetHeader>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(&mut *buffer)?;
                let (mut ip_response, ip_response_body): (
                    LayoutVerified<&mut [u8], IpV4Header>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(eth_response_body)?;
                let (mut icmp_response, icmp_response_body): (
                    LayoutVerified<&mut [u8], IcmpV4Header>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(ip_response_body)?;

                *icmp_response = IcmpV4Header {
                    icmp_type,
                    icmp_code,
                    checksum: 0.into(),
                    rest: icmp_rest,
                };
                let icmp_response_body = &mut icmp_response_body[..body.len()];
                icmp_response_body.copy_from_slice(body);

                icmp_response.checksum =
                    checksum([icmp_response.bytes(), icmp_response_body]).into();

                *ip_response = IpV4Header {
                    version_ihl: (4 << 4) | 5,
                    dsp_ecn: 0,
                    total_len: (20u16 + icmp_response.bytes().len() as u16 + body.len() as u16)
                        .into(),
                    ident: 0.into(),
                    flags_fragment: 0.into(),
                    ttl: 20,
                    protocol: 0x01,
                    checksum: 0.into(),
                    source: self.my_ipv4_addr,
                    destination: dest_ip_address,
                };
                ip_response.checksum = checksum([ip_response.bytes()]).into();

                *eth_header_resp = EthernetHeader {
                    dst_mac: dst_mac_address.into(),
                    src_mac: self.my_mac_addr,
                    ethertype: 0x800u16.into(),
                };

                eth_header_resp.bytes().len()
                    + ip_response.bytes().len()
                    + icmp_response.bytes().len()
                    + body.len()
            };
            if let Err((_, buffer)) = self.adapter.transmit_frame(buffer, len as u16, 0) {
                kernel::debug!("Uh oh");
                self.buffer.replace(buffer);
            }
            Some(len)
        })
    }

    fn handle_icmpv4(&self, eth_header: &EthernetHeader, ip_header: &IpV4Header, ip_body: &[u8]) {
        let (icmp, icmp_body): (LayoutVerified<&[u8], IcmpV4Header>, _) =
            LayoutVerified::new_unaligned_from_prefix(ip_body).unwrap();
        let body_size: u16 = ip_header.total_len.get() + 20 + 8;
        let icmp_body = &icmp_body[..core::cmp::min(body_size as usize, icmp_body.len())];

        match (icmp.icmp_type, icmp.icmp_code) {
            (8, 0) => {
                //echo request
                self.send_icmpv4(
                    eth_header.src_mac,
                    ip_header.source,
                    0,
                    0,
                    icmp.rest,
                    icmp_body,
                );
            }
            _ => {}
        }
    }

    fn send_udpv4(
        &self,
        dst_mac_address: [u8; 6],
        dest_ip_address: [u8; 4],
        source_port: u16,
        dest_port: u16,
        body: &[u8],
    ) -> Option<usize> {
        self.buffer.take().and_then(|buffer| {
            let len = {
                let (mut eth_header_resp, eth_response_body): (
                    LayoutVerified<&mut [u8], EthernetHeader>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(&mut *buffer)?;
                let (mut ip_response, ip_response_body): (
                    LayoutVerified<&mut [u8], IpV4Header>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(eth_response_body)?;
                let (mut udp_response, udp_response_body): (
                    LayoutVerified<&mut [u8], UDPHeader>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(ip_response_body)?;

                *udp_response = UDPHeader {
                    source_port: source_port.into(),
                    destination_port: dest_port.into(),
                    length: (8 + body.len() as u16).into(),
                    checksum: 0.into(),
                };
                let udp_response_body = &mut udp_response_body[..body.len()];
                udp_response_body.copy_from_slice(body);

                //udp_response.checksum = checksum(todo!()).into(); // not necessary in UDP if set to zero

                *ip_response = IpV4Header {
                    version_ihl: (4 << 4) | 5,
                    dsp_ecn: 0,
                    total_len: (20u16
                        + udp_response.bytes().len() as u16
                        + udp_response_body.len() as u16)
                        .into(),
                    ident: 0.into(),
                    flags_fragment: 0.into(),
                    ttl: 20,
                    protocol: 0x11,
                    checksum: 0.into(),
                    source: self.my_ipv4_addr,
                    destination: dest_ip_address,
                };
                ip_response.checksum = checksum([ip_response.bytes()]).into();

                *eth_header_resp = EthernetHeader {
                    dst_mac: dst_mac_address,
                    src_mac: self.my_mac_addr,
                    ethertype: 0x800u16.into(),
                };

                eth_header_resp.bytes().len()
                    + ip_response.bytes().len()
                    + udp_response.bytes().len()
                    + udp_response_body.len()
            };
            if let Err((_, buffer)) = self.adapter.transmit_frame(buffer, len as u16, 0) {
                kernel::debug!("Uh oh");
                self.buffer.replace(buffer);
            }
            Some(len)
        })
    }

    fn handle_udpv4(&self, eth_header: &EthernetHeader, ip_header: &IpV4Header, ip_body: &[u8]) {
        let (udp_header, udp_body): (LayoutVerified<&[u8], UDPHeader>, _) =
            LayoutVerified::new_unaligned_from_prefix(ip_body).unwrap();
        let udp_body =
            &udp_body[..core::cmp::min(udp_body.len(), udp_header.length.get() as usize - 8)];
        match udp_header.destination_port.get() {
            11 => {
                // Echo server
                self.send_udpv4(
                    eth_header.src_mac,
                    ip_header.source,
                    11,
                    udp_header.source_port.into(),
                    udp_body,
                );
            }
            _ => {}
        }
    }

    fn send_tcpv4(
        &self,
        dst_mac_address: [u8; 6],
        dest_ip_address: [u8; 4],
        source_port: u16,
        dest_port: u16,
        body: &[u8],
        seq_number: u32,
        ack_number: Option<u32>,
        flags: u8,
        options: &[u8],
    ) -> Option<usize> {
        self.buffer.take().and_then(|buffer| {
            let len = {
                let (mut eth_header_resp, eth_response_body): (
                    LayoutVerified<&mut [u8], EthernetHeader>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(&mut *buffer)?;
                let (mut ip_response, ip_response_body): (
                    LayoutVerified<&mut [u8], IpV4Header>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(eth_response_body)?;
                let (mut tcp_response, tcp_response_body): (
                    LayoutVerified<&mut [u8], TCPHeader>,
                    _,
                ) = LayoutVerified::new_unaligned_from_prefix(ip_response_body)?;
                let tcp_response_body = &mut tcp_response_body[..(options.len() + body.len())];

                *tcp_response = TCPHeader {
                    source_port: source_port.into(),
                    destination_port: dest_port.into(),
                    sequence_number: seq_number.into(),
                    ack_number: ack_number.unwrap_or(0).into(),
                    data_offset: ((5 + (options.len() as u8 / 4)) << 4),
                    flags: flags | if ack_number.is_some() { 0b10000 } else { 0 },
                    window_size: 64240.into(),
                    checksum: 0.into(),
                    urgent_pointer: 0.into(),
                };

                {
                    let (options_response, body_response) =
                        tcp_response_body.split_at_mut(options.len());
                    options_response.copy_from_slice(options);
                    body_response.copy_from_slice(body);
                }

                let tcp_length: U16 = (20 + (options.len() + body.len()) as u16).into();
                tcp_response.checksum = checksum([
                    &self.my_ipv4_addr,
                    &dest_ip_address,
                    &[0],
                    &[0x06],
                    tcp_length.as_ref(),
                    tcp_response.bytes(),
                    tcp_response_body,
                ])
                .into();

                *ip_response = IpV4Header {
                    version_ihl: (4 << 4) | 5,
                    dsp_ecn: 0,
                    total_len: (20u16
                        + tcp_response.bytes().len() as u16
                        + tcp_response_body.len() as u16)
                        .into(),
                    ident: 0.into(),
                    flags_fragment: 0.into(),
                    ttl: 20,
                    protocol: 0x06,
                    checksum: 0.into(),
                    source: self.my_ipv4_addr,
                    destination: dest_ip_address,
                };
                ip_response.checksum = checksum([ip_response.bytes()]).into();

                *eth_header_resp = EthernetHeader {
                    dst_mac: dst_mac_address,
                    src_mac: self.my_mac_addr,
                    ethertype: 0x800u16.into(),
                };

                eth_header_resp.bytes().len()
                    + ip_response.bytes().len()
                    + tcp_response.bytes().len()
                    + tcp_response_body.len()
            };
            if let Err((_, buffer)) = self.adapter.transmit_frame(buffer, len as u16, 0) {
                kernel::debug!("Uh oh");
                self.buffer.replace(buffer);
            }
            Some(len)
        })
    }

    fn handle_tcpv4(&self, eth_header: &EthernetHeader, ip_header: &IpV4Header, ip_body: &[u8]) {
        let (tcp_header, tcp_body): (LayoutVerified<&[u8], TCPHeader>, _) =
            LayoutVerified::new_unaligned_from_prefix(ip_body).unwrap();
        let data_offset = (tcp_header.data_offset >> 4) * 4 - 20;
        let (_options, body) =
            tcp_body.split_at(core::cmp::min(data_offset as usize, tcp_body.len()));
        match tcp_header.destination_port.get() {
            11 => {
                if tcp_header.flags & 0b10 != 0 {
                    // SYN
                    self.send_tcpv4(
                        eth_header.src_mac,
                        ip_header.source,
                        11,
                        tcp_header.source_port.get(),
                        &[],
                        22,
                        Some(tcp_header.sequence_number.get() + 1),
                        0b10,
                        &[],
                    );
                } else if tcp_header.flags & 0b10000 != 0 {
                    // ACK
                    if tcp_header.flags & 0b1 != 0 {
                        // FIN
                        self.send_tcpv4(
                            eth_header.src_mac,
                            ip_header.source,
                            11,
                            tcp_header.source_port.get(),
                            &[],
                            tcp_header.ack_number.get(),
                            Some(tcp_header.sequence_number.get() + 1),
                            0b1,
                            &[],
                        );
                    } else if tcp_header.flags & 0b1000 != 0 {
                        // PSH
                        kernel::debug!("TCP out: {}", core::str::from_utf8(body).unwrap());
                        let response_body = &[];
                        self.send_tcpv4(
                            eth_header.src_mac,
                            ip_header.source,
                            11,
                            tcp_header.source_port.get(),
                            response_body,
                            tcp_header.ack_number.get() + response_body.len() as u32,
                            Some(tcp_header.sequence_number.get() + body.len() as u32),
                            0,
                            &[],
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

impl EthernetAdapterDatapathClient for NetworkTest {
    fn transmit_frame_done(
        &self,
        _err: Result<(), kernel::ErrorCode>,
        packet_buffer: &'static mut [u8],
        _len: u16,
        _packet_identifier: usize,
        _timestamp: Option<u64>,
    ) {
        self.buffer.replace(packet_buffer);
    }

    fn received_frame(&self, frame: &[u8], _timestamp: Option<u64>) {
        if frame.len() < 12 {
            kernel::debug!("frame: {:#x?}", frame);
        } else {
            let (eth_header, eth_body): (LayoutVerified<&[u8], EthernetHeader>, _) =
                LayoutVerified::new_unaligned_from_prefix(frame).unwrap();
            match eth_header.ethertype.get() {
                0x0806 => {
                    //ARP
                    self.handle_arp(&eth_header, eth_body);
                }
                0x0800 => {
                    // IPv4
                    self.handle_ipv4(&eth_header, eth_body);
                }
                a => kernel::debug!("Unknown {:#x}", a),
            }
        }
    }
}

/// Initializes the `NetworkTest` capsule using USB EEM over the Nordic's USB controller.
///
/// It self-assigns the device an IP address of `192.168.1.50` and uses the
/// `SRC_ADDR` MAC address.
#[allow(dead_code)]
pub(crate) unsafe fn setup(nrf_peripherals: &'static Nrf52840DefaultPeripherals<'static>) {
    // Create the strings we include in the USB descriptor. We use the hardcoded
    // DEVICEADDR register on the nRF52 to set the serial number.
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str =
        nrf52840::ficr::FICR_INSTANCE.address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Tock",               // Manufacturer
            "NRF52840DK Eth",     // Product
            serial_number_string, // Serial number
        ]
    );

    let eem = components::eem::CdcEemComponent::new(
        &nrf_peripherals.usbd,
        capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x2341,
        0x005a,
        strings,
    )
    .finalize(components::cdc_eem_component_static!(nrf52840::usbd::Usbd,));

    let ping_buffer = static_init!([u8; 1522], [0; 1522]);
    let ping_client = static_init!(
        NetworkTest,
        NetworkTest::new(eem, ping_buffer, [192, 168, 1, 50], SRC_ADDR)
    );
    eem.set_client(ping_client);
    eem.enable();
    eem.attach();
}
