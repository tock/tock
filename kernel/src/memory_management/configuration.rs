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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::memory_management::pages::Page4KiB;
    use crate::memory_management::permissions::Permissions;
    use crate::memory_management::pointers::{
        MutablePhysicalPointer,
        ValidMutableVirtualPointer,
        VirtualPointer,
    };
    use crate::memory_management::regions::{
        AllocatedRegion,
        ProtectedAllocatedRegion,
    };
    use crate::memory_management::slices::{
        MutablePhysicalSlice,
    };
    use crate::utilities;
    use crate::utilities::misc::create_non_zero_usize;

    fn create_physical_pointer<T>(address: usize) -> MutablePhysicalPointer<T> {
        // Allocated region
        let pointer = utilities::pointers::MutablePointer::new(address as *mut T).unwrap();
        // SAFETY: let's assume it's a valid physical pointer
        unsafe { MutablePhysicalPointer::new(pointer) }
    }

    fn create_virtual_pointer<const IS_USER: bool, T>(address: usize) -> ValidMutableVirtualPointer<IS_USER, T> {
        // Allocated region
        let pointer = utilities::pointers::MutablePointer::new(address as *mut T).unwrap();
        // SAFETY: let's assume it's a valid physical pointer
        let virtual_pointer = unsafe { VirtualPointer::new(pointer) };
        // SAFETY: let's assume it's a valid virtual pointer
        unsafe { ValidMutableVirtualPointer::new(virtual_pointer) }
    }

    fn create_region<const IS_USER: bool>(
        starting_physical_address: usize,
        starting_virtual_address: usize,
        permissions: Permissions,
    ) -> MappedProtectedAllocatedRegion<'static, IS_USER, Page4KiB> {
        let starting_physical_pointer = create_physical_pointer(starting_physical_address);
        let physical_length = create_non_zero_usize(4);
        // SAFETY: let's assume it's a valid physical slice
        let physical_slice = unsafe { MutablePhysicalSlice::from_raw_parts(starting_physical_pointer, physical_length) };
        let allocated_region = AllocatedRegion::new(physical_slice);

        // Protected allocated region
        let protected_length = create_non_zero_usize(2);
        let protected_allocated_region = ProtectedAllocatedRegion::new(
            allocated_region,
            protected_length,
            permissions,
        ).unwrap();


        // Allocated region
        let starting_virtual_pointer = create_virtual_pointer(starting_virtual_address);
        MappedProtectedAllocatedRegion::new_from_protected(
            protected_allocated_region,
            starting_virtual_pointer,
        ).unwrap()
    }

    fn create_process_configuration<'a>() -> ValidProcessConfiguration<'a, Page4KiB> {
        let flash_region = create_region::<true>(0x9000_0000, 0x3000_0000, Permissions::ReadExecute);
        let ram_region = create_region::<true>(0x3000_0000, 0x4000_0000, Permissions::ReadWrite);
        let process_configuration = ProcessConfiguration::new(
            Asid::new(0),
            flash_region,
            ram_region,
        );
        // SAFETY: let's assume the configuration is valid
        unsafe { ValidProcessConfiguration::new(process_configuration) }
    }

    fn create_kernel_configuration<'a>() -> KernelConfiguration<'a, Page4KiB> {
        let rom_region = create_region::<false>(0x1000_0000, 0xC000_0000, Permissions::ReadExecute);
        let prog_region = create_region::<false>(0x2000_0000, 0xD000_0000, Permissions::ReadExecute);
        let ram_region = create_region::<false>(0x3000_0000, 0xE000_0000, Permissions::ReadWrite);
        let peripheral_region = create_region::<false>(0x4000_0000, 0xF000_0000, Permissions::ReadWrite);
        KernelConfiguration::new(
            rom_region,
            prog_region,
            ram_region,
            peripheral_region
        )
    }

    #[test]
    fn test_process_configuration_translate_protected_physical_pointer_byte() {
        let process_configuration = create_process_configuration();

        let physical_byte = create_physical_pointer::<u8>(0x8FFF_FFFF);
        assert!(process_configuration.translate_protected_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x9000_0000);
        let virtual_byte = process_configuration.translate_protected_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0x3000_0000, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x9000_1FFF);
        let virtual_byte = process_configuration.translate_protected_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0x3000_1FFF, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x9000_2000);
        assert!(process_configuration.translate_protected_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x2FFF_FFFF);
        assert!(process_configuration.translate_protected_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x3000_0000);
        let virtual_byte = process_configuration.translate_protected_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0x4000_0000, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x3000_1FFF);
        let virtual_byte = process_configuration.translate_protected_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0x4000_1FFF, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x3000_2000);
        assert!(process_configuration.translate_protected_physical_pointer_byte(physical_byte).is_err());
    }

    #[test]
    fn test_process_configuration_translate_protected_virtual_pointer_byte() {
        let process_configuration = create_process_configuration();

        let virtual_byte = create_virtual_pointer::<true, u8>(0x2FFF_FFFF);
        assert!(process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3000_0000);
        let physical_byte = process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x9000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3000_1FFF);
        let physical_byte = process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x9000_1FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3000_2000);
        assert!(process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3FFF_FFFF);
        assert!(process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x4000_0000);
        let physical_byte = process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x3000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x4000_1FFF);
        let physical_byte = process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x3000_1FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x4000_2000);
        assert!(process_configuration.translate_protected_virtual_pointer_byte(virtual_byte).is_err());
    }

    #[test]
    fn test_process_configuration_translate_allocated_virtual_pointer_byte() {
        let process_configuration = create_process_configuration();

        let virtual_byte = create_virtual_pointer::<true, u8>(0x2FFF_FFFF);
        assert!(process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3000_0000);
        let physical_byte = process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x9000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3000_3FFF);
        let physical_byte = process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x9000_3FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3000_4000);
        assert!(process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x3FFF_FFFF);
        assert!(process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x4000_0000);
        let physical_byte = process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x3000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x4000_3FFF);
        let physical_byte = process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x3000_3FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<true, u8>(0x4000_4000);
        assert!(process_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());
    }

    #[test]
    fn test_kernel_configuration_is_intersecting_user_virtual_region() {
        let kernel_configuration = create_kernel_configuration();

        // ROM
        let user_region = create_region::<true>(
            0x5000_0000,
            0xBFFF_C000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xBFFF_D000,
            Permissions::ReadWrite,
        );
        assert!(kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xC000_4000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        // PROG
        let user_region = create_region::<true>(
            0x5000_0000,
            0xCFFF_C000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xCFFF_D000,
            Permissions::ReadWrite,
        );
        assert!(kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xD000_4000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        // RAM
        let user_region = create_region::<true>(
            0x5000_0000,
            0xDFFF_C000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xDFFF_D000,
            Permissions::ReadWrite,
        );
        assert!(kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xE000_4000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        // PERIPHERAL
        let user_region = create_region::<true>(
            0x5000_0000,
            0xEFFF_C000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xEFFF_D000,
            Permissions::ReadWrite,
        );
        assert!(kernel_configuration.is_intersecting_user_virtual_region(&user_region));

        let user_region = create_region::<true>(
            0x5000_0000,
            0xF000_4000,
            Permissions::ReadWrite,
        );
        assert!(!kernel_configuration.is_intersecting_user_virtual_region(&user_region));
    }

    #[test]
    fn test_kernel_configuration_translate_allocated_virtual_pointer_byte() {
        let kernel_configuration = create_kernel_configuration();

        // ROM
        let virtual_byte = create_virtual_pointer::<false, u8>(0xBFFF_FFFF);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xC000_0000);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x1000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xC000_3FFF);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x1000_3FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xC000_4000);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        // PROG
        let virtual_byte = create_virtual_pointer::<false, u8>(0xCFFF_FFFF);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_0000);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x2000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_3FFF);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x2000_3FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_4000);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        // RAM
        let virtual_byte = create_virtual_pointer::<false, u8>(0xCFFF_FFFF);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_0000);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x2000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_3FFF);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x2000_3FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_4000);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        // PERIPHERAL
        let virtual_byte = create_virtual_pointer::<false, u8>(0xCFFF_FFFF);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_0000);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x2000_0000, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_3FFF);
        let physical_byte = kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).unwrap();
        assert_eq!(0x2000_3FFF, physical_byte.get_address().get());

        let virtual_byte = create_virtual_pointer::<false, u8>(0xD000_4000);
        assert!(kernel_configuration.translate_allocated_virtual_pointer_byte(virtual_byte).is_err());
    }

    #[test]
    fn test_kernel_configuration_translate_allocated_physical_pointer_byte() {
        let kernel_configuration = create_kernel_configuration();

        // ROM
        let physical_byte = create_physical_pointer::<u8>(0x0FFF_FFFF);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x1000_0000);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xC000_0000, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x1000_3FFF);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xC000_3FFF, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x1000_4000);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        // PROG
        let physical_byte = create_physical_pointer::<u8>(0x1FFF_FFFF);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x2000_0000);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xD000_0000, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x2000_3FFF);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xD000_3FFF, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x2000_4000);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        // RAM
        let physical_byte = create_physical_pointer::<u8>(0x2FFF_FFFF);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x3000_0000);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xE000_0000, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x3000_3FFF);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xE000_3FFF, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x3000_4000);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        // PERIPHERAL
        let physical_byte = create_physical_pointer::<u8>(0x3FFF_FFFF);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());

        let physical_byte = create_physical_pointer::<u8>(0x4000_0000);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xF000_0000, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x4000_3FFF);
        let virtual_byte = kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).unwrap();
        assert_eq!(0xF000_3FFF, virtual_byte.get_address().get());

        let physical_byte = create_physical_pointer::<u8>(0x4000_4000);
        assert!(kernel_configuration.translate_allocated_physical_pointer_byte(physical_byte).is_err());
    }
}
