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
use crate::net::thread::thread::ThreadState;

use crate::net::network_capabilities::NetworkCapability;
use crate::net::stream::encode_bytes;
use crate::net::stream::encode_u16;
use crate::net::stream::encode_u32;
use crate::net::stream::encode_u8;
use crate::net::stream::SResult;
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
use kernel::utilities::leasable_buffer::LeasableMutableBuffer;
use kernel::{ErrorCode, ProcessId};

use capsules_core::driver;
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

    /// List of IP Addresses of the interfaces on the device
    interface_list: &'static [IPAddr],

    /// Maximum length payload that an app can transmit via this driver
    max_tx_pyld_len: usize,

    /// UDP bound port table (manages kernel bindings)
    port_table: &'static UdpPortManager,

    send_buffer: MapCell<LeasableMutableBuffer<'static, u8>>,

    recv_buffer: MapCell<LeasableMutableBuffer<'static, u8>>,

    state: ThreadState,

    driver_send_cap: &'static dyn UdpDriverCapability,

    net_cap: &'static NetworkCapability,
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
        interface_list: &'static [IPAddr],
        max_tx_pyld_len: usize,
        port_table: &'static UdpPortManager,
        send_buffer: LeasableMutableBuffer<'static, u8>,
        recv_buffer: LeasableMutableBuffer<'static, u8>,
        driver_send_cap: &'static dyn UdpDriverCapability,
        net_cap: &'static NetworkCapability,
    ) -> ThreadNetworkDriver<'a> {
        ThreadNetworkDriver {
            sender: sender,
            crypto: crypto,
            apps: grant,
            current_app: Cell::new(None),
            interface_list: interface_list,
            max_tx_pyld_len: max_tx_pyld_len,
            port_table: port_table,
            send_buffer: MapCell::new(send_buffer),
            recv_buffer: MapCell::new(recv_buffer),
            state: ThreadState::new(),
            driver_send_cap: driver_send_cap,
            net_cap: net_cap,
        }
    }

    pub fn init_thread_binding(&self) -> (UdpPortBindingRx, UdpPortBindingTx) {
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
        self.send_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                // let mle_msg: [u8; 32] = [
                //     0x00, 0x15, 0xc4, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x09, 0x01,
                //     0x01, 0x0f, 0x03, 0x08, 0xfa, 0x67, 0x49, 0xbb, 0x48, 0x91, 0x3f, 0xf6, 0x0e,
                //     0x01, 0x80, 0x12, 0x02, 0x00, 0x04,
                // ];

                let mle_msg: [u8; 63] = [
                    0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0xb5, 0xa6, 0x91, 0xee,
                    0x42, 0x56, 0x36, 0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x15, 0xc4, 0x31, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x01, 0x09, 0x01, 0x01, 0x0f, 0x03, 0x08, 0xfa, 0x67, 0x49, 0xbb,
                    0x48, 0x91, 0x3f, 0xf6, 0x0e, 0x01, 0x80, 0x12, 0x02, 0x00, 0x04,
                ];

                send_buffer[..mle_msg.len()].copy_from_slice(&mle_msg);
                send_buffer.slice(0..(mle_msg.len() + 4));

                // Hardcoded for now, this should probably be moved elsewhere (already stored in kernel)
                let device_addr: [u8; 8] = [0xa2, 0xb5, 0xa6, 0x91, 0xee, 0x42, 0x56, 0x36];

                // let key = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
                let key: [u8; 16] = [
                    0x54, 0x45, 0xf4, 0x15, 0x8f, 0xd7, 0x59, 0x12, 0x17, 0x58, 0x09, 0xf8, 0xb5,
                    0x7a, 0x66, 0xa4,
                ];

                let nonce = get_ccm_nonce(&device_addr, 12740, SecurityLevel::EncMic32);

                self.crypto.set_key(&key);
                self.crypto.set_nonce(&nonce);

                self.crypto
                    .crypt(send_buffer.take(), 0, 42, 21, 4, true, true);

                Ok(())
            })
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
        kernel::debug!("we have received a command for thread");
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
    fn send_done(
        &self,
        result: Result<(), ErrorCode>,
        mut dgram: LeasableMutableBuffer<'static, u8>,
    ) {
        // Replace the returned kernel buffer. Now we can send the next msg.
        dgram.reset();
        self.send_buffer.replace(dgram);
        self.current_app.get().map(|processid| {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                upcalls
                    .schedule_upcall(1, (kernel::errorcode::into_statuscode(result), 0, 0))
                    .ok();
            });
        });
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
        // Obtain frame counter from the UDP packet
        let frame_counter = decode_u32(&payload[2..6]).done().unwrap().1.to_be();

        // Obtain MLE payload from received UDP packet
        let payload = &payload[11..payload.len() - 4];

        // relevant values for encryption
        let a_off = 0;
        let m_off = 0;
        let m_len = payload.len();
        let mic_len = 4;
        let confidential = false;
        let encrypting = true;
        let level = 5; // hardcoded for now

        // Hardcoded for now, this should probably be moved elsewhere (already stored in kernel)
        let device_addr: [u8; 8] = [0xa2, 0xb5, 0xa6, 0x91, 0xee, 0x42, 0x56, 0x36];

        // let key = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let key = [
            0x54, 0x45, 0xf4, 0x15, 0x8f, 0xd7, 0x59, 0x12, 0x17, 0x58, 0x09, 0xf8, 0xb5, 0x7a,
            0x66, 0xa4,
        ];

        // generate nonce
        let nonce = get_ccm_nonce(&device_addr, frame_counter, SecurityLevel::EncMic32);

        // set nonce/key for encryption
        if self.crypto.set_key(&key) != Ok(()) || self.crypto.set_nonce(&nonce) != Ok(()) {
            kernel::debug!("FAIL KEY SET AND NONCE");
        }

        let crypto_res =
            self.recv_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |mut recv_buffer| {
                    if payload.len() > recv_buffer.len() {
                        kernel::debug!("no space!");
                        self.recv_buffer.replace(recv_buffer);
                        return Err(ErrorCode::SIZE);
                    }

                    recv_buffer[..payload.len()].copy_from_slice(payload);

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
        kernel::debug!("TAG IS VALID {:?}", tag_is_valid);
        kernel::debug!("CUR BUF {:?}", buf);
        if self.send_buffer.is_none() {
            // let aux_sec_header: &[u8; 11] = &[
            //     0x00, 0x15, 0xc4, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            // ];
            // buf.copy_within(0..25, aux_sec_header.len());
            // buf[0..aux_sec_header.len()].copy_from_slice(aux_sec_header);
            // buf[(32)..(36)].copy_from_slice(&[0xe3, 0x5d, 0xc4, 0x7f]);

            buf.copy_within(32..67, 1);

            let zero_slic: &[u8; 1] = &[0];
            buf[0..1].copy_from_slice(zero_slic);

            self.send_buffer.replace(LeasableMutableBuffer::new(buf));

            self.send_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |mut send_buffer| {
                    kernel::debug!("ENTERED INNER");

                    send_buffer.slice(0..36);

                    kernel::debug!("THIS IS OUR KERNEL BUF {:?}", send_buffer);

                    self.sender.driver_send_to(
                        IPAddr([
                            0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x02,
                        ]),
                        MacAddress::Short(0xFFFF),
                        19788,
                        19788,
                        send_buffer,
                        self.driver_send_cap,
                        self.net_cap,
                    );

                    Ok(())
                });
        }
    }
}
