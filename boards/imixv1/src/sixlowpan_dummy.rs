//! A dummy sixlowpan/IP sender

use capsules::net::ip;
use capsules::net::lowpan::{DummyStore, LoWPAN};
use capsules::net::ip::{IP6Header, MacAddr, IPAddr};
use core::mem;
use kernel::hil::radio;

pub const SRC_ADDR: IPAddr = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 
                              0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
pub const DST_ADDR: IPAddr = [0; 16];
pub const SRC_MAC_ADDR: [u8; 8] = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17];
pub const DST_MAC_ADDR: [u8; 8] = [0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f];

pub const IP6_HDR_SIZE: usize = 40;
pub const PAYLOAD_LEN: usize = 10;
pub static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0 as u8; radio::MAX_BUF_SIZE];

pub unsafe fn sixlowpan_dummy_test<R: radio::Radio>(rf233: &R) {
    let store = DummyStore {};
    let lowpan = LoWPAN::new(&store);
    let mut ip6_datagram = [0 as u8; IP6_HDR_SIZE + PAYLOAD_LEN];
    {
        let mut payload = &mut ip6_datagram[IP6_HDR_SIZE..];
        for i in 0..PAYLOAD_LEN {
            payload[i] = i as u8;
        }
    }
    {
        let mut ip6_header: &mut IP6Header = mem::transmute(ip6_datagram.as_mut_ptr());
        *ip6_header = IP6Header::new();
        ip6_header.set_src_addr(SRC_ADDR);
        ip6_header.set_dst_addr(DST_ADDR);
        ip6_header.set_payload_len(PAYLOAD_LEN as u16);
    }

    let offset = rf233.payload_offset(true, true) as usize;
    let (ip6_offset, header_size) = lowpan.compress(&ip6_datagram,
                                                    MacAddr::LongAddr(SRC_MAC_ADDR),
                                                    MacAddr::LongAddr(DST_MAC_ADDR),
                                                    &mut RF233_BUF[offset..])
                                                    .expect("Error");

    for i in 0..PAYLOAD_LEN {
        RF233_BUF[offset+header_size+i] = ip6_datagram[ip6_offset + i];
    }
    rf233.config_set_pan(0xABCD);
    rf233.config_set_address_long(SRC_MAC_ADDR);
    //rf233.config_commit();
    //while !rf233.is_on() {}
    rf233.transmit_long(DST_MAC_ADDR,
                        &mut RF233_BUF,
                        (PAYLOAD_LEN + header_size) as u8
                        + rf233.header_size(true, true),
                        true);
}
