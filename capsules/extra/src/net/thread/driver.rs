// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! UDP userspace interface for transmit and receive.
//!
//! Implements a userspace interface for sending and receiving UDP messages.
//! Processes use this driver to send UDP packets from a common interface
//! and bind to UDP ports for receiving packets.
//! Also exposes a list of interface addresses to the application (currently
//! hard-coded).

use crate::ieee802154::framer::get_ccm_nonce;
use crate::net::ieee802154::MacAddress;
use crate::net::ieee802154::SecurityLevel;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ip_utils::MacAddr;
use crate::net::ipv6::ip_utils::DEFAULT_DST_MAC_ADDR;
use crate::net::thread::thread::find_challenge;
use crate::net::thread::thread::generate_src_ipv6;
use crate::net::thread::thread::ThreadState;
use crate::net::thread::thread::MULTICAST_IPV6;

use crate::net::network_capabilities::NetworkCapability;
use crate::net::stream::encode_bytes;
use crate::net::stream::encode_bytes_be;
use crate::net::stream::encode_u16;
use crate::net::stream::encode_u32;
use crate::net::stream::encode_u8;
use crate::net::stream::SResult;
use crate::net::thread::tlv;
use crate::net::thread::tlv::LinkMode;
use crate::net::udp::udp_port_table::UdpPortBindingRx;
use crate::net::udp::udp_port_table::UdpPortBindingTx;
use crate::net::udp::udp_port_table::{PortQuery, UdpPortManager};
use crate::net::udp::udp_recv::UDPRecvClient;
use crate::net::udp::udp_send::{UDPSendClient, UDPSender};
use crate::net::util::host_slice_to_u16;
use capsules_core::stream::decode_bytes;
use capsules_core::stream::decode_u32;
use kernel::hil::symmetric_encryption::CCMClient;
use kernel::hil::symmetric_encryption::AES128CCM;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::SubSliceMut;

use core::cell::Cell;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::iter::Map;
use core::mem::size_of;
use core::{cmp, mem};

use kernel::capabilities::UdpDriverCapability;
use kernel::debug;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::MapCell;
use kernel::{ErrorCode, ProcessId};

use capsules_core::driver;

use super::thread::mac_from_ipv6;
pub const DRIVER_NUM: usize = driver::NUM::Thread as usize;

const THREAD_PORT_NUMBER: u16 = 19788;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const READ: usize = 0;
    pub const CFG: usize = 1;
    pub const RX_CFG: usize = 2;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 3;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UDPEndpoint {
    addr: IPAddr,
    port: u16,
}

impl UDPEndpoint {
    /// This function serializes the `UDPEndpoint` into the provided buffer.
    ///
    /// # Arguments
    ///
    /// `buf` - A mutable buffer to serialize the `UDPEndpoint` into
    /// `offset` - The current offset into the provided buffer
    ///
    /// # Return Value
    ///
    /// This function returns the new offset into the buffer wrapped in an
    /// SResult.
    pub fn encode(&self, buf: &mut [u8], offset: usize) -> SResult<usize> {
        stream_len_cond!(buf, size_of::<UDPEndpoint>() + offset);

        let mut off = offset;
        for i in 0..16 {
            off = enc_consume!(buf, off; encode_u8, self.addr.0[i]);
        }
        off = enc_consume!(buf, off; encode_u16, self.port);
        stream_done!(off, off);
    }

    /// This function checks if the UDPEndpoint specified is the 0 address + 0 port.
    pub fn is_zero(&self) -> bool {
        self.addr.is_unspecified() && self.port == 0
    }
}

#[derive(Default)]
pub struct App {
    pending_tx: Option<[UDPEndpoint; 2]>,
    bound_port: Option<UDPEndpoint>,
}

#[allow(dead_code)]
pub struct ThreadNetworkDriver<'a> {
    /// UDP sender
    sender: &'a dyn UDPSender<'a>,
    crypto: &'a dyn AES128CCM<'a>,

    /// Grant of apps that use this radio driver.
    apps: Grant<
        App,
        UpcallCount<2>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    /// ID of app whose transmission request is being processed.
    current_app: Cell<Option<ProcessId>>,

    src_mac_addr: [u8; 8],

    /// List of IP Addresses of the interfaces on the device
    interface_list: &'static [IPAddr],

    /// Maximum length payload that an app can transmit via this driver
    max_tx_pyld_len: usize,

    /// UDP bound port table (manages kernel bindings)
    port_table: &'static UdpPortManager,

    send_buffer: MapCell<SubSliceMut<'static, u8>>,

    recv_buffer: MapCell<SubSliceMut<'static, u8>>,

    state: MapCell<ThreadState>,

    driver_send_cap: &'static dyn UdpDriverCapability,

    net_cap: &'static NetworkCapability,

    send: Cell<bool>,

    second_send: Cell<bool>,

    frame_count: u32,

    networkkey: Cell<[u8; 16]>,
}

impl<'a> ThreadNetworkDriver<'a> {
    pub fn new(
        sender: &'a dyn UDPSender<'a>,
        crypto: &'a dyn AES128CCM<'a>,

        grant: Grant<
            App,
            UpcallCount<2>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        src_mac_addr: [u8; 8],
        interface_list: &'static [IPAddr],
        max_tx_pyld_len: usize,
        port_table: &'static UdpPortManager,
        send_buffer: SubSliceMut<'static, u8>,
        recv_buffer: SubSliceMut<'static, u8>,
        driver_send_cap: &'static dyn UdpDriverCapability,
        net_cap: &'static NetworkCapability,
    ) -> ThreadNetworkDriver<'a> {
        ThreadNetworkDriver {
            sender: sender,
            crypto: crypto,
            apps: grant,
            current_app: Cell::new(None),
            src_mac_addr: src_mac_addr,
            interface_list: interface_list,
            max_tx_pyld_len: max_tx_pyld_len,
            port_table: port_table,
            send_buffer: MapCell::new(send_buffer),
            recv_buffer: MapCell::new(recv_buffer),
            state: MapCell::new(ThreadState::CryptReady),
            driver_send_cap: driver_send_cap,
            net_cap: net_cap,
            send: Cell::new(true),
            second_send: Cell::new(false),
            frame_count: 5,
            networkkey: Cell::new([0; 16]),
        }
    }

    fn set_send(&self) {
        self.send.set(false);
    }

    pub fn set_networkkey(&self, key: [u8; 16]) {
        self.networkkey.set(key);
    }

    pub fn get_networkkey(&self) -> [u8; 16] {
        self.networkkey.get()
    }

    pub fn init_thread_binding(&self) -> (UdpPortBindingRx, UdpPortBindingTx) {
        let key: [u8; 16] = [
            0x54, 0x45, 0xf4, 0x15, 0x8f, 0xd7, 0x59, 0x12, 0x17, 0x58, 0x09, 0xf8, 0xb5, 0x7a,
            0x66, 0xa4,
        ];
        self.set_networkkey(key);
        match self.port_table.create_socket() {
            Ok(socket) => match self
                .port_table
                .bind(socket, THREAD_PORT_NUMBER, self.net_cap)
            {
                Ok((udp_tx, udp_rx)) => (udp_rx, udp_tx),
                Err(_) => panic!("failed bind to port"),
            },
            Err(_) => panic!("Error in retrieving socket!"),
        }
    }

    fn send_parent_req(&self) -> Result<(), ErrorCode> {
        let state_check = self.state.take().map_or(Err(ErrorCode::BUSY), |state| {
            if let ThreadState::CryptReady = state {
                self.state.replace(ThreadState::CryptSend(
                    IPAddr(MULTICAST_IPV6),
                    MacAddress::Short(0xFFFF),
                    35,
                ));
                Ok(())
            } else {
                Err(ErrorCode::BUSY)
            }
        });

        if let Err(ErrorCode::BUSY) = state_check {
            kernel::debug!("Sending parent req failed; crypt is busy");
            return state_check;
        }

        let src_ipv6 = generate_src_ipv6(&self.src_mac_addr);
        let prepared_buf: &mut [u8; 67] = &mut [0; 67];

        // ENCODE SRC/DESTINATION AUTH DATA //
        encode_bytes(&mut prepared_buf[..16], &src_ipv6);
        encode_bytes(&mut prepared_buf[16..32], &MULTICAST_IPV6);

        // ENCODE AUXILARY SUITE //
        encode_u8(&mut prepared_buf[32..33], 0x15); // security control field (replace this later..needs to be more robust)
        let mut frame_count_bytes: [u8; 4] = [0; 4];
        encode_u32(&mut frame_count_bytes, self.frame_count); // frame counter
        encode_bytes_be(&mut prepared_buf[33..37], &frame_count_bytes);
        let key_ident_field: [u8; 5] = [0, 0, 0, 0, 1];
        encode_bytes(&mut prepared_buf[37..42], &key_ident_field);

        // ENCODE PAYLOAD //
        encode_u8(&mut prepared_buf[42..43], 9); //Command Parent Request
        let mode_tlv: [u8; 3] = [1, 1, 0x0f];
        let challenge_tlv: [u8; 10] = [3, 8, 0, 0, 0, 0, 0, 0, 0, 0];
        let scan_mask_tlv: [u8; 3] = [0x0e, 0x01, 0x80];
        let version_tlv: [u8; 4] = [0x12, 0x02, 0x00, 0x04];
        encode_bytes(&mut prepared_buf[43..46], &mode_tlv);
        encode_bytes(&mut prepared_buf[46..56], &challenge_tlv);
        encode_bytes(&mut prepared_buf[56..59], &scan_mask_tlv);
        encode_bytes(&mut prepared_buf[59..63], &version_tlv);

        let nonce = get_ccm_nonce(
            &self.src_mac_addr,
            self.frame_count,
            SecurityLevel::EncMic32,
        );

        self.crypto.set_key(&self.get_networkkey());
        self.crypto.set_nonce(&nonce);
        self.send_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                send_buffer[..prepared_buf.len()].copy_from_slice(prepared_buf);
                send_buffer.slice(0..(prepared_buf.len()));

                // add error check here
                self.crypto
                    .crypt(send_buffer.take(), 0, 42, 21, 4, true, true);

                Ok(())
            })
    }

    fn recv_logic(&self, sender_ip: IPAddr) {
        let mut output: [u8; 200] = [0; 200];
        let mut offset: usize = 0;

        let _ = self
            .recv_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |recv_buf| {
                // Command for Parent Response
                if recv_buf[0] == 10 {
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

                    kernel::debug!("parent response received");

                    // Command Child ID Request //
                    output[0..1].copy_from_slice(&[11]);
                    offset += 1;

                    // Response TLV //
                    let received_challenge_tlv: Result<&[u8], ErrorCode> =
                        find_challenge(&recv_buf[1..]);

                    if received_challenge_tlv.is_err() {
                        // Challenge TLV not found; malformed request
                        return Err(ErrorCode::FAIL);
                    } else {
                        // Encode response into output
                        let mut rsp_buf: [u8; 8] = [0; 8];
                        rsp_buf.copy_from_slice(received_challenge_tlv.unwrap());
                        rsp_buf.reverse(); // NEED TO DISCUSS BIG/LITTLE ENDIAN ASSUMPTIONS
                        kernel::debug!("RECEIVED CHALLENGE {:?}", rsp_buf);
                        offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                            &tlv::Tlv::Response(rsp_buf),
                            &mut output[offset..],
                        ));
                    }

                    // Link-layer Frame Counter TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::LinkLayerFrameCounter(0),
                        &mut output[offset..],
                    ));

                    // Mode TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::Mode(
                            LinkMode::SecureDataRequests as u8
                                + LinkMode::FullNetworkDataRequired as u8
                                + LinkMode::FullThreadDevice as u8
                                + LinkMode::ReceiverOnWhenIdle as u8,
                        ),
                        &mut output[offset..],
                    ));

                    // Timeout TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::Timeout((0xf0 as u32).to_be()),
                        &mut output[offset..],
                    ));

                    // Version TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::Version(4),
                        &mut output[offset..],
                    ));

                    output[offset..offset + 4].copy_from_slice(&mut [0x1b, 0x02, 0x00, 0x81]);
                    offset += 4;

                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::TlvRequest(&[0x0a, 0x0c, 0x09]),
                        &mut output[offset..],
                    ));
                }
                Ok(())
            });

        let dest_mac_addr = mac_from_ipv6(sender_ip);

        let state_check = self.state.take().map_or(Err(ErrorCode::BUSY), |state| {
            if let ThreadState::CryptReady = state {
                self.state.replace(ThreadState::CryptSend(
                    sender_ip,
                    MacAddress::Long(dest_mac_addr),
                    offset + 4 + 10,
                ));
                Ok(())
            } else {
                Err(ErrorCode::BUSY)
            }
        });

        let src_ipv6 = generate_src_ipv6(&self.src_mac_addr);

        output.copy_within(0..offset, 42);

        // AUTH DATA //
        // src ipv6
        encode_bytes(&mut output[0..16], &src_ipv6);

        // destination ipv6
        encode_bytes(&mut output[16..32], &sender_ip.0);

        // ENCODE AUXILARY SUITE //
        encode_u8(&mut output[32..33], 0x15); // security control field (replace this later..needs to be more robust)
        let mut frame_count_bytes: [u8; 4] = [0; 4];
        encode_u32(&mut frame_count_bytes, self.frame_count); // frame counter
        encode_bytes_be(&mut output[33..37], &frame_count_bytes);
        let key_ident_field: [u8; 5] = [0, 0, 0, 0, 1];
        encode_bytes(&mut output[37..42], &key_ident_field);

        let nonce = get_ccm_nonce(
            &self.src_mac_addr,
            self.frame_count,
            SecurityLevel::EncMic32,
        );
        self.crypto.set_key(&self.get_networkkey());
        self.crypto.set_nonce(&nonce);

        let _ = self
            .send_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                send_buffer[0..42 + offset].copy_from_slice(&mut output[0..42 + offset]);
                send_buffer.slice(0..42 + offset + 4);
                // add error check here
                self.crypto
                    .crypt(send_buffer.take(), 0, 42, offset, 4, true, true);

                Ok(())
            });
    }
}

impl<'a> SyscallDriver for ThreadNetworkDriver<'a> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            // Init Thread Network for Device
            1 => {
                self.send_parent_req();

                CommandReturn::success_u32(self.max_tx_pyld_len as u32)
            }

            // Transmits UDP packet stored in tx_buf
            2 => CommandReturn::success_u32(self.max_tx_pyld_len as u32),
            3 => CommandReturn::success_u32(self.max_tx_pyld_len as u32),
            4 => CommandReturn::success_u32(self.max_tx_pyld_len as u32),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a> UDPSendClient for ThreadNetworkDriver<'a> {
    fn send_done(&self, result: Result<(), ErrorCode>, mut dgram: SubSliceMut<'static, u8>) {
        // Replace the returned kernel buffer. Now we can send the next msg.
        dgram.reset();
        self.send_buffer.replace(dgram);
    }
}

impl<'a> UDPRecvClient for ThreadNetworkDriver<'a> {
    fn receive(
        &self,
        src_addr: IPAddr,
        dst_addr: IPAddr,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) {
        let state_check = self.state.take().map_or(Err(ErrorCode::BUSY), |state| {
            if let ThreadState::CryptReady = state {
                self.state
                    .replace(ThreadState::CryptReceive(src_addr, payload.len()));
                Ok(())
            } else {
                Err(ErrorCode::BUSY)
            }
        });

        if let Err(ErrorCode::BUSY) = state_check {
            kernel::debug!("Received failed; crypt is busy");
        }

        // Obtain frame counter from the UDP packet
        let frame_counter = decode_u32(&payload[2..6]).done().unwrap().1.to_be();

        let payload_slice = &payload[11..(payload.len() - 4)];
        // relevant values for encryption
        let a_off = 0;
        let m_off = 0;
        let m_len = payload_slice.len();
        let mic_len = 0;
        let confidential = true;
        let encrypting = true;
        let level = 5; // hardcoded for now

        let src_device_addr: [u8; 8] = mac_from_ipv6(src_addr);

        let key = [
            0x54, 0x45, 0xf4, 0x15, 0x8f, 0xd7, 0x59, 0x12, 0x17, 0x58, 0x09, 0xf8, 0xb5, 0x7a,
            0x66, 0xa4,
        ];

        // generate nonce
        let nonce = get_ccm_nonce(&src_device_addr, frame_counter, SecurityLevel::EncMic32);

        // set nonce/key for encryption
        if self.crypto.set_key(&self.get_networkkey()) != Ok(())
            || self.crypto.set_nonce(&nonce) != Ok(())
        {
            kernel::debug!("FAIL KEY SET AND NONCE");
        }

        let crypto_res =
            self.recv_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |mut recv_buffer| {
                    if payload_slice.len() > recv_buffer.len() {
                        kernel::debug!("no space!");
                        self.recv_buffer.replace(recv_buffer);
                        return Err(ErrorCode::SIZE);
                    }

                    recv_buffer[..payload_slice.len()].copy_from_slice(payload_slice);

                    let cryp_out = self.crypto.crypt(
                        recv_buffer.take(),
                        a_off,
                        m_off,
                        m_len,
                        mic_len,
                        confidential,
                        encrypting,
                    );

                    if !cryp_out.is_ok() {
                        kernel::debug!("error with crypto")
                    }

                    Ok(())
                });
    }
}

impl<'a> PortQuery for ThreadNetworkDriver<'a> {
    // Returns true if |port| is bound (on any iface), false otherwise.
    fn is_bound(&self, port: u16) -> bool {
        let mut port_bound = false;
        for app in self.apps.iter() {
            app.enter(|other_app, _| {
                if other_app.bound_port.is_some() {
                    let other_addr_opt = other_app.bound_port.clone();
                    let other_addr = other_addr_opt.unwrap(); // Unwrap fail = Missing other_addr
                    if other_addr.port == port {
                        port_bound = true;
                    }
                }
            });
        }
        port_bound
    }
}

impl<'a> CCMClient for ThreadNetworkDriver<'a> {
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        const AuthDataLen: usize = 32;

        let res = self
            .state
            .take()
            .map_or(Err(ErrorCode::BUSY), |state| match state {
                ThreadState::CryptSend(dst_ipv6, dst_mac, payload_len) => {
                    kernel::debug!("Entered Crypt Send");
                    buf.copy_within(AuthDataLen..(AuthDataLen + payload_len), 1);

                    buf[0..1].copy_from_slice(&[0]);

                    self.send_buffer.replace(SubSliceMut::new(buf));
                    self.send_buffer
                        .take()
                        .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                            send_buffer.slice(0..(payload_len + 1));

                            kernel::debug!("udp sending...");
                            self.sender.driver_send_to(
                                dst_ipv6,
                                dst_mac,
                                19788,
                                19788,
                                send_buffer,
                                self.driver_send_cap,
                                self.net_cap,
                            );

                            self.state.replace(ThreadState::CryptReady);
                            Ok(())
                        })
                }
                ThreadState::CryptReceive(sender_ipv6, payload_len) => {
                    let mut new_recv_buffer = SubSliceMut::new(buf);
                    new_recv_buffer.slice(0..payload_len);
                    self.recv_buffer.replace(new_recv_buffer);

                    self.state.replace(ThreadState::CryptReady);
                    self.recv_logic(sender_ipv6);
                    Ok(())
                }
                _ => panic!("This should not be possible"),
            });
        /*
        if self.second_send.get() {
            kernel::debug!("ENTERING CHILD REQUEST");
            buf.copy_within(32..91, 1);

            let zero_slic: &[u8; 1] = &[0];
            buf[0..1].copy_from_slice(zero_slic);

            self.send_buffer.replace(SubSliceMut::new(buf));
            self.send_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                    send_buffer.slice(0..60);

                    kernel::debug!("SENDING CHILD REQUEST");

                    self.sender.driver_send_to(
                        IPAddr([
                            0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0xb5, 0xa6, 0x91,
                            0xee, 0x42, 0x56, 0x35,
                        ]),
                        MacAddress::Long([0xa2, 0xb5, 0xa6, 0x91, 0xee, 0x42, 0x56, 0x35]),
                        19788,
                        19788,
                        send_buffer,
                        self.driver_send_cap,
                        self.net_cap,
                    );

                    self.send.set(false);
                    Ok(())
                });
        } else {
            kernel::debug!("THS IS THE RECV BUF {:?}", buf);
            let link_layer_frame_ct: [u8; 6] = [5, 4, 0, 0, 0, 0];
            let mle_frame_ct: [u8; 6] = [8, 4, 0, 0, 0, 0x09];
            let mode: [u8; 3] = [1, 1, 0x0f];
            let timeout: [u8; 6] = [2, 4, 0, 0, 0, 0xf0];
            let version: [u8; 4] = [0x12, 2, 0, 4];
            let a: [u8; 4] = [0x1b, 2, 0, 0x81];
            let elev_slice: [u8; 3] = [11, 4, 8];
            let tlv_request: [u8; 5] = [0x0d, 0x03, 0x0a, 0x0c, 0x09];
            let start_auth: [u8; 32] = [
                0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0xb5, 0xa6, 0x91, 0xee, 0x42,
                0x56, 0x36, 0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0xb5, 0xa6, 0x91,
                0xee, 0x42, 0x56, 0x35,
            ];

            let aux_sec_header: [u8; 11] = [
                0x00, 0x15, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            ];

            self.send_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                    kernel::debug!("PREPARING TO SEND");
                    send_buffer.reset();
                    send_buffer[0..32].copy_from_slice(&start_auth);
                    send_buffer[32..42].copy_from_slice(&aux_sec_header[1..11]);
                    send_buffer[42..45].copy_from_slice(&elev_slice);
                    send_buffer[45..53].copy_from_slice(&buf[39..47]);
                    send_buffer[53..59].copy_from_slice(&link_layer_frame_ct);
                    send_buffer[59..65].copy_from_slice(&mle_frame_ct);
                    send_buffer[65..68].copy_from_slice(&mode);
                    send_buffer[68..74].copy_from_slice(&timeout);
                    send_buffer[74..78].copy_from_slice(&version);
                    send_buffer[78..82].copy_from_slice(&a);
                    send_buffer[82..87].copy_from_slice(&tlv_request);

                    send_buffer.slice(0..91);

                    // Hardcoded for now, this should probably be moved elsewhere (already stored in kernel)
                    let device_addr: [u8; 8] = [0xa2, 0xb5, 0xa6, 0x91, 0xee, 0x42, 0x56, 0x36];

                    // let key = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
                    let key = [
                        0x54, 0x45, 0xf4, 0x15, 0x8f, 0xd7, 0x59, 0x12, 0x17, 0x58, 0x09, 0xf8,
                        0xb5, 0x7a, 0x66, 0xa4,
                    ];

                    let nonce = get_ccm_nonce(&device_addr, 9, SecurityLevel::EncMic32);

                    self.crypto.set_nonce(&nonce);
                    self.crypto.set_key(&key);
                    kernel::debug!("CRYPTO IS IN PROGRESS!");

                    self.crypto
                        .crypt(send_buffer.take(), 0, 42, 45, 4, true, true);
                    self.second_send.set(true);
                    Ok(())
                });
        } */
    }
}
