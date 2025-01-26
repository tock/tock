// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Mechanism for managing storage read & write permissions.
//!
//! These permissions are intended for userspace applications so the kernel can
//! restrict which stored elements the apps have access to.

use crate::capabilities::ApplicationStorageCapability;
use crate::capabilities::KerneluserStorageCapability;

/// Permissions for accessing persistent storage.
///
/// This is a general type capable of representing permissions in different
/// ways. Users of storage permissions do not need to understand the different
/// ways permissions are stored internally. Instead, layers that need to enforce
/// permissions only use the following API:
///
/// ```rust,ignore
/// fn StoragePermissions::check_read_permission(&self, stored_id: u32) -> bool;
/// fn StoragePermissions::check_modify_permission(&self, stored_id: u32) -> bool;
/// fn StoragePermissions::get_write_id(&self) -> Option<u32>;
/// ```
#[derive(Clone, Copy)]
pub struct StoragePermissions(StoragePermissionsPrivate);

/// Inner enum type for types of permissions.
///
/// Private so permissions can only be created with capability-restricted
/// constructors.
#[derive(Clone, Copy)]
enum StoragePermissionsPrivate {
    /// This permission grants an application full access to its own stored
    /// state. The application may write state, and read and modify anything it
    /// has written.
    ///
    /// The `NonZeroU32` is the `ShortId::Fixed` of the application.
    SelfOnly(core::num::NonZeroU32),

    /// This permission supports setting whether an application can write and
    /// supports setting up to eight storage identifiers the application can
    /// read and eight storage identifiers the application can modify. This
    /// permission also includes a flag allowing an application to read and
    /// modify its own state.
    FixedSize(FixedSizePermissions),

    /// This permission supports setting whether an application can write and
    /// supports storing references to static buffers that contain an arbitrary
    /// list of storage identifiers the application can read and modify. This
    /// permission also includes a flag allowing an application to read and
    /// modify its own state.
    Listed(ListedPermissions),

    /// This permission is designed for only the kernel use, and allows the
    /// kernel to store and read/modify its own state. Note, this permission
    /// does not give the kernel access to application state.
    Kernel,

    /// This permission grants an application no access to any persistent
    /// storage.
    Null,
}

/// `StoragePermissions` with a fixed size number of read and modify
/// permissions.
///
/// For simplicity, a we store to eight read and eight write permissions. The
/// first `X_count` `u32` values in `X_permissions` are valid.
#[derive(Clone, Copy)]
pub struct FixedSizePermissions {
    /// The `ShortId::Fixed` of the application these permissions belong to.
    app_id: core::num::NonZeroU32,
    /// Whether this permission grants write access.
    write_permission: bool,
    /// If true, these permissions grant read and modify access to any stored
    /// state where this AppId matches the storage identifier.
    read_modify_self: bool,
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
}

/// `StoragePermissions` with arbitrary static arrays holding read and modify
/// permissions.
#[derive(Clone, Copy)]
pub struct ListedPermissions {
    /// The `ShortId::Fixed` of the application these permissions belong to.
    app_id: core::num::NonZeroU32,
    /// Whether this permission grants write access.
    write_permission: bool,
    /// If true, these permissions grant read and modify access to any stored
    /// state where this AppId matches the storage identifier.
    read_modify_self: bool,
    /// The 32 bit identifiers of storage items the process can read.
    read_permissions: &'static [u32],
    /// The 32 bit identifiers of storage items the process can modify
    modify_permissions: &'static [u32],
}

impl StoragePermissions {
    pub fn new_self_only(
        short_id_fixed: core::num::NonZeroU32,
        _cap: &dyn ApplicationStorageCapability,
    ) -> Self {
        Self(StoragePermissionsPrivate::SelfOnly(short_id_fixed))
    }

    pub fn new_fixed_size(
        app_id: core::num::NonZeroU32,
        write_permission: bool,
        read_modify_self: bool,
        read_count: usize,
        read_permissions: [u32; 8],
        modify_count: usize,
        modify_permissions: [u32; 8],
        _cap: &dyn ApplicationStorageCapability,
    ) -> Self {
        Self(StoragePermissionsPrivate::FixedSize(FixedSizePermissions {
            app_id,
            write_permission,
            read_modify_self,
            read_count,
            read_permissions,
            modify_count,
            modify_permissions,
        }))
    }

    pub fn new_listed(
        app_id: core::num::NonZeroU32,
        write_permission: bool,
        read_modify_self: bool,
        read_permissions: &'static [u32],
        modify_permissions: &'static [u32],
        _cap: &dyn ApplicationStorageCapability,
    ) -> Self {
        Self(StoragePermissionsPrivate::Listed(ListedPermissions {
            app_id,
            write_permission,
            read_modify_self,
            read_permissions,
            modify_permissions,
        }))
    }

    pub fn new_kernel(_cap: &dyn KerneluserStorageCapability) -> Self {
        Self(StoragePermissionsPrivate::Kernel)
    }

    pub fn new_null() -> Self {
        Self(StoragePermissionsPrivate::Null)
    }

    /// Check if these storage permissions grant read access to the stored state
    /// marked with identifier `stored_id`.
    pub fn check_read_permission(&self, stored_id: u32) -> bool {
        match self.0 {
            StoragePermissionsPrivate::SelfOnly(id) => stored_id == id.into(),
            StoragePermissionsPrivate::FixedSize(p) => {
                (stored_id == p.app_id.into() && p.read_modify_self)
                    || (stored_id != 0
                        && p.read_permissions
                            .get(0..p.read_count)
                            .unwrap_or(&[])
                            .contains(&stored_id))
            }
            StoragePermissionsPrivate::Listed(p) => {
                (stored_id == p.app_id.into() && p.read_modify_self)
                    || (stored_id != 0 && p.read_permissions.contains(&stored_id))
            }
            StoragePermissionsPrivate::Kernel => stored_id == 0,
            StoragePermissionsPrivate::Null => false,
        }
    }

    /// Check if these storage permissions grant modify access to the stored
    /// state marked with identifier `stored_id`.
    pub fn check_modify_permission(&self, stored_id: u32) -> bool {
        match self.0 {
            StoragePermissionsPrivate::SelfOnly(id) => stored_id == id.into(),
            StoragePermissionsPrivate::FixedSize(p) => {
                (stored_id == p.app_id.into() && p.read_modify_self)
                    || (stored_id != 0
                        && p.modify_permissions
                            .get(0..p.modify_count)
                            .unwrap_or(&[])
                            .contains(&stored_id))
            }
            StoragePermissionsPrivate::Listed(p) => {
                (stored_id == p.app_id.into() && p.read_modify_self)
                    || (stored_id != 0 && p.modify_permissions.contains(&stored_id))
            }
            StoragePermissionsPrivate::Kernel => stored_id == 0,
            StoragePermissionsPrivate::Null => false,
        }
    }

    /// Retrieve the identifier to use when storing state, if the application
    /// has permission to write. Returns `None` if the application cannot write.
    pub fn get_write_id(&self) -> Option<u32> {
        match self.0 {
            StoragePermissionsPrivate::SelfOnly(id) => Some(id.into()),
            StoragePermissionsPrivate::FixedSize(p) => {
                p.write_permission.then_some(p.app_id.into())
            }
            StoragePermissionsPrivate::Listed(p) => p.write_permission.then_some(p.app_id.into()),
            StoragePermissionsPrivate::Kernel => Some(0),
            StoragePermissionsPrivate::Null => None,
        }
    }
}
