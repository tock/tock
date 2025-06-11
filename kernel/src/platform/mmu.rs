// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for configuring the Memory Protection Unit.

use crate::memory_management::granules::Granule as GranuleLike;
use crate::memory_management::regions::{
    PhysicalProtectedAllocatedRegion, UserMappedProtectedAllocatedRegion,
};

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

pub trait MpuMmuCommon {
    type Granule: GranuleLike;

    fn enable_user_protection(&self, asid: Asid);

    fn disable_user_protection(&self);
}

pub trait MPU: MpuMmuCommon {
    fn protect_user_prog_region(
        &self,
        protected_region: &PhysicalProtectedAllocatedRegion<Self::Granule>,
    );

    fn protect_user_ram_region(
        &self,
        protected_region: &PhysicalProtectedAllocatedRegion<Self::Granule>,
    );
}

pub trait MMU: MpuMmuCommon {
    fn create_asid(&self) -> Asid;

    fn flush(&self, asid: Asid);

    fn map_user_prog_region(
        &self,
        mapped_protected_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    );

    fn map_user_ram_region(
        &self,
        mapped_protected_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    );
}

impl<T: MPU> MMU for T {
    fn create_asid(&self) -> Asid {
        // The returned value doesn't matter.
        Asid::new(0)
    }

    fn flush(&self, _asid: Asid) {}

    fn map_user_prog_region(
        &self,
        mapped_protected_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    ) {
        let protected_region = mapped_protected_region.as_physical_protected_allocated_region();
        self.protect_user_prog_region(protected_region);
    }

    fn map_user_ram_region(
        &self,
        mapped_protected_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    ) {
        let protected_region = mapped_protected_region.as_physical_protected_allocated_region();
        self.protect_user_ram_region(protected_region);
    }
}
