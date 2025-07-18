// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Structs and methods associated with the Thread networking layer.
//!
//! This represents a first attempt in Tock to support Thread
//! networking. The current implementation successfully joins a Tock
//! device as a child node to a Thread parent (tested using
//! OpenThread). This Thread capsule is a client to the UDP Mux.  The
//! associated ThreadNetwork struct must be created in the
//! `thread_network.rs` component.
//!
//! The Userland interface is incredibly simple at this juncture. An application
//! can begin the Thread child/parent joining by issuing a syscall command
//! with the MLE/MAC key as an argument. Only one userspace application can use/join
//! the Thread network. Once a userspace application has joined the Thread network,
//! the Thread network is considered locked. After the Thread network
//! is "locked", other userspace applications attempting to join the network
//! will return a failure. This is temporary and will eventually be replaced.

// ------------------------------------------------------------------------------
// Current Limitations
// ------------------------------------------------------------------------------
// (1) A majority of the TLV fields used in the parent request/child id request
//     are hardcoded. Future implementations need to provide options for specifying
//     varied security policies.
// (2) Current implementation joins the Thread network sucessfully and consistently
//     but does not send update/heart beat messages to the parent prior to the child
//     timing out.
// (3) Currently no support for sending UDP messages across Thread interface. The
//     current interface is unusable for sending data. It can only be used to
//     join a network.

use crate::ieee802154::framer::{self, get_ccm_nonce};
use crate::net::ieee802154::{KeyId, MacAddress, Security, SecurityLevel};
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::network_capabilities::NetworkCapability;

use crate::net::ieee802154;
use crate::net::thread::thread_utils::generate_src_ipv6;
use crate::net::thread::thread_utils::ThreadState;
use crate::net::thread::thread_utils::MULTICAST_IPV6;
use crate::net::thread::thread_utils::THREAD_PORT_NUMBER;
use crate::net::thread::thread_utils::{
    encode_cryp_data, form_child_id_req, form_parent_req, mac_from_ipv6, MleCommand, NetworkKey,
    AUTH_DATA_LEN, AUX_SEC_HEADER_LENGTH, IPV6_LEN, SECURITY_SUITE_LEN,
};
use crate::net::udp::udp_port_table::UdpPortManager;
use crate::net::udp::udp_recv::UDPRecvClient;
use crate::net::udp::udp_send::{UDPSendClient, UDPSender};
use capsules_core::driver;

use core::cell::Cell;

use kernel::capabilities::UdpDriverCapability;
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::symmetric_encryption::CCMClient;
use kernel::hil::symmetric_encryption::AES128CCM;
use kernel::hil::time;
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::MapCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

const SECURITY_SUITE_ENCRYP: u8 = 0;
pub const DRIVER_NUM: usize = driver::NUM::Thread as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

// /// Ids for read-write allow buffers
// mod rw_allow {
//     pub const READ: usize = 0;
//     pub const CFG: usize = 1;
//     pub const RX_CFG: usize = 2;
//     /// The number of allow buffers the kernel stores for this grant
//     pub const COUNT: u8 = 3;
// }

/// IDs for subscribed upcalls.
mod upcall {
    pub const JOINCOMPLETE: usize = 0;
}

#[derive(Default)]
pub struct App {}

#[allow(dead_code)]
pub struct ThreadNetworkDriver<'a, A: time::Alarm<'a>> {
    /// UDP sender
    sender: &'a dyn UDPSender<'a>,

    /// AES crypto engine for MLE encryption
    aes_crypto: &'a dyn AES128CCM<'a>,

    /// Alarm for timeouts
    alarm: &'a A,

    /// Grant of apps that use this thread driver.
    apps: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,

    /// mac address of device
    src_mac_addr: [u8; 8],

    /// Maximum length payload that an app can transmit via this driver
    max_tx_pyld_len: usize,

    /// UDP bound port table (manages kernel bindings)
    port_table: &'static UdpPortManager,

    /// kernel buffer used for sending
    send_buffer: MapCell<SubSliceMut<'static, u8>>,

    /// kernel buffer used for receiving
    recv_buffer: MapCell<SubSliceMut<'static, u8>>,

    /// state machine for the Thread device
    state: MapCell<ThreadState>,

    /// UDP driver capability
    driver_send_cap: &'static dyn UdpDriverCapability,

    /// Network capability
    net_cap: &'static NetworkCapability,

    /// Frame counter for Thread MLE
    frame_count: Cell<u32>,

    /// Stored Thread network containing mac/MLE key
    networkkey: MapCell<NetworkKey>,

    /// Length of the message passed to the crypto engine
    crypto_sizelock: MapCell<usize>,
}

// Note: For now, we initialize the Thread state as empty.
// We replace the Thread state when the first userspace
// application calls the Thread capsule to initiate a Thread network.
// This serves to "lock" the Thread capsule to only one application.
// For now, Tock only supports one application using the Thread network.
// After the network is "locked" to one application, other userspace
// applications requesting to join a Thread network will fail.
impl<'a, A: time::Alarm<'a>> ThreadNetworkDriver<'a, A> {
    pub fn new(
        sender: &'a dyn UDPSender<'a>,
        aes_crypto: &'a dyn AES128CCM<'a>,
        alarm: &'a A,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,
        src_mac_addr: [u8; 8],
        max_tx_pyld_len: usize,
        port_table: &'static UdpPortManager,
        send_buffer: SubSliceMut<'static, u8>,
        recv_buffer: SubSliceMut<'static, u8>,
        driver_send_cap: &'static dyn UdpDriverCapability,
        net_cap: &'static NetworkCapability,
    ) -> ThreadNetworkDriver<'a, A> {
        ThreadNetworkDriver {
            sender,
            aes_crypto,
            alarm,
            apps: grant,
            src_mac_addr,
            max_tx_pyld_len,
            port_table,
            send_buffer: MapCell::new(send_buffer),
            recv_buffer: MapCell::new(recv_buffer),
            state: MapCell::empty(),
            driver_send_cap,
            net_cap,
            frame_count: Cell::new(5),
            networkkey: MapCell::empty(),
            crypto_sizelock: MapCell::empty(),
        }
    }

    /// Takes the MLE and MAC keys and replaces the networkkey
    pub fn set_networkkey(&self, mle_key: [u8; 16], mac_key: [u8; 16]) {
        self.networkkey.replace(NetworkKey { mle_key, mac_key });
    }

    fn send_parent_req(&self) {
        // UNCOMMENT TO DEBUG THREAD //
        // kernel::debug!("[Thread] Sending parent request...");

        // Panicking on unwrap indicates the state was taken without replacement
        // (unreachable with proper state machine implementation)
        let curr_state = self.state.take().unwrap();

        match curr_state {
            ThreadState::Detached => {
                // A parent request can only begin from a detached state. We utilize
                // helper functions to form the request and send the parent request
                // to the multicast IP/Mac Address
                self.state.replace(ThreadState::SendParentReq);
                let parent_req_mle = form_parent_req();
                let src_ipv6 = generate_src_ipv6(&self.src_mac_addr);
                self.thread_mle_send(&parent_req_mle, MULTICAST_IPV6, src_ipv6)
                    .err()
                    .map(|code| {
                        // Thread send failed sending parent req so we terminate and return
                        // to a detached state.
                        self.state.replace(ThreadState::Detached);

                        // UNCOMMENT TO DEBUG THREAD //
                        // kernel::debug!(
                        //     "[Thread] Failed sending MLE parent request - crypto operation error."
                        // );
                        self.terminate_child_join(Err(code));
                    });
            }
            ThreadState::SEDActive(_, _)
            | ThreadState::SendUpdate(_, _)
            | ThreadState::SendUDPMsg => {
                // These states constitute a device that has previously sucessfully
                // joined the network. There is no need to issue a new parent request.
                // Replace state, and terminate.
                self.state.replace(curr_state);
                self.terminate_child_join(Err(ErrorCode::ALREADY));
            }
            _ => {
                // All other Thread states indicate that the thread device has already
                // begun the process of connecting to a parent device. Terminate the parent request.
                // Replace state, and terminate.
                self.state.replace(curr_state);
                self.terminate_child_join(Err(ErrorCode::BUSY));
            }
        }
    }

    fn thread_mle_send(
        &self,
        mle_buf: &[u8],
        dest_addr: IPAddr,
        src_addr: IPAddr,
    ) -> Result<(), ErrorCode> {
        // TODO: Hardcoded encryption suite and auxiliary security; add support to send encrypted/unencrypted MLE

        // We hardcode the auxiliary security for now
        let security = Security {
            level: SecurityLevel::EncMic32,
            asn_in_nonce: false,
            frame_counter: Some(self.frame_count.get()),
            key_id: KeyId::Source4Index([0, 0, 0, 0], 1),
        };

        // Begin cryptographic and sending procedure for the MLE message
        self.send_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |send_buffer| {
                self.perform_crypt_op(src_addr, dest_addr, security, mle_buf, send_buffer.take())
                    .map_err(|(code, buf)| {
                        // Error occured with cryptographic operation, replace buffer
                        // for future transmissions and return error code
                        self.send_buffer.replace(SubSliceMut::new(buf));
                        code
                    })
            })
    }

    fn recv_logic(&self, sender_ip: IPAddr) -> Result<(), ErrorCode> {
        // This function is called once the received MLE payload has been placed
        // into the recv_buffer. The function handles the message and responds accordingly

        self.recv_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut recv_buf| {
                if recv_buf[0] == MleCommand::ParentResponse as u8 {
                    // Received Parent Response -> form Child ID Request

                    // UNCOMMENT TO DEBUG THREAD //
                    // kernel::debug!("[Thread] Received Parent Response.");
                    // kernel::debug!("[Thread] Sending Child ID Request...");

                    let src_ipv6 = generate_src_ipv6(&self.src_mac_addr);

                    let (output, offset) =
                        form_child_id_req(recv_buf.as_slice(), self.frame_count.get())?;

                    // Advance state machine
                    self.state.replace(ThreadState::SendChildIdReq(sender_ip));

                    self.thread_mle_send(&output[..offset], sender_ip, src_ipv6)?;
                } else if recv_buf[0] == MleCommand::ChildIdResponse as u8 {
                    // Receive child id response -> advance state machine
                    self.state.replace(ThreadState::SEDActive(
                        sender_ip,
                        MacAddress::Long(mac_from_ipv6(sender_ip)),
                    ));

                    // TODO: once heart beats are implemented, we will set
                    // the timer here (as seen below)
                    // let curr_time = self.alarm.now();
                    // self.alarm.set_alarm(
                    //     curr_time,
                    //     time::ConvertTicks::ticks_from_seconds(self.alarm, 5),
                    // );
                }

                recv_buf.reset();
                self.recv_buffer.replace(recv_buf);
                Ok(())
            })
    }

    fn terminate_child_join(&self, res: Result<(), ErrorCode>) {
        // Function to schedule upcall to userland on parent request termination. Notifies
        // userland of the reason for termination with the first argument.

        self.apps.each(|_, _, kernel_data| {
            let _ = kernel_data.schedule_upcall(upcall::JOINCOMPLETE, (into_statuscode(res), 0, 0));
        });
    }

    fn perform_crypt_op(
        &self,
        src_addr: IPAddr,
        dst_addr: IPAddr,
        security: Security,
        payload: &[u8],
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Wrapper function for performing the AES-128CCM encryption. This function generates the nonce,
        // sets the nonce/key for the crypto engine, generates the authenticated data, and initiates
        // the crypto operation.

        // Note: The payload argument does not include aux sec header

        // Obtain and unwrap frame counter
        let frame_counter = security.frame_counter;
        if frame_counter.is_none() {
            // UNCOMMENT TO DEBUG THREAD //
            // kernel::debug!("[Thread] Malformed auxiliary security header");
            return Err((ErrorCode::INVAL, buf));
        }

        // Generate nonce, obtain network key and set crypto engine accordingly
        let nonce = get_ccm_nonce(
            &mac_from_ipv6(src_addr),
            frame_counter.unwrap(),
            security.level,
        );
        let mle_key = self.networkkey.get();
        let mic_len = security.level.mic_len();
        match mle_key {
            Some(netkey) => {
                if self.aes_crypto.set_key(&netkey.mle_key).is_err()
                    || self.aes_crypto.set_nonce(&nonce).is_err()
                {
                    // UNCOMMENT TO DEBUG THREAD //
                    // kernel::debug!("[Thread] Failure setting networkkey and/or nonce.");
                    return Err((ErrorCode::FAIL, buf));
                }
            }
            None => {
                // UNCOMMENT TO DEBUG THREAD //
                // kernel::debug!("[Thread] Attempt to access networkkey when no networkkey set.");
                return Err((ErrorCode::NOSUPPORT, buf));
            }
        }

        // Thread MLE security utilizes the AES128 CCM security used by 802.15.4 link layer security.
        // Notably, there are a few minor modifications. AES128 requires authentication data (a data)
        // and the secured message data (m data). Together, the a data is used to encrypt the m data
        // while also generating a message integrity code (MIC). Thread subtly changes the
        // a data from the 802.15.4 specification. For Thread MLE, the a data consists of a concatenation
        // of the IP source address || IP destination address || auxiliary security header. It is especially important
        // to note that the security control field is not included in the auxiliary security header. Likewise,
        // the first byte of the payload is not included in the a data. For further information, refer to
        // (Thread Spec v1.3.0 -- sect. 4.9)
        //
        // Because all MLE messages must be encrypted with MLE security (v1.3.0 sect 4.10), all MLE messages
        // that are processed must possess an auxiliary security header of 10 bytes. The payload therefore consists of:
        //
        // |-----(1 byte)-----|----(10 bytes)------|---(UNKNOWN)---|--(DEPENDENT ON PROTOCOL)--|
        // |  SECURITY SUITE  |   AUX SEC HEADER   |      MLE      |            Mic            |
        //
        // Since the aux sec header is fixed, the auth data is always 32 bytes [16 (IPV6) + 16 (IPV6) + 10 aux sec header].
        // The m data len can be determined because it is the only unknown length.

        let aux_sec_header = &mut [0u8; AUX_SEC_HEADER_LENGTH];
        Security::encode(&security, aux_sec_header);

        let m_data_len = payload.len();

        // Encode auth data and payload into `buf`
        let encode_res = encode_cryp_data(src_addr, dst_addr, aux_sec_header, payload, buf).done();

        // Error check on result from encoding, failure likely means buf was not large enough
        if encode_res.is_none() {
            // UNCOMMENT TO DEBUG THREAD //
            // kernel::debug!("[Thread] Error encoding cryptographic data into buffer");
            return Err((ErrorCode::FAIL, buf));
        }

        let (offset, ()) = encode_res.unwrap();

        // GENERAL NOTE: `self.crypto_sizelock`
        // This does not seem to be the most elegant solution. The `crypto_sizelock` arose from the fact
        // that we must know the length of the payload when the `crypt_done` callback
        // occurs in order to only send/receive the portion of the 200 byte buffer that is the message.
        // The other option to `crypto_sizelock` is to only pass to the crypto engine a buffer
        // that is the size of the transmission/reception. This however is flawed
        // as it leads to an inability to replace/restore the 200 byte buffer. The crypto engine
        // requires a reference to static memory (leading to the recv/send buf being taken).
        // This is only able to be replaced when the `crypt_done` callback occurs and returns the
        // buf used by the crypto engine. If a partial buffer is used, it is impossible to then replace
        // the full sized buffer to the send/recv buf. Likewise, I choose to use the `crypt_sizelock`
        // and pass the whole 200 byte buffer. The `crypto_sizelock` works for
        // now until a more elegant solution is implemented.

        // The sizelock is empty except when a crypto operation
        // is underway. If the sizelock is not empty, return error
        if self.crypto_sizelock.is_some() {
            // UNCOMMENT TO DEBUG THREAD //
            // kernel::debug!(
            //     "[Thread] Error - cryptographic resources in use; crypto_sizelock occupied"
            // );
            return Err((ErrorCode::BUSY, buf));
        }

        // Store the length of the payload.
        self.crypto_sizelock.replace(offset + mic_len);
        self.aes_crypto
            .crypt(buf, 0, AUTH_DATA_LEN, m_data_len, mic_len, true, true)
    }
}

impl<'a, A: time::Alarm<'a>> framer::KeyProcedure for ThreadNetworkDriver<'a, A> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    // TODO: This implementation only supports one key
    fn lookup_key(&self, _level: SecurityLevel, _key_id: KeyId) -> Option<[u8; 16]> {
        if let Some(netkey) = self.networkkey.get() {
            Some(netkey.mac_key)
        } else {
            None
        }
    }
}

impl<'a, A: time::Alarm<'a>> framer::DeviceProcedure for ThreadNetworkDriver<'a, A> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    // TODO: This implementation only supports one key
    fn lookup_addr_long(&self, _addr: MacAddress) -> Option<[u8; 8]> {
        Some(self.src_mac_addr)
    }
}

impl<'a, A: time::Alarm<'a>> SyscallDriver for ThreadNetworkDriver<'a, A> {
    /// ### `command_num`
    /// - `0`: Driver Check
    /// - `1`: Add a new mle/mac networkkey and initiate a parent request.

    fn command(
        &self,
        command_num: usize,
        _arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            1 => self
                .apps
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readonly_processbuffer(ro_allow::WRITE)
                        .and_then(|ro_buf| {
                            ro_buf.enter(|src_key| {
                                // check Thread state, if thread state is not empty,
                                // another userspace application has control of the Thread
                                // network and other requesting applications should fail.
                                if self.state.is_some() {
                                    return CommandReturn::failure(ErrorCode::BUSY);
                                }

                                // src key consists of the mle and mac keys; Thread
                                // hash is performed in userland and 32 byte hash is
                                // passed to thread capsule and entered as mac/mle key
                                // (For key generation see Thread spec v1.3.0 7.1.4)
                                if src_key.len() != 32 {
                                    return CommandReturn::failure(ErrorCode::SIZE);
                                }
                                let mut mle_key = [0u8; 16];
                                let mut mac_key = [0u8; 16];
                                src_key[..16].copy_to_slice(&mut mle_key);
                                src_key[16..32].copy_to_slice(&mut mac_key);
                                self.set_networkkey(mle_key, mac_key);

                                // Thread state begins as detached if sucessfully joined
                                self.state.replace(ThreadState::Detached);
                                CommandReturn::success()
                            })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .map_or_else(
                    |err| CommandReturn::failure(err.into()),
                    |ok_val| {
                        // If no failure in saving the mle/mac key, initiate
                        // sending the parent request
                        self.send_parent_req();
                        ok_val
                    },
                ),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, A: time::Alarm<'a>> UDPSendClient for ThreadNetworkDriver<'a, A> {
    fn send_done(&self, _result: Result<(), ErrorCode>, mut dgram: SubSliceMut<'static, u8>) {
        // TODO: handle result from send done and respond accordingly

        // Panicking on unwrap indicates the state was taken without replacement
        // (unreachable with proper state machine implementation)
        let curr_state = self.state.take().unwrap();

        // Advance state machine
        let next_state = match curr_state {
            ThreadState::SendUpdate(dst_ip, dst_mac) => ThreadState::SEDActive(dst_ip, dst_mac),
            ThreadState::SendUDPMsg => unimplemented!(),
            ThreadState::SendChildIdReq(_) => ThreadState::WaitingChildRsp,
            ThreadState::SendParentReq => {
                // UNCOMMENT TO DEBUG THREAD //
                // kernel::debug!("[Thread] Completed sending parent request to multicast IP");
                ThreadState::WaitingParentRsp
            }
            _ => panic!("Thread state machine diverged"),
        };

        self.frame_count.set(self.frame_count.get() + 1);

        // Replace the returned buffer and state
        dgram.reset();
        self.send_buffer.replace(dgram);
        self.state.replace(next_state);
    }
}

impl<'a, A: time::Alarm<'a>> time::AlarmClient for ThreadNetworkDriver<'a, A> {
    // TODO: This function is mainly here as a place holder as it will be needed
    // for implementing timeouts/timing for sending heartbeat messages to the parent
    // node
    fn alarm(&self) {
        match self.state.take().unwrap() {
            // TODO: Implement retries as defined in the thread spec (when timeouts occur)
            ThreadState::Detached => unimplemented!("[Thread ALARM] Detached"),
            ThreadState::SendParentReq => unimplemented!("[Thread ALARM] Send Parent Req"),
            ThreadState::SendChildIdReq(_) => unimplemented!("[Thread ALARM] Send Child ID Req"),
            ThreadState::SEDActive(_ipaddr, _mac) => {
                // TODO: SEND HEARTBEAT to parent node
                unimplemented!("[Thread ALARM] Send Heartbeat")
            }
            _ => panic!(""),
        }
    }
}

impl<'a, A: time::Alarm<'a>> UDPRecvClient for ThreadNetworkDriver<'a, A> {
    fn receive(
        &self,
        src_addr: IPAddr,
        dst_addr: IPAddr,
        _src_port: u16,
        _dst_port: u16,
        payload: &[u8],
    ) {
        if payload[0] != SECURITY_SUITE_ENCRYP {
            // Tock's current implementation of Thread ignores all messages that do not possess MLE encryption. This
            // is due to the Thread spec stating "Except for when specifically indicated, incoming
            // messages that are not secured with either MLE or link-layer security SHOULD be ignored." (v.1.3.0 sect 4.10)
            // UNCOMMENT TO DEBUG THREAD //
            // kernel::debug!("[Thread] DROPPED PACKET - Received unencrypted MLE packet.");
        }

        // decode aux security header from packet into Security data type
        let sec_res = ieee802154::Security::decode(&payload[1..]).done();

        // Guard statement for improperly formated aux sec header
        if sec_res.is_none() {
            // UNCOMMENT TO DEBUG THREAD //
            // kernel::debug!("[Thread] DROPPED PACKET - Malformed auxiliary security header.");
            return;
        }

        let security = sec_res.unwrap().1;

        // Take the receive buffer and pass to the `perform_crypto_op` wrapper function. This
        // initiates encoding all relevant auth data, setting crypto engine and initiating the
        // crypto operation.
        self.recv_buffer.take().map_or_else(
            || {
                // UNCOMMENT TO DEBUG THREAD //
                // kernel::debug!("[Thread] DROPPED PACKET - Receive buffer not available")
            },
            |recv_buf| {
                self.perform_crypt_op(
                    src_addr,
                    dst_addr,
                    security,
                    &payload[SECURITY_SUITE_LEN + AUX_SEC_HEADER_LENGTH
                        ..payload.len() - security.level.mic_len()],
                    recv_buf.take(),
                )
                .map_or_else(
                    // Error check on crypto operation. If the crypto operation
                    // fails, we log the error and replace the receive buffer for
                    // future receptions
                    |(_code, buf)| {
                        // UNCOMMENT TO DEBUG THREAD alter _code to code//
                        // kernel::debug!(
                        //     "[Thread] DROPPED PACKET - Crypto Operation Error *{:?}",
                        //     code
                        // );
                        self.recv_buffer.replace(SubSliceMut::new(buf));
                    },
                    |()| (),
                )
            },
        );
    }
}

impl<'a, A: time::Alarm<'a>> CCMClient for ThreadNetworkDriver<'a, A> {
    fn crypt_done(&self, buf: &'static mut [u8], _res: Result<(), ErrorCode>, _tag_is_valid: bool) {
        // TODO: check validity of result/tag and handle accordingly

        // Obtain the length of the payload from the sizelock
        let buf_len = self.crypto_sizelock.take().unwrap();

        // The auth data contains the src_addr || dest_addr || aux_sec_header;
        // Recover src/dst addr from the auth data
        let mut src_ipv6 = [0u8; IPV6_LEN];
        let mut dst_ipv6 = [0u8; IPV6_LEN];
        src_ipv6.copy_from_slice(&buf[..IPV6_LEN]);
        dst_ipv6.copy_from_slice(&buf[IPV6_LEN..(2 * IPV6_LEN)]);

        // The crypto operation requires 32 bytes (2 IPV6 addresses) as part of the auth data. We
        // do not care about this data so we shift the data to overwrite this data with the aux sec header
        // and payload. We shift this to an offset of 1 so that we can add the security suite field
        let auth_addr_offset = AUTH_DATA_LEN - AUX_SEC_HEADER_LENGTH;
        buf.copy_within(auth_addr_offset.., SECURITY_SUITE_LEN);

        // Recover the length of the mic from the security information encoded in the aux_sec_header
        let mic_len = ieee802154::Security::decode(&buf[SECURITY_SUITE_LEN..])
            .done()
            .unwrap()
            .1
            .level
            .mic_len();

        // We hard code the security suite to `0` for now as all messages are
        // assumed to be encrypted for the current implementation
        buf[..SECURITY_SUITE_LEN].copy_from_slice(&[SECURITY_SUITE_ENCRYP]);

        // the assembled_buf_len is the length of: security suite || aux sec header || mle payload || mic
        let assembled_buf_len = buf_len - auth_addr_offset + SECURITY_SUITE_LEN;

        // We create a new subslice that we will slice accordingly depending on if we are sending/receiving
        let mut assembled_subslice = SubSliceMut::new(buf);

        // Panicking on unwrap indicates the state was taken without replacement
        // (unreachable with proper state machine implementation)
        let curr_state = self.state.take().unwrap();

        match curr_state {
            ThreadState::SendParentReq | ThreadState::SendChildIdReq(_) => {
                //TODO: Add alarm for timeouts

                // To send, we need to send: security suite || aux sec header || mle payload || mic
                // which correlates to the assembled_buf_len
                assembled_subslice.slice(..assembled_buf_len);

                let dest_ipv6 = match curr_state {
                    // Determine destination IP depending on message type
                    ThreadState::SendParentReq => MULTICAST_IPV6,
                    ThreadState::SendChildIdReq(dst_ipv6) => dst_ipv6,
                    _ => unreachable!(),
                };

                // we replace the state with the current state here
                // because we cannot advance the state machine until
                // after the `send_done` callback is received
                self.state.replace(curr_state);

                // Begin sending the transmission
                self.sender
                    .driver_send_to(
                        dest_ipv6,
                        THREAD_PORT_NUMBER,
                        THREAD_PORT_NUMBER,
                        assembled_subslice,
                        self.driver_send_cap,
                        self.net_cap,
                    )
                    .map_err(|buf| {
                        // if the sending fails prior to transmission, replace
                        // the buffer and pass error accordingly to terminate_child_join
                        // in following unwrap statement
                        self.send_buffer.replace(buf);
                        ErrorCode::FAIL
                    })
                    .unwrap_or_else(|code| self.terminate_child_join(Err(code)));
            }
            ThreadState::WaitingChildRsp => {
                // TODO: Receive child response
            }
            ThreadState::WaitingParentRsp => {
                // Upon receiving messages, the receive logic only requires the MLE payload. Subsequently,
                // we slice the assembled_subslice to exclude the security suite, aux sec header, and mic.
                assembled_subslice
                    .slice(AUX_SEC_HEADER_LENGTH + SECURITY_SUITE_LEN..assembled_buf_len - mic_len);

                // Move the decrypted MLE message into the recv_buf and execute the receiving logic. Upon
                // an error in `recv_logic`, joining the network fails and schedule termination upcall
                self.recv_buffer.replace(assembled_subslice);
                if let Err(code) = self.recv_logic(IPAddr(src_ipv6)) {
                    self.terminate_child_join(Err(code))
                }
            }
            _ => (),
        }
    }
}
