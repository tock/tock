//! A dummy sixlowpan/IP sender

use capsules::net::lowpan::{DummyStore, LoWPAN};
use capsules::net::ip::{IP6Header, MacAddr, IPAddr};
use core::mem;
use kernel::hil::radio;

pub const SRC_ADDR: IPAddr = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 
                              0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
pub const DST_ADDR: IPAddr = [0; 16];
pub const SRC_MAC_ADDR: MacAddr = MacAddr::LongAddr([0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17]);
pub const DST_MAC_ADDR: MacAddr = MacAddr::LongAddr([0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f]);

pub const IP6_HDR_SIZE: usize = 40;
pub const PAYLOAD_LEN: usize = 10;
pub static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0 as u8; radio::MAX_BUF_SIZE];

pub fn sixlowpan_dummy_test<R: radio::Radio>(radio: &R) {
    let mut ip6_datagram = [0 as u8; IP6_HDR_SIZE + PAYLOAD_LEN];
    {
        let mut payload = &mut ip6_datagram[IP6_HDR_SIZE..];
        for i in 0..PAYLOAD_LEN {
            payload[i] = i as u8;
        }
    }
    {
        let mut ip6_header: &mut IP6Header = unsafe {
            mem::transmute(ip6_datagram.as_mut_ptr())
        };
        *ip6_header = IP6Header::new();
        ip6_header.set_src_addr(SRC_ADDR);
        ip6_header.set_dst_addr(DST_ADDR);
        ip6_header.set_payload_len(PAYLOAD_LEN as u16);
    }
    unsafe {
        send_ipv6_packet(radio, SRC_MAC_ADDR, DST_MAC_ADDR,
                         &ip6_datagram[0..IP6_HDR_SIZE + PAYLOAD_LEN]);
    }
}

pub unsafe fn send_ipv6_packet<R: radio::Radio>(radio: &R,
                                                src_mac_addr: MacAddr,
                                                dst_mac_addr: MacAddr,
                                                ip6_datagram: &[u8]) {
    radio.config_set_pan(0xABCD);
    match src_mac_addr {
        MacAddr::ShortAddr(addr) => radio.config_set_address(addr),
        MacAddr::LongAddr(addr) => radio.config_set_address_long(addr)
    };

    let src_long = match src_mac_addr {
        MacAddr::ShortAddr(_) => false,
        MacAddr::LongAddr(_) => true
    };
    let dst_long = match dst_mac_addr {
        MacAddr::ShortAddr(_) => false,
        MacAddr::LongAddr(_) => true
    };
    let offset = radio.payload_offset(src_long, dst_long) as usize;

    let store = DummyStore {};
    let lowpan = LoWPAN::new(&store);
    let (consumed, written) = lowpan
        .compress(&ip6_datagram,
                  src_mac_addr,
                  dst_mac_addr,
                  &mut RF233_BUF[offset..])
        .expect("Error");
    let payload_len = ip6_datagram.len() - consumed;
    RF233_BUF[offset + written..offset + written + payload_len]
        .copy_from_slice(&ip6_datagram[consumed..ip6_datagram.len()]);

    // Transmit len is 802.15.4 header + LoWPAN-compressed packet size
    let transmit_len = radio.header_size(src_long, dst_long)
        + (written + payload_len) as u8;
    match dst_mac_addr {
        MacAddr::ShortAddr(addr) => radio.transmit(addr,
                                                   &mut RF233_BUF,
                                                   transmit_len,
                                                   src_long),
        MacAddr::LongAddr(addr) => radio.transmit_long(addr,
                                                       &mut RF233_BUF,
                                                       transmit_len,
                                                       src_long)
    };
}
