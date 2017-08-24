//! Implements a userspace interface for sending and receiving IEEE 802.15.4
//! frames. Also provides a minimal list-based interface for managing keys and
//! known link neighbors, which is needed for 802.15.4 security.

use core::cell::Cell;
use ieee802154::mac;
use kernel::{AppId, Driver, Callback, AppSlice, Shared, Container, ReturnCode};
use kernel::common::take_cell::{MapCell, TakeCell};

use net::ieee802154::{MacAddress, Header, SecurityLevel, KeyId};

const MAX_NEIGHBORS: usize = 4;
const MAX_KEYS: usize = 4;

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

pub struct App {
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    app_cfg: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<(usize, u16)>,
    tx_security: Option<(SecurityLevel, KeyId)>,
}

impl Default for App {
    fn default() -> Self {
        App {
            tx_callback: None,
            rx_callback: None,
            app_read: None,
            app_write: None,
            app_cfg: None,
            pending_tx: None,
            tx_security: None,
        }
    }
}

pub struct RadioDriver<'a> {
    /// Underlying MAC device, possibly multiplexed
    mac: &'a mac::Mac<'a>,

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

    /// Container of apps that use this radio driver.
    apps: Container<App>,
    /// ID of app whose transmission request is being processed.
    current_app: Cell<Option<AppId>>,

    /// Buffer that stores the IEEE 802.15.4 frame to be transmitted.
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a> RadioDriver<'a> {
    pub fn new(mac: &'a mac::Mac<'a>,
               container: Container<App>,
               kernel_tx: &'static mut [u8])
               -> RadioDriver<'a> {
        RadioDriver {
            mac: mac,
            neighbors: MapCell::new(Default::default()),
            num_neighbors: Cell::new(0),
            keys: MapCell::new(Default::default()),
            num_keys: Cell::new(0),
            apps: container,
            current_app: Cell::new(None),
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
            self.neighbors.map(|neighbors| for i in index..(num_neighbors - 1) {
                neighbors[i] = neighbors[i + 1];
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
            let position = keys[..num_keys]
                .iter()
                .position(|key| *key == new_key);
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
            self.keys.map(|keys| for i in index..(num_keys - 1) {
                keys[i] = keys[i + 1];
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

    #[inline]
    fn do_with_app<F>(&self, appid: AppId, closure: F) -> ReturnCode
        where F: FnOnce(&mut App) -> ReturnCode
    {
        self.apps
            .enter(appid, |app, _| closure(app))
            .unwrap_or_else(|err| err.into())
    }

    #[inline]
    fn do_with_cfg<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
        where F: FnOnce(&[u8]) -> ReturnCode
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

    #[inline]
    fn do_with_cfg_mut<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
        where F: FnOnce(&mut [u8]) -> ReturnCode
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
}

impl<'a> mac::DeviceProcedure for RadioDriver<'a> {
    /// Gets the long address corresponding to the neighbor that matches the given
    /// MAC address. If no such neighbor exists, returns `None`.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<([u8; 8])> {
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

impl<'a> mac::KeyProcedure for RadioDriver<'a> {
    /// Gets the key corresponding to the key that matches the given security
    /// level `level` and key ID `key_id`. If no such key matches, returns
    /// `None`.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<([u8; 16])> {
        self.keys.and_then(|keys| {
            keys[..self.num_keys.get()]
                .iter()
                .find(|key| key.level == level && key.key_id == key_id)
                .map(|key| key.key)
        })
    }
}

impl<'a> Driver for RadioDriver<'a> {
    /// Setup buffers to read/write from.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Read buffer. Will contain the received frame.
    /// - `1`: Write buffer. Contains the frame payload to be transmitted.
    /// - `2`: Config buffer. Used to contain miscellaneous data associated with
    ///        some commands because the system call parameters / return codes are
    ///        not enough to convey the desired information.
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 | 1 | 2 => {
                self.do_with_app(appid, |app| {
                    match allow_num {
                        0 => app.app_read = Some(slice),
                        1 => app.app_write = Some(slice),
                        2 => app.app_cfg = Some(slice),
                        _ => {}
                    }
                    ReturnCode::SUCCESS
                })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
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
    /// - `0`: Return radio status. SUCCESS/EOFF = on/off.
    /// - `1`: Set short MAC address.
    /// - `2`: Set long MAC address.
    ///        app_cfg (in): 8 bytes: the long MAC address.
    /// - `3`: Set PAN ID.
    /// - `4`: Set channel.
    /// - `5`: Set transmission power.
    /// - `6`: Commit any configuration changes.
    /// - `7`: Get the short MAC address.
    /// - `8`: Get the long MAC address.
    ///        app_cfg (out): 8 bytes: the long MAC address.
    /// - `9`: Get the PAN ID.
    /// - `10`: Get the channel.
    /// - `11`: Get the transmission power.
    fn command(&self, command_num: usize, arg1: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => {
                if self.mac.is_on() {
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EOFF
                }
            }
            1 => {
                self.mac.set_address(arg1 as u16);
                ReturnCode::SUCCESS
            }
            2 => {
                self.do_with_cfg(appid, 8, |cfg| {
                    let mut addr_long = [0u8; 8];
                    addr_long.copy_from_slice(cfg.as_ref());
                    self.mac.set_address_long(addr_long);
                    ReturnCode::SUCCESS
                })
            }
            3 => {
                self.mac.set_pan(arg1 as u16);
                ReturnCode::SUCCESS
            }
            4 => self.mac.set_channel(arg1 as u8),
            5 => {
                // Userspace casts the i8 to a u8 before casting to u32, so this works.
                self.mac.set_tx_power(arg1 as i8);
                ReturnCode::SUCCESS
            }
            6 => {
                self.mac.config_commit();
                ReturnCode::SUCCESS
            }
            7 => {
                // Guarantee that address is positive by adding 1
                let addr = self.mac.get_address();
                ReturnCode::SuccessWithValue { value: (addr as usize) + 1 }
            }
            8 => {
                self.do_with_cfg_mut(appid, 8, |cfg| {
                    cfg.copy_from_slice(&self.mac.get_address_long());
                    ReturnCode::SUCCESS
                })
            }
            9 => {
                // Guarantee that the PAN is positive by adding 1
                let pan = self.mac.get_pan();
                ReturnCode::SuccessWithValue { value: (pan as usize) + 1 }
            }
            10 => {
                // Guarantee that the PAN is positive by adding 1
                let channel = self.mac.get_channel();
                ReturnCode::SuccessWithValue { value: (channel as usize) + 1 }
            }
            11 => {
                // Cast the power to unsigned, then ensure it is positive by
                // adding 1
                let power = self.mac.get_tx_power() as u8;
                ReturnCode::SuccessWithValue { value: (power as usize) + 1 }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a> mac::TxClient for RadioDriver<'a> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        unimplemented!();
    }
}

impl<'a> mac::RxClient for RadioDriver<'a> {
    fn receive<'b>(&self, buf: &'b [u8], header: Header<'b>, data_offset: usize, data_len: usize) {
        unimplemented!();
    }
}
