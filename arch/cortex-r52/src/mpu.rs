// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! PMSAv8 Memory Protection Unit support for Cortex-R52.
//!
//! This module is currently a skeleton copied from Cortex M and does nothing.

// TODO: Implement PMSAv8 MPU register interface and kernel::platform::mpu::MPU trait.

use core::cmp;

use kernel::platform::mpu;

pub struct MPU {}

unsafe impl mpu::MPU for MPU {
    type MpuConfig = kernel::platform::mpu::MpuConfigDefault;

    fn enable_app_mpu(&self) {
        //sh::p!("++++ Enabling app MPU\n");
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        //self.registers
        //    .ctrl
        //    .write(Control::ENABLE::SET + Control::HFNMIENA::CLEAR + Control::PRIVDEFENA::SET);
    }

    unsafe fn disable_app_mpu(&self) {
        //sh::p!("++++ Disabling app MPU\n");
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        //self.registers.ctrl.write(Control::ENABLE::CLEAR);
    }

    fn number_total_regions(&self) -> usize {
        //sh::p!("++++ Number of total regions\n");
        //self.registers.mpu_type.read(Type::DREGION) as usize
        0
    }

    fn new_config(&self) -> Option<Self::MpuConfig> {
        //sh::p!("++++ Creating new MPU config\n");
        //let id = self.config_count.get();
        //self.config_count.set(id.checked_add(1)?);
        //
        //// Allocate the regions with index `0` first, then use `reset_config` to
        //// write the properly-indexed `CortexMRegion`s:
        //let mut ret = CortexMConfig {
        //    id,
        //    regions: [CortexMRegion::empty(0); NUM_REGIONS],
        //    is_dirty: Cell::new(true),
        //};
        //
        //self.reset_config(&mut ret);

        Some(Self::MpuConfig {})
    }

    fn reset_config(&self, _config: &mut Self::MpuConfig) {
        //sh::p!("++++ Resetting MPU config\n");
        //for i in 0..NUM_REGIONS {
        //    config.regions[i] = CortexMRegion::empty(i);
        //}
        //
        //config.is_dirty.set(true);
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        _min_region_size: usize,
        _permissions: mpu::Permissions,
        _config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        //sh::p!("++++ Allocating MPU region\n");
        Some(mpu::Region::new(
            unallocated_memory_start,
            unallocated_memory_size,
        ))

        /*
        // Check that no previously allocated regions overlap the unallocated memory.
        for region in config.regions.iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        let region_num = config.unused_region_number()?;

        // Logical region
        let mut start = unallocated_memory_start as usize;
        let mut size = min_region_size;

        // Region start always has to align to minimum region size bytes
        start = start.next_multiple_of(MIN_REGION_SIZE);

        // Regions must be at least minimum region size bytes
        if size < MIN_REGION_SIZE {
            size = MIN_REGION_SIZE;
        }

        // Physical MPU region (might be larger than logical region if some subregions are disabled)
        let mut region_start = start;
        let mut region_size = size;
        let mut subregions = None;

        // We can only create an MPU region if the size is a power of two and it divides
        // the start address. If this is not the case, the first thing we try to do to
        // cover the memory region is to use a larger MPU region and expose certain subregions.
        if size.count_ones() > 1 || !start.is_multiple_of(size) {
            // Which (power-of-two) subregion size would align with the start
            // address?
            //
            // We find this by taking smallest binary substring of the start
            // address with exactly one bit:
            //
            //      1 << (start.trailing_zeros())
            let subregion_size = {
                let tz = start.trailing_zeros();
                if tz < 32 {
                    // Find the largest power of two that divides `start`
                    1_usize << tz
                } else {
                    // This case means `start` is 0.
                    let mut ceil = (size as u32).next_power_of_two() as usize;
                    if ceil < 256 {
                        ceil = 256
                    }
                    ceil / 8
                }
            };

            // Once we have a subregion size, we get a region size by
            // multiplying it by the number of subregions per region.
            let underlying_region_size = subregion_size * 8;

            // Finally, we calculate the region base by finding the nearest
            // address below `start` that aligns with the region size.
            let underlying_region_start = start - (start % underlying_region_size);

            // If `size` doesn't align to the subregion size, extend it.
            size = size.next_multiple_of(subregion_size);

            let end = start + size;
            let underlying_region_end = underlying_region_start + underlying_region_size;

            // To use subregions, the region must be at least 256 bytes. Also, we need
            // the amount of left over space in the region after `start` to be at least as
            // large as the memory region we want to cover.
            if subregion_size >= 32 && underlying_region_end >= end {
                // The index of the first subregion to activate is the number of
                // regions between `region_start` (MPU) and `start` (memory).
                let min_subregion = (start - underlying_region_start) / subregion_size;

                // The index of the last subregion to activate is the number of
                // regions that fit in `len`, plus the `min_subregion`, minus one
                // (because subregions are zero-indexed).
                let max_subregion = min_subregion + size / subregion_size - 1;

                region_start = underlying_region_start;
                region_size = underlying_region_size;
                subregions = Some((min_subregion, max_subregion));
            } else {
                // In this case, we can't use subregions to solve the alignment
                // problem. Instead, we round up `size` to a power of two and
                // shift `start` up in memory to make it align with `size`.
                size = (size as u32).next_power_of_two() as usize;
                start += size - (start % size);

                region_start = start;
                region_size = size;
            }

        }

        // Check that our logical region fits in memory.
        if start + size > (unallocated_memory_start as usize) + unallocated_memory_size {
            return None;
        }

        let region = CortexMRegion::new(
            start as *const u8,
            size,
            region_start as *const u8,
            region_size,
            region_num,
            subregions,
            permissions,
        )?;

        config.regions[region_num] = region;
        config.is_dirty.set(true);

        Some(mpu::Region::new(start as *const u8, size))
        */
    }

    fn remove_memory_region(
        &self,
        _region: mpu::Region,
        _config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        //sh::p!("++++ Removing MPU region\n");
        /*
        let (idx, _r) = config
            .regions
            .iter()
            .enumerate()
            .find(|(_idx, r)| **r == region)
            .ok_or(())?;

        if idx <= APP_MEMORY_REGION_MAX_NUM {
            return Err(());
        }

        config.regions[idx] = CortexMRegion::empty(idx);
        config.is_dirty.set(true);
        */
        Ok(())
    }

    // When allocating memory for apps, we use two regions, each a power of two
    // in size. By using two regions we halve their size, and also halve their
    // alignment restrictions.
    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        _unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        _permissions: mpu::Permissions,
        _config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        //sh::p!("++++ Allocating app memory region\n");

        // Make sure there is enough memory for app memory and kernel memory.
        let memory_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );

        // Size must be a power of two, so:
        // https://www.youtube.com/watch?v=ovo6zwv6DX4.
        let memory_size_po2 = (memory_size as u32).next_power_of_two() as usize;

        Some((unallocated_memory_start, memory_size_po2))

        /*
        // Check that no previously allocated regions overlap the unallocated
        // memory.
        for region in config.regions.iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        // Make sure there is enough memory for app memory and kernel memory.
        let memory_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );

        // Size must be a power of two, so:
        // https://www.youtube.com/watch?v=ovo6zwv6DX4.
        let mut memory_size_po2 = (memory_size as u32).next_power_of_two() as usize;
        let exponent = math::log_base_two(memory_size_po2 as u32);

        // Check for compliance with the constraints of the MPU.
        if exponent < 9 {
            // Region sizes must be 256 bytes or larger to support subregions.
            // Since we are using two regions, and each must be at least 256
            // bytes, we need the entire memory region to be at least 512 bytes.
            memory_size_po2 = 512;
        } else if exponent > 32 {
            // Region sizes must be 4GB or smaller.
            return None;
        }

        // Region size is the actual size the MPU region will be set to, and is
        // half of the total power of two size we are allocating to the app.
        let mut region_size = memory_size_po2 / 2;

        // The region should start as close as possible to the start of the
        // unallocated memory.
        let mut region_start = unallocated_memory_start as usize;

        // If the start and length don't align, move region up until it does.
        region_start = region_start.next_multiple_of(region_size);

        // We allocate two MPU regions exactly over the process memory block,
        // and we disable subregions at the end of this region to disallow
        // access to the memory past the app break. As the app break later
        // increases, we will be able to linearly grow the logical region
        // covering app-owned memory by enabling more and more subregions. The
        // Cortex-M MPU supports 8 subregions per region, so the size of this
        // logical region is always a multiple of a sixteenth of the MPU region
        // length.

        // Determine the number of subregions to enable.
        // Want `round_up(app_memory_size / subregion_size)`.
        let mut num_enabled_subregions = initial_app_memory_size * 8 / region_size + 1;

        let subregion_size = region_size / 8;

        // Calculates the end address of the enabled subregions and the initial
        // kernel memory break.
        let subregions_enabled_end = region_start + num_enabled_subregions * subregion_size;
        let kernel_memory_break = region_start + memory_size_po2 - initial_kernel_memory_size;

        // If the last subregion covering app-owned memory overlaps the start of
        // kernel-owned memory, we make the entire process memory block twice as
        // big so there is plenty of space between app-owned and kernel-owned
        // memory.
        if subregions_enabled_end > kernel_memory_break {
            memory_size_po2 *= 2;
            region_size *= 2;

            region_start = region_start.next_multiple_of(region_size);

            num_enabled_subregions = initial_app_memory_size * 8 / region_size + 1;
        }

        // Make sure the region fits in the unallocated memory.
        if region_start + memory_size_po2
            > (unallocated_memory_start as usize) + unallocated_memory_size
        {
            return None;
        }

        // Get the number of subregions enabled in each of the two MPU regions.
        let num_enabled_subregions0 = cmp::min(num_enabled_subregions, 8);
        let num_enabled_subregions1 = num_enabled_subregions.saturating_sub(8);

        let region0 = CortexMRegion::new(
            region_start as *const u8,
            region_size,
            region_start as *const u8,
            region_size,
            0,
            Some((0, num_enabled_subregions0 - 1)),
            permissions,
        )?;

        // We cannot have a completely unused MPU region
        let region1 = if num_enabled_subregions1 == 0 {
            CortexMRegion::empty(1)
        } else {
            CortexMRegion::new(
                (region_start + region_size) as *const u8,
                region_size,
                (region_start + region_size) as *const u8,
                region_size,
                1,
                Some((0, num_enabled_subregions1 - 1)),
                permissions,
            )?
        };

        config.regions[0] = region0;
        config.regions[1] = region1;
        config.is_dirty.set(true);

        Some((region_start as *const u8, memory_size_po2))
        */
    }

    fn update_app_memory_region(
        &self,
        _app_memory_break: *const u8,
        _kernel_memory_break: *const u8,
        _permissions: mpu::Permissions,
        _config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        //sh::p!("++++ Updating app memory region\n");
        /*
        // Get first region, or error if the process tried to update app memory
        // MPU region before it was created.
        let (region_start_ptr, region_size) = config.regions[0].location().ok_or(())?;
        let region_start = region_start_ptr as usize;

        let app_memory_break = app_memory_break as usize;
        let kernel_memory_break = kernel_memory_break as usize;

        // Out of memory
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        // Number of bytes the process wants access to.
        let app_memory_size = app_memory_break - region_start;

        // There are eight subregions for every region in the Cortex-M3/4 MPU.
        let subregion_size = region_size / 8;

        // Determine the number of subregions to enable.
        // Want `round_up(app_memory_size / subregion_size)`.
        let num_enabled_subregions = app_memory_size.div_ceil(subregion_size);

        let subregions_enabled_end = region_start + subregion_size * num_enabled_subregions;

        // If we can no longer cover app memory with an MPU region without
        // overlapping kernel memory, we fail.
        if subregions_enabled_end > kernel_memory_break {
            return Err(());
        }

        // Get the number of subregions enabled in each of the two MPU regions.
        let num_enabled_subregions0 = cmp::min(num_enabled_subregions, 8);
        let num_enabled_subregions1 = num_enabled_subregions.saturating_sub(8);

        let region0 = CortexMRegion::new(
            region_start as *const u8,
            region_size,
            region_start as *const u8,
            region_size,
            0,
            Some((0, num_enabled_subregions0 - 1)),
            permissions,
        )
        .ok_or(())?;

        let region1 = if num_enabled_subregions1 == 0 {
            CortexMRegion::empty(1)
        } else {
            CortexMRegion::new(
                (region_start + region_size) as *const u8,
                region_size,
                (region_start + region_size) as *const u8,
                region_size,
                1,
                Some((0, num_enabled_subregions1 - 1)),
                permissions,
            )
            .ok_or(())?
        };

        config.regions[0] = region0;
        config.regions[1] = region1;
        config.is_dirty.set(true);
        */
        Ok(())
    }

    unsafe fn configure_mpu(&self, _config: &Self::MpuConfig) {
        //sh::p!("++++ Configuring MPU");
        /*
        // If the hardware is already configured for this app and the app's MPU
        // configuration has not changed, then skip the hardware update.
        if !self.hardware_is_configured_for.contains(&config.id) || config.is_dirty.get() {
            // Set MPU regions
            for region in config.regions.iter() {
                self.registers.rbar.write(region.base_address());
                self.registers.rasr.write(region.attributes());
            }
            self.hardware_is_configured_for.set(config.id);
            config.is_dirty.set(false);
        }
        */
    }
}
