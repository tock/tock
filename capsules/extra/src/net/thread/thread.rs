// pub struct ThreadState {
//     pending_parent_req: bool,
// }

// impl ThreadState {
//     pub fn new() -> ThreadState {
//         ThreadState {
//             pending_parent_req: false,
//         }
//     }
// }

use capsules_core::stream::encode_bytes;
use kernel::ErrorCode;

use crate::net::{ieee802154::MacAddress, ipv6::ip_utils::IPAddr};

pub const MULTICAST_IPV6: [u8; 16] = [
    0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
];

/* THREAD SPEC 5.2.2.4 (V1.3.0) -- A Thread Device MUST assign a link-local IPv6 address where the
interface identifier is set to the MAC Extended Address with the universal/local bit inverted.
------------------------------------------------------------------------------------------------
The universal/local bit is the 2nd least significant bit; invert by xor first byte of MAC addr
with 2 */
pub fn generate_src_ipv6(macaddr: &[u8; 8]) -> [u8; 16] {
    let mut output: [u8; 16] = [0; 16];
    let mut lower_bytes = macaddr.clone();
    lower_bytes[0] = macaddr[0] ^ 2;
    let upper_bytes: [u8; 8] = [0xfe, 0x80, 0, 0, 0, 0, 0, 0];

    encode_bytes(&mut output[..8], &upper_bytes);
    encode_bytes(&mut output[8..16], &lower_bytes);
    output
}

pub fn mac_from_ipv6(ipv6: IPAddr) -> [u8; 8] {
    let mut output: [u8; 8] = [0; 8];
    let mut lower_bytes = ipv6.clone().0;
    lower_bytes[8] = lower_bytes[8] ^ 2;

    encode_bytes(&mut output[..8], &lower_bytes[8..16]);
    output
}

pub fn find_challenge(buf: &[u8]) -> Result<&[u8], ErrorCode> {
    let mut index = 0;
    while index < buf.len() {
        let tlv_len = buf[index + 1] as usize;
        if buf[index] == 3 {
            return Ok(&buf[index + 2..index + 2 + tlv_len]);
        } else {
            index += tlv_len + 2;
        }
    }
    Err(ErrorCode::FAIL)
}

pub enum ThreadState {
    CryptSend(IPAddr, MacAddress, usize),
    CryptReceive(IPAddr, usize),
    CryptReady,
    Sending,
}
