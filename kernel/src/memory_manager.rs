// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Memory region allocator for process loading.
//!
//! [`MemoryManager`] finds a best-fit free region in the board's memory
//! pool by querying the kernel for currently running processes.
//! Callers supply the minimum memory size from the TBF header.
//!
//! The allocator holds no state. Used memory regions are derived
//! fresh on every call to [`MemoryManager::alloc`] by scanning
//! `kernel.get_process_iter()`. Reclaim after a process fault or unload is
//! therefore automatic.

use crate::config;
use crate::debug;
use crate::kernel::Kernel;

/// A free contiguous region of memory described by its start address and
/// available size from that address to the end of the gap.
#[derive(Copy, Clone, Debug)]
struct FreeRegion {
    start: usize,
    size: usize,
}

/// Allocates memory regions for processes out of the board's app-memory pool.
/// Construct once in `main.rs` alongside the kernel and pass into
/// `SequentialProcessLoaderMachine`.
/// # Usage
///
/// ```rust
/// let ram = memory_manager
///     .alloc(process_binary.header.get_minimum_app_ram_size() as usize)
///     .ok_or(ProcessLoadError::NotEnoughMemory)?;
///
/// load_process(kernel, chip, process_binary, ram, ...);
/// ```
pub struct MemoryManager {
    app_memory: &'static [u8],
    kernel: &'static Kernel,
}

impl MemoryManager {
    pub fn new(app_memory: &'static [u8], kernel: &'static Kernel) -> Self {
        Self { app_memory, kernel }
    }

    /// Find the best-fit free region for a process that needs `required_size`
    /// bytes of memory.
    ///
    /// Active process regions are queried from the kernel on every call. 
    /// The allocated region is sized and aligned to 
    // `required_size.next_power_of_two()` to satisfy Cortex-M MPU 
    // constraints. The returned slice has exactly that length.
    ///
    /// Returns `None` if no region large enough and correctly aligned exists
    /// in the pool.
    pub(crate) fn alloc(&self, required_size: usize) -> Option<&'static mut [u8]> {
        if required_size == 0 {
            return None;
        }

        let aligned_size = required_size.next_power_of_two();

        // Collect active regions directly from the kernel.
        const MAX_PROCS: usize = 16;
        let mut active: [(usize, usize); MAX_PROCS] = [(0, 0); MAX_PROCS];
        for (i, proc) in self.kernel.get_process_iter().enumerate() {
            if i >= MAX_PROCS {
                break;
            }
            let addr = proc.get_addresses();
            active[i] = (addr.sram_start, addr.sram_end);
        }

        let best = self.find_best_fit(aligned_size, &active)?;

        if config::CONFIG.debug_load_processes {
            debug!(
                "MemoryManager: allocating {:#x} bytes at {:#010x} (requested {:#x})",
                aligned_size, best.start, required_size
            );
        }

        // `find_best_fit` guarantees:
        //   1. `best.start` is within `self.app_memory`.
        //   2. `best.start + aligned_size` does not exceed the pool end.
        //   3. The region does not overlap any active process region.
        //   4. `best.start` is aligned to `aligned_size` (power-of-two natural
        //      alignment required by the Cortex-M MPU).
        Some(unsafe {
            core::slice::from_raw_parts_mut(best.start as *mut u8, aligned_size)
        })
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Collect, sort, and deduplicate active regions, then scan gaps for the
    /// best fit at the required alignment.
    fn find_best_fit(
        &self,
        aligned_size: usize,
        active_regions: &[(usize, usize)],
    ) -> Option<FreeRegion> {
        let pool_start = self.app_memory.as_ptr() as usize;
        let pool_end = pool_start + self.app_memory.len();

        const MAX_PROCS: usize = 16;
        let mut occupied: [(usize, usize); MAX_PROCS] = [(0, 0); MAX_PROCS];
        let mut count = 0;

        for &(start, end) in active_regions {
            if start == 0 || start >= end {
                continue;
            }
            if count < MAX_PROCS {
                occupied[count] = (start, end);
                count += 1;
            }
        }

        for i in 1..count {
            let key = occupied[i];
            let mut j = i;
            while j > 0 && occupied[j - 1].0 > key.0 {
                occupied[j] = occupied[j - 1];
                j -= 1;
            }
            occupied[j] = key;
        }

        let mut best: Option<FreeRegion> = None;

        let mut try_region = |gap_start: usize, gap_end: usize| {
            if let Some(region) = self.fit_in_gap(gap_start, gap_end, aligned_size) {
                match best {
                    None => best = Some(region),
                    Some(current) if region.size < current.size => best = Some(region),
                    _ => {}
                }
            }
        };

        if count == 0 {
            // No active processes - the entire memory region is available.
            try_region(pool_start, pool_end);
        } else {
            // Gap before the first active region.
            try_region(pool_start, occupied[0].0);

            // Gaps between consecutive active regions.
            for i in 0..count - 1 {
                try_region(occupied[i].1, occupied[i + 1].0);
            }

            // Gap after the last active region.
            try_region(occupied[count - 1].1, pool_end);
        }

        if config::CONFIG.debug_load_processes {
            match best {
                Some(region) => debug!(
                    "Ram Manager: best fit at {:#010x}, gap size {:#x}, needed {:#x}",
                    region.start, region.size, aligned_size
                ),
                None => debug!(
                    "Ram Manager: no suitable region found for {:#x} bytes",
                    aligned_size
                ),
            }
        }

        best
    }

    /// Given a gap `[gap_start, gap_end)`, find the lowest aligned
    /// address within the gap.
    ///
    /// Returns `None` if the gap is too small or alignment pushes the
    /// candidate past `gap_end`
    fn fit_in_gap(&self, gap_start: usize, gap_end: usize, aligned_size: usize) -> Option<FreeRegion> {
        if gap_end <= gap_start {
            return None;
        }

        // Round gap_start up to the next multiple of aligned_size.
        let mask = aligned_size - 1; 
        let candidate_start = (gap_start + mask) & !mask;

        if candidate_start.checked_add(aligned_size)? <= gap_end {
            Some(FreeRegion {
                start: candidate_start,
                size: gap_end - candidate_start,
            })
        } else {
            None
        }
    }
}