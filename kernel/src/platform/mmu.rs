// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for configuring the Memory Protection Unit.

use crate::memory_management::granules::Granule as GranuleLike;
use crate::memory_management::regions::{
    PhysicalProtectedAllocatedRegion, UserMappedProtectedAllocatedRegion,
};

/// Addres space identifier.
#[derive(Clone, Copy)]
pub struct Asid(u16);

impl Asid {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

/// Methods common to both MPUs and MMUs.
pub trait MpuMmuCommon {
    type Granule: GranuleLike;

    fn enable_user_protection(&self, asid: Asid);

    fn disable_user_protection(&self);
}

/// MPU support.
pub trait MPU: MpuMmuCommon {
    /// Applies the memory protections of the given PROG region.
    ///
    /// Implementors may ignore the request if invalid. For instance if the region's protected
    /// length is not a power of two as mandated by the hardware.
    ///
    /// If the index is invalid, the region must be ignored.
    fn protect_user_region(
        &self,
        region_index: usize,
        protected_region: &PhysicalProtectedAllocatedRegion<Self::Granule>,
    );
}

pub trait MMU: MpuMmuCommon {
    /// Create a new ASID.
    ///
    /// This method should return an unique ASID each time is invoked.
    // TODO: What if all ASIDs are exhausted?
    //
    // 1. ARMv8-A supports 8-bit or 16-bit ASIDs.
    // 2. x64 supports 12-bit ASIDs.
    // 3. RV32 supports up to 9-bit ASIDs and RV64 up to 16-bit ASIDs.
    fn create_asid(&self) -> Asid;

    /// Flush TLB entries with the given ASID.
    ///
    /// If the underlying hardware does not support TLB, nor ASIDs, this should be implemented as a no-op.
    fn flush(&self, asid: Asid);

    /// Map and protect the given region.
    ///
    /// If the index is invalid, the region must be ignored.
    fn map_user_region(
        &self,
        index: usize,
        mapped_protected_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    );
}

/// MMU implementation for MPU hardware.
///
/// Tock kernel is designed to run with a MMU by default. However, it can also run on MMU-less
/// architectures with limited capabilities.
impl<T: MPU> MMU for T {
    fn create_asid(&self) -> Asid {
        // The returned value doesn't matter.
        Asid::new(0)
    }

    fn flush(&self, _asid: Asid) {}

    fn map_user_region(
        &self,
        index: usize,
        mapped_protected_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    ) {
        // Discard the memory mapping for MPUs.
        let protected_region = mapped_protected_region.as_physical_protected_allocated_region();
        self.protect_user_region(index, protected_region);
    }
}
