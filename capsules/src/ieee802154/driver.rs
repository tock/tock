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
use core::mem;
use kernel::common::cells::{MapCell, OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::{
    CommandReturn, Driver, ErrorCode, Grant, ProcessId, ReadOnlyProcessBuffer,
    ReadWriteProcessBuffer, ReadableProcessBuffer, WriteableProcessBuffer,
};

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

#[derive(Default)]
pub struct App {
    app_read: ReadWriteProcessBuffer,
    app_write: ReadOnlyProcessBuffer,
    app_cfg: ReadWriteProcessBuffer,
    pending_tx: Option<(u16, Option<(SecurityLevel, KeyId)>)>,
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
    apps: Grant<App, 2>,
    /// ID of app whose transmission request is being processed.
    current_app: OptionalCell<ProcessId>,

    /// Buffer that stores the IEEE 802.15.4 frame to be transmitted.
    kernel_tx: TakeCell<'static, [u8]>,

    /// Used to ensure callbacks are delivered during upcalls
    deferred_caller: &'a DynamicDeferredCall,

    /// Also used for deferred calls
    handle: OptionalCell<DeferredCallHandle>,

    /// Used to deliver callbacks to the correct app during deferred calls
    saved_appid: OptionalCell<ProcessId>,

    /// Used to save result for passing a callback from a deferred call.
    saved_result: OptionalCell<Result<(), ErrorCode>>,
}

impl<'a> RadioDriver<'a> {
    pub fn new(
        mac: &'a dyn device::MacDevice<'a>,
        grant: Grant<App, 2>,
        kernel_tx: &'static mut [u8],
        deferred_caller: &'a DynamicDeferredCall,
    ) -> RadioDriver<'a> {
        RadioDriver {
            mac,
            neighbors: MapCell::new(Default::default()),
            num_neighbors: Cell::new(0),
            keys: MapCell::new(Default::default()),
            num_keys: Cell::new(0),
            apps: grant,
            current_app: OptionalCell::empty(),
            kernel_tx: TakeCell::new(kernel_tx),
            deferred_caller,
            saved_appid: OptionalCell::empty(),
            saved_result: OptionalCell::empty(),
            handle: OptionalCell::empty(),
        }
    }

    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
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
    /// `Ok()`. Otherwise, returns `Err(ErrorCode::INVAL)`.  Ensures
    /// that the `neighbors` list is compact by shifting forward any elements
    /// after the index.
    fn remove_neighbor(&self, index: usize) -> Result<(), ErrorCode> {
        let num_neighbors = self.num_neighbors.get();
        if index < num_neighbors {
            self.neighbors.map(|neighbors| {
                for i in index..(num_neighbors - 1) {
                    neighbors[i] = neighbors[i + 1];
                }
            });
            self.num_neighbors.set(num_neighbors - 1);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
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
    /// `Ok(())`. Otherwise, returns `Err(ErrorCode::INVAL)`.  Ensures
    /// that the `keys` list is compact by shifting forward any elements
    /// after the index.
    fn remove_key(&self, index: usize) -> Result<(), ErrorCode> {
        let num_keys = self.num_keys.get();
        if index < num_keys {
            self.keys.map(|keys| {
                for i in index..(num_keys - 1) {
                    keys[i] = keys[i + 1];
                }
            });
            self.num_keys.set(num_keys - 1);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
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
    fn do_with_app<F>(&self, appid: ProcessId, closure: F) -> Result<(), ErrorCode>
    where
        F: FnOnce(&mut App) -> Result<(), ErrorCode>,
    {
        self.apps
            .enter(appid, |app, _| closure(app))
            .unwrap_or_else(|err| err.into())
    }

    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `ProcessId`.
    fn get_next_tx_if_idle(&self) -> Option<ProcessId> {
        if self.current_app.is_some() {
            return None;
        }
        let mut pending_app = None;
        for app in self.apps.iter() {
            let appid = app.processid();
            app.enter(|app, _| {
                if app.pending_tx.is_some() {
                    pending_app = Some(appid);
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
    fn perform_tx_async(&self, appid: ProcessId) {
        let result = self.perform_tx_sync(appid);
        if result != Ok(()) {
            self.saved_appid.set(appid);
            self.saved_result.set(result);
            self.handle.map(|handle| self.deferred_caller.set(*handle));
        }
    }

    /// Performs `appid`'s pending transmission synchronously. The result is
    /// returned immediately to the app. Assumes that the driver is currently
    /// idle and the app has a pending transmission.
    #[inline]
    fn perform_tx_sync(&self, appid: ProcessId) -> Result<(), ErrorCode> {
        self.do_with_app(appid, |app| {
            let (dst_addr, security_needed) = match app.pending_tx.take() {
                Some(pending_tx) => pending_tx,
                None => {
                    return Ok(());
                }
            };
            let result = self.kernel_tx.take().map_or(Err(ErrorCode::NOMEM), |kbuf| {
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
                        return Err(ErrorCode::FAIL);
                    }
                };

                // Append the payload: there must be one
                let result = app
                    .app_write
                    .enter(|payload| frame.append_payload_process(payload))
                    .unwrap_or(Err(ErrorCode::INVAL));
                if result != Ok(()) {
                    return result;
                }

                // Finally, transmit the frame
                match self.mac.transmit(frame) {
                    Ok(()) => Ok(()),
                    Err((ecode, buf)) => {
                        self.kernel_tx.put(Some(buf));
                        Err(ecode)
                    }
                }
            });
            if result == Ok(()) {
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
    fn do_next_tx_sync(&self, new_appid: ProcessId) -> Result<(), ErrorCode> {
        self.get_next_tx_if_idle().map_or(Ok(()), |appid| {
            if appid == new_appid {
                self.perform_tx_sync(appid)
            } else {
                self.perform_tx_async(appid);
                Ok(())
            }
        })
    }
}

impl DynamicDeferredCallClient for RadioDriver<'_> {
    fn call(&self, _handle: DeferredCallHandle) {
        let _ = self
            .apps
            .enter(self.saved_appid.expect("missing appid"), |_app, upcalls| {
                upcalls.schedule_upcall(
                    1,
                    kernel::into_statuscode(self.saved_result.expect("missing result")),
                    0,
                    0,
                );
            });
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

impl Driver for RadioDriver<'_> {
    /// Setup buffers to read/write from.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Read buffer. Will contain the received frame.
    /// - `1`: Config buffer. Used to contain miscellaneous data associated with
    ///        some commands because the system call parameters / return codes are
    ///        not enough to convey the desired information.
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        match allow_num {
            0 | 1 => {
                let res = self.apps.enter(appid, |app, _| match allow_num {
                    0 => mem::swap(&mut app.app_read, &mut slice),
                    1 => mem::swap(&mut app.app_cfg, &mut slice),
                    _ => unreachable!(),
                });
                match res {
                    Ok(_) => Ok(slice),
                    Err(err) => Err((slice, err.into())),
                }
            }
            _ => Err((slice, ErrorCode::NOSUPPORT)),
        }
    }

    /// - `0`: Write buffer. Contains the frame payload to be transmitted.
    fn allow_readonly(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        match allow_num {
            0 => {
                let res = self.apps.enter(appid, |app, _| {
                    mem::swap(&mut app.app_write, &mut slice);
                });
                match res {
                    Ok(_) => Ok(slice),
                    Err(err) => Err((slice, err.into())),
                }
            }
            _ => Err((slice, ErrorCode::NOSUPPORT)),
        }
    }

    // Setup callbacks.
    //
    // ### `subscribe_num`
    //
    // - `0`: Setup callback for when frame is received.
    // - `1`: Setup callback for when frame is transmitted.

    /// IEEE 802.15.4 MAC device control.
    ///
    /// For some of the below commands, one 32-bit argument is not enough to
    /// contain the desired input parameters or output data. For those commands,
    /// the config slice `app_cfg` is used as a channel to shuffle information
    /// between kernel space and user space. The expected size of the slice
    /// varies by command, and acts essentially like a custom FFI. That is, the
    /// userspace library MUST `allow()` a buffer of the correct size, otherwise
    /// the call is INVAL. When used, the expected format is described below.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Return radio status. Ok(())/OFF = on/off.
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
    fn command(
        &self,
        command_number: usize,
        arg1: usize,
        _: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => {
                if self.mac.is_on() {
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::OFF)
                }
            }
            2 => {
                self.mac.set_address(arg1 as u16);
                CommandReturn::success()
            }
            3 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .enter(|cfg| {
                            if cfg.len() != 8 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }
                            let mut addr_long = [0u8; 8];
                            cfg.copy_to_slice(&mut addr_long);
                            self.mac.set_address_long(addr_long);
                            CommandReturn::success()
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            4 => {
                self.mac.set_pan(arg1 as u16);
                CommandReturn::success()
            }
            // XXX: Setting channel DEPRECATED by MAC layer channel control
            5 => CommandReturn::failure(ErrorCode::NOSUPPORT),
            // XXX: Setting tx power DEPRECATED by MAC layer tx power control
            6 => CommandReturn::failure(ErrorCode::NOSUPPORT),
            7 => {
                self.mac.config_commit();
                CommandReturn::success()
            }
            8 => {
                // Guarantee that address is positive by adding 1
                let addr = self.mac.get_address();
                CommandReturn::success_u32(addr as u32 + 1)
            }
            9 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .mut_enter(|cfg| {
                            if cfg.len() != 8 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }
                            cfg.copy_from_slice(&self.mac.get_address_long());
                            CommandReturn::success()
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            10 => {
                // Guarantee that the PAN is positive by adding 1
                let pan = self.mac.get_pan();
                CommandReturn::success_u32(pan as u32 + 1)
            }
            // XXX: Getting channel DEPRECATED by MAC layer channel control
            11 => CommandReturn::failure(ErrorCode::NOSUPPORT),
            // XXX: Getting tx power DEPRECATED by MAC layer tx power control
            12 => CommandReturn::failure(ErrorCode::NOSUPPORT),
            13 => {
                // Guarantee that it is positive by adding 1
                CommandReturn::success_u32(MAX_NEIGHBORS as u32 + 1)
            }
            14 => {
                // Guarantee that it is positive by adding 1
                CommandReturn::success_u32(self.num_neighbors.get() as u32 + 1)
            }
            15 => self
                .get_neighbor(arg1)
                .map_or(CommandReturn::failure(ErrorCode::INVAL), |neighbor| {
                    CommandReturn::success_u32(neighbor.short_addr as u32 + 1)
                }),
            16 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .mut_enter(|cfg| {
                            if cfg.len() != 8 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }
                            self.get_neighbor(arg1).map_or(
                                CommandReturn::failure(ErrorCode::INVAL),
                                |neighbor| {
                                    cfg.copy_from_slice(&neighbor.long_addr);
                                    CommandReturn::success()
                                },
                            )
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            17 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .enter(|cfg| {
                            if cfg.len() != 8 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }
                            let mut new_neighbor: DeviceDescriptor = DeviceDescriptor::default();
                            new_neighbor.short_addr = arg1 as u16;
                            cfg.copy_to_slice(&mut new_neighbor.long_addr);
                            self.add_neighbor(new_neighbor)
                                .map_or(CommandReturn::failure(ErrorCode::INVAL), |index| {
                                    CommandReturn::success_u32(index as u32 + 1)
                                })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            18 => match self.remove_neighbor(arg1) {
                Ok(_) => CommandReturn::success(),
                Err(e) => CommandReturn::failure(e),
            },
            19 => {
                // Guarantee that it is positive by adding 1
                CommandReturn::success_u32(MAX_KEYS as u32 + 1)
            }
            20 => {
                // Guarantee that it is positive by adding 1
                CommandReturn::success_u32(self.num_keys.get() as u32 + 1)
            }
            21 => self
                .get_key(arg1)
                .map_or(CommandReturn::failure(ErrorCode::INVAL), |key| {
                    CommandReturn::success_u32(key.level as u32 + 1)
                }),
            22 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .mut_enter(|cfg| {
                            if cfg.len() != 10 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }

                            let mut tmp_cfg: [u8; 10] = [0; 10];
                            let res = self
                                .get_key(arg1)
                                .and_then(|key| encode_key_id(&key.key_id, &mut tmp_cfg).done())
                                .map_or(CommandReturn::failure(ErrorCode::INVAL), |_| {
                                    CommandReturn::success()
                                });
                            cfg.copy_from_slice(&tmp_cfg);

                            res
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            23 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .mut_enter(|cfg| {
                            if cfg.len() != 16 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }
                            self.get_key(arg1).map_or(
                                CommandReturn::failure(ErrorCode::INVAL),
                                |key| {
                                    cfg.copy_from_slice(&key.key);
                                    CommandReturn::success()
                                },
                            )
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            24 => self
                .apps
                .enter(appid, |app, _| {
                    app.app_cfg
                        .mut_enter(|cfg| {
                            if cfg.len() != 27 {
                                return CommandReturn::failure(ErrorCode::SIZE);
                            }

                            // The cfg userspace buffer is exactly 27
                            // bytes long, copy it into a proper slice
                            // for decoding
                            let mut tmp_cfg: [u8; 27] = [0; 27];
                            cfg.copy_to_slice(&mut tmp_cfg);

                            KeyDescriptor::decode(&tmp_cfg)
                                .done()
                                .and_then(|(_, new_key)| self.add_key(new_key))
                                .map_or(CommandReturn::failure(ErrorCode::INVAL), |index| {
                                    CommandReturn::success_u32(index as u32 + 1)
                                })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            25 => self.remove_key(arg1).into(),
            26 => {
                self.apps
                    .enter(appid, |app, _| {
                        if app.pending_tx.is_some() {
                            // Cannot support more than one pending tx per process.
                            return Err(ErrorCode::BUSY);
                        }
                        let next_tx = app
                            .app_cfg
                            .enter(|cfg| {
                                if cfg.len() != 11 {
                                    return None;
                                }
                                let dst_addr = arg1 as u16;
                                let level = match SecurityLevel::from_scf(cfg[0].get()) {
                                    Some(level) => level,
                                    None => {
                                        return None;
                                    }
                                };
                                if level == SecurityLevel::None {
                                    Some((dst_addr, None))
                                } else {
                                    let mut tmp_key_id_buffer: [u8; 10] = [0; 10];
                                    cfg[1..].copy_to_slice(&mut tmp_key_id_buffer);
                                    let key_id = match decode_key_id(&tmp_key_id_buffer).done() {
                                        Some((_, key_id)) => key_id,
                                        None => {
                                            return None;
                                        }
                                    };
                                    Some((dst_addr, Some((level, key_id))))
                                }
                            })
                            .unwrap_or(None);
                        if next_tx.is_none() {
                            return Err(ErrorCode::INVAL);
                        }
                        app.pending_tx = next_tx;
                        Ok(())
                    })
                    .map_or_else(
                        |err| CommandReturn::failure(err.into()),
                        |setup_tx| match setup_tx {
                            Ok(_) => self.do_next_tx_sync(appid).into(),
                            Err(e) => CommandReturn::failure(e.into()),
                        },
                    )
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl device::TxClient for RadioDriver<'_> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: Result<(), ErrorCode>) {
        self.kernel_tx.replace(spi_buf);
        self.current_app.take().map(|appid| {
            let _ = self.apps.enter(appid, |_app, upcalls| {
                upcalls.schedule_upcall(1, kernel::into_statuscode(result), acked as usize, 0);
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
        self.apps.each(|_, app, upcalls| {
            let read_present = app
                .app_read
                .mut_enter(|rbuf| {
                    let len = min(rbuf.len(), data_offset + data_len);
                    // Copy the entire frame over to userland, preceded by two
                    // bytes: the data offset and the data length.
                    rbuf[..len].copy_from_slice(&buf[..len]);
                    rbuf[0].set(data_offset as u8);
                    rbuf[1].set(data_len as u8);
                    true
                })
                .unwrap_or(false);
            if read_present {
                // Encode useful parts of the header in 3 usizes
                let pans = encode_pans(&header.dst_pan, &header.src_pan);
                let dst_addr = encode_address(&header.dst_addr);
                let src_addr = encode_address(&header.src_addr);
                upcalls.schedule_upcall(0, pans, dst_addr, src_addr);
            }
        });
    }
}
