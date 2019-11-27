//! UDP userspace interface for transmit and receive.
//!
//! Implements a userspace interface for sending and receiving UDP messages.
//! Processes use this driver to send UDP packets from a common interface
//! and bind to UDP ports for receiving packets.
//! Also exposes a list of interface addresses to the application (currently
//! hard-coded).

/// Syscall number
use crate::driver;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::stream::encode_u16;
use crate::net::stream::encode_u8;
use crate::net::stream::SResult;
use crate::net::udp::udp_port_table::{PortQuery, UdpPortManager};
use crate::net::udp::udp_recv::UDPRecvClient;
use crate::net::udp::udp_send::{UDPSendClient, UDPSender};
use crate::net::util::host_slice_to_u16;
use core::cell::Cell;
use core::{cmp, mem};
use kernel::capabilities::UdpDriverCapability;
use kernel::common::buffer::Buffer;
use kernel::common::cells::MapCell;
use kernel::{debug, AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};
pub const DRIVER_NUM: usize = driver::NUM::Udp as usize;

#[derive(Debug, Clone, Eq, PartialEq)]
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
        stream_len_cond!(buf, mem::size_of::<UDPEndpoint>() + offset);

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
    rx_callback: Option<Callback>,
    tx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    app_cfg: Option<AppSlice<Shared, u8>>,
    app_rx_cfg: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<[UDPEndpoint; 2]>,
    bound_port: Option<UDPEndpoint>,
}

#[allow(dead_code)]
pub struct UDPDriver<'a> {
    /// UDP sender
    sender: &'a dyn UDPSender<'a>,

    /// Grant of apps that use this radio driver.
    apps: Grant<App>,
    /// ID of app whose transmission request is being processed.
    current_app: Cell<Option<AppId>>,

    /// List of IP Addresses of the interfaces on the device
    interface_list: &'static [IPAddr],

    /// Maximum length payload that an app can transmit via this driver
    max_tx_pyld_len: usize,

    /// UDP bound port table (manages kernel bindings)
    port_table: &'static UdpPortManager,

    kernel_buffer: MapCell<Buffer<'static, u8>>,

    driver_send_cap: &'static dyn UdpDriverCapability,
}

impl<'a> UDPDriver<'a> {
    pub fn new(
        sender: &'a dyn UDPSender<'a>,
        grant: Grant<App>,
        interface_list: &'static [IPAddr],
        max_tx_pyld_len: usize,
        port_table: &'static UdpPortManager,
        kernel_buffer: Buffer<'static, u8>,
        driver_send_cap: &'static dyn UdpDriverCapability,
    ) -> UDPDriver<'a> {
        UDPDriver {
            sender: sender,
            apps: grant,
            current_app: Cell::new(None),
            interface_list: interface_list,
            max_tx_pyld_len: max_tx_pyld_len,
            port_table: port_table,
            kernel_buffer: MapCell::new(kernel_buffer),
            driver_send_cap: driver_send_cap,
        }
    }

    /// Utility function to perform an action on an app in a system call.
    #[inline]
    fn do_with_app<F>(&self, appid: AppId, closure: F) -> ReturnCode
    where
        F: FnOnce(&mut App) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| closure(app))
            .unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform an action using an app's config buffer.
    #[inline]
    #[allow(dead_code)]
    fn do_with_cfg<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&[u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_cfg.as_ref().map_or(ReturnCode::EINVAL, |cfg| {
                    if cfg.len() != len {
                        return ReturnCode::EINVAL;
                    }
                    closure(cfg.as_ref())
                })
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform a write to an app's config buffer.
    #[inline]
    fn do_with_cfg_mut<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&mut [u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_cfg.as_mut().map_or(ReturnCode::EINVAL, |cfg| {
                    if cfg.len() != len {
                        return ReturnCode::EINVAL;
                    }
                    closure(cfg.as_mut())
                })
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform an action using an app's RX config buffer.
    /// (quick and dirty ctrl-c, ctrl-v from above)
    #[inline]
    #[allow(dead_code)]
    fn do_with_rx_cfg<F>(&self, appid: AppId, closure: F) -> ReturnCode
    where
        F: FnOnce(&[u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_rx_cfg
                    .as_ref()
                    .map_or(ReturnCode::EINVAL, |cfg| closure(cfg.as_ref()))
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform a write to an app's RX config buffer.
    /// (also a quick and dirty ctrl-c)
    #[inline]
    #[allow(dead_code)]
    fn do_with_rx_cfg_mut<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&mut [u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_rx_cfg.as_mut().map_or(ReturnCode::EINVAL, |cfg| {
                    if cfg.len() != len {
                        return ReturnCode::EINVAL;
                    }
                    closure(cfg.as_mut())
                })
            })
            .unwrap_or_else(|err| err.into())
    }

    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `AppId`.
    fn get_next_tx_if_idle(&self) -> Option<AppId> {
        if self.current_app.get().is_some() {
            // Tx already in progress
            return None;
        }
        let mut pending_app = None;
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if app.pending_tx.is_some() {
                    pending_app = Some(app.appid());
                }
            });
            if pending_app.is_some() {
                break;
            }
        }
        pending_app
    }

    /// Performs `appid`'s pending transmission asynchronously. If the
    /// transmission is not successful, the error is returned to the app via its
    /// `tx_callback`. Assumes that the driver is currently idle and the app has
    /// a pending transmission.
    #[inline]
    fn perform_tx_async(&self, appid: AppId) {
        let result = self.perform_tx_sync(appid);
        if result != ReturnCode::SUCCESS {
            let _ = self.apps.enter(appid, |app, _| {
                app.tx_callback
                    .map(|mut cb| cb.schedule(result.into(), 0, 0));
            });
        }
    }

    /// Performs `appid`'s pending transmission synchronously. The result is
    /// returned immediately to the app. Assumes that the driver is currently
    /// idle and the app has a pending transmission.
    #[inline]
    fn perform_tx_sync(&self, appid: AppId) -> ReturnCode {
        self.do_with_app(appid, |app| {
            let addr_ports = match app.pending_tx.take() {
                Some(pending_tx) => pending_tx,
                None => {
                    return ReturnCode::SUCCESS;
                }
            };
            let dst_addr = addr_ports[1].addr;
            let dst_port = addr_ports[1].port;
            let src_port = addr_ports[0].port;

            // Send UDP payload. Copy payload into packet buffer held by this driver, then queue
            // it on the udp_mux.
            let result = app
                .app_write
                .as_ref()
                .map_or(ReturnCode::ENOMEM, |payload| {
                    self.kernel_buffer
                        .take()
                        .map_or(ReturnCode::ENOMEM, |mut kernel_buffer| {
                            kernel_buffer[0..payload.len()].copy_from_slice(payload.as_ref());
                            kernel_buffer.slice(0..payload.len());
                            match self.sender.driver_send_to(
                                dst_addr,
                                dst_port,
                                src_port,
                                kernel_buffer,
                                self.driver_send_cap,
                            ) {
                                Ok(_) => ReturnCode::SUCCESS,
                                Err(mut buf) => {
                                    buf.reset();
                                    self.kernel_buffer.replace(buf);
                                    ReturnCode::FAIL
                                }
                            }
                        })
                });
            if result == ReturnCode::SUCCESS {
                self.current_app.set(Some(appid));
            }
            result
        })
    }

    /// Schedule the next transmission if there is one pending. Performs the
    /// transmission eventually, returning any errors via asynchronous callbacks.
    #[inline]
    #[allow(dead_code)]
    fn do_next_tx_queued(&self) {
        self.get_next_tx_if_idle()
            .map(|appid| self.perform_tx_async(appid));
    }

    /// Schedule the next transmission if there is one pending. If the next
    /// transmission happens to be the one that was just queued, then the
    /// transmission is immediate. Hence, errors must be returned immediately.
    /// On the other hand, if it is some other app, then return any errors via
    /// callbacks.
    #[inline]
    fn do_next_tx_immediate(&self, new_appid: AppId) -> ReturnCode {
        self.get_next_tx_if_idle()
            .map_or(ReturnCode::SUCCESS, |appid| {
                if appid == new_appid {
                    let sync_result = self.perform_tx_sync(appid);
                    if sync_result == ReturnCode::SUCCESS {
                        return ReturnCode::SuccessWithValue { value: 1 }; //Indicates packet passed to radio
                    }
                    sync_result
                } else {
                    self.perform_tx_async(appid);
                    ReturnCode::SUCCESS
                }
            })
    }

    #[inline]
    fn parse_ip_port_pair(&self, buf: &[u8]) -> Option<UDPEndpoint> {
        if buf.len() != mem::size_of::<UDPEndpoint>() {
            debug!(
                "[parse] len is {:?}, not {:?} as expected",
                buf.len(),
                mem::size_of::<UDPEndpoint>()
            );
            None
        } else {
            let (a, p) = buf.split_at(mem::size_of::<IPAddr>());
            let mut addr = IPAddr::new();
            addr.0.copy_from_slice(a);

            let pair = UDPEndpoint {
                addr: addr,
                port: host_slice_to_u16(p),
            };
            Some(pair)
        }
    }
}

impl<'a> Driver for UDPDriver<'a> {
    /// Setup buffers to read/write from.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Read buffer. Will contain the received payload.
    /// - `1`: Write buffer. Contains the UDP payload to be transmitted.
    /// - `2`: Config buffer. Used to contain miscellaneous data associated with
    ///        some commands, namely source/destination addresses and ports.
    /// - `3`: Rx config buffer. Used to contain source/destination addresses
    ///        and ports for receives (separate from `2` because receives may
    ///        be waiting for an incoming packet asynchronously).
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            0 | 1 | 2 | 3 => self.do_with_app(appid, |app| {
                let mut success = true;
                match allow_num {
                    0 => app.app_read = slice,
                    1 => match slice {
                        Some(s) => {
                            if s.len() > self.max_tx_pyld_len {
                                success = false;
                            } else {
                                app.app_write = Some(s);
                            }
                        }
                        None => {}
                    },
                    2 => app.app_cfg = slice,
                    3 => app.app_rx_cfg = slice,
                    _ => {}
                }
                if success {
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EINVAL //passed tx buffer too long
                }
            }),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup callback for when packet is received. If no port has
    ///        been bound, return ERESERVE to indicate that port binding is
    ///        is a prerequisite to reception.
    /// - `1`: Setup callback for when packet is transmitted. Notably,
    ///        this callback receives the result of the send_done callback
    ///        from udp_send.rs, which does not currently pass information
    ///        regarding whether packets were acked at the link layer.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self.do_with_app(app_id, |app| {
                if app.bound_port.is_some() {
                    app.rx_callback = callback;
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::ERESERVE
                }
            }),
            1 => self.do_with_app(app_id, |app| {
                app.tx_callback = callback;
                ReturnCode::SUCCESS
            }),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// UDP control
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Get the interface list
    ///        app_cfg (out): 16 * `n` bytes: the list of interface IPv6 addresses, length
    ///                       limited by `app_cfg` length.
    /// - `2`: Transmit payload.
    ///        Returns EBUSY is this process already has a pending tx.
    ///        Returns EINVAL if no valid buffer has been loaded into the write buffer,
    ///        or if the config buffer is the wrong length, or if the destination and source
    ///        port/address pairs cannot be parsed.
    ///        Otherwise, returns the result of do_next_tx_immediate(). Notably, a successful
    ///        transmit can produce two different success values. If success is returned,
    ///        this simply means that the packet was queued. In this case, the app still
    ///        still needs to wait for a callback to check if any errors occurred before
    ///        the packet was passed to the radio. However, if SuccessWithValue
    ///        is returned with value 1, this means the the packet was successfully passed
    ///        the radio without any errors, which tells the userland application that it does
    ///        not need to wait for a callback to check if any errors occured while the packet
    ///        was being passed down to the radio. Any successful return value indicates that
    ///        the app should wait for a send_done() callback before attempting to queue another
    ///        packet.
    ///        Currently, only will transmit if the app has bound to the port passed in the tx_cfg
    ///        buf as the source address. If no port is bound, returns ERESERVE, if it tries to
    ///        send on a port other than the port which is bound, returns EINVALID.
    ///
    ///        Notably, the currently transmit implementation allows for starvation - an
    ///        an app with a lower app id can send constantly and starve an app with a
    ///        later ID.
    /// - `3`: Bind to the address in rx_cfg. Returns SUCCESS if that addr/port combo is free,
    ///        returns EINVAL if the address requested is not a local interface, or if the port
    ///        requested is 0. Returns EBUSY if that port is already bound to by another app.
    ///        This command should be called after allow() is called on the rx_cfg buffer, and
    ///        before subscribe() is used to set up the recv callback. Additionally, apps can only
    ///        send on ports after they have bound to said port. If this command is called
    ///        and the address in rx_cfg is 0::0 : 0, this command will reset the option
    ///        containing the bound port to None and set the rx callback to None. Notably,
    ///        the current implementation of this only allows for each app to bind to a single
    ///        port at a time, as such an implementation conserves memory (and is similar
    ///        to the approach applied by TinyOS and Riot). Further, there is
    ///        currently no mechanism for anything in the kernel to bind to ports, and there
    ///        is no distinction between ephemeral ports and reserved ports.
    /// - `4`: Returns the maximum payload that can be transmitted by apps using this driver.
    ///        This represents the size of the payload buffer in the kernel. Apps can use this
    ///        syscall to ensure they do not attempt to send too-large messages.

    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,

            //  Writes the requested number of network interface addresses
            // `arg1`: number of interfaces requested that will fit into the buffer
            1 => self.do_with_cfg_mut(appid, arg1 * mem::size_of::<IPAddr>(), |cfg| {
                let n_ifaces_to_copy = cmp::min(arg1, self.interface_list.len());
                let iface_size = mem::size_of::<IPAddr>();
                for i in 0..n_ifaces_to_copy {
                    cfg[i * iface_size..(i + 1) * iface_size]
                        .copy_from_slice(&self.interface_list[i].0);
                }
                // Returns total number of interfaces
                ReturnCode::SuccessWithValue {
                    value: self.interface_list.len(),
                }
            }),

            // Transmits UDP packet stored in tx_buf
            2 => {
                self.do_with_app(appid, |app| {
                    if app.pending_tx.is_some() {
                        // Cannot support more than one pending tx per process.
                        return ReturnCode::EBUSY;
                    }
                    if app.bound_port.is_none() {
                        // Currently, apps need to bind to a port before they can send from said port
                        return ReturnCode::ERESERVE;
                    }
                    let next_tx = app.app_cfg.as_ref().and_then(|cfg| {
                        if cfg.len() != 2 * mem::size_of::<UDPEndpoint>() {
                            return None;
                        }

                        if let (Some(dst), Some(src)) = (
                            self.parse_ip_port_pair(&cfg.as_ref()[mem::size_of::<UDPEndpoint>()..]),
                            self.parse_ip_port_pair(&cfg.as_ref()[..mem::size_of::<UDPEndpoint>()]),
                        ) {
                            if Some(src.clone()) == app.bound_port {
                                Some([src, dst])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    });
                    if next_tx.is_none() {
                        return ReturnCode::EINVAL;
                    }
                    app.pending_tx = next_tx;
                    self.do_next_tx_immediate(appid)
                })
            }
            3 => {
                self.do_with_app(appid, |app| {
                    // Move UDPEndpoint into udp.rs?
                    let mut requested_addr_opt = app.app_rx_cfg.as_ref().and_then(|cfg| {
                        if cfg.len() != 2 * mem::size_of::<UDPEndpoint>() {
                            None
                        } else if let Some(local_iface) =
                            self.parse_ip_port_pair(&cfg.as_ref()[mem::size_of::<UDPEndpoint>()..])
                        {
                            Some(local_iface)
                        } else {
                            None
                        }
                    });
                    if requested_addr_opt.is_none() {
                        return ReturnCode::EINVAL;
                    }
                    if requested_addr_opt.is_some() {
                        let requested_addr = requested_addr_opt.expect("missing address.");
                        // If zero address, close any already bound socket
                        if requested_addr.is_zero() {
                            app.rx_callback = None;
                            app.bound_port = None;
                            return ReturnCode::SUCCESS;
                        }
                        // Check that requested addr is a local interface
                        let mut requested_is_local = false;
                        for i in 0..self.interface_list.len() {
                            if requested_addr.addr == self.interface_list[i] {
                                requested_is_local = true;
                            }
                        }
                        if !requested_is_local {
                            return ReturnCode::EINVAL;
                        }
                        let mut addr_already_bound = false;
                        // This checks the bound ports in the other grants.
                        // This code needs to be replicated in the bound port
                        // table when checking the userspace apps.
                        for app in self.apps.iter() {
                            app.enter(|other_app, _| {
                                if other_app.bound_port.is_some() {
                                    let other_addr_opt = other_app.bound_port.clone();
                                    let other_addr =
                                        other_addr_opt.expect("Missing other address.");
                                    if other_addr.port == requested_addr.port {
                                        if other_addr.addr == requested_addr.addr {
                                            addr_already_bound = true;
                                        }
                                    }
                                }
                            });
                        }
                        // Check bound ports in the kernel.
                        match self.port_table.is_bound(requested_addr.port) {
                            Ok(bound) => {
                                addr_already_bound = bound;
                            }
                            Err(_) => {
                                return ReturnCode::FAIL;
                            } //error in port table
                        }
                        // Also check the bound port table here.
                        if addr_already_bound {
                            ReturnCode::EBUSY
                        } else {
                            requested_addr_opt = Some(requested_addr);
                            // If this point is reached, the requested addr is free and valid
                            app.bound_port = requested_addr_opt;
                            ReturnCode::SUCCESS
                        }
                    } else {
                        ReturnCode::EINVAL
                    }
                })
            }
            4 => ReturnCode::SuccessWithValue {
                value: self.max_tx_pyld_len,
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a> UDPSendClient for UDPDriver<'a> {
    fn send_done(&self, result: ReturnCode, mut dgram: Buffer<'static, u8>) {
        // Replace the returned kernel buffer. Now we can send the next msg.
        dgram.reset();
        self.kernel_buffer.replace(dgram);
        self.current_app.get().map(|appid| {
            let _ = self.apps.enter(appid, |app, _| {
                app.tx_callback
                    .map(|mut cb| cb.schedule(result.into(), 0, 0));
            });
        });
        self.current_app.set(None);
        self.do_next_tx_queued();
    }
}

impl<'a> UDPRecvClient for UDPDriver<'a> {
    fn receive(
        &self,
        src_addr: IPAddr,
        dst_addr: IPAddr,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) {
        self.apps.each(|app| {
            if app.bound_port.is_some() {
                let appid = app.appid();
                self.do_with_app(app.appid(), |app| {
                    let mut for_me = false;
                    app.bound_port.as_ref().map(|requested_addr| {
                        if requested_addr.addr == dst_addr && requested_addr.port == dst_port {
                            for_me = true;
                        }
                    });
                    if for_me {
                        let mut app_read = app.app_read.take();
                        app_read.as_mut().map(|rbuf| {
                            let rbuf = rbuf.as_mut();
                            let len = payload.len();
                            if rbuf.len() >= len {
                                // silently ignore packets that don't fit?
                                rbuf[..len].copy_from_slice(&payload[..len]);

                                // Write address of sender into rx_cfg so it can be read by client
                                let sender_addr = UDPEndpoint {
                                    addr: src_addr,
                                    port: src_port,
                                };
                                let cfg_len = 2 * mem::size_of::<UDPEndpoint>();
                                self.do_with_rx_cfg_mut(appid, cfg_len, |cfg| {
                                    sender_addr.encode(cfg, 0);
                                    ReturnCode::SUCCESS
                                });
                                app.rx_callback.map(|mut cb| cb.schedule(len, 0, 0));
                            }
                        });
                        app.app_read = app_read;
                    }
                    ReturnCode::SUCCESS
                });
            }
        });
    }
}

impl<'a> PortQuery for UDPDriver<'a> {
    // Returns true if |port| is bound (on any iface), false otherwise.
    fn is_bound(&self, port: u16) -> bool {
        let mut port_bound = false;
        for app in self.apps.iter() {
            app.enter(|other_app, _| {
                if other_app.bound_port.is_some() {
                    let other_addr_opt = other_app.bound_port.clone();
                    let other_addr = other_addr_opt.expect("Missing other_addr");
                    if other_addr.port == port {
                        port_bound = true;
                    }
                }
            });
        }
        port_bound
    }
}
