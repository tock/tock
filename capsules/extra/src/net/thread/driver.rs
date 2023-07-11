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

use crate::net::ieee802154::MacAddress;
use crate::net::ieee802154::SecurityLevel;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ip_utils::MacAddr;
use crate::net::ipv6::ip_utils::DEFAULT_DST_MAC_ADDR;

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

    kernel_buffer: MapCell<LeasableMutableBuffer<'static, u8>>,

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
        kernel_buffer: LeasableMutableBuffer<'static, u8>,
        driver_send_cap: &'static dyn UdpDriverCapability,
        net_cap: &'static NetworkCapability,
    ) -> ThreadNetworkDriver<'a> {
        let a = ThreadNetworkDriver {
            sender: sender,
            crypto: crypto,
            apps: grant,
            current_app: Cell::new(None),
            interface_list: interface_list,
            max_tx_pyld_len: max_tx_pyld_len,
            port_table: port_table,
            kernel_buffer: MapCell::new(kernel_buffer),
            driver_send_cap: driver_send_cap,
            net_cap: net_cap,
        };
        a
    }

    pub fn init_binding(&self) -> (UdpPortBindingRx, UdpPortBindingTx) {
        match self.port_table.create_socket() {
            Ok(socket) => {
                kernel::debug!("successfully created a socket");
                // 16123
                match self.port_table.bind(socket, 5000, self.net_cap) {
                    Ok((udp_tx, udp_rx)) => {
                        kernel::debug!("successfully bound to port");
                        kernel::debug!("UDP RX {:?} ;; UDP TX {:?}", udp_rx, udp_tx);
                        (udp_rx, udp_tx)
                    }
                    Err(_) => panic!("failed bind to port"),
                }
            }
            Err(_) => panic!("Error in retrieving socket!"),
        }
    }
    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `ProcessId`.
    fn get_next_tx_if_idle(&self) -> Option<ProcessId> {
        if self.current_app.get().is_some() {
            // Tx already in progress
            return None;
        }
        let mut pending_app = None;
        for app in self.apps.iter() {
            let processid = app.processid();
            app.enter(|app, _| {
                if app.pending_tx.is_some() {
                    pending_app = Some(processid);
                }
            });
            if pending_app.is_some() {
                break;
            }
        }
        pending_app
    }

    /// Performs `processid`'s pending transmission asynchronously. If the
    /// transmission is not successful, the error is returned to the app via its
    /// `tx_callback`. Assumes that the driver is currently idle and the app has
    /// a pending transmission.
    #[inline]
    fn perform_tx_async(&self, processid: ProcessId) {
        let result = self.perform_tx_sync(processid);
        if result != Ok(()) {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                upcalls
                    .schedule_upcall(1, (kernel::errorcode::into_statuscode(result), 0, 0))
                    .ok();
            });
        }
    }

    /// Performs `processid`'s pending transmission synchronously. The result is
    /// returned immediately to the app. Assumes that the driver is currently
    /// idle and the app has a pending transmission.
    #[inline]
    fn perform_tx_sync(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, kernel_data| {
            let addr_ports = match app.pending_tx.take() {
                Some(pending_tx) => pending_tx,
                None => {
                    return Ok(());
                }
            };
            let dst_addr = addr_ports[1].addr;
            let dst_port = addr_ports[1].port;
            let src_port = addr_ports[0].port;
            let dst_mac = MacAddress::Short(0xFFFF);

            // Send UDP payload. Copy payload into packet buffer held by this driver, then queue
            // it on the udp_mux.
            let result = kernel_data
                .get_readonly_processbuffer(ro_allow::WRITE)
                .and_then(|write| {
                    write.enter(|payload| {
                        self.kernel_buffer.take().map_or(
                            Err(ErrorCode::NOMEM),
                            |mut kernel_buffer| {
                                if payload.len() > kernel_buffer.len() {
                                    self.kernel_buffer.replace(kernel_buffer);
                                    return Err(ErrorCode::SIZE);
                                }
                                payload.copy_to_slice(&mut kernel_buffer[0..payload.len()]);
                                kernel_buffer.slice(0..payload.len());
                                match self.sender.driver_send_to(
                                    dst_addr,
                                    DEFAULT_DST_MAC_ADDR,
                                    dst_port,
                                    src_port,
                                    kernel_buffer,
                                    self.driver_send_cap,
                                    self.net_cap,
                                ) {
                                    Ok(_) => Ok(()),
                                    Err(mut buf) => {
                                        buf.reset();
                                        self.kernel_buffer.replace(buf);
                                        Err(ErrorCode::FAIL)
                                    }
                                }
                            },
                        )
                    })
                })
                .unwrap_or(Err(ErrorCode::NOMEM));
            if result == Ok(()) {
                self.current_app.set(Some(processid));
            }
            result
        })?
    }

    /// Schedule the next transmission if there is one pending. Performs the
    /// transmission eventually, returning any errors via asynchronous callbacks.
    #[inline]
    #[allow(dead_code)]
    fn do_next_tx_queued(&self) {
        self.get_next_tx_if_idle()
            .map(|processid| self.perform_tx_async(processid));
    }

    /// Schedule the next transmission if there is one pending. If the next
    /// transmission happens to be the one that was just queued, then the
    /// transmission is immediate. Hence, errors must be returned immediately.
    /// On the other hand, if it is some other app, then return any errors via
    /// callbacks.
    #[inline]
    fn do_next_tx_immediate(&self, new_processid: ProcessId) -> Result<u32, ErrorCode> {
        self.get_next_tx_if_idle().map_or(Ok(0), |processid| {
            if processid == new_processid {
                let sync_result = self.perform_tx_sync(processid);
                if sync_result == Ok(()) {
                    Ok(1) //Indicates packet passed to radio
                } else {
                    Err(ErrorCode::try_from(sync_result).unwrap())
                }
            } else {
                self.perform_tx_async(processid);
                Ok(0) //indicates async transmission
            }
        })
    }

    #[inline]
    fn parse_ip_port_pair(&self, buf: &[u8]) -> Option<UDPEndpoint> {
        if buf.len() != size_of::<UDPEndpoint>() {
            debug!(
                "[parse] len is {:?}, not {:?} as expected",
                buf.len(),
                size_of::<UDPEndpoint>()
            );
            None
        } else {
            let (a, p) = buf.split_at(size_of::<IPAddr>());
            let mut addr = IPAddr::new();
            addr.0.copy_from_slice(a);

            let pair = UDPEndpoint {
                addr: addr,
                port: host_slice_to_u16(p),
            };
            Some(pair)
        }
    }

    pub fn send_mle(&self) {
        kernel::debug!("sending MLE");

        let mode_tlv: [u8; 3] = [0x01, 0x01, 0x0f];
        let challenge_tlv: [u8; 10] = [0x03, 0x08, 0xe0, 0xda, 0xf4, 0xea, 0x58, 0x39, 0xe2, 0x66];
        let scan_mask_tlv: [u8; 3] = [0x0e, 0x01, 0x80];
        let version_tlv: [u8; 4] = [0x12, 0x02, 0x00, 0x04];
        let command: [u8; 1] = [0x09];

        let mle_msg: [u8; 21] = [
            0x09, 0x01, 0x01, 0x0f, 0x03, 0x08, 0xe0, 0xda, 0xf4, 0xea, 0x58, 0x39, 0xe2, 0x66,
            0x0e, 0x01, 0x80, 0x12, 0x02, 0x00, 0x04,
        ];

        let dest = IPAddr([
            0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02,
        ]);
        let dst_port: u16 = 10;
        let src_port: u16 = 10;

        self.kernel_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut kernel_buffer| {
                // kernel::debug!("VALS TO SEND {:?}", kernel_buffer);
                // kernel::debug!("dst_port {:?}", dst_port);
                kernel_buffer[..mle_msg.len()].copy_from_slice(&mle_msg);

                kernel_buffer.slice(0..mle_msg.len());

                let res = self.sender.driver_send_to(
                    dest,
                    MacAddress::Short(0xFFFF),
                    dst_port,
                    src_port,
                    kernel_buffer,
                    self.driver_send_cap,
                    self.net_cap,
                );

                kernel::debug!("Send to res {:?}", res);

                Ok(())
            });
    }
}

fn get_ccm_nonce(device_addr: &[u8; 8], frame_counter: u32, level: SecurityLevel) -> [u8; 13] {
    let mut nonce = [0u8; 13];
    let encode_ccm_nonce = |buf: &mut [u8]| {
        let off = enc_consume!(buf; encode_bytes, device_addr.as_ref());
        let off = enc_consume!(buf, off; encode_u32, frame_counter);
        let off = enc_consume!(buf, off; encode_u8, level as u8);
        stream_done!(off);
    };
    match encode_ccm_nonce(&mut nonce).done() {
        None => {
            // This should not be possible
            panic!("Failed to produce ccm nonce");
        }
        Some(_) => nonce,
    }
}

impl<'a> SyscallDriver for ThreadNetworkDriver<'a> {
    /// Setup buffers to read/write from.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Read buffer. Will contain the received payload.
    /// - `1`: Config buffer. Used to contain miscellaneous data associated with
    ///        some commands, namely source/destination addresses and ports.
    /// - `2`: Rx config buffer. Used to contain source/destination addresses
    ///        and ports for receives (separate from `2` because receives may
    ///        be waiting for an incoming packet asynchronously).

    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Write buffer. Contains the UDP payload to be transmitted.
    ///        Returns SIZE if the passed buffer is too long, and NOSUPPORT
    ///        if an invalid `allow_num` is passed.

    // Setup callbacks.
    //
    // ### `subscribe_num`
    //
    // - `0`: Setup callback for when packet is received. If no port has
    //        been bound, return RESERVE to indicate that port binding is
    //        is a prerequisite to reception.
    // - `1`: Setup callback for when packet is transmitted. Notably,
    //        this callback receives the result of the send_done callback
    //        from udp_send.rs, which does not currently pass information
    //        regarding whether packets were acked at the link layer.

    /// UDP control
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Get the interface list
    ///        app_cfg (out): 16 * `n` bytes: the list of interface IPv6 addresses, length
    ///                       limited by `app_cfg` length.
    ///        Returns INVAL if the cfg buffer is the wrong size, or not available.
    /// - `2`: Transmit payload.
    ///        Returns BUSY is this process already has a pending tx.
    ///        Returns INVAL if no valid buffer has been loaded into the write buffer,
    ///        or if the config buffer is the wrong length, or if the destination and source
    ///        port/address pairs cannot be parsed.
    ///        Otherwise, returns the result of do_next_tx_immediate(). Notably, a successful
    ///        transmit can produce two different success values. If success is returned,
    ///        this simply means that the packet was queued. In this case, the app still
    ///        still needs to wait for a callback to check if any errors occurred before
    ///        the packet was passed to the radio. However, if Success_U32
    ///        is returned with value 1, this means the the packet was successfully passed
    ///        the radio without any errors, which tells the userland application that it does
    ///        not need to wait for a callback to check if any errors occurred while the packet
    ///        was being passed down to the radio. Any successful return value indicates that
    ///        the app should wait for a send_done() callback before attempting to queue another
    ///        packet.
    ///        Currently, only will transmit if the app has bound to the port passed in the tx_cfg
    ///        buf as the source address. If no port is bound, returns RESERVE, if it tries to
    ///        send on a port other than the port which is bound, returns INVALID.
    ///        Notably, the currently transmit implementation allows for starvation - an
    ///        an app with a lower app id can send constantly and starve an app with a
    ///        later ID.
    /// - `3`: Bind to the address in rx_cfg. Returns Ok(()) if that addr/port combo is free,
    ///        returns INVAL if the address requested is not a local interface, or if the port
    ///        requested is 0. Returns BUSY if that port is already bound to by another app.
    ///        This command should be called after allow() is called on the rx_cfg buffer, and
    ///        before subscribe() is used to set up the recv callback. Additionally, apps can only
    ///        send on ports after they have bound to said port. If this command is called
    ///        and the address in rx_cfg is 0::0 : 0, this command will reset the option
    ///        containing the bound port to None. Notably,
    ///        the current implementation of this only allows for each app to bind to a single
    ///        port at a time, as such an implementation conserves memory (and is similar
    ///        to the approach applied by TinyOS and Riot).
    ///        /// - `4`: Returns the maximum payload that can be transmitted by apps using this driver.
    ///        This represents the size of the payload buffer in the kernel. Apps can use this
    ///        syscall to ensure they do not attempt to send too-large messages.

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

            //  Writes the requested number of network interface addresses
            // `arg1`: number of interfaces requested that will fit into the buffer
            1 => {
                self.kernel_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |mut kernel_buffer| {
                        // payload.copy_to_slice(&mut kernel_buffer[0..payload.len()]);
                        // let mle_msg: [u8; 27] = [
                        //     0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x01, 0x01, 0x0f, 0x03, 0x08,
                        //     0xe0, 0xda, 0xf4, 0xea, 0x58, 0x39, 0xe2, 0x66, 0x0e, 0x01, 0x80, 0x12,
                        //     0x02, 0x00, 0x04,
                        // ];

                        let mle_msg: [u8; 21] = [
                            0x09, 0x01, 0x01, 0x0f, 0x03, 0x08, 0xfa, 0x67, 0x49, 0xbb, 0x48, 0x91,
                            0x3f, 0xf6, 0x0e, 0x01, 0x80, 0x12, 0x02, 0x00, 0x04,
                        ];

                        kernel_buffer[..mle_msg.len()].copy_from_slice(&mle_msg);
                        kernel_buffer.slice(0..(mle_msg.len()));

                        // Hardcoded for now, this should probably be moved elsewhere (already stored in kernel)
                        let device_addr: [u8; 8] = [0xa2, 0xb5, 0xa6, 0x91, 0xee, 0x42, 0x56, 0x35];

                        // let key = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
                        let key: [u8; 16] = [
                            0x54, 0x45, 0xf4, 0x15, 0x8f, 0xd7, 0x59, 0x12, 0x17, 0x58, 0x09, 0xf8,
                            0xb5, 0x7a, 0x66, 0xa4,
                        ];

                        let nonce = get_ccm_nonce(&device_addr, 12740, SecurityLevel::EncMic32);

                        kernel::debug!("OUR NONCE VALUE IS {:?}", nonce);
                        self.crypto.set_key(&key);
                        self.crypto.set_nonce(&nonce);

                        self.crypto
                            .crypt(kernel_buffer.take(), 0, 0, mle_msg.len(), 4, true, true);

                        Ok(())
                    });
                /*
                kernel::debug!("SUCCESSFULLY ENTERED THREAD CAPSULE..");
                self.apps
                    .enter(processid, |_, kernel_data| {
                        kernel::debug!("good news...");
                        kernel_data
                            .get_readwrite_processbuffer(rw_allow::CFG)
                            .and_then(|cfg| {
                                cfg.mut_enter(|cfg| {
                                    if cfg.len() != arg1 * size_of::<IPAddr>() {
                                        return CommandReturn::failure(ErrorCode::INVAL);
                                    }
                                    let n_ifaces_to_copy =
                                        cmp::min(arg1, self.interface_list.len());
                                    let iface_size = size_of::<IPAddr>();
                                    for i in 0..n_ifaces_to_copy {
                                        cfg[i * iface_size..(i + 1) * iface_size]
                                            .copy_from_slice(&self.interface_list[i].0);
                                    }
                                    // Returns total number of interfaces
                                    CommandReturn::success_u32(self.interface_list.len() as u32)
                                })
                            })
                            .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                    })
                    .unwrap_or_else(|err| CommandReturn::failure(err.into()))
                    */
                CommandReturn::success_u32(self.max_tx_pyld_len as u32)
            }

            // Transmits UDP packet stored in tx_buf
            2 => {
                let res = self
                    .apps
                    .enter(processid, |app, kernel_data| {
                        if app.pending_tx.is_some() {
                            // Cannot support more than one pending tx per process.
                            return Err(ErrorCode::BUSY);
                        }
                        if app.bound_port.is_none() {
                            // Currently, apps need to bind to a port before they can send from said port
                            return Err(ErrorCode::RESERVE);
                        }
                        let next_tx = kernel_data
                            .get_readwrite_processbuffer(rw_allow::CFG)
                            .and_then(|cfg| {
                                cfg.enter(|cfg| {
                                    if cfg.len() != 2 * size_of::<UDPEndpoint>() {
                                        return None;
                                    }

                                    let mut tmp_cfg_buffer: [u8; size_of::<UDPEndpoint>() * 2] =
                                        [0; size_of::<UDPEndpoint>() * 2];
                                    cfg.copy_to_slice(&mut tmp_cfg_buffer);

                                    if let (Some(dst), Some(src)) = (
                                        self.parse_ip_port_pair(
                                            &tmp_cfg_buffer[size_of::<UDPEndpoint>()..],
                                        ),
                                        self.parse_ip_port_pair(
                                            &tmp_cfg_buffer[..size_of::<UDPEndpoint>()],
                                        ),
                                    ) {
                                        if Some(src.clone()) == app.bound_port {
                                            Some([src, dst])
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                            })
                            .unwrap_or(None);
                        if next_tx.is_none() {
                            return Err(ErrorCode::INVAL);
                        }
                        app.pending_tx = next_tx;
                        Ok(())
                    })
                    .unwrap_or_else(|err| Err(err.into()));
                match res {
                    Ok(_) => self.do_next_tx_immediate(processid).map_or_else(
                        |err| CommandReturn::failure(err.into()),
                        |v| CommandReturn::success_u32(v),
                    ),
                    Err(e) => CommandReturn::failure(e),
                }
            }
            3 => {
                let err = self
                    .apps
                    .enter(processid, |app, kernel_data| {
                        // Move UDPEndpoint into udp.rs?
                        let requested_addr_opt = kernel_data
                            .get_readwrite_processbuffer(rw_allow::RX_CFG)
                            .and_then(|rx_cfg| {
                                rx_cfg.enter(|cfg| {
                                    if cfg.len() != 2 * mem::size_of::<UDPEndpoint>() {
                                        None
                                    } else {
                                        let mut tmp_endpoint: [u8; mem::size_of::<UDPEndpoint>()] =
                                            [0; mem::size_of::<UDPEndpoint>()];
                                        cfg[mem::size_of::<UDPEndpoint>()..]
                                            .copy_to_slice(&mut tmp_endpoint);

                                        if let Some(local_iface) =
                                            self.parse_ip_port_pair(&tmp_endpoint)
                                        {
                                            Some(local_iface)
                                        } else {
                                            None
                                        }
                                    }
                                })
                            })
                            .unwrap_or(None);
                        requested_addr_opt.map_or(Err(Err(ErrorCode::INVAL)), |requested_addr| {
                            // If zero address, close any already bound socket
                            if requested_addr.is_zero() {
                                app.bound_port = None;
                                return Ok(None);
                            }
                            // Check that requested addr is a local interface
                            let mut requested_is_local = false;
                            for i in 0..self.interface_list.len() {
                                if requested_addr.addr == self.interface_list[i] {
                                    requested_is_local = true;
                                }
                            }
                            if !requested_is_local {
                                return Err(Err(ErrorCode::INVAL));
                            }
                            Ok(Some(requested_addr))
                        })
                    })
                    .unwrap_or_else(|err| Err(err.into()));
                match err {
                    Ok(requested_addr_opt) => {
                        requested_addr_opt.map_or(CommandReturn::success(), |requested_addr| {
                            // Check bound ports in the kernel.
                            match self.port_table.is_bound(requested_addr.port) {
                                Ok(bound) => {
                                    if bound {
                                        CommandReturn::failure(ErrorCode::BUSY)
                                    } else {
                                        self.apps
                                            .enter(processid, |app, _| {
                                                // The requested addr is free and valid
                                                app.bound_port = Some(requested_addr);
                                                CommandReturn::success()
                                            })
                                            .unwrap_or_else(|err| {
                                                CommandReturn::failure(err.into())
                                            })
                                    }
                                }
                                Err(_) => CommandReturn::failure(ErrorCode::FAIL), //error in port table
                            }
                        })
                    }
                    Err(retcode) => CommandReturn::failure(retcode.try_into().unwrap()),
                }
            }
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
        self.kernel_buffer.replace(dgram);
        self.current_app.get().map(|processid| {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                upcalls
                    .schedule_upcall(1, (kernel::errorcode::into_statuscode(result), 0, 0))
                    .ok();
            });
        });
        self.current_app.set(None);
        self.do_next_tx_queued();
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
        let confidential = true;
        let encrypting = true;
        let level = 5; // hardcoded for now

        // Hardcoded for now, this should probably be moved elsewhere (already stored in kernel)
        let device_addr: [u8; 8] = [0xa2, 0xb5, 0xa6, 0x91, 0xee, 0x42, 0x56, 0x35];

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
            self.kernel_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |mut kernel_buffer| {
                    if payload.len() > kernel_buffer.len() {
                        kernel::debug!("no space!");
                        self.kernel_buffer.replace(kernel_buffer);
                        return Err(ErrorCode::SIZE);
                    }

                    kernel_buffer[..payload.len()].copy_from_slice(payload);

                    let cryp_out = self.crypto.crypt(
                        kernel_buffer.take(),
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
        // if buf[0] == 9 {

        // }
        // For now place
        kernel::debug!("CRYPT DOWN");
        let aux_sec_header: &[u8; 11] = &[
            0x00, 0x15, 0xc4, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        buf.copy_within(0..21, aux_sec_header.len());
        buf[0..aux_sec_header.len()].copy_from_slice(aux_sec_header);
        buf[(aux_sec_header.len() + 21)..(aux_sec_header.len() + 25)]
            .copy_from_slice(&[0xe3, 0x5d, 0xc4, 0x7f]);

        self.kernel_buffer.replace(LeasableMutableBuffer::new(buf));

        self.kernel_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut kernel_buffer| {
                kernel::debug!("ENTERED INNER");

                kernel_buffer.slice(0..(aux_sec_header.len() + 25));

                kernel::debug!("THIS IS OUR KERNEL BUF {:?}", kernel_buffer);

                self.sender.driver_send_to(
                    IPAddr([
                        0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x02,
                    ]),
                    MacAddress::Short(0xFFFF),
                    19788,
                    19788,
                    kernel_buffer,
                    self.driver_send_cap,
                    self.net_cap,
                );

                Ok(())
            });
    }
}
