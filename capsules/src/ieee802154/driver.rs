//! IEEE 802.15.4 userspace interface for configuration and transmit/receive.
//!
//! Implements a userspace interface for sending and receiving IEEE 802.15.4
//! frames. Also provides a minimal list-based interface for managing keys and
//! known link neighbors, which is needed for 802.15.4 security.

use crate::ieee802154::{device, framer};
use crate::net::ieee802154::{AddressMode, Header, KeyId, MacAddress, PanID, SecurityLevel};
use crate::net::stream::{decode_bytes, decode_u8, encode_bytes, encode_u8, SResult};
use core::cell::Cell;
use core::cmp::min;
use kernel::common::cells::{MapCell, OptionalCell, TakeCell};
use kernel::{AppId, AppSlice, Callback, Grant, LegacyDriver, ReturnCode, SharedReadWrite};

const MAX_NEIGHBORS: usize = 4;
const MAX_KEYS: usize = 4;

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Ieee802154 as usize;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct DeviceDescriptor {
    short_addr: u16,
    long_addr: [u8; 8],
}

impl Default for DeviceDescriptor {
    fn default() -> Self {
        DeviceDescriptor {
            short_addr: 0,
            long_addr: [0; 8],
        }
    }
}

/// The Key ID mode mapping expected by the userland driver
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum KeyIdModeUserland {
    Implicit = 0,
    Index = 1,
    Source4Index = 2,
    Source8Index = 3,
}

impl KeyIdModeUserland {
    pub fn from_u8(byte: u8) -> Option<KeyIdModeUserland> {
        match byte {
            0 => Some(KeyIdModeUserland::Implicit),
            1 => Some(KeyIdModeUserland::Index),
            2 => Some(KeyIdModeUserland::Source4Index),
            3 => Some(KeyIdModeUserland::Source8Index),
            _ => None,
        }
    }
}

/// Encodes a key ID into a buffer in the format expected by the userland driver.
fn encode_key_id(key_id: &KeyId, buf: &mut [u8]) -> SResult {
    let off = enc_consume!(buf; encode_u8, KeyIdModeUserland::from(key_id) as u8);
    let off = match *key_id {
        KeyId::Implicit => 0,
        KeyId::Index(index) => enc_consume!(buf, off; encode_u8, index),
        KeyId::Source4Index(ref src, index) => {
            let off = enc_consume!(buf, off; encode_bytes, src);
            enc_consume!(buf, off; encode_u8, index)
        }
        KeyId::Source8Index(ref src, index) => {
            let off = enc_consume!(buf, off; encode_bytes, src);
            enc_consume!(buf, off; encode_u8, index)
        }
    };
    stream_done!(off);
}

/// Decodes a key ID that is in the format produced by the userland driver.
fn decode_key_id(buf: &[u8]) -> SResult<KeyId> {
    stream_len_cond!(buf, 1);
    let mode = stream_from_option!(KeyIdModeUserland::from_u8(buf[0]));
    match mode {
        KeyIdModeUserland::Implicit => stream_done!(0, KeyId::Implicit),
        KeyIdModeUserland::Index => {
            let (off, index) = dec_try!(buf; decode_u8);
            stream_done!(off, KeyId::Index(index));
        }
        KeyIdModeUserland::Source4Index => {
            let mut src = [0u8; 4];
            let off = dec_consume!(buf; decode_bytes, &mut src);
            let (off, index) = dec_try!(buf, off; decode_u8);
            stream_done!(off, KeyId::Source4Index(src, index));
        }
        KeyIdModeUserland::Source8Index => {
            let mut src = [0u8; 8];
            let off = dec_consume!(buf; decode_bytes, &mut src);
            let (off, index) = dec_try!(buf, off; decode_u8);
            stream_done!(off, KeyId::Source8Index(src, index));
        }
    }
}

impl From<&KeyId> for KeyIdModeUserland {
    fn from(key_id: &KeyId) -> Self {
        match *key_id {
            KeyId::Implicit => KeyIdModeUserland::Implicit,
            KeyId::Index(_) => KeyIdModeUserland::Index,
            KeyId::Source4Index(_, _) => KeyIdModeUserland::Source4Index,
            KeyId::Source8Index(_, _) => KeyIdModeUserland::Source8Index,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct KeyDescriptor {
    level: SecurityLevel,
    key_id: KeyId,
    key: [u8; 16],
}

impl Default for KeyDescriptor {
    fn default() -> Self {
        KeyDescriptor {
            level: SecurityLevel::None,
            key_id: KeyId::Implicit,
            key: [0; 16],
        }
    }
}

impl KeyDescriptor {
    fn decode(buf: &[u8]) -> SResult<KeyDescriptor> {
        stream_len_cond!(buf, 27);
        let level = stream_from_option!(SecurityLevel::from_scf(buf[0]));
        let (_, key_id) = dec_try!(buf, 1; decode_key_id);
        let mut key = [0u8; 16];
        let off = dec_consume!(buf, 11; decode_bytes, &mut key);
        stream_done!(
            off,
            KeyDescriptor {
                level: level,
                key_id: key_id,
                key: key,
            }
        );
    }
}

pub struct App {
    rx_callback: Option<Callback>,
    tx_callback: Option<Callback>,
    app_read: Option<AppSlice<SharedReadWrite, u8>>,
    app_write: Option<AppSlice<SharedReadWrite, u8>>,
    app_cfg: Option<AppSlice<SharedReadWrite, u8>>,
    pending_tx: Option<(u16, Option<(SecurityLevel, KeyId)>)>,
}

impl Default for App {
    fn default() -> Self {
        App {
            rx_callback: None,
            tx_callback: None,
            app_read: None,
            app_write: None,
            app_cfg: None,
            pending_tx: None,
        }
    }
}

pub struct RadioDriver<'a> {
    /// Underlying MAC device, possibly multiplexed
    mac: &'a dyn device::MacDevice<'a>,

    /// List of (short address, long address) pairs representing IEEE 802.15.4
    /// neighbors.
    neighbors: MapCell<[DeviceDescriptor; MAX_NEIGHBORS]>,
    /// Actual number of neighbors in the fixed size array of neighbors.
    num_neighbors: Cell<usize>,

    /// List of (security level, key_id, key) tuples representing IEEE 802.15.4
    /// key descriptors.
    keys: MapCell<[KeyDescriptor; MAX_KEYS]>,
    /// Actual number of keys in the fixed size array of keys.
    num_keys: Cell<usize>,

    /// Grant of apps that use this radio driver.
    apps: Grant<App>,
    /// ID of app whose transmission request is being processed.
    current_app: OptionalCell<AppId>,

    /// Buffer that stores the IEEE 802.15.4 frame to be transmitted.
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a> RadioDriver<'a> {
    pub fn new(
        mac: &'a dyn device::MacDevice<'a>,
        grant: Grant<App>,
        kernel_tx: &'static mut [u8],
    ) -> RadioDriver<'a> {
        RadioDriver {
            mac: mac,
            neighbors: MapCell::new(Default::default()),
            num_neighbors: Cell::new(0),
            keys: MapCell::new(Default::default()),
            num_keys: Cell::new(0),
            apps: grant,
            current_app: OptionalCell::empty(),
            kernel_tx: TakeCell::new(kernel_tx),
        }
    }

    // Neighbor management functions

    /// Add a new neighbor to the end of the list if there is still space
    /// for one, returning its new index. If the neighbor already exists,
    /// returns the index of the existing neighbor. Returns `None` if there is
    /// no remaining space.
    fn add_neighbor(&self, new_neighbor: DeviceDescriptor) -> Option<usize> {
        self.neighbors.and_then(|neighbors| {
            let num_neighbors = self.num_neighbors.get();
            let position = neighbors[..num_neighbors]
                .iter()
                .position(|neighbor| *neighbor == new_neighbor);
            match position {
                Some(index) => Some(index),
                None => {
                    if num_neighbors == MAX_NEIGHBORS {
                        None
                    } else {
                        neighbors[num_neighbors] = new_neighbor;
                        self.num_neighbors.set(num_neighbors + 1);
                        Some(num_neighbors)
                    }
                }
            }
        })
    }

    /// Deletes the neighbor at `index` if `index` is valid, returning
    /// `ReturnCode::SUCCESS`. Otherwise, returns `ReturnCode::EINVAL`.  Ensures
    /// that the `neighbors` list is compact by shifting forward any elements
    /// after the index.
    fn remove_neighbor(&self, index: usize) -> ReturnCode {
        let num_neighbors = self.num_neighbors.get();
        if index < num_neighbors {
            self.neighbors.map(|neighbors| {
                for i in index..(num_neighbors - 1) {
                    neighbors[i] = neighbors[i + 1];
                }
            });
            self.num_neighbors.set(num_neighbors - 1);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EINVAL
        }
    }

    /// Gets the `DeviceDescriptor` corresponding to the neighbor at a
    /// particular `index`, if the `index` is valid. Otherwise, returns `None`
    fn get_neighbor(&self, index: usize) -> Option<DeviceDescriptor> {
        if index < self.num_neighbors.get() {
            self.neighbors.map(|neighbors| neighbors[index])
        } else {
            None
        }
    }

    // Key management functions

    /// Add a new key to the end of the list if there is still space
    /// for one, returning its new index. If the key already exists,
    /// returns the index of the existing key. Returns `None` if there
    /// is no remaining space.
    fn add_key(&self, new_key: KeyDescriptor) -> Option<usize> {
        self.keys.and_then(|keys| {
            let num_keys = self.num_keys.get();
            let position = keys[..num_keys].iter().position(|key| *key == new_key);
            match position {
                Some(index) => Some(index),
                None => {
                    if num_keys == MAX_KEYS {
                        None
                    } else {
                        keys[num_keys] = new_key;
                        self.num_keys.set(num_keys + 1);
                        Some(num_keys)
                    }
                }
            }
        })
    }

    /// Deletes the key at `index` if `index` is valid, returning
    /// `ReturnCode::SUCCESS`. Otherwise, returns `ReturnCode::EINVAL`.  Ensures
    /// that the `keys` list is compact by shifting forward any elements
    /// after the index.
    fn remove_key(&self, index: usize) -> ReturnCode {
        let num_keys = self.num_keys.get();
        if index < num_keys {
            self.keys.map(|keys| {
                for i in index..(num_keys - 1) {
                    keys[i] = keys[i + 1];
                }
            });
            self.num_keys.set(num_keys - 1);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EINVAL
        }
    }

    /// Gets the `DeviceDescriptor` corresponding to the key at a
    /// particular `index`, if the `index` is valid. Otherwise, returns `None`
    fn get_key(&self, index: usize) -> Option<KeyDescriptor> {
        if index < self.num_keys.get() {
            self.keys.map(|keys| keys[index])
        } else {
            None
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

    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `AppId`.
    fn get_next_tx_if_idle(&self) -> Option<AppId> {
        if self.current_app.is_some() {
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
            let (dst_addr, security_needed) = match app.pending_tx.take() {
                Some(pending_tx) => pending_tx,
                None => {
                    return ReturnCode::SUCCESS;
                }
            };
            let result = self.kernel_tx.take().map_or(ReturnCode::ENOMEM, |kbuf| {
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
                let result = app
                    .app_write
                    .take()
                    .as_ref()
                    .map_or(ReturnCode::EINVAL, |payload| {
                        frame.append_payload(payload.as_ref())
                    });
                if result != ReturnCode::SUCCESS {
                    return result;
                }

                // Finally, transmit the frame
                let (result, mbuf) = self.mac.transmit(frame);
                if let Some(buf) = mbuf {
                    self.kernel_tx.replace(buf);
                }
                result
            });
            if result == ReturnCode::SUCCESS {
                self.current_app.set(appid);
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
            .map_or(ReturnCode::SUCCESS, |appid| {
                if appid == new_appid {
                    self.perform_tx_sync(appid)
                } else {
                    self.perform_tx_async(appid);
                    ReturnCode::SUCCESS
                }
            })
    }
}

impl framer::DeviceProcedure for RadioDriver<'_> {
    /// Gets the long address corresponding to the neighbor that matches the given
    /// MAC address. If no such neighbor exists, returns `None`.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<[u8; 8]> {
        self.neighbors.and_then(|neighbors| {
            neighbors[..self.num_neighbors.get()]
                .iter()
                .find(|neighbor| match addr {
                    MacAddress::Short(addr) => addr == neighbor.short_addr,
                    MacAddress::Long(addr) => addr == neighbor.long_addr,
                })
                .map(|neighbor| neighbor.long_addr)
        })
    }
}

impl framer::KeyProcedure for RadioDriver<'_> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<[u8; 16]> {
        self.keys.and_then(|keys| {
            keys[..self.num_keys.get()]
                .iter()
                .find(|key| key.level == level && key.key_id == key_id)
                .map(|key| key.key)
        })
    }
}

impl LegacyDriver for RadioDriver<'_> {
    /// Setup buffers to read/write from.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Read buffer. Will contain the received frame.
    /// - `1`: Write buffer. Contains the frame payload to be transmitted.
    /// - `2`: Config buffer. Used to contain miscellaneous data associated with
    ///        some commands because the system call parameters / return codes are
    ///        not enough to convey the desired information.
    fn allow_readwrite(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<SharedReadWrite, u8>>,
    ) -> ReturnCode {
        match allow_num {
            0 | 1 | 2 => self.do_with_app(appid, |app| {
                match allow_num {
                    0 => app.app_read = slice,
                    1 => app.app_write = slice,
                    2 => app.app_cfg = slice,
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

    /// IEEE 802.15.4 MAC device control.
    ///
    /// For some of the below commands, one 32-bit argument is not enough to
    /// contain the desired input parameters or output data. For those commands,
    /// the config slice `app_cfg` is used as a channel to shuffle information
    /// between kernel space and user space. The expected size of the slice
    /// varies by command, and acts essentially like a custom FFI. That is, the
    /// userspace library MUST `allow()` a buffer of the correct size, otherwise
    /// the call is EINVAL. When used, the expected format is described below.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Return radio status. SUCCESS/EOFF = on/off.
    /// - `2`: Set short MAC address.
    /// - `3`: Set long MAC address.
    ///        app_cfg (in): 8 bytes: the long MAC address.
    /// - `4`: Set PAN ID.
    /// - `5`: Set channel.
    /// - `6`: Set transmission power.
    /// - `7`: Commit any configuration changes.
    /// - `8`: Get the short MAC address.
    /// - `9`: Get the long MAC address.
    ///        app_cfg (out): 8 bytes: the long MAC address.
    /// - `10`: Get the PAN ID.
    /// - `11`: Get the channel.
    /// - `12`: Get the transmission power.
    /// - `13`: Get the maximum number of neighbors.
    /// - `14`: Get the current number of neighbors.
    /// - `15`: Get the short address of the neighbor at an index.
    /// - `16`: Get the long address of the neighbor at an index.
    ///        app_cfg (out): 8 bytes: the long MAC address.
    /// - `17`: Add a new neighbor with the given short and long address.
    ///        app_cfg (in): 8 bytes: the long MAC address.
    /// - `18`: Remove the neighbor at an index.
    /// - `19`: Get the maximum number of keys.
    /// - `20`: Get the current number of keys.
    /// - `21`: Get the security level of the key at an index.
    /// - `22`: Get the key id of the key at an index.
    ///        app_cfg (out): 1 byte: the key ID mode +
    ///                       up to 9 bytes: the key ID.
    /// - `23`: Get the key at an index.
    ///        app_cfg (out): 16 bytes: the key.
    /// - `24`: Add a new key with the given descripton.
    ///        app_cfg (in): 1 byte: the security level +
    ///                      1 byte: the key ID mode +
    ///                      9 bytes: the key ID (might not use all bytes) +
    ///                      16 bytes: the key.
    /// - `25`: Remove the key at an index.
    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            1 => {
                if self.mac.is_on() {
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EOFF
                }
            }
            2 => {
                self.mac.set_address(arg1 as u16);
                ReturnCode::SUCCESS
            }
            3 => self.do_with_cfg(appid, 8, |cfg| {
                let mut addr_long = [0u8; 8];
                addr_long.copy_from_slice(cfg);
                self.mac.set_address_long(addr_long);
                ReturnCode::SUCCESS
            }),
            4 => {
                self.mac.set_pan(arg1 as u16);
                ReturnCode::SUCCESS
            }
            // XXX: Setting channel DEPRECATED by MAC layer channel control
            5 => ReturnCode::ENOSUPPORT,
            // XXX: Setting tx power DEPRECATED by MAC layer tx power control
            6 => ReturnCode::ENOSUPPORT,
            7 => {
                self.mac.config_commit();
                ReturnCode::SUCCESS
            }
            8 => {
                // Guarantee that address is positive by adding 1
                let addr = self.mac.get_address();
                ReturnCode::SuccessWithValue {
                    value: (addr as usize) + 1,
                }
            }
            9 => self.do_with_cfg_mut(appid, 8, |cfg| {
                cfg.copy_from_slice(&self.mac.get_address_long());
                ReturnCode::SUCCESS
            }),
            10 => {
                // Guarantee that the PAN is positive by adding 1
                let pan = self.mac.get_pan();
                ReturnCode::SuccessWithValue {
                    value: (pan as usize) + 1,
                }
            }
            // XXX: Getting channel DEPRECATED by MAC layer channel control
            11 => ReturnCode::ENOSUPPORT,
            // XXX: Getting tx power DEPRECATED by MAC layer tx power control
            12 => ReturnCode::ENOSUPPORT,
            13 => {
                // Guarantee that it is positive by adding 1
                ReturnCode::SuccessWithValue {
                    value: MAX_NEIGHBORS + 1,
                }
            }
            14 => {
                // Guarantee that it is positive by adding 1
                ReturnCode::SuccessWithValue {
                    value: self.num_neighbors.get() + 1,
                }
            }
            15 => self
                .get_neighbor(arg1)
                .map_or(ReturnCode::EINVAL, |neighbor| {
                    ReturnCode::SuccessWithValue {
                        value: (neighbor.short_addr as usize) + 1,
                    }
                }),
            16 => self.do_with_cfg_mut(appid, 8, |cfg| {
                self.get_neighbor(arg1)
                    .map_or(ReturnCode::EINVAL, |neighbor| {
                        cfg.copy_from_slice(&neighbor.long_addr);
                        ReturnCode::SUCCESS
                    })
            }),
            17 => self.do_with_cfg(appid, 8, |cfg| {
                let mut new_neighbor: DeviceDescriptor = DeviceDescriptor::default();
                new_neighbor.short_addr = arg1 as u16;
                new_neighbor.long_addr.copy_from_slice(cfg);
                self.add_neighbor(new_neighbor)
                    .map_or(ReturnCode::EINVAL, |index| ReturnCode::SuccessWithValue {
                        value: index + 1,
                    })
            }),
            18 => self.remove_neighbor(arg1),
            19 => {
                // Guarantee that it is positive by adding 1
                ReturnCode::SuccessWithValue {
                    value: MAX_KEYS + 1,
                }
            }
            20 => {
                // Guarantee that it is positive by adding 1
                ReturnCode::SuccessWithValue {
                    value: self.num_keys.get() + 1,
                }
            }
            21 => {
                self.get_key(arg1)
                    .map_or(ReturnCode::EINVAL, |key| ReturnCode::SuccessWithValue {
                        value: (key.level as usize) + 1,
                    })
            }
            22 => self.do_with_cfg_mut(appid, 10, |cfg| {
                self.get_key(arg1)
                    .and_then(|key| encode_key_id(&key.key_id, cfg).done())
                    .map_or(ReturnCode::EINVAL, |_| ReturnCode::SUCCESS)
            }),
            23 => self.do_with_cfg_mut(appid, 16, |cfg| {
                self.get_key(arg1).map_or(ReturnCode::EINVAL, |key| {
                    cfg.copy_from_slice(&key.key);
                    ReturnCode::SUCCESS
                })
            }),
            24 => self.do_with_cfg(appid, 27, |cfg| {
                KeyDescriptor::decode(cfg)
                    .done()
                    .and_then(|(_, new_key)| self.add_key(new_key))
                    .map_or(ReturnCode::EINVAL, |index| ReturnCode::SuccessWithValue {
                        value: index + 1,
                    })
            }),
            25 => self.remove_key(arg1),
            26 => {
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
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl device::TxClient for RadioDriver<'_> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.kernel_tx.replace(spi_buf);
        self.current_app.take().map(|appid| {
            let _ = self.apps.enter(appid, |app, _| {
                app.tx_callback
                    .take()
                    .map(|mut cb| cb.schedule(result.into(), acked as usize, 0));
            });
        });
        self.do_next_tx_async();
    }
}

/// Encode two PAN IDs into a single usize.
#[inline]
fn encode_pans(dst_pan: &Option<PanID>, src_pan: &Option<PanID>) -> usize {
    ((dst_pan.unwrap_or(0) as usize) << 16) | (src_pan.unwrap_or(0) as usize)
}

/// Encodes as much as possible about an address into a single usize.
#[inline]
fn encode_address(addr: &Option<MacAddress>) -> usize {
    let short_addr_only = match *addr {
        Some(MacAddress::Short(addr)) => addr as usize,
        _ => 0,
    };
    ((AddressMode::from(addr) as usize) << 16) | short_addr_only
}

impl device::RxClient for RadioDriver<'_> {
    fn receive<'b>(&self, buf: &'b [u8], header: Header<'b>, data_offset: usize, data_len: usize) {
        self.apps.each(|app| {
            app.app_read.take().as_mut().map(|rbuf| {
                let rbuf = rbuf.as_mut();
                let len = min(rbuf.len(), data_offset + data_len);
                // Copy the entire frame over to userland, preceded by two
                // bytes: the data offset and the data length.
                rbuf[..len].copy_from_slice(&buf[..len]);
                rbuf[0] = data_offset as u8;
                rbuf[1] = data_len as u8;

                // Encode useful parts of the header in 3 usizes
                let pans = encode_pans(&header.dst_pan, &header.src_pan);
                let dst_addr = encode_address(&header.dst_addr);
                let src_addr = encode_address(&header.src_addr);
                app.rx_callback
                    .take()
                    .map(|mut cb| cb.schedule(pans, dst_addr, src_addr));
            });
        });
    }
}
