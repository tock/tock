// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Mechanism for managing storage read & write permissions.

use core::cmp;

/// List of storage permissions for a storage user.
///
/// These identifiers signify what permissions a storage user has. The storage
/// mechanism defines how the identifiers are assigned and how they relate to
/// stored objects.
///
/// For simplicity, a we store to eight read and eight write permissions. The
/// first `count` `u32` values in `permissions` are valid.
///
/// Mar, 2022: This interface is considered experimental and for initial
/// prototyping. As we learn more about how these permissions are set and used
/// we may want to revamp this interface.
#[derive(Clone, Copy)]
pub struct StoragePermissions {
    // How many entries in the `read_permissions` slice are valid, starting at
    // index 0.
    read_count: usize,
    // Up to eight 32 bit identifiers of storage items the process has read
    // access to.
    read_permissions: [u32; 8],
    // How many entries in the `write_permissions` slice are valid, starting at
    // index 0.
    write_count: usize,
    // Up to eight 32 bit identifiers of storage items the process has write
    // (update) access to.
    write_permissions: [u32; 8],
    // The identifier for this storage user when creating new objects. If `None`
    // there is no `write_id` for these permissions.
    write_id: Option<u32>,
}

impl StoragePermissions {
    pub(crate) fn new(
        read_count: usize,
        read_permissions: [u32; 8],
        write_count: usize,
        write_permissions: [u32; 8],
        write_id: Option<u32>,
    ) -> Self {
        let read_count_capped = cmp::min(read_count, 8);
        let write_count_capped = cmp::min(write_count, 8);
        StoragePermissions {
            read_count: read_count_capped,
            read_permissions,
            write_count: write_count_capped,
            write_permissions,
            write_id,
        }
    }

    /// Check if this permission object grants read access to the specified
    /// `storage_id`. Returns `true` if access is permitted, `false` otherwise.
    pub fn check_read_permission(&self, storage_id: u32) -> bool {
        self.read_permissions
            .get(0..self.read_count)
            .unwrap_or(&[])
            .contains(&storage_id)
    }

    /// Check if this permission object grants write access to the specified
    /// `storage_id`. Returns `true` if access is permitted, `false` otherwise.
    pub fn check_write_permission(&self, storage_id: u32) -> bool {
        self.write_permissions
            .get(0..self.write_count)
            .unwrap_or(&[])
            .contains(&storage_id)
    }

    /// Get the `write_id` for saving items to the storage.
    pub fn get_write_id(&self) -> Option<u32> {
        self.write_id
    }
}
