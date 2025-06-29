// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

// todo: this module needs some polish

use crate::registers::bits32::paging::{PAddr, PDEntry, PTEntry, PD, PDFLAGS, PT, PTFLAGS};
use crate::registers::controlregs::{self, CR0, CR4};
use crate::registers::tlb;
use core::fmt;
use kernel::memory_management::pages::Page4KiB;
use kernel::memory_management::permissions::Permissions;
use kernel::memory_management::pointers::{MutablePhysicalPointer, MutableUserVirtualPointer};
use kernel::memory_management::regions::UserMappedProtectedAllocatedRegion;
use kernel::platform::mmu::Asid;
use kernel::utilities::cells::OptionalCell;
use tock_registers::LocalRegisterCopy;

use core::cell::RefCell;
use core::num::NonZero;

//
// Information about the page table and virtual addresses can be found here:
// https://wiki.osdev.org/Paging
//
const MAX_PTE_ENTRY: usize = 1024;
const PAGE_BITS_4K: usize = 12;
const PAGE_SIZE_4K: usize = 1 << PAGE_BITS_4K;
const PAGE_SIZE_4M: usize = 0x400000;
const PAGE_TABLE_MASK: usize = MAX_PTE_ENTRY - 1;

#[derive(Copy, Clone)]
struct PageTableConfig {
    start_ram_section: usize,
    ram_pages: usize,
    start_app_section: usize,
    app_pages: usize,
    last_page_owned: usize,
    kernel_first_page: usize,
}

impl PageTableConfig {
    pub fn new() -> Self {
        Self {
            start_ram_section: 0,
            ram_pages: 0,
            start_app_section: 0,
            app_pages: 0,
            last_page_owned: 0,
            kernel_first_page: 0,
        }
    }
}

pub struct MemoryProtectionConfig {
    num_regions: usize,
    ram_regions: usize,
    page_information: PageTableConfig,
}

impl Default for MemoryProtectionConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryProtectionConfig {
    pub fn new() -> Self {
        Self {
            num_regions: 0,
            ram_regions: 0,
            page_information: PageTableConfig::new(),
        }
    }
}

impl fmt::Display for MemoryProtectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, " Paging Configuration:")?;

        writeln!(
            f,
            "  Total regions: {:10}   RAM regions: {:10}",
            self.num_regions, self.ram_regions
        )?;

        let flash_start = self.page_information.start_app_section * PAGE_SIZE_4K;
        let flash_length = self.page_information.app_pages * PAGE_SIZE_4K;
        writeln!(
            f,
            "  Flash start:   {:#010x}   Length:      {:#10x}",
            flash_start, flash_length
        )?;

        let ram_start = self.page_information.start_ram_section * PAGE_SIZE_4K;
        let ram_length = self.page_information.ram_pages * PAGE_SIZE_4K;
        writeln!(
            f,
            "  RAM start:     {:#010x}   Length:      {:#10x}",
            ram_start, ram_length
        )?;

        let kernel_start = self.page_information.kernel_first_page * PAGE_SIZE_4K;
        let kernel_length = (self.page_information.last_page_owned + 1
            - self.page_information.kernel_first_page)
            * PAGE_SIZE_4K;
        writeln!(
            f,
            "  Kernel start:  {:#010x}   Length:      {:#10x}",
            kernel_start, kernel_length
        )?;
        writeln!(f)?;

        Ok(())
    }
}

struct CachedRegion {
    starting_physical_pointer: MutablePhysicalPointer<Page4KiB>,
    starting_virtual_pointer: MutableUserVirtualPointer<Page4KiB>,
    page_count: NonZero<usize>,
    permissions: Permissions,
}

impl CachedRegion {
    fn new(mapped_region: &UserMappedProtectedAllocatedRegion<Page4KiB>) -> Self {
        let starting_physical_pointer = *mapped_region.get_starting_physical_pointer();
        let starting_virtual_pointer = *mapped_region.get_starting_virtual_pointer();
        let page_count = mapped_region.get_protected_length();
        let permissions = mapped_region.get_permissions();

        Self {
            starting_physical_pointer,
            starting_virtual_pointer,
            page_count,
            permissions,
        }
    }

    fn get_starting_physical_pointer(&self) -> &MutablePhysicalPointer<Page4KiB> {
        &self.starting_physical_pointer
    }

    fn get_starting_virtual_pointer(&self) -> &MutableUserVirtualPointer<Page4KiB> {
        &self.starting_virtual_pointer
    }

    fn get_page_count(&self) -> NonZero<usize> {
        self.page_count
    }

    fn get_permissions(&self) -> Permissions {
        self.permissions
    }
}

pub struct MMU<'a, const NUMBER_OF_REGIONS: usize> {
    cached_regions: [OptionalCell<CachedRegion>; NUMBER_OF_REGIONS],
    page_dir_paddr: usize,
    page_table_paddr: usize,
    pd: RefCell<&'a mut PD>,
    pt: RefCell<&'a mut PT>,
}

fn calc_page_index(memory_address: usize) -> usize {
    memory_address / PAGE_SIZE_4K
}

impl<'a, const NUMBER_OF_REGIONS: usize> MMU<'a, NUMBER_OF_REGIONS> {
    pub unsafe fn new(
        page_dir: &'a mut PD,
        page_dir_paddr: usize,
        page_table: &'a mut PT,
        page_table_paddr: usize,
    ) -> Self {
        let page_dir = RefCell::new(page_dir);
        let page_table = RefCell::new(page_table);

        Self {
            cached_regions: [const { OptionalCell::empty() }; NUMBER_OF_REGIONS],
            page_dir_paddr,
            page_table_paddr,
            pd: page_dir,
            pt: page_table,
        }
    }

    ///
    /// Basic iterator to walk through all page table entries
    ///
    pub unsafe fn iterate_pt<C>(&self, mut closure: C)
    where
        C: FnMut(usize, &mut PTEntry),
    {
        let mut page_table = self.pt.borrow_mut();
        for (n, entry) in page_table.iter_mut().enumerate() {
            closure(n, entry);
        }
    }

    ///
    /// Get a page table entry from a virtual address
    ///
    pub fn pt_from_addr<C>(&self, mut closure: C, virtual_addr: usize)
    where
        C: FnMut(&mut PTEntry),
    {
        let mut page_table = self.pt.borrow_mut();
        let mut page_index = virtual_addr >> PAGE_BITS_4K;
        page_index &= PAGE_TABLE_MASK;

        closure(&mut page_table[page_index]);
    }

    ///
    /// initializes the page directory & page table
    ///
    pub unsafe fn initialize_page_tables(&self) {
        let mut page_directory = self.pd.borrow_mut();

        // This should set the Page directory to point directly to a 4M opening on system doing 1 - 1 Mapping
        // so that all 32-bit space is accesible by the kernel
        // Starts at 0x0000_0000 to 0xFFFF_FFFF covering full
        for (n, entry) in page_directory.iter_mut().enumerate() {
            let mut entry_flags = LocalRegisterCopy::new(0);
            entry_flags.modify(PDFLAGS::PS::SET + PDFLAGS::RW::SET + PDFLAGS::P::SET);
            // Set up the page directory with
            *entry = PDEntry::new(PAddr::from(PAGE_SIZE_4M * n), entry_flags);
        }

        // This Page Directory Entry maps the space from 0x0000_0000 until 0x40_0000
        // this entry needs to be marked as User Accessible so entries in page table can be accessible to user.
        let mut page_directory_flags = LocalRegisterCopy::new(0);
        page_directory_flags.modify(PDFLAGS::P::SET + PDFLAGS::RW::SET + PDFLAGS::US::SET);
        page_directory[0] = PDEntry::new(PAddr::from(self.page_table_paddr), page_directory_flags);

        //  Map the first 4 MiB of memory into 4 KiB entries
        let mut page_table = self.pt.borrow_mut();
        let mut page_table_flags = LocalRegisterCopy::new(0);
        page_table_flags.modify(PTFLAGS::P::SET + PTFLAGS::RW::SET);
        for (n, entry) in page_table.iter_mut().enumerate() {
            *entry = PTEntry::new(PAddr::from(PAGE_SIZE_4K * n), page_table_flags);
        }
    }

    ///
    /// Performs basic x86-32 bit paging enablement
    ///
    /// This function enables Paging (CR4) and sets the page directory ptr (in physical address)
    /// into CR3.
    ///
    unsafe fn enable_paging(&self) {
        // In order to enable a 4M make sure PSE is enabled in CR4
        let mut cr4_value = unsafe { controlregs::cr4() };
        if !cr4_value.is_set(CR4::CR4_ENABLE_PSE) {
            cr4_value.modify(CR4::CR4_ENABLE_PSE::SET);
            unsafe {
                controlregs::cr4_write(cr4_value);
            }
        }

        unsafe {
            // Now with the page directory and page table mapped load it to CR3
            controlregs::cr3_write(self.page_dir_paddr as u64);

            // Finally enable paging setting the value in CR0
            let mut cr0_value = controlregs::cr0();
            cr0_value.modify(CR0::CR0_ENABLE_PAGING::SET);
            controlregs::cr0_write(cr0_value);
        }
    }

    ///
    /// General init function
    /// This function automatically sets the page directory and page table, and then enables
    /// Paging
    pub fn init(&self) {
        unsafe {
            self.initialize_page_tables();
            self.enable_paging();
        }
    }

    fn remove_user_cached_region(&self, cached_region: &CachedRegion) {
        let starting_virtual_pointer = cached_region.get_starting_virtual_pointer();
        let starting_physical_pointer = cached_region.get_starting_physical_pointer();
        let page_count = cached_region.get_page_count();
        let permissions = cached_region.get_permissions();

        let starting_virtual_address = starting_virtual_pointer.get_address();
        let starting_index = calc_page_index(starting_virtual_address.get());
        let ending_index = starting_index + page_count.get();

        let starting_physical_address = starting_physical_pointer.get_address();

        let mut sram_page_table = self.pt.borrow_mut();
        let mut permissions_flags = LocalRegisterCopy::new(0);
        permissions_flags.modify(PTFLAGS::P::SET);

        if Permissions::ReadWrite == permissions {
            permissions_flags.modify(PTFLAGS::RW::SET);
        }

        for page_index in starting_index..ending_index {
            let paddr = PAddr(starting_physical_address.get() as u32);
            sram_page_table[page_index] = PTEntry::new(paddr, permissions_flags);
        }
    }

    fn add_user_region(&self, mapped_region: &UserMappedProtectedAllocatedRegion<Page4KiB>) {
        let starting_virtual_pointer = mapped_region.get_starting_virtual_pointer();
        let starting_physical_pointer = mapped_region.get_starting_physical_pointer();
        let page_count = mapped_region.get_protected_length();
        let permissions = mapped_region.get_permissions();

        let starting_virtual_address = starting_virtual_pointer.get_address();
        let starting_index = calc_page_index(starting_virtual_address.get());
        let ending_index = starting_index + page_count.get();

        let starting_physical_address = starting_physical_pointer.get_address();

        let mut sram_page_table = self.pt.borrow_mut();
        let mut permissions_flags = LocalRegisterCopy::new(0);
        permissions_flags.modify(PTFLAGS::P::SET + PTFLAGS::US::SET + PTFLAGS::RW::SET);

        if Permissions::ReadWrite == permissions {
            permissions_flags.modify(PTFLAGS::RW::SET);
        }

        for page_index in starting_index..ending_index {
            let paddr = PAddr(starting_physical_address.get() as u32);
            sram_page_table[page_index] = PTEntry::new(paddr, permissions_flags);
        }
    }
}

impl<const NUMBER_OF_REGIONS: usize> kernel::platform::mmu::MpuMmuCommon
    for MMU<'_, NUMBER_OF_REGIONS>
{
    type Granule = Page4KiB;

    // Once the MMU is active, the user protection is always active
    fn enable_user_protection(&self, _asid: Asid) {}

    // Paging stays enabled for Ring0/Ring3
    fn disable_user_protection(&self) {}
}

impl<const NUMBER_OF_REGIONS: usize> kernel::platform::mmu::MMU for MMU<'_, NUMBER_OF_REGIONS> {
    fn create_asid(&self) -> Asid {
        // The current implementation doesn't use ASIDs. This function returns a placeholder value.
        Asid::new(0)
    }

    // The current implementation doesn't use ASIDs.
    fn flush(&self, _asid: Asid) {}

    fn map_user_region(
        &self,
        region_index: usize,
        mapped_region: &UserMappedProtectedAllocatedRegion<Self::Granule>,
    ) {
        let cached_region = match self.cached_regions.get(region_index) {
            // Ignore an invalid index
            None => return,
            Some(cached_region) => cached_region,
        };

        if let Some(cached_user_prog_region) = cached_region.take() {
            self.remove_user_cached_region(&cached_user_prog_region);
        }

        self.add_user_region(mapped_region);
        cached_region.set(CachedRegion::new(mapped_region));

        unsafe { tlb::flush_all() };
    }
}
