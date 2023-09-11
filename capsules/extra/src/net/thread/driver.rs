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

use crate::ieee802154::framer;
use crate::ieee802154::RadioDriver;
use crate::net::ieee802154::{KeyId, MacAddress, SecurityLevel};
use crate::net::ipv6::ip_utils::IPAddr;

const PARENT_REQUEST_SIZE: usize = 21;
use crate::net::thread::thread::find_challenge;
use crate::net::thread::thread::generate_src_ipv6;
use crate::net::thread::thread::ThreadRadioState;
use crate::net::thread::thread::ThreadState;
use crate::net::thread::thread::MULTICAST_IPV6;
use kernel::hil::time;

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
use capsules_core::stream::decode_u32;
use kernel::hil::symmetric_encryption::CCMClient;
use kernel::hil::symmetric_encryption::AES128CCM;
use kernel::utilities::leasable_buffer::SubSliceMut;

use core::cell::Cell;
use core::mem::size_of;

use kernel::capabilities::UdpDriverCapability;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
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
pub struct ThreadNetworkDriver<'a, A: time::Alarm<'a>> {
    /// UDP sender
    sender: &'a dyn UDPSender<'a>,
    crypto: &'a dyn AES128CCM<'a>,
    alarm: &'a A,
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

    crypt_state: MapCell<ThreadRadioState>,

    state: MapCell<ThreadState>,

    driver_send_cap: &'static dyn UdpDriverCapability,

    net_cap: &'static NetworkCapability,

    frame_count: Cell<u32>,

    networkkey: MapCell<[u8; 16]>,
}

impl<'a, A: time::Alarm<'a>> ThreadNetworkDriver<'a, A> {
    pub fn new(
        sender: &'a dyn UDPSender<'a>,
        crypto: &'a dyn AES128CCM<'a>,
        alarm: &'a A,
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
    ) -> ThreadNetworkDriver<'a, A> {
        ThreadNetworkDriver {
            sender: sender,
            crypto: crypto,
            alarm: alarm,
            apps: grant,
            current_app: Cell::new(None),
            src_mac_addr: src_mac_addr,
            interface_list: interface_list,
            max_tx_pyld_len: max_tx_pyld_len,
            port_table: port_table,
            send_buffer: MapCell::new(send_buffer),
            recv_buffer: MapCell::new(recv_buffer),
            crypt_state: MapCell::new(ThreadRadioState::CryptReady),
            state: MapCell::new(ThreadState::Detached),
            driver_send_cap: driver_send_cap,
            net_cap: net_cap,
            frame_count: Cell::new(5),
            networkkey: MapCell::new([0; 16]),
        }
    }

    pub fn set_networkkey(&self, key: [u8; 16]) {
        self.networkkey.replace(key);
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
        // Parent Request has now begun, save previous state and then
        // update state machine accordingly
        let original_state = self.state.take().unwrap();
        self.state.replace(ThreadState::SendParentReq);

        kernel::debug!("Sending parent request...");

        // Before beginning process to construct Parent Request, confirm that
        // the cryptography resources allocated to thread are idle/ready
        let crypt_state_check =
            self.crypt_state
                .take()
                .map_or(Err(ErrorCode::BUSY), |crypt_state| {
                    if let ThreadRadioState::CryptReady = crypt_state {
                        self.crypt_state.replace(ThreadRadioState::CryptSend(
                            MULTICAST_IPV6,
                            MacAddress::Short(0xFFFF),
                            35,
                        ));
                        Ok(())
                    } else {
                        Err(ErrorCode::BUSY)
                    }
                });

        // Handle error of busy thread cryptographic resources; fail parent request
        if let Err(ErrorCode::BUSY) = crypt_state_check {
            self.state.replace(ThreadState::Detached);
            kernel::debug!("Sending parent req failed; crypt is busy");
            return crypt_state_check;
        }

        let prepared_buf: &mut [u8; PARENT_REQUEST_SIZE] = &mut [0; PARENT_REQUEST_SIZE];

        // ENCODE PAYLOAD //
        encode_u8(&mut prepared_buf[0..1], 9); //Command Parent Request
        let mode_tlv: [u8; 3] = [1, 1, 0x0f];
        let challenge_tlv: [u8; 10] = [3, 8, 0, 0, 0, 0, 0, 0, 0, 0];
        let scan_mask_tlv: [u8; 3] = [0x0e, 0x01, 0x80];
        let version_tlv: [u8; 4] = [0x12, 0x02, 0x00, 0x04];
        encode_bytes(&mut prepared_buf[1..4], &mode_tlv);
        encode_bytes(&mut prepared_buf[4..14], &challenge_tlv);
        encode_bytes(&mut prepared_buf[14..17], &scan_mask_tlv);
        encode_bytes(&mut prepared_buf[17..21], &version_tlv);

        self.thread_send(prepared_buf, MULTICAST_IPV6)
    }

    fn thread_send(&self, buf: &[u8], dest_addr: IPAddr) -> Result<(), ErrorCode> {
        let mut aux_buf: [u8; 42] = [0; 42];
        let src_ipv6 = generate_src_ipv6(&self.src_mac_addr);
        // ENCODE SRC/DESTINATION AUTH DATA //
        encode_bytes(&mut aux_buf[..16], &src_ipv6);
        encode_bytes(&mut aux_buf[16..32], &dest_addr.0);

        // ENCODE AUXILARY SUITE //
        encode_u8(&mut aux_buf[32..33], 0x15); // security control field (replace this later..needs to be more robust)
        let mut frame_count_bytes: [u8; 4] = [0; 4];
        encode_u32(&mut frame_count_bytes, self.frame_count.get()); // frame counter
        encode_bytes_be(&mut aux_buf[33..37], &frame_count_bytes);
        let key_ident_field: [u8; 5] = [0, 0, 0, 0, 1];
        encode_bytes(&mut aux_buf[37..42], &key_ident_field);

        let nonce = framer::get_ccm_nonce(
            &self.src_mac_addr,
            self.frame_count.get(),
            SecurityLevel::EncMic32,
        );

        let networkkey = self.networkkey.get();

        match networkkey {
            Some(key) => {
                if let Err(code) = self.crypto.set_key(&key) {
                    return Err(code);
                } else if let Err(code) = self.crypto.set_nonce(&nonce) {
                    return Err(code);
                }
            }
            None => {
                kernel::debug!("Attempt to access networkkey when no networkkey set.");
                self.state.replace(ThreadState::Detached); // This is not necissarily correct
                return Err(ErrorCode::NOSUPPORT);
            }
        }

        self.frame_count.set(self.frame_count.get() + 1);

        self.send_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                send_buffer.reset();

                send_buffer[0..42].copy_from_slice(&aux_buf);
                send_buffer[42..42 + buf.len()].copy_from_slice(buf);

                send_buffer.slice(0..(aux_buf.len() + buf.len() + 4));

                let crypto_res =
                    self.crypto
                        .crypt(send_buffer.take(), 0, 42, buf.len(), 4, true, true);

                if let Err((code, buf)) = crypto_res {
                    self.send_buffer.replace(SubSliceMut::new(buf));
                    Err(code)
                } else {
                    Ok(())
                }
            })
    }

    fn recv_logic(&self, sender_ip: IPAddr) {
        let mut output: [u8; 200] = [0; 200];
        let mut offset: usize = 0;

        let _ = self
            .recv_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut recv_buf| {
                kernel::debug!("RECV LOGIC CALLED {:?}", recv_buf[0]);

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

                    // MLE Frame Counter TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::MleFrameCounter(self.frame_count.get().to_be()),
                        &mut output[offset..],
                    ));

                    // Mode TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::Mode(
                            LinkMode::FullThreadDevice as u8 + LinkMode::ReceiverOnWhenIdle as u8,
                        ),
                        &mut output[offset..],
                    ));

                    // Timeout TLV //
                    offset += tlv::unwrap_tlv_offset(tlv::Tlv::encode(
                        &tlv::Tlv::Timeout((10 as u32).to_be()),
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

                if recv_buf[0] == 12 {
                    kernel::debug!("Received child id response")
                }

                recv_buf.reset();
                self.recv_buffer.replace(recv_buf);
                Ok(())
            });

        let dest_mac_addr = mac_from_ipv6(sender_ip);

        // NEED TO HANDLE THIS POTENTIAL ERROR
        let crypt_state_res = self
            .crypt_state
            .take()
            .map_or(Err(ErrorCode::BUSY), |crypt_state| {
                if let ThreadRadioState::CryptReady = crypt_state {
                    self.crypt_state.replace(ThreadRadioState::CryptSend(
                        sender_ip,
                        MacAddress::Long(dest_mac_addr),
                        offset + 4 + 10,
                    ));
                    Ok(())
                } else {
                    Err(ErrorCode::BUSY)
                }
            });

        self.thread_send(&output[0..offset], sender_ip); // need to add error checks
    }
}

impl<'a, A: time::Alarm<'a>> framer::KeyProcedure for ThreadNetworkDriver<'a, A> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<[u8; 16]> {
        // self.networkkey.get()
        Some([
            0xde, 0x89, 0xc5, 0x3a, 0xf3, 0x82, 0xb4, 0x21, 0xe0, 0xfd, 0xe5, 0xa9, 0xba, 0xe3,
            0xbe, 0xf0,
        ])
    }
}

impl<'a, A: time::Alarm<'a>> framer::DeviceProcedure for ThreadNetworkDriver<'a, A> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<[u8; 8]> {
        Some(self.src_mac_addr.clone())
    }
}

impl<'a, A: time::Alarm<'a>> SyscallDriver for ThreadNetworkDriver<'a, A> {
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
                // self.alarm
                //     .set_alarm(self.alarm.now(), self.alarm.ticks_from_seconds(5));
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

impl<'a, A: time::Alarm<'a>> UDPSendClient for ThreadNetworkDriver<'a, A> {
    fn send_done(&self, result: Result<(), ErrorCode>, mut dgram: SubSliceMut<'static, u8>) {
        // Replace the returned kernel buffer. Now we can send the next msg.
        dgram.reset();
        self.send_buffer.replace(dgram);
        self.crypt_state.replace(ThreadRadioState::CryptReady);
        kernel::debug!("SENDING DONE!!");
    }
}

impl<'a, A: time::Alarm<'a>> time::AlarmClient for ThreadNetworkDriver<'a, A> {
    // handle case here where state is empty?
    fn alarm(&self) {
        match self.state.take().unwrap() {
            ThreadState::Detached => (),
            ThreadState::SendParentReq => panic!("Sending PR timeout"),
            ThreadState::SendChildIdReq => panic!("Sending CR timeout"),
            ThreadState::SEDActive => {
                // we need to ping parent again
                ()
            }
        }
    }
}

impl<'a, A: time::Alarm<'a>> UDPRecvClient for ThreadNetworkDriver<'a, A> {
    fn receive(
        &self,
        src_addr: IPAddr,
        dst_addr: IPAddr,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) {
        kernel::debug!("RECEIVE!!");
        let state_check = self
            .crypt_state
            .take()
            .map_or(Err(ErrorCode::BUSY), |crypt_state| {
                if let ThreadRadioState::CryptReady = crypt_state {
                    self.crypt_state
                        .replace(ThreadRadioState::CryptReceive(src_addr, payload.len()));
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
        let nonce = framer::get_ccm_nonce(&src_device_addr, frame_counter, SecurityLevel::EncMic32);

        let networkkey = self.networkkey.get();
        match networkkey {
            Some(key) => {
                if self.crypto.set_key(&key).is_err() || self.crypto.set_nonce(&nonce).is_err() {
                    kernel::debug!("Error configuring crypt.");
                }
            }
            None => {
                kernel::debug!("Attempt to access networkkey when no networkkey set.");
            }
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

                    kernel::debug!("*****************crypto take");
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

        kernel::debug!("Crypto res value {:?}", crypto_res);
    }
}

impl<'a, A: time::Alarm<'a>> PortQuery for ThreadNetworkDriver<'a, A> {
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

impl<'a, A: time::Alarm<'a>> CCMClient for ThreadNetworkDriver<'a, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        kernel::debug!("CRYPTO DONE");
        const AuthDataLen: usize = 32;

        let res = self
            .crypt_state
            .take()
            .map_or(Err(ErrorCode::BUSY), |crypt_state| match crypt_state {
                ThreadRadioState::CryptSend(dst_ipv6, dst_mac, payload_len) => {
                    buf.copy_within(AuthDataLen..(AuthDataLen + payload_len), 1);

                    buf[0..1].copy_from_slice(&[0]);

                    self.send_buffer.replace(SubSliceMut::new(buf));
                    self.send_buffer
                        .take()
                        .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                            send_buffer.slice(0..(payload_len + 1));

                            kernel::debug!("udp sending...");

                            // Set alarm for sending timeout. The value is not correct
                            // self.alarm
                            //     .set_alarm(self.alarm.now(), self.alarm.ticks_from_seconds(2));

                            self.sender.driver_send_to(
                                dst_ipv6,
                                dst_mac,
                                19788,
                                19788,
                                send_buffer,
                                self.driver_send_cap,
                                self.net_cap,
                            );

                            self.crypt_state.replace(ThreadRadioState::CryptReady);

                            Ok(())
                        })
                }
                ThreadRadioState::CryptReceive(sender_ipv6, payload_len) => {
                    kernel::debug!("cryptreceive enter");
                    let mut new_recv_buffer = SubSliceMut::new(buf);
                    new_recv_buffer.slice(0..payload_len);
                    self.recv_buffer.replace(new_recv_buffer);

                    self.crypt_state.replace(ThreadRadioState::CryptReady);
                    self.recv_logic(sender_ipv6);
                    Ok(())
                }
                _ => panic!("This should not be possible"),
            });
    }
}
