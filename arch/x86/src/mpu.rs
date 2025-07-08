// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

// todo: this module needs some polish

use crate::registers::bits32::paging::{
    PAddr, PDEntry, PTEntry, PTFlags, PD, PDFLAGS, PT, PTFLAGS,
};
use crate::registers::controlregs::{self, CR0, CR4};
use crate::registers::tlb;
use core::{cmp, fmt, mem};
use kernel::platform::mpu::{Permissions, Region, MPU};
use kernel::utilities::cells::MapCell;
use tock_registers::LocalRegisterCopy;

use core::cell::RefCell;

//
// Information about the page table and virtual addresses can be found here:
// https://wiki.osdev.org/Paging
//
const MAX_PTE_ENTRY: usize = 1024;
const PAGE_BITS_4K: usize = 12;
const PAGE_SIZE_4K: usize = 1 << PAGE_BITS_4K;
const PAGE_SIZE_4M: usize = 0x400000;
const MAX_REGIONS: usize = 8;
const PAGE_TABLE_MASK: usize = MAX_PTE_ENTRY - 1;

#[derive(Copy, Clone)]
struct AllocateRegion {
    start_index_page: usize,
    pages: usize,
    flags_set: PTFlags,
    flags_clear: PTFlags,
}

#[derive(Copy, Clone)]
struct PageTableConfig {
    start_ram_section: usize,
    ram_pages: usize,
    start_app_section: usize,
    app_pages: usize,
    last_page_owned: usize,
    kernel_first_page: usize,
    app_ram_region: usize,
    alloc_regions: [Option<AllocateRegion>; MAX_REGIONS],
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
            app_ram_region: 0,
            alloc_regions: [None; MAX_REGIONS],
        }
    }
    pub fn set_app(&mut self, start: usize, sections: usize) {
        self.start_app_section = start;
        self.app_pages = sections;
    }
    pub fn set_ram(&mut self, start: usize, sections: usize) {
        self.start_ram_section = start;
        self.ram_pages = sections;
    }
    pub fn get_ram(&self) -> usize {
        self.start_ram_section
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

pub struct PagingMPU<'a> {
    num_regions: usize,
    config_pages: MapCell<PageTableConfig>,
    page_dir_paddr: usize,
    page_table_paddr: usize,
    pd: RefCell<&'a mut PD>,
    pt: RefCell<&'a mut PT>,
}

fn calc_page_index(memory_address: usize) -> usize {
    memory_address / PAGE_SIZE_4K
}

// It will calculate the required pages doing a round up to Page size.
fn calc_alloc_pages(memory_size: usize) -> usize {
    memory_size.next_multiple_of(PAGE_SIZE_4K) / PAGE_SIZE_4K
}

impl<'a> PagingMPU<'a> {
    pub unsafe fn new(
        page_dir: &'a mut PD,
        page_dir_paddr: usize,
        page_table: &'a mut PT,
        page_table_paddr: usize,
    ) -> Self {
        let page_dir = RefCell::new(page_dir);
        let page_table = RefCell::new(page_table);

        Self {
            num_regions: 0,
            config_pages: MapCell::empty(),
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
            entry_flags.write(PDFLAGS::PS::SET + PDFLAGS::RW::SET + PDFLAGS::P::SET);
            // Set up the page directory with
            *entry = PDEntry::new(PAddr::from(PAGE_SIZE_4M * n), entry_flags);
        }

        // This Page Directory Entry maps the space from 0x0000_0000 until 0x40_0000
        // this entry needs to be marked as User Accessible so entries in page table can be accessible to user.
        let mut page_directory_flags = LocalRegisterCopy::new(0);
        page_directory_flags.write(PDFLAGS::P::SET + PDFLAGS::RW::SET + PDFLAGS::US::SET);
        page_directory[0] = PDEntry::new(PAddr::from(self.page_table_paddr), page_directory_flags);

        //  Map the first 4 MiB of memory into 4 KiB entries
        let mut page_table = self.pt.borrow_mut();
        let mut page_table_flags = LocalRegisterCopy::new(0);
        page_table_flags.write(PTFLAGS::P::SET + PTFLAGS::RW::SET);
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
}

impl fmt::Display for PagingMPU<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Num_regions: {:?}, ...", self.num_regions,)
    }
}

impl MPU for PagingMPU<'_> {
    type MpuConfig = MemoryProtectionConfig;

    fn new_config(&self) -> Option<Self::MpuConfig> {
        Some(MemoryProtectionConfig {
            num_regions: 0,
            ram_regions: 0,
            page_information: PageTableConfig::new(),
        })
    }

    fn reset_config(&self, config: &mut Self::MpuConfig) {
        config.num_regions = 0;
        config.ram_regions = 0;
        config.page_information = PageTableConfig::new();
    }

    // Once paging is enabled it is enabled for Ring0/Ring3
    fn enable_app_mpu(&self) {}

    // Paging stays enabled for Ring0/Ring3
    fn disable_app_mpu(&self) {}

    /// Returns the maximum number of regions supported by the MPU.
    fn number_total_regions(&self) -> usize {
        mem::size_of::<PT>() / mem::size_of::<PTEntry>()
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<Region> {
        // Check for the start of the unallocated memory as to be 4K Page aligned.
        let aligned_address_start: usize =
            (unallocated_memory_start as usize).next_multiple_of(PAGE_SIZE_4K);
        let page_index: usize = calc_page_index(aligned_address_start);

        let pages_alloc_requested: usize = calc_alloc_pages(min_region_size);

        let total_page_aligned_size: usize = pages_alloc_requested * PAGE_SIZE_4K;

        if aligned_address_start + total_page_aligned_size
            > unallocated_memory_start as usize + unallocated_memory_size
        {
            return None;
        }

        // check to see if this is an exact duplicate region allocation
        for r in config.page_information.alloc_regions.iter().flatten() {
            if r.start_index_page == page_index && r.pages == pages_alloc_requested {
                return Some(Region::new(
                    aligned_address_start as *const u8,
                    total_page_aligned_size,
                ));
            }
        }

        // Execution protection needs to enable PAE on the system needs support from CPU.
        // Need to check then the pages entry will go to use NXE bit

        let mut pages_attr = LocalRegisterCopy::new(0);
        match permissions {
            Permissions::ReadWriteExecute => {
                pages_attr.write(PTFLAGS::P::SET + PTFLAGS::RW::SET + PTFLAGS::US::SET)
            }
            Permissions::ReadWriteOnly => {
                pages_attr.write(PTFLAGS::P::SET + PTFLAGS::RW::SET + PTFLAGS::US::SET)
            }
            Permissions::ReadExecuteOnly => pages_attr.write(PTFLAGS::P::SET + PTFLAGS::US::SET),
            Permissions::ReadOnly => pages_attr.write(PTFLAGS::P::SET + PTFLAGS::US::SET),
            Permissions::ExecuteOnly => pages_attr.write(PTFLAGS::P::SET + PTFLAGS::US::SET),
        }

        // For allocating a region we also need the right level to set it back to
        // if is a shared region in RAM memory this region needs to be WR to Kernel
        // anything else should be just Present
        let mut pages_clear = LocalRegisterCopy::new(0);
        match permissions {
            Permissions::ReadWriteOnly => pages_clear.write(PTFLAGS::P::SET + PTFLAGS::RW::SET),
            _ => pages_clear.write(PTFLAGS::P::SET),
        }

        // Calculate the page offset based on the init.
        if page_index > MAX_PTE_ENTRY || page_index + pages_alloc_requested > MAX_PTE_ENTRY {
            return None;
        }

        // check for the start and end to be within limits
        let end_of_unallocated_memory: usize =
            unallocated_memory_start as usize + unallocated_memory_size;
        let end_of_allocated_memory: usize = aligned_address_start + total_page_aligned_size - 1;
        if calc_page_index(end_of_allocated_memory) > calc_page_index(end_of_unallocated_memory) {
            None
        } else {
            // Find the next free region that is not used
            let index = config
                .page_information
                .alloc_regions
                .iter_mut()
                .position(|r| r.is_none());

            match index {
                Some(i) => {
                    config.page_information.alloc_regions[i] = Some(AllocateRegion {
                        flags_set: pages_attr,
                        flags_clear: pages_clear,
                        start_index_page: page_index,
                        pages: pages_alloc_requested,
                    });
                }
                None => return None,
            }

            let last_page = page_index + pages_alloc_requested;

            let mut sram_page_table = self.pt.borrow_mut();

            for current_page in page_index..=last_page {
                sram_page_table[current_page] =
                    PTEntry::new(sram_page_table[current_page].address(), pages_attr);
                config.num_regions += 1;
            }

            config
                .page_information
                .set_app(page_index, config.num_regions);

            Some(Region::new(
                aligned_address_start as *const u8,
                total_page_aligned_size,
            ))
        }
    }

    fn remove_memory_region(&self, region: Region, config: &mut Self::MpuConfig) -> Result<(), ()> {
        unsafe {
            let start_page = calc_page_index(region.start_address() as usize);
            let last_page = start_page + calc_alloc_pages(region.size());

            // Find the region that is used
            let index = config.page_information.alloc_regions.iter().position(|r| {
                if let Some(r) = r {
                    if r.start_index_page == start_page && r.pages == last_page - start_page {
                        return true;
                    }
                }
                false
            });

            // If the region is not found return an error, otherwise remove it
            match index {
                Some(i) => {
                    config.page_information.alloc_regions[i] = None;
                }
                None => return Err(()),
            }

            // Update the page table to remove the region
            let mut sram_page_table = self.pt.borrow_mut();
            for page_index in start_page..=last_page {
                // Reset using the same Address but modify flags
                let mut sram_page_table_flags = LocalRegisterCopy::new(0);
                sram_page_table_flags.write(PTFLAGS::P::SET);
                sram_page_table[page_index] =
                    PTEntry::new(sram_page_table[page_index].address(), sram_page_table_flags);

                // invalidate the TLB to the virtual address
                let inv_page = page_index * PAGE_SIZE_4K;
                tlb::flush(inv_page);
                config.num_regions -= 1;
            }
        }
        Ok(())
    }

    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // this should allocate memory in a continous block right after the user
        // the kernel should be there

        let aligned_address_app: usize =
            (unallocated_memory_start as usize).next_multiple_of(PAGE_SIZE_4K);
        let last_unallocated_memory: usize =
            (unallocated_memory_start as usize) + unallocated_memory_size;
        let start_mem_page: usize = calc_page_index(aligned_address_app);

        let last_page_app_mem: usize = calc_page_index(last_unallocated_memory);

        // for x86 the minimal granularity is a 4k page
        let aligned_app_mem_size: usize = initial_app_memory_size.next_multiple_of(PAGE_SIZE_4K);
        let aligned_kernel_mem_size: usize =
            initial_kernel_memory_size.next_multiple_of(PAGE_SIZE_4K);
        let aligned_min_mem_size: usize = min_memory_size.next_multiple_of(PAGE_SIZE_4K);

        let mut pages_attr = LocalRegisterCopy::new(0);
        match permissions {
            Permissions::ReadWriteExecute => {
                pages_attr.write(PTFLAGS::P::SET + PTFLAGS::RW::SET + PTFLAGS::US::SET)
            }
            Permissions::ReadWriteOnly => {
                pages_attr.write(PTFLAGS::P::SET + PTFLAGS::RW::SET + PTFLAGS::US::SET)
            }
            Permissions::ReadExecuteOnly => pages_attr.write(PTFLAGS::P::SET + PTFLAGS::US::SET),
            Permissions::ReadOnly => pages_attr.write(PTFLAGS::P::SET + PTFLAGS::US::SET),
            Permissions::ExecuteOnly => pages_attr.write(PTFLAGS::P::SET + PTFLAGS::US::SET),
        }

        // Compute what the maximum should be at this point all should be page-aligned.

        let total_memory_size = cmp::max(
            aligned_min_mem_size + aligned_kernel_mem_size,
            aligned_app_mem_size + aligned_kernel_mem_size,
        );
        let pages_alloc_requested: usize = calc_alloc_pages(total_memory_size);
        let kernel_alloc_pages: usize = calc_alloc_pages(aligned_kernel_mem_size);

        // Check the page offset based on the init and last page
        if start_mem_page > MAX_PTE_ENTRY || start_mem_page + pages_alloc_requested > MAX_PTE_ENTRY
        {
            return None;
        }
        // Check the boundary to the end of the calculated data size.
        let end_of_unallocated_memory: usize =
            unallocated_memory_start as usize + unallocated_memory_size;
        let end_of_allocated_memory: usize = aligned_address_app + total_memory_size;
        if end_of_allocated_memory > end_of_unallocated_memory {
            None
        } else {
            let allocate_index = config
                .page_information
                .alloc_regions
                .iter_mut()
                .position(|r| r.is_none());

            allocate_index?;

            let allocate_index = allocate_index.unwrap();

            let mut alloc_regions_flags_clear = LocalRegisterCopy::new(0);
            alloc_regions_flags_clear.write(PTFLAGS::P::SET + PTFLAGS::RW::SET);
            config.page_information.alloc_regions[allocate_index] = Some(AllocateRegion {
                flags_set: pages_attr,
                flags_clear: alloc_regions_flags_clear,
                start_index_page: start_mem_page,
                pages: calc_alloc_pages(aligned_app_mem_size),
            });

            let last_page = start_mem_page + calc_alloc_pages(aligned_app_mem_size);
            let mut sram_page_table = self.pt.borrow_mut();
            for page_index in start_mem_page..=last_page {
                // Reset
                sram_page_table[page_index] =
                    PTEntry::new(sram_page_table[page_index].address(), pages_attr);
                config.ram_regions += 1;
            }

            config
                .page_information
                .set_ram(start_mem_page, config.ram_regions);
            config.page_information.last_page_owned = last_page_app_mem;
            config.page_information.kernel_first_page = last_page_app_mem - kernel_alloc_pages;
            config.page_information.app_ram_region = allocate_index;
            Some((aligned_address_app as *const u8, total_memory_size))
        }
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        _permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        // Given how x86 page are tied to a 4k page app memory can't include
        // parts in the same page, check if new break is lurking to kernel page.
        // Depending on App memory grants is the memory waste on kernel assigned page.
        let page_in_app_break = calc_page_index(app_memory_break as usize);

        let page_in_kernel_break = calc_page_index(kernel_memory_break as usize);

        // Last page currently owned should include the ram it self to correctly calculate
        // we have moved from the last page app currently owns
        let last_page_currently =
            config.page_information.get_ram() + config.page_information.ram_pages - 1;
        let num_of_ram_pages = page_in_app_break - config.page_information.get_ram();

        // Check for boundaries on last page we had assigned as well as it doesn't pass
        // user request to a kernel owned page

        if (app_memory_break as usize) > (kernel_memory_break as usize)
            || (page_in_app_break >= page_in_kernel_break)
            || page_in_kernel_break > config.page_information.last_page_owned
        {
            return Err(());
        }
        // Now lets check if there are changes which will trigger a reconfig
        if last_page_currently != page_in_app_break
            || num_of_ram_pages != config.page_information.ram_pages
        {
            if let Some(r) = config.page_information.alloc_regions
                [config.page_information.app_ram_region]
                .as_mut()
            {
                r.pages = num_of_ram_pages;
            }
            config.page_information.ram_pages = num_of_ram_pages;
            config.page_information.kernel_first_page = page_in_kernel_break;
        }

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        self.config_pages.map(|current_config| {
            unsafe {
                let mut sram_page_table = self.pt.borrow_mut();
                for r in current_config.alloc_regions.iter().flatten() {
                    let init_region_page = r.start_index_page;
                    let last_region_page = init_region_page + r.pages;
                    for page_index in init_region_page..=last_region_page {
                        // Reset using the same Address setting flags
                        sram_page_table[page_index] =
                            PTEntry::new(sram_page_table[page_index].address(), r.flags_clear);
                    }
                }

                // Moving to a single operation so it can refresh TLB's at the same time
                tlb::flush_all();
            }
        });
        // Now set the current config as the one being used in the app id
        self.config_pages.put(config.page_information);

        self.config_pages.map(|app_config| {
            unsafe {
                let mut sram_page_table = self.pt.borrow_mut();
                for r in app_config.alloc_regions.iter().flatten() {
                    let init_region_page = r.start_index_page;
                    let last_region_page = init_region_page + r.pages;
                    for page_index in init_region_page..=last_region_page {
                        // Reset using the same Address setting flags
                        sram_page_table[page_index] =
                            PTEntry::new(sram_page_table[page_index].address(), r.flags_set);
                    }
                }
                // Moving to a single operation so it can refresh TLB's at the same time
                tlb::flush_all();
            }
        });
    }
}
