// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Memory managers.

use super::allocators::Allocator as AllocatorTrait;
use super::configuration::{KernelConfiguration, ProcessConfiguration, ValidProcessConfiguration};
use super::granules::Granule as GranuleTrait;
use super::pointers::{KernelVirtualPointer, PhysicalPointer};
use super::regions::{
    KernelMappedProtectedAllocatedRegion, PhysicalAllocatedRegion,
    UserMappedProtectedAllocatedRegion,
};

use crate::platform::mmu::Asid;
use crate::utilities::alignment::AlwaysAligned;

use core::marker::PhantomData;
use core::num::NonZero;

/// A memory manager.
///
/// A memory manager currently handles only memory allocation.
struct MemoryManager<'a, Granule, Allocator: AllocatorTrait<'a, Granule>> {
    allocator: Allocator,
    phantom_data: PhantomData<&'a Granule>,
}

impl<'a, Granule, Allocator: AllocatorTrait<'a, Granule>> MemoryManager<'a, Granule, Allocator> {
    const fn new(allocator: Allocator) -> Self {
        Self {
            allocator,
            phantom_data: PhantomData,
        }
    }

    fn allocate(
        &self,
        granule_count: NonZero<usize>,
    ) -> Result<PhysicalAllocatedRegion<'a, Granule>, ()> {
        let mutable_physical_slice = self.allocator.allocate(granule_count)?;
        let allocated_region = PhysicalAllocatedRegion::new(mutable_physical_slice);
        Ok(allocated_region)
    }
}

/// Errors on process memory mapping.
#[derive(Debug)]
pub enum ProcessMemoryMappingError {
    /// The PROG memory is not a valid user virtual memory.
    InvalidProgVirtualMemory,
    /// The RAM memory is not a valid user virtual memory,
    InvalidRamVirtualMemory,
}

/// Process memory manager.
///
/// Handles process memory allocations and process memory configurations.
pub struct ProcessMemoryManager<'a, Granule, Allocator: AllocatorTrait<'a, Granule>> {
    memory_manager: MemoryManager<'a, Granule, Allocator>,
}

impl<'a, Granule: 'a + GranuleTrait, Allocator: AllocatorTrait<'a, Granule>>
    ProcessMemoryManager<'a, Granule, Allocator>
{
    pub const fn new(allocator: Allocator) -> Self {
        Self {
            memory_manager: MemoryManager::new(allocator),
        }
    }

    pub(crate) fn new_configuration(
        &self,
        asid: Asid,
        flash_region: UserMappedProtectedAllocatedRegion<'a, Granule>,
        ram_region: UserMappedProtectedAllocatedRegion<'a, Granule>,
    ) -> ProcessConfiguration<'a, Granule> {
        ProcessConfiguration::new(asid, flash_region, ram_region)
    }

    pub(crate) fn allocate(
        &self,
        granule_count: NonZero<usize>,
    ) -> Result<PhysicalAllocatedRegion<Granule>, ()> {
        let allocated_region = self.memory_manager.allocate(granule_count)?;
        Ok(allocated_region)
    }
}

/// Kernel memory manager.
///
/// Validates process memory configurations and performs offline user -> kernel
/// address translation.
pub(crate) struct KernelMemoryManager<'a, Granule, const USING_MPU: bool> {
    configuration: KernelConfiguration<'a, Granule>,
}

impl<'a, Granule, const USING_MPU: bool> KernelMemoryManager<'a, Granule, USING_MPU> {
    pub(crate) const fn new(
        rom_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
        prog_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
        ram_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
        peripheral_region: KernelMappedProtectedAllocatedRegion<'a, Granule>,
    ) -> Self {
        let configuration =
            KernelConfiguration::new(rom_region, prog_region, ram_region, peripheral_region);

        Self { configuration }
    }

    fn get_configuration(&self) -> &KernelConfiguration<Granule> {
        &self.configuration
    }

    fn is_user_mapping_valid(
        &self,
        region: &UserMappedProtectedAllocatedRegion<'a, Granule>,
    ) -> bool {
        if USING_MPU {
            let configuration = self.get_configuration();
            let dirty_prog_region = configuration.get_prog_region();
            let mapped_prog_region = dirty_prog_region.as_mapped_protected_allocated_region();
            let dirty_ram_region = configuration.get_ram_region();
            let mapped_ram_region = dirty_ram_region.as_mapped_protected_allocated_region();

            mapped_prog_region.is_intersecting_virtually(region)
                || mapped_ram_region.is_intersecting_virtually(region)
        } else {
            !self
                .get_configuration()
                .is_intersecting_user_virtual_region(region)
        }
    }

    pub(crate) fn is_process_configuration_valid(
        &self,
        process_configuration: ProcessConfiguration<'a, Granule>,
    ) -> Result<ValidProcessConfiguration<'a, Granule>, ProcessMemoryMappingError> {
        let prog_region = process_configuration.get_prog_region();

        if !self.is_user_mapping_valid(prog_region.as_mapped_protected_allocated_region()) {
            return Err(ProcessMemoryMappingError::InvalidProgVirtualMemory);
        }

        let ram_region = process_configuration.get_ram_region();

        if !self.is_user_mapping_valid(ram_region.as_mapped_protected_allocated_region()) {
            return Err(ProcessMemoryMappingError::InvalidRamVirtualMemory);
        }

        // SAFETY: because of the previous checks, the process configuration does not overlap the
        // kernel's virtual memory
        let valid_process_configuration =
            unsafe { ValidProcessConfiguration::new(process_configuration) };

        Ok(valid_process_configuration)
    }

    pub(crate) fn translate_allocated_physical_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        physical_pointer: PhysicalPointer<IS_MUTABLE, U>,
    ) -> Result<KernelVirtualPointer<IS_MUTABLE, U>, PhysicalPointer<IS_MUTABLE, U>> {
        self.get_configuration()
            .translate_allocated_physical_pointer_byte(physical_pointer)
    }

    pub(crate) fn translate_allocated_virtual_pointer_byte<
        const IS_MUTABLE: bool,
        U: AlwaysAligned,
    >(
        &self,
        physical_pointer: KernelVirtualPointer<IS_MUTABLE, U>,
    ) -> Result<PhysicalPointer<IS_MUTABLE, U>, KernelVirtualPointer<IS_MUTABLE, U>> {
        self.get_configuration()
            .translate_allocated_virtual_pointer_byte(physical_pointer)
    }
}
