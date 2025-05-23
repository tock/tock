// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

use crate::net::stream::{encode_bytes, SResult};
use crate::net::thread::tlv::{unwrap_tlv_offset, LinkMode, MulticastResponder, Tlv};
use crate::net::{ieee802154::MacAddress, ipv6::ip_utils::IPAddr};
pub const THREAD_PORT_NUMBER: u16 = 19788;

use kernel::ErrorCode;

pub const SECURITY_SUITE_LEN: usize = 1;
pub const AUX_SEC_HEADER_LENGTH: usize = 10;
pub const AUTH_DATA_LEN: usize = 42;
pub const IPV6_LEN: usize = 16;
const PARENT_REQUEST_MLE_SIZE: usize = 21;
pub const MULTICAST_IPV6: IPAddr = IPAddr([
    0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
]);

#[derive(Clone, Copy)]
pub struct NetworkKey {
    pub mle_key: [u8; 16],
    pub mac_key: [u8; 16],
}

pub enum ThreadState {
    SendParentReq,
    WaitingParentRsp,
    RecvParentRsp(IPAddr),
    SendChildIdReq(IPAddr),
    WaitingChildRsp,
    RecvChildRsp(IPAddr),
    SEDActive(IPAddr, MacAddress),
    SendUpdate(IPAddr, MacAddress),
    SendUDPMsg,
    Detached,
}

pub enum MleCommand {
    LinkRequest = 0,
    LinkAccept = 1,
    LinkAcceptAndRequest = 2,
    LinkAdvertisement = 4,
    DataRequest = 7,
    DataResponse = 8,
    ParentRequest = 9,
    ParentResponse = 10,
    ChildIdRequest = 11,
    ChildIdResponse = 12,
    ChildUpdateRequest = 13,
    ChildUpdateResponse = 14,
    Announce = 15,
    DiscoverRequest = 16,
    DiscoveryResponse = 17,
    LinkMetricsManagReq = 18,
    LinkMetricsManagResp = 19,
    LinkProbe = 20,
}

/// Helper function to generate a link-local IPV6 address
/// from the device's mac address.
pub fn generate_src_ipv6(macaddr: &[u8; 8]) -> IPAddr {
    // -----------------------------------------------------------------------------------------------
    // THREAD SPEC 5.2.2.4 (V1.3.0) -- A Thread Device MUST assign a link-local IPv6 address where the
    // interface identifier is set to the MAC Extended Address with the universal/local bit inverted.
    // ------------------------------------------------------------------------------------------------

    let mut output: [u8; 16] = [0; 16];
    let mut lower_bytes = *macaddr;

    // The universal/local bit is the 2nd least significant bit.
    // Invert by xor first byte of MAC addr with 2
    lower_bytes[0] = macaddr[0] ^ 2;
    let upper_bytes: [u8; 8] = [0xfe, 0x80, 0, 0, 0, 0, 0, 0];

    encode_bytes(&mut output[..8], &upper_bytes);
    encode_bytes(&mut output[8..16], &lower_bytes);
    IPAddr(output)
}

/// Helper function to recover the mac address from
/// an IPV6 address.
pub fn mac_from_ipv6(ipv6: IPAddr) -> [u8; 8] {
    // Helper function to generate the mac address from the mac address;
    // reversing the tranformation used/described in `generate_src_ipv6`
    let mut output: [u8; 8] = [0; 8];
    let mut lower_bytes = ipv6.0;
    lower_bytes[8] ^= 2;

    encode_bytes(&mut output[..8], &lower_bytes[8..16]);
    output
}

/// Helper function to locate the challenge TLV in a received
/// MLE packet. Return the challenge to be used as a response
/// TLV in reply.
fn find_challenge(buf: &[u8]) -> Result<&[u8], ErrorCode> {
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

/// Function to encode the crypt data into a/m data
pub fn encode_cryp_data(
    src_addr: IPAddr,
    dst_addr: IPAddr,
    aux_sec_header: &[u8; AUX_SEC_HEADER_LENGTH],
    payload: &[u8],
    output: &mut [u8],
) -> SResult {
    // --------------AUTH DATA----------------||-- M DATA--
    // SRC IPV6 || DST IPV6 || AUX SEC HEADER ||  PAYLOAD
    let mut off = enc_consume!(output; encode_bytes, &src_addr.0);
    off = enc_consume!(output, off; encode_bytes, &dst_addr.0);
    off = enc_consume!(output, off; encode_bytes, aux_sec_header);
    off = enc_consume!(output, off; encode_bytes, payload);
    stream_done!(off)
}

/// This helper function creates a parent request. For now,
/// this implementation hard codes all values for the parent request
pub fn form_parent_req() -> [u8; PARENT_REQUEST_MLE_SIZE] {
    // TODO: form parent request from alterable values, generate
    // challenge from random number generator
    let mut output = [0u8; PARENT_REQUEST_MLE_SIZE];
    let mut offset = 0;

    // Command: Parent Request //
    output[0..1].copy_from_slice(&[MleCommand::ParentRequest as u8]);
    offset += 1;

    // Mode TLV //
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::Mode(
            LinkMode::FullNetworkDataRequired as u8
                + LinkMode::FullThreadDevice as u8
                + LinkMode::SecureDataRequests as u8
                + LinkMode::ReceiverOnWhenIdle as u8,
        ),
        &mut output[offset..],
    ));

    // Challenge TLV //
    // TODO: challenge is hardcoded currently; randomly generate number
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::Challenge([0, 0, 0, 0, 0, 0, 0, 0]),
        &mut output[offset..],
    ));

    // Scan Mask TLV //
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::ScanMask(MulticastResponder::Router as u8),
        &mut output[offset..],
    ));

    // Version TLV //
    unwrap_tlv_offset(Tlv::encode(&Tlv::Version(4), &mut output[offset..]));

    output
}

/// This helper function creates a child id request. For now,
/// this implementation hard codes many of the values
pub fn form_child_id_req(
    recv_buf: &[u8],
    frame_count: u32,
) -> Result<([u8; 200], usize), ErrorCode> {
    let mut output: [u8; 200] = [0; 200];
    let mut offset = 0;

    /* -- Child ID Request TLVs (Thread Spec 4.5.1 (v1.3.0)) --
    Response TLV
    Link-layer Frame Counter TLV
    [MLE Frame Counter TLV] **optional if Link-layer frame counter is the same**
    Mode TLV
    Timeout TLV
    Version TLV
    [Address Registration TLV]
    [TLV Request TLV: Address16 (Network Data and/or Route)]
    [Active Timestamp TLV]
    [Pending Timestamp TLV]
    */

    // Command Child ID Request //
    output[0..1].copy_from_slice(&[MleCommand::ChildIdRequest as u8]);
    offset += 1;

    // Response TLV //
    let received_challenge_tlv: Result<&[u8], ErrorCode> = find_challenge(&recv_buf[1..]);

    if let Ok(received_challenge_tlv_inner) = received_challenge_tlv {
        // Encode response into output
        let mut rsp_buf: [u8; 8] = [0; 8];
        rsp_buf.copy_from_slice(received_challenge_tlv_inner);
        rsp_buf.reverse(); // NEED TO DISCUSS BIG/LITTLE ENDIAN ASSUMPTIONS
        offset += unwrap_tlv_offset(Tlv::encode(&Tlv::Response(rsp_buf), &mut output[offset..]));
    } else {
        // Challenge TLV not found; malformed request
        return Err(ErrorCode::FAIL);
    }

    // Link-layer Frame Counter TLV //
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::LinkLayerFrameCounter(0),
        &mut output[offset..],
    ));

    // MLE Frame Counter TLV //
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::MleFrameCounter(frame_count.to_be()),
        &mut output[offset..],
    ));

    // Mode TLV //
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::Mode(LinkMode::FullThreadDevice as u8 + LinkMode::ReceiverOnWhenIdle as u8),
        &mut output[offset..],
    ));

    // Timeout TLV //
    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::Timeout((10_u32).to_be()),
        &mut output[offset..],
    ));

    // Version TLV //
    offset += unwrap_tlv_offset(Tlv::encode(&Tlv::Version(4), &mut output[offset..]));

    // TODO: hardcoded for now, but replace in future
    output[offset..offset + 4].copy_from_slice(&[0x1b, 0x02, 0x00, 0x81]);
    offset += 4;

    offset += unwrap_tlv_offset(Tlv::encode(
        &Tlv::TlvRequest(&[0x0a, 0x0c, 0x09]),
        &mut output[offset..],
    ));

    Ok((output, offset))
}

/*
This is just here as a note for when retries are added
==================================================================================================
THREAD SPEC v1.3.0 -- section 4.5.1
A Thread Device attempting to attach MUST first attempt to attach with the Scan Mask TLV of
the Parent Request set to only solicit responses from Routers. If no responses are received, or
there is no response with the highest link quality, this request is deemed to have failed. If it
failed, the Device MUST attempt to attach again using the same request. If it still failed, the device
MUST attempt to attach with the Scan Mask TLV of the Parent Request set to solicit responses from both
Routers and REEDs. There is no requirement on minimum link quality for
responses to this request. If no responses are received, this request is deemed to have failed. If
this Parent Request failed it MUST be retried up to three times.

If the Thread Device is a REED and it still fails to successfully attach to a parent after all retries,
it forms a new Partition as described in Section 5.16, Thread Network Partitions in Chapter 5,
Network Layer.

If the Thread Device is not a REED and it fails to successfully attach to a parent after all retries,
then it SHOULD first wait for a vendor-specific timeout and then attempt to attach again using a
Parent Request set to only solicit responses from Routers. If no responses are received, or if
there is no response with the highest link quality, this request is deemed to have failed. If it
failed, the Device MUST attempt to attach with the Scan Mask TLV of the Parent Request set to
solicit responses from both Routers and REEDs. There is no requirement on minimum link quality
for responses to this request. If this request still failed, the Thread Device again waits for a
vendor-specific timeout and repeats the cycle defined in this paragraph.

SENDING/RETRYING PARENT REQUESTS THREAD SPEC v1.3.0 -- section 4.5.1
**Attempt 1/2 considered to fail if no response received or no response with highest link quality
**Attempt 3-6 considered to fail if no response received
    (Attempt 1) Send parent request with scan mask only set to routers
    (Attempt 2) Repeat attempt 1
    (Attempt 3) Send parent request with scan mask set to routers and REEDs
    (Attempt 4) Repeat attempt 3
    (Attempt 5) Repeat attempt 3
    (Attempt 6) Repeat attempt 3



*/
