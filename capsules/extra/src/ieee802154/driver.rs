// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! IEEE 802.15.4 userspace interface for configuration and transmit/receive.
//!
//! Implements a userspace interface for sending and receiving IEEE 802.15.4
//! frames. Also provides a minimal list-based interface for managing keys and
//! known link neighbors, which is needed for 802.15.4 security.
//!
//! The driver functionality can be divided into three aspects: sending
//! packets, receiving packets, and managing the 15.4 state (i.e. keys, neighbors,
//! buffers, addressing, etc). The general design and procedure for sending and
//! receiving is discussed below.
//!
//! Sending - The driver supports two modes of sending: Raw and Parse. In Raw mode,
//! the userprocess fully forms the 15.4 frame and passes it to the driver. In Parse
//! mode, the userprocess provides the payload and relevant metadata. From this
//! the driver forms the 15.4 header and secures the payload. To send a packet,
//! the userprocess issues the respective send command syscall (corresponding to
//! raw or parse mode of sending). The 15.4 capsule will then schedule an upcall,
//! upon completion of the transmission, to notify the process.
//!
//! Receiving - The driver receives 15.4 frames and passes them to the userprocess.
//! To accomplish this, the userprocess must first `allow` a read/write ring buffer
//! to the kernel. The kernel will then fill this buffer with received frames and
//! schedule an upcall upon receipt of the first packet. When handling the upcall
//! the userprocess must first `unallow` the buffer as described in section 4.4 of
//! TRD104-syscalls. After unallowing the buffer, the userprocess must then immediately
//! clear all pending/scheduled receive upcalls. This is done by either unsubscribing
//! the receive upcall or subscribing a new receive upcall. Because the userprocess
//! provides the buffer, it is responsible for adhering to this procedure. Failure
//! to comply may result in dropped or malformed packets.
//!
//! The ring buffer provided by the userprocess must be of the form:
//!
//! ```text
//! | read index | write index | user_frame 0 | user_frame 1 | ... | user_frame n |
//! ```
//!
//! `user_frame` denotes the 15.4 frame in addition to the relevant 3 bytes of
//! metadata (offset to data payload, length of data payload, and the MIC len). The
//! capsule assumes that this is the form of the buffer. Errors or deviation in
//! the form of the provided buffer will likely result in incomplete or dropped packets.
//!
//! Because the scheduled receive upcall must be handled by the userprocess, there is
//! no guarantee as to when this will occur and if additional packets will be received
//! prior to the upcall being handled. Without a ring buffer (or some equivalent data
//! structure), the original packet will be lost. The ring buffer allows for the upcall
//! to be scheduled and for all received packets to be passed to the process. The ring
//! buffer is designed to overwrite old packets if the buffer becomes full. If the
//! userprocess notices a high number of "dropped" packets, this may be the cause. The
//! userproceess can mitigate this issue by increasing the size of the ring buffer
//! provided to the capsule.

use crate::ieee802154::{device, framer};
use crate::net::ieee802154::{Header, KeyId, MacAddress, SecurityLevel};
use crate::net::stream::{decode_bytes, decode_u8, encode_bytes, encode_u8, SResult};

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::radio;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

const MAX_NEIGHBORS: usize = 4;
const MAX_KEYS: usize = 4;

const USER_FRAME_METADATA_SIZE: usize = 3; // 3B metadata (offset, len, mic_len)
const USER_FRAME_MAX_SIZE: usize = USER_FRAME_METADATA_SIZE + radio::MAX_FRAME_SIZE; // 3B metadata + 127B max payload

/// IDs for subscribed upcalls.
mod upcall {
    /// Frame is received
    pub const FRAME_RECEIVED: usize = 0;
    /// Frame is transmitted
    pub const FRAME_TRANSMITTED: usize = 1;
    /// Number of upcalls.
    pub const COUNT: u8 = 2;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// Write buffer. Contains the frame payload to be transmitted.
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Read buffer. Will contain the received frame.
    pub const READ: usize = 0;
    /// Config buffer.
    ///
    /// Used to contain miscellaneous data associated with some commands because
    /// the system call parameters / return codes are not enough to convey the
    /// desired information.
    pub const CFG: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Ieee802154 as usize;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
struct DeviceDescriptor {
    short_addr: u16,
    long_addr: [u8; 8],
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
    pending_tx: Option<(u16, Option<(SecurityLevel, KeyId)>)>,
}

pub struct RadioDriver<'a, M: device::MacDevice<'a>> {
    /// Underlying MAC device, possibly multiplexed
    mac: &'a M,

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
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    /// ID of app whose transmission request is being processed.
    current_app: OptionalCell<ProcessId>,

    /// Buffer that stores the IEEE 802.15.4 frame to be transmitted.
    kernel_tx: TakeCell<'static, [u8]>,

    /// Used to ensure callbacks are delivered during upcalls
    deferred_call: DeferredCall,

    /// Used to deliver callbacks to the correct app during deferred calls
    saved_processid: OptionalCell<ProcessId>,

    /// Used to save result for passing a callback from a deferred call.
    saved_result: OptionalCell<Result<(), ErrorCode>>,

    /// Used to allow Thread to specify a key procedure for 15.4 to use for link layer encryption
    backup_key_procedure: OptionalCell<&'a dyn framer::KeyProcedure>,

    /// Used to allow Thread to specify the 15.4 device procedure as used in nonce generation
    backup_device_procedure: OptionalCell<&'a dyn framer::DeviceProcedure>,
}

impl<'a, M: device::MacDevice<'a>> RadioDriver<'a, M> {
    pub fn new(
        mac: &'a M,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        kernel_tx: &'static mut [u8],
    ) -> Self {
        Self {
            mac,
            neighbors: MapCell::new(Default::default()),
            num_neighbors: Cell::new(0),
            keys: MapCell::new(Default::default()),
            num_keys: Cell::new(0),
            apps: grant,
            current_app: OptionalCell::empty(),
            kernel_tx: TakeCell::new(kernel_tx),
            deferred_call: DeferredCall::new(),
            saved_processid: OptionalCell::empty(),
            saved_result: OptionalCell::empty(),
            backup_key_procedure: OptionalCell::empty(),
            backup_device_procedure: OptionalCell::empty(),
        }
    }

    pub fn set_key_procedure(&self, key_procedure: &'a dyn framer::KeyProcedure) {
        self.backup_key_procedure.set(key_procedure);
    }

    pub fn set_device_procedure(&self, device_procedure: &'a dyn framer::DeviceProcedure) {
        self.backup_device_procedure.set(device_procedure);
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

    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `ProcessId`.
    fn get_next_tx_if_idle(&self) -> Option<ProcessId> {
        if self.current_app.is_some() {
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
            self.saved_processid.set(processid);
            self.saved_result.set(result);
            self.deferred_call.set();
        }
    }

    /// Performs `processid`'s pending transmission synchronously. The result is
    /// returned immediately to the app. Assumes that the driver is currently
    /// idle and the app has a pending transmission.
    #[inline]
    fn perform_tx_sync(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, kerel_data| {
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
                let result = kerel_data
                    .get_readonly_processbuffer(ro_allow::WRITE)
                    .and_then(|write| write.enter(|payload| frame.append_payload_process(payload)))
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
                self.current_app.set(processid);
            }
            result
        })?
    }

    /// Schedule the next transmission if there is one pending. Performs the
    /// transmission asynchronously, returning any errors via callbacks.
    #[inline]
    fn do_next_tx_async(&self) {
        self.get_next_tx_if_idle()
            .map(|processid| self.perform_tx_async(processid));
    }

    /// Schedule the next transmission if there is one pending. If the next
    /// transmission happens to be the one that was just queued, then the
    /// transmission is synchronous. Hence, errors must be returned immediately.
    /// On the other hand, if it is some other app, then return any errors via
    /// callbacks.
    #[inline]
    fn do_next_tx_sync(&self, new_processid: ProcessId) -> Result<(), ErrorCode> {
        self.get_next_tx_if_idle().map_or(Ok(()), |processid| {
            if processid == new_processid {
                self.perform_tx_sync(processid)
            } else {
                self.perform_tx_async(processid);
                Ok(())
            }
        })
    }
}

impl<'a, M: device::MacDevice<'a>> DeferredCallClient for RadioDriver<'a, M> {
    fn handle_deferred_call(&self) {
        let _ = self
            .apps
            .enter(self.saved_processid.unwrap_or_panic(), |_app, upcalls| {
                // Unwrap fail = missing processid
                upcalls
                    .schedule_upcall(
                        upcall::FRAME_TRANSMITTED,
                        (
                            kernel::errorcode::into_statuscode(
                                self.saved_result.unwrap_or_panic(), // Unwrap fail = missing result
                            ),
                            0,
                            0,
                        ),
                    )
                    .ok();
            });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a, M: device::MacDevice<'a>> framer::DeviceProcedure for RadioDriver<'a, M> {
    /// Gets the long address corresponding to the neighbor that matches the given
    /// MAC address. If no such neighbor exists, returns `None`.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<[u8; 8]> {
        self.neighbors
            .and_then(|neighbors| {
                neighbors[..self.num_neighbors.get()]
                    .iter()
                    .find(|neighbor| match addr {
                        MacAddress::Short(addr) => addr == neighbor.short_addr,
                        MacAddress::Long(addr) => addr == neighbor.long_addr,
                    })
                    .map(|neighbor| neighbor.long_addr)
            })
            .map_or_else(
                // This serves the same purpose as the KeyProcedure lookup (see comment).
                // This is kept as a remnant of 15.4, but should potentially be removed moving forward
                // as Thread does not have a use to add a Device procedure.
                || {
                    self.backup_device_procedure
                        .and_then(|procedure| procedure.lookup_addr_long(addr))
                },
                |res| Some(res),
            )
    }
}

impl<'a, M: device::MacDevice<'a>> framer::KeyProcedure for RadioDriver<'a, M> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<[u8; 16]> {
        self.keys
            .and_then(|keys| {
                keys[..self.num_keys.get()]
                    .iter()
                    .find(|key| key.level == level && key.key_id == key_id)
                    .map(|key| key.key)
            })
            .map_or_else(
                // Thread needs to add a MAC key to the 15.4 network keys so that the 15.4 framer
                // can decrypt incoming Thread 15.4 frames. The backup_device_procedure was added
                // so that if the lookup procedure failed to find a key here, it would check a
                // "backup" procedure (Thread in this case). This is somewhat clunky and removing
                // the network keys being stored in the 15.4 driver is a longer term TODO.
                || {
                    self.backup_key_procedure.and_then(|procedure| {
                        // TODO: security_level / keyID are hardcoded for now
                        procedure.lookup_key(SecurityLevel::EncMic32, KeyId::Index(2))
                    })
                },
                |res| Some(res),
            )
    }
}

impl<'a, M: device::MacDevice<'a>> SyscallDriver for RadioDriver<'a, M> {
    /// IEEE 802.15.4 MAC device control.
    ///
    /// For some of the below commands, one 32-bit argument is not enough to
    /// contain the desired input parameters or output data. For those commands,
    /// the config slice `app_cfg` (RW allow num 1) is used as a channel to shuffle information
    /// between kernel space and user space. The expected size of the slice
    /// varies by command, and acts essentially like a custom FFI. That is, the
    /// userspace library MUST `allow()` a buffer of the correct size, otherwise
    /// the call is INVAL. When used, the expected format is described below.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
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
    /// - `24`: Add a new key with the given description.
    ///        app_cfg (in): 1 byte: the security level +
    ///                      1 byte: the key ID mode +
    ///                      9 bytes: the key ID (might not use all bytes) +
    ///                      16 bytes: the key.
    /// - `25`: Remove the key at an index.
    /// - `26`: Transmit a frame (parse required). Take the provided payload and
    ///        parameters to encrypt, form headers, and transmit the frame.
    /// - `28`: Set long address.
    /// - `29`: Get the long MAC address.
    fn command(
        &self,
        command_number: usize,
        arg1: usize,
        arg2: usize,
        processid: ProcessId,
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
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.enter(|cfg| {
                                if cfg.len() != 8 {
                                    return CommandReturn::failure(ErrorCode::SIZE);
                                }
                                let mut addr_long = [0u8; 8];
                                cfg.copy_to_slice(&mut addr_long);
                                self.mac.set_address_long(addr_long);
                                CommandReturn::success()
                            })
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
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.mut_enter(|cfg| {
                                if cfg.len() != 8 {
                                    return CommandReturn::failure(ErrorCode::SIZE);
                                }
                                cfg.copy_from_slice(&self.mac.get_address_long());
                                CommandReturn::success()
                            })
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
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.mut_enter(|cfg| {
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
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            17 => self
                .apps
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.enter(|cfg| {
                                if cfg.len() != 8 {
                                    return CommandReturn::failure(ErrorCode::SIZE);
                                }
                                let mut new_neighbor: DeviceDescriptor =
                                    DeviceDescriptor::default();
                                new_neighbor.short_addr = arg1 as u16;
                                cfg.copy_to_slice(&mut new_neighbor.long_addr);
                                self.add_neighbor(new_neighbor)
                                    .map_or(CommandReturn::failure(ErrorCode::INVAL), |index| {
                                        CommandReturn::success_u32(index as u32 + 1)
                                    })
                            })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            18 => match self.remove_neighbor(arg1) {
                Ok(()) => CommandReturn::success(),
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
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.mut_enter(|cfg| {
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
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            23 => self
                .apps
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.mut_enter(|cfg| {
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
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),
            24 => self
                .apps
                .enter(processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::CFG)
                        .and_then(|cfg| {
                            cfg.mut_enter(|cfg| {
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
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            25 => self.remove_key(arg1).into(),
            26 => {
                self.apps
                    .enter(processid, |app, kernel_data| {
                        if app.pending_tx.is_some() {
                            // Cannot support more than one pending tx per process.
                            return Err(ErrorCode::BUSY);
                        }
                        let next_tx = kernel_data
                            .get_readwrite_processbuffer(rw_allow::CFG)
                            .and_then(|cfg| {
                                cfg.enter(|cfg| {
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
                                        let key_id = match decode_key_id(&tmp_key_id_buffer).done()
                                        {
                                            Some((_, key_id)) => key_id,
                                            None => {
                                                return None;
                                            }
                                        };
                                        Some((dst_addr, Some((level, key_id))))
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
                    .map_or_else(
                        |err| CommandReturn::failure(err.into()),
                        |setup_tx| match setup_tx {
                            Ok(()) => self.do_next_tx_sync(processid).into(),
                            Err(e) => CommandReturn::failure(e),
                        },
                    )
            }
            28 => {
                let addr_upper: u64 = arg2 as u64;
                let addr_lower: u64 = arg1 as u64;
                let addr = addr_upper << 32 | addr_lower;
                self.mac.set_address_long(addr.to_be_bytes());
                CommandReturn::success()
            }
            29 => {
                let addr = u64::from_be_bytes(self.mac.get_address_long());
                CommandReturn::success_u64(addr)
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, M: device::MacDevice<'a>> device::TxClient for RadioDriver<'a, M> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: Result<(), ErrorCode>) {
        self.kernel_tx.replace(spi_buf);
        self.current_app.take().map(|processid| {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                upcalls
                    .schedule_upcall(
                        upcall::FRAME_TRANSMITTED,
                        (
                            kernel::errorcode::into_statuscode(result),
                            acked as usize,
                            0,
                        ),
                    )
                    .ok();
            });
        });
        self.do_next_tx_async();
    }
}

impl<'a, M: device::MacDevice<'a>> device::RxClient for RadioDriver<'a, M> {
    fn receive<'b>(
        &self,
        buf: &'b [u8],
        header: Header<'b>,
        lqi: u8,
        data_offset: usize,
        data_len: usize,
    ) {
        self.apps.each(|_, _, kernel_data| {
            let read_present = kernel_data
                .get_readwrite_processbuffer(rw_allow::READ)
                .and_then(|read| {
                    read.mut_enter(|rbuf| {
                        ///////////////////////////////////////////////////////////////////////////////////////////
                        // NOTE: context for the ring buffer and assumptions regarding the ring buffer
                        // format and usage can be found in the detailed comment at the top of this file.
                        //      Ring buffer format:
                        //          | read index | write index | user_frame 0 | user_frame 1 | ... | user_frame n |
                        //      user_frame format:
                        //          | header_len | payload_len | mic_len | 15.4 frame |
                        ///////////////////////////////////////////////////////////////////////////////////////////

                        // 2 bytes for the readwrite buffer metadata (read / write index)
                        const RING_BUF_METADATA_SIZE: usize = 2;

                        // Confirm the availability of the buffer. A buffer of len 0 is indicative
                        // of the userprocess not allocating a readwrite buffer. We must also
                        // confirm that the userprocess correctly formatted the buffer to be of length
                        // 2 + n * USER_FRAME_MAX_SIZE, where n is the number of user frames that the
                        // buffer can store. We combine checking the buffer's non-zero length and the
                        // case of the buffer being shorter than the `RING_BUF_METADATA_SIZE` as an
                        // invalid buffer (e.g. of length 1) may otherwise errantly pass the second
                        // conditional check (due to unsigned integer arithmetic).
                        if rbuf.len() <= RING_BUF_METADATA_SIZE
                            || (rbuf.len() - RING_BUF_METADATA_SIZE) % USER_FRAME_MAX_SIZE != 0
                        {
                            // kernel::debug!("[15.4 Driver] Error - improperly formatted readwrite buffer provided");
                            return false;
                        }

                        let mic_len = header.security.map_or(0, |sec| sec.level.mic_len());
                        let frame_len = data_offset + data_len + mic_len;

                        let mut read_index = rbuf[0].get() as usize;
                        let mut write_index = rbuf[1].get() as usize;

                        let max_pending_rx =
                            (rbuf.len() - RING_BUF_METADATA_SIZE) / USER_FRAME_MAX_SIZE;

                        // confirm user modifiable metadata is valid (i.e. within bounds of the provided buffer)
                        if read_index >= max_pending_rx || write_index >= max_pending_rx {
                            // kernel::debug!("[15.4 driver] Invalid read or write index");
                            return false;
                        }

                        let offset = RING_BUF_METADATA_SIZE + (write_index * USER_FRAME_MAX_SIZE);

                        // Copy the entire frame over to userland, preceded by three metadata bytes:
                        // the header length, the data length, and the MIC length.
                        rbuf[(offset + USER_FRAME_METADATA_SIZE)
                            ..(offset + frame_len + USER_FRAME_METADATA_SIZE)]
                            .copy_from_slice(&buf[..frame_len]);

                        rbuf[offset].set(data_offset as u8);
                        rbuf[offset + 1].set(data_len as u8);
                        rbuf[offset + 2].set(mic_len as u8);

                        // Prepare the ring buffer for the next write. The current design favors newness;
                        // newly received packets will begin to overwrite the oldest data in the event
                        // of the buffer becoming full. The read index must always point to the "oldest"
                        // data. If we have overwritten the oldest data, the next oldest data is now at
                        // the read index + 1. We must update the read index to reflect this.
                        write_index = (write_index + 1) % max_pending_rx;
                        if write_index == read_index {
                            read_index = (read_index + 1) % max_pending_rx;
                            rbuf[0].set(read_index as u8);
                            // kernel::debug!("[15.4 driver] Provided RX buffer is full");
                        }

                        // update write index metadata (we do not modify the read index
                        // in the recv functionality so we do not need to update this metadata)
                        rbuf[1].set(write_index as u8);
                        true
                    })
                })
                .unwrap_or(false);
            if read_present {
                // Place lqi as argument to be included in upcall.
                kernel_data
                    .schedule_upcall(upcall::FRAME_RECEIVED, (lqi as usize, 0, 0))
                    .ok();
            }
        });
    }
}
