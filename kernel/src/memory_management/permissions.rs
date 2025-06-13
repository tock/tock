// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Memory permissions.

/// Permissions that might be associated with a region of memory.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    ReadOnly,
    ReadWrite,
    ReadExecute,
}

impl core::fmt::Display for Permissions {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Permissions::ReadOnly => write!(formatter, "read-only"),
            Permissions::ReadWrite => write!(formatter, "read-write"),
            Permissions::ReadExecute => write!(formatter, "read-execute"),
        }
    }
}
