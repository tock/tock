// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Mechanism for managing storage read & write permissions.
//!
//! These permissions are intended for userspace applications so the kernel can
//! restrict which stored elements the apps have access to.

use core::cmp;
use core::num::NonZeroU32;

use crate::capabilities;

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
    /// How many entries in the `read_permissions` slice are valid, starting at
    /// index 0.
    read_count: usize,
    /// Up to eight 32 bit identifiers of storage items the process has read
    /// access to.
    read_permissions: [u32; 8],
    /// How many entries in the `modify_permissions` slice are valid, starting
    /// at index 0.
    modify_count: usize,
    /// Up to eight 32 bit identifiers of storage items the process has modify
    /// (update) access to.
    modify_permissions: [u32; 8],
    /// The identifier for this storage user when creating new objects. If
    /// `None` there is no `write_id` for these permissions.
    write_id: Option<NonZeroU32>,
    /// If `kerneluser` is true, this permission grants access to all objects
    /// stored stored with `write_id` 0. New items created with `kerneluser ==
    /// true` will use the specified ID if `write_id.is_some()`, otherwise new
    /// items will be created with the reserved ID (i.e., 0).
    kerneluser: bool,
}

impl StoragePermissions {
    pub(crate) fn new(
        read_count: usize,
        read_permissions: [u32; 8],
        modify_count: usize,
        modify_permissions: [u32; 8],
        write_id: Option<NonZeroU32>,
    ) -> Self {
        let read_count_capped = cmp::min(read_count, 8);
        let modify_count_capped = cmp::min(modify_count, 8);
        StoragePermissions {
            read_count: read_count_capped,
            read_permissions,
            modify_count: modify_count_capped,
            modify_permissions,
            write_id,
            kerneluser: false,
        }
    }

    /// Create superuser permissions suitable for the kernel. This allows the
    /// kernel to read/update any stored item, and allows the kernel to write
    /// items that will not be accessible to any clients without superuser
    /// permissions.
    pub fn new_kernel_permissions(_cap: &dyn capabilities::KerneluserStorageCapability) -> Self {
        let read_permissions: [u32; 8] = [0; 8];
        let modify_permissions: [u32; 8] = [0; 8];
        StoragePermissions {
            read_count: 0,
            read_permissions,
            modify_count: 0,
            modify_permissions,
            write_id: None,
            kerneluser: true,
        }
    }

    /// Check if this permission object grants read access to the specified
    /// `storage_id`. Returns `true` if access is permitted, `false` otherwise.
    pub fn check_read_permission(&self, storage_id: u32) -> bool {
        if storage_id == 0 {
            // Only kerneluser can read ID 0.
            self.kerneluser
        } else {
            // Otherwise check if given storage_id is in read permissions
            // array.
            self.read_permissions
                .get(0..self.read_count)
                .unwrap_or(&[])
                .contains(&storage_id)
        }
    }

    /// Check if this permission object grants modify access to the specified
    /// `storage_id`. Returns `true` if access is permitted, `false` otherwise.
    pub fn check_write_permission(&self, storage_id: u32) -> bool {
        if storage_id == 0 {
            // Only kerneluser can access ID 0.
            self.kerneluser
        } else {
            // Otherwise check if given storage_id is in read permissions
            // array.
            self.modify_permissions
                .get(0..self.modify_count)
                .unwrap_or(&[])
                .contains(&storage_id)
        }
    }

    /// Get the `write_id` for saving items to the storage.
    pub fn get_write_id(&self) -> Option<u32> {
        if self.kerneluser {
            // If kerneluser, write_id is 0 unless specifically set.
            Some(self.write_id.map_or(0, |wid| wid.get()))
        } else {
            self.write_id.map(|wid| wid.get())
        }
    }
}
