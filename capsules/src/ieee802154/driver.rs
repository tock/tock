//! Implements a userspace interface for sending and receiving IEEE 802.15.4
//! frames. Also provides a minimal list-based interface for managing keys and
//! known link neighbors, which is needed for 802.15.4 security.

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, Driver, Callback, AppSlice, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::ReturnCode;

use net::ieee802154::{MacAddress, Header, SecurityLevel, KeyId};
use ieee802154::mac;

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
}

impl<'a> RadioDriver<'a> {
    pub fn new(mac: &'a mac::Mac<'a>) -> RadioDriver<'a> {
        RadioDriver {
            mac: mac,
            neighbors: MapCell::new(Default::default()),
            num_neighbors: Cell::new(0),
            keys: MapCell::new(Default::default()),
            num_keys: Cell::new(0),
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
            let position = neighbors[..num_neighbors].iter()
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
            let position = keys[..num_keys].iter()
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
}

impl<'a> mac::DeviceProcedure for RadioDriver<'a> {
    /// Gets the long address corresponding to the neighbor that matches the given
    /// MAC address. If no such neighbor exists, returns `None`.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<([u8; 8])> {
        self.neighbors.and_then(|neighbors| {
            neighbors[..self.num_neighbors.get()].iter()
                .find(|neighbor| {
                    match addr {
                        MacAddress::Short(addr) => addr == neighbor.short_addr,
                        MacAddress::Long(addr) => addr == neighbor.long_addr,
                    }
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
            keys[..self.num_keys.get()].iter()
                .find(|key| key.level == level && key.key_id == key_id)
                .map(|key| key.key)
        })
    }
}

impl<'a> Driver for RadioDriver<'a> {
    fn allow(&self, _appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            _ => ReturnCode::ENOSUPPORT
        }
    }

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> ReturnCode {
        match cmd_num {
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
