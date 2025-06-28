// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Memory configurations.

use super::pointers::{
    KernelVirtualPointer, PhysicalPointer, UserVirtualPointer, ValidVirtualPointer,
};
use super::regions::{
    DirtyMappedProtectedAllocatedRegion, KernelDirtyMappedProtectedAllocatedRegion,
    KernelMappedProtectedAllocatedRegion, MappedProtectedAllocatedRegion,
    UserDirtyMappedProtectedAllocatedRegion, UserMappedProtectedAllocatedRegion,
};

use crate::platform::mmu::Asid;

use crate::utilities::alignment::AlwaysAligned;

/// Memory configuration.
#[repr(transparent)]
pub(super) struct Configuration<'a, const IS_USER: bool, const NUMBER_REGIONS: usize, Granule> {
    regions: [DirtyMappedProtectedAllocatedRegion<'a, IS_USER, Granule>; NUMBER_REGIONS],
}

impl<'a, const IS_USER: bool, const NUMBER_REGIONS: usize, Granule>
    Configuration<'a, IS_USER, NUMBER_REGIONS, Granule>
{
    pub(super) const fn new(
        regions: [DirtyMappedProtectedAllocatedRegion<'a, IS_USER, Granule>; NUMBER_REGIONS],
    ) -> Self {
        Self { regions }
    }

    pub(super) fn get_region(
        &self,
        index: usize,
    ) -> Option<&DirtyMappedProtectedAllocatedRegion<'a, IS_USER, Granule>> {
        self.regions.get(index)
    }

    pub(super) fn is_intersecting_virtual_region<const OTHER_IS_USER: bool>(
        &self,
        target_region: &MappedProtectedAllocatedRegion<'a, OTHER_IS_USER, Granule>,
    ) -> bool {
        for region in &self.regions {
            if region
                .as_mapped_protected_allocated_region()
                .is_intersecting_virtually(target_region)
            {
                return true;
            }
        }

        false
    }

    pub(super) fn translate_allocated_physical_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        mut physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<ValidVirtualPointer<IS_USER, IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        for region in &self.regions {
            physical_pointer =
                match region.translate_allocated_physical_pointer_byte(physical_pointer) {
                    Err(physical_pointer) => physical_pointer,
                    Ok(virtual_pointer) => return Ok(virtual_pointer),
                }
        }

        Err(physical_pointer)
    }

    pub(super) fn translate_allocated_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        mut virtual_pointer: ValidVirtualPointer<IS_USER, IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, ValidVirtualPointer<IS_USER, IS_MUTABLE, U>> {
        for region in &self.regions {
            virtual_pointer = match region.translate_allocated_virtual_pointer_byte(virtual_pointer)
            {
                Err(virtual_pointer) => virtual_pointer,
                Ok(physical_pointer) => return Ok(physical_pointer),
            }
        }

        Err(virtual_pointer)
    }
}

/// Process memory configuration.
pub(crate) struct ProcessConfiguration<'a, Granule> {
    asid: Asid,
    configuration: Configuration<'a, true, 2, Granule>,
}

const FLASH_REGION_INDEX: usize = 0;
const RAM_REGION_INDEX: usize = 1;

impl<'a, Granule> ProcessConfiguration<'a, Granule> {
    pub(super) const fn new(
        asid: Asid,
        flash_region: UserMappedProtectedAllocatedRegion<'a, Granule>,
        ram_region: UserMappedProtectedAllocatedRegion<'a, Granule>,
    ) -> Self {
        let dirty_flash_region = UserDirtyMappedProtectedAllocatedRegion::new(flash_region);
        let dirty_ram_region = UserDirtyMappedProtectedAllocatedRegion::new(ram_region);

        Self {
            asid,
            configuration: Configuration::new([dirty_flash_region, dirty_ram_region]),
        }
    }

    fn as_configuration(&self) -> &Configuration<'a, true, 2, Granule> {
        &self.configuration
    }

    fn get_asid(&self) -> Asid {
        self.asid
    }

    pub(super) fn get_prog_region(&self) -> &UserDirtyMappedProtectedAllocatedRegion<'a, Granule> {
        // PANIC: FLASH_REGION_INDEX < 2
        self.as_configuration()
            .get_region(FLASH_REGION_INDEX)
            .unwrap()
    }

    pub(super) fn get_ram_region(&self) -> &UserDirtyMappedProtectedAllocatedRegion<'a, Granule> {
        // PANIC: RAM_REGION_INDEX < 2
        self.as_configuration()
            .get_region(RAM_REGION_INDEX)
            .unwrap()
    }

    fn translate_protected_physical_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<UserVirtualPointer<IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        let ram_region = self.get_ram_region();

        let physical_pointer =
            match ram_region.translate_protected_physical_pointer_byte(physical_pointer) {
                Err(physical_pointer) => physical_pointer,
                ok => return ok,
            };

        let prog_region = self.get_prog_region();

        prog_region.translate_protected_physical_pointer_byte(physical_pointer)
    }

    fn translate_protected_virtual_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        virtual_pointer: UserVirtualPointer<IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, UserVirtualPointer<IS_MUTABLE, U>> {
        let ram_region = self.get_ram_region();
        let virtual_pointer =
            match ram_region.translate_protected_virtual_pointer_byte(virtual_pointer) {
                Err(virtual_pointer) => virtual_pointer,
                ok => return ok,
            };

        let prog_region = self.get_prog_region();
        prog_region.translate_protected_virtual_pointer_byte(virtual_pointer)
    }

    fn translate_allocated_virtual_pointer_byte<const IS_MUTABLE: bool, U: AlwaysAligned>(
        &self,
        virtual_pointer: UserVirtualPointer<IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, UserVirtualPointer<IS_MUTABLE, U>> {
        let ram_region = self.get_ram_region();

        let virtual_pointer =
            match ram_region.translate_allocated_virtual_pointer_byte(virtual_pointer) {
                Err(virtual_pointer) => virtual_pointer,
                ok => return ok,
            };

        let prog_region = self.get_prog_region();

        prog_region.translate_allocated_virtual_pointer_byte(virtual_pointer)
    }
}

/// Valid process memory configuration, that is, it doesn't overlap kernel's
/// virtual address space.
#[repr(transparent)]
pub struct ValidProcessConfiguration<'a, Granule>(ProcessConfiguration<'a, Granule>);

impl<'a, Granule> ValidProcessConfiguration<'a, Granule> {
    /// # Safety
    ///
    /// The caller must ensure that the process configuration does not overlap the kernel's virtual
    /// memory.
    pub(super) const unsafe fn new(
        process_configuration: ProcessConfiguration<'a, Granule>,
    ) -> Self {
        Self(process_configuration)
    }

    pub(crate) fn get_prog_region(&self) -> &UserDirtyMappedProtectedAllocatedRegion<'a, Granule> {
        self.0.get_prog_region()
    }

    pub(crate) fn get_ram_region(&self) -> &UserDirtyMappedProtectedAllocatedRegion<'a, Granule> {
        self.0.get_ram_region()
    }

    pub(crate) fn get_asid(&self) -> Asid {
        self.0.get_asid()
    }

    pub(crate) fn translate_protected_physical_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<UserVirtualPointer<IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        self.0
            .translate_protected_physical_pointer_byte(physical_pointer)
    }

    pub(crate) fn translate_protected_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        virtual_pointer: UserVirtualPointer<IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, UserVirtualPointer<IS_MUTABLE, U>> {
        self.0
            .translate_protected_virtual_pointer_byte(virtual_pointer)
    }

    pub(crate) fn translate_allocated_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        virtual_pointer: UserVirtualPointer<IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, UserVirtualPointer<IS_MUTABLE, U>> {
        self.0
            .translate_allocated_virtual_pointer_byte(virtual_pointer)
    }
}

impl<Granule> core::fmt::Display for ValidProcessConfiguration<'_, Granule> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let header = r"
+---------------------------------+
|                                 |
|  PROCESS MEMORY CONFIGURATION   |
|                                 |
+---------------------------------+
";

        write!(formatter, "\n{}\n", header)?;
        writeln!(formatter, "PROG region: {}", self.get_prog_region())?;
        write!(formatter, "RAM region: {}", self.get_ram_region())
    }
}

//const KERNEL_ROM_REGION_INDEX: usize = 0;
const KERNEL_PROG_REGION_INDEX: usize = 1;
const KERNEL_RAM_REGION_INDEX: usize = 2;
//const KERNEL_PERIPHERAL_REGION_INDEX: usize = 3;

/// Kernel memory configuration.
#[repr(transparent)]
pub(crate) struct KernelConfiguration<'a, Granule>(Configuration<'a, false, 4, Granule>);

impl<'a, Granule> KernelConfiguration<'a, Granule> {
    pub(crate) const fn new(
        rom_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
        prog_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
        ram_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
        peripheral_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
    ) -> Self {
        let dirty_rom_region = KernelDirtyMappedProtectedAllocatedRegion::new(rom_region);
        let dirty_prog_region = KernelDirtyMappedProtectedAllocatedRegion::new(prog_region);
        let dirty_ram_region = KernelDirtyMappedProtectedAllocatedRegion::new(ram_region);
        let dirty_peripheral_region =
            KernelDirtyMappedProtectedAllocatedRegion::new(peripheral_region);

        let configuration = Configuration::new([
            dirty_rom_region,
            dirty_prog_region,
            dirty_ram_region,
            dirty_peripheral_region,
        ]);

        Self(configuration)
    }

    pub(super) fn is_intersecting_user_virtual_region(
        &self,
        region: &UserMappedProtectedAllocatedRegion<'a, Granule>,
    ) -> bool {
        self.0.is_intersecting_virtual_region(region)
    }

    pub(super) fn get_prog_region(&self) -> &KernelDirtyMappedProtectedAllocatedRegion<Granule> {
        // PANIC: KERNEL_PROG_REGION_INDEX < 4
        self.0.get_region(KERNEL_PROG_REGION_INDEX).unwrap()
    }

    pub(super) fn get_ram_region(&self) -> &KernelDirtyMappedProtectedAllocatedRegion<Granule> {
        // PANIC: KERNEL_RAM_REGION_INDEX < 4
        self.0.get_region(KERNEL_RAM_REGION_INDEX).unwrap()
    }

    pub(super) fn translate_allocated_physical_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<KernelVirtualPointer<IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        self.0
            .translate_allocated_physical_pointer_byte(physical_pointer)
    }

    pub(super) fn translate_allocated_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        virtual_pointer: KernelVirtualPointer<IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, KernelVirtualPointer<IS_MUTABLE, U>> {
        self.0
            .translate_allocated_virtual_pointer_byte(virtual_pointer)
    }
}
