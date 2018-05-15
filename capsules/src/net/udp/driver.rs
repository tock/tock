//! UDP userspace interface for transmit and receive.
//!
//! Implements a userspace interface for sending and receiving UDP messages.
//! Also exposes a list of interface addresses to the application (currently
//! hard-coded.

// use net::stream::{decode_bytes, decode_u8, encode_bytes, encode_u8, SResult};
use core::cell::Cell;
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};
use net::ipv6::ip_utils::IPAddr;
use net::udp::udp_send::UDPSender;

/// Syscall number
pub const DRIVER_NUM: usize = 0x30002;

pub struct App {
    rx_callback: Option<Callback>,
    tx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    app_cfg: Option<AppSlice<Shared, u8>>,
    app_rx_cfg: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<IPAddr>,
}

impl Default for App {
    fn default() -> Self {
        App {
            rx_callback: None,
            tx_callback: None,
            app_read: None,
            app_write: None,
            app_cfg: None,
            app_rx_cfg: None,
            pending_tx: None,
        }
    }
}

pub struct UDPDriver<'a> {
    /// UDP sender
    sender: &'a UDPSender<'a>,

    /// Grant of apps that use this radio driver.
    apps: Grant<App>,
    /// ID of app whose transmission request is being processed.
    current_app: Cell<Option<AppId>>,

    /// Buffer that stores the UDP frame to be transmitted.
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a> UDPDriver<'a> {
    pub fn new(
        sender: &'a UDPSender<'a>,
        grant: Grant<App>,
        kernel_tx: &'static mut [u8],
    ) -> UDPDriver<'a> {
        UDPDriver {
            sender: sender,
            apps: grant,
            current_app: Cell::new(None),
            kernel_tx: TakeCell::new(kernel_tx),
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
    fn do_with_cfg<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&[u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_cfg
                    .take()
                    .as_ref()
                    .map_or(ReturnCode::EINVAL, |cfg| {
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
                app.app_cfg
                    .take()
                    .as_mut()
                    .map_or(ReturnCode::EINVAL, |cfg| {
                        if cfg.len() != len {
                            return ReturnCode::EINVAL;
                        }
                        closure(cfg.as_mut())
                    })
            })
            .unwrap_or_else(|err| err.into())
    }
    
    /// Utility function to perform an action using an app's RX config buffer.
    /// (quick and dirty ctrl-c, ctrl-v from above
    #[inline]
    fn do_with_rx_cfg<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&[u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_rx_cfg
                    .take()
                    .as_ref()
                    .map_or(ReturnCode::EINVAL, |cfg| {
                        if cfg.len() != len {
                            return ReturnCode::EINVAL;
                        }
                        closure(cfg.as_ref())
                    })
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform a write to an app's RX config buffer.
    #[inline]
    fn do_with_rx_cfg_mut<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&mut [u8]) -> ReturnCode,
    {
        self.apps
            .enter(appid, |app, _| {
                app.app_rx_cfg
                    .take()
                    .as_mut()
                    .map_or(ReturnCode::EINVAL, |cfg| {
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
                    .take()
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
            let dst_addr = match app.pending_tx.take() {
                Some(pending_tx) => pending_tx,
                None => {
                    return ReturnCode::SUCCESS;
                }
            };
            let result = self.kernel_tx.take().map_or(ReturnCode::ENOMEM, |kbuf| {
                // TODO: do transmission stuff 
                /*
                // Prepare the frame headers
                let pan = self.mac.get_pan();
                let dst_addr = MacAddress::Short(dst_addr);
                let src_addr = MacAddress::Short(self.mac.get_address());
                let mut frame = match self.mac.prepare_data_frame(
                    kbuf,
                    pan,
                    dst_addr,
                    pan,
                    src_addr,
                    security_needed,
                ) {
                    Ok(frame) => frame,
                    Err(kbuf) => {
                        self.kernel_tx.replace(kbuf);
                        return ReturnCode::FAIL;
                    }
                };

                // Append the payload: there must be one
                let result = app.app_write
                    .take()
                    .as_ref()
                    .map(|payload| frame.append_payload(payload.as_ref()))
                    .unwrap_or(ReturnCode::EINVAL);
                if result != ReturnCode::SUCCESS {
                    return result;
                }

                // Finally, transmit the frame
                let (result, mbuf) = self.mac.transmit(frame);
                if let Some(buf) = mbuf {
                    self.kernel_tx.replace(buf);
                }
                result
                */
                ReturnCode::SUCCESS
            });
            if result == ReturnCode::SUCCESS {
                self.current_app.set(Some(appid));
            }
            result
        })
    }

    /// Schedule the next transmission if there is one pending. Performs the
    /// transmission asynchronously, returning any errors via callbacks.
    #[inline]
    fn do_next_tx_async(&self) {
        self.get_next_tx_if_idle()
            .map(|appid| self.perform_tx_async(appid));
    }

    /// Schedule the next transmission if there is one pending. If the next
    /// transmission happens to be the one that was just queued, then the
    /// transmission is synchronous. Hence, errors must be returned immediately.
    /// On the other hand, if it is some other app, then return any errors via
    /// callbacks.
    #[inline]
    fn do_next_tx_sync(&self, new_appid: AppId) -> ReturnCode {
        self.get_next_tx_if_idle()
            .map(|appid| {
                if appid == new_appid {
                    self.perform_tx_sync(appid)
                } else {
                    self.perform_tx_async(appid);
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or(ReturnCode::SUCCESS)
    }
}

impl<'a> Driver for RadioDriver<'a> {
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
    ///        be asynchronous).
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            0 | 1 | 2 => self.do_with_app(appid, |app| {
                match allow_num {
                    0 => app.app_read = slice,
                    1 => app.app_write = slice,
                    2 => app.app_cfg = slice,
                    3 => app.app_rx_cfg = slice,
                    _ => {}
                }
                ReturnCode::SUCCESS
            }),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup callback for when frame is received.
    /// - `1`: Setup callback for when frame is transmitted.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self.do_with_app(app_id, |app| {
                app.rx_callback = callback;
                ReturnCode::SUCCESS
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
    /// - `2`: Transmit payload 
    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            /* 
            // TODO: generate interface list 
            1 => self.do_with_cfg_mut(appid, 8, |cfg| {
                cfg.copy_from_slice(&self.mac.get_address_long());
                ReturnCode::SUCCESS
            }),
            // TODO: transmit UDP packet
            2 => {
                self.do_with_app(appid, |app| {
                    if app.pending_tx.is_some() {
                        // Cannot support more than one pending tx per process.
                        return ReturnCode::EBUSY;
                    }
                    let next_tx = app.app_cfg.as_ref().and_then(|cfg| {
                        if cfg.len() != 11 {
                            return None;
                        }
                        let dst_addr = arg1 as u16;
                        let level = match SecurityLevel::from_scf(cfg.as_ref()[0]) {
                            Some(level) => level,
                            None => {
                                return None;
                            }
                        };
                        if level == SecurityLevel::None {
                            Some((dst_addr, None))
                        } else {
                            let key_id = match decode_key_id(&cfg.as_ref()[1..]).done() {
                                Some((_, key_id)) => key_id,
                                None => {
                                    return None;
                                }
                            };
                            Some((dst_addr, Some((level, key_id))))
                        }
                    });
                    if next_tx.is_none() {
                        return ReturnCode::EINVAL;
                    }
                    app.pending_tx = next_tx;

                    self.do_next_tx_sync(appid)
                })
            }
            */
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

/*
// TODO: UDP send_done
impl<'a> device::TxClient for RadioDriver<'a> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.kernel_tx.replace(spi_buf);
        self.current_app.get().map(|appid| {
            let _ = self.apps.enter(appid, |app, _| {
                app.tx_callback
                    .take()
                    .map(|mut cb| cb.schedule(result.into(), acked as usize, 0));
            });
        });
        self.current_app.set(None);
        self.do_next_tx_async();
    }
}
*/

// TODO: UDP receive path

