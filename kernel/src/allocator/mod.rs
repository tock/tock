use core::{cmp, fmt::Display, ptr::NonNull};

use cortexm_mpu::CortexMRegion;
use flux_support::{max_ptr, max_usize, FluxPtrU8, FluxPtrU8Mut, RArray, FluxPtrExt};

use crate::{
    platform::mpu, process::{Error, ProcessCustomGrantIdentifier}, utilities::math
};

pub(crate) mod cortexm_mpu;
pub use cortexm_mpu::MPU;

pub type MPU8 = MPU<8>;
impl IntoCortexMPU for MPU8 {
    fn into_cortex_mpu(&self) -> CortexMpuTypes {
        CortexMpuTypes::Eight(self)
    }
}

pub type MPU16 = MPU<16>;
impl IntoCortexMPU for MPU16 {
    fn into_cortex_mpu(&self) -> CortexMpuTypes {
        CortexMpuTypes::Sixteen(self)
    }
}

pub enum CortexMpuTypes<'a> {
    Sixteen(&'a cortexm_mpu::MPU<16>),
    Eight(&'a cortexm_mpu::MPU<8>),
}

pub trait IntoCortexMPU {
    fn into_cortex_mpu(&self) -> CortexMpuTypes;
}

// VTOCK-TODO: NUM_REGIONS currently fixed to 8. Need to also handle 16
flux_rs::defs! {

    fn region_can_access(region: CortexMRegion, start: int, end: int, perms: mpu::Permissions) -> bool {
        // region set
        region.set &&
        // region's accesible block contains the start..end (exclusive) checked
        start >= region.astart &&
        end <= region.astart + region.asize &&
        // and perms are correct
        region.perms == perms
    }

    fn region_cant_access_at_all(region: CortexMRegion, start: int, end: int) -> bool {
        // WHY is this different than !region_can_access:
        //  1. We don't want to talk about permissions at all here - it shouldn't be allocated at all
        //  2. region_can_access talks about everything from start..(start + size) being
        //  included in one region. However, here we want to say that there is no subslice of
        //  start..(start + size) that is accessible via the current region we are looking at
        let region_start = region.astart;
        let region_end = region.astart + region.asize;
        // Either the region is not set
        !region.set ||
        // or NO slice of start..(start + size) is included in the region
        // i.e. the start..end is entirely before the region start
        !(region_start < start && start < region_end)
    }

    fn app_regions_can_access_flash(regions: RArray<CortexMRegion>, fstart: int, fend: int) -> bool {
        let flash_region = map_select(regions, FLASH_REGION_NUMBER);
        region_can_access(flash_region, fstart, fend, mpu::Permissions { r: true, w: false, x: true }) 
    }

    fn app_regions_can_access_ram(regions: RArray<CortexMRegion>, astart: int, aend: int) -> bool {
        let ram_region = map_select(regions, RAM_REGION_NUMBER);
        region_can_access(ram_region, astart, aend, mpu::Permissions { r: true, w: true, x: false })
    }

    fn app_regions_cant_access_at_all(regions: RArray<CortexMRegion>, start: int, end: int) -> bool {
        forall i in 0..8 {
            region_cant_access_at_all(map_select(regions, i), start, end)
        }
    }

    fn regions_overlap(region1: CortexMRegion, region2: CortexMRegion) -> bool {
        if region1.set && region2.set {
            let fst_region_start = region1.rstart;
            let fst_region_end = region1.rstart + region1.rsize;
            let snd_region_start = region2.rstart;
            let snd_region_end = region2.rstart + region2.rsize;
            fst_region_start < snd_region_end && snd_region_start < fst_region_end
        } else {
            false
        }
    }

    fn no_app_regions_overlap(regions: RArray<CortexMRegion>) -> bool {
        forall i in 0..8 {
            forall j in 0..8 {
                (i != j) => (
                    !regions_overlap(map_select(regions, i), map_select(regions, j))
                )
            }
        }
    }

    fn rnum(region: CortexMRegion) -> int { region.region_no}
    fn rbar(region: CortexMRegion) -> bitvec<32>{ region.rbar.value }
    fn rasr(region: CortexMRegion) -> bitvec<32> { region.rasr.value }

    fn set(region: CortexMRegion) -> bool { region.set }
    fn astart(region: CortexMRegion) -> int { region.astart }
    fn asize(region: CortexMRegion) -> int { region.asize }
}

const MIN_REGION_SIZE: usize = 32;

pub(crate) enum AllocateAppMemoryError {
    HeapError,
    FlashError,
}

#[derive(Debug, Clone, Copy)]
#[flux_rs::refined_by(
    memory_start: int,
    memory_size: int,
    app_break: int, 
    high_water_mark: int, 
    kernel_break: int, 
    flash_start: int, 
    flash_size: int
)]
pub(crate) struct AppBreaks {
    #[field(FluxPtrU8[memory_start])]
    pub memory_start: FluxPtrU8,
    #[field(usize[memory_size])]
    pub memory_size: usize,
    #[field(FluxPtrU8[app_break])]
    pub app_break: FluxPtrU8,
    #[field(FluxPtrU8[high_water_mark])]
    pub high_water_mark: FluxPtrU8,
    #[field(FluxPtrU8[kernel_break])]
    pub kernel_break: FluxPtrU8,
    #[field(FluxPtrU8[flash_start])]
    pub flash_start: FluxPtrU8,
    #[field(usize[flash_size])]
    pub flash_size: usize,
}

const RAM_REGION_NUMBER: usize = 0;
const FLASH_REGION_NUMBER: usize = 1;

#[flux_rs::refined_by(regions: Map<int, CortexMRegion>, breaks: AppBreaks)]
#[flux_rs::invariant(
    app_regions_can_access_flash(regions, breaks.flash_start, breaks.flash_start + breaks.flash_size) &&
    app_regions_can_access_ram(regions, breaks.memory_start, breaks.app_break) &&
    app_regions_cant_access_at_all(regions, 0, breaks.flash_start - 1) &&
    app_regions_cant_access_at_all(regions, breaks.flash_start + breaks.flash_size, breaks.memory_start - 1) &&
    app_regions_cant_access_at_all(regions, breaks.app_break + 1, u32::MAX) &&
    no_app_regions_overlap(regions)
)]
#[flux_rs::invariant(breaks.memory_start + breaks.memory_size <= u32::MAX)]
#[flux_rs::invariant(breaks.kernel_break < breaks.memory_start + breaks.memory_size)]
#[flux_rs::invariant(breaks.flash_start + breaks.flash_size < breaks.memory_start)]
#[flux_rs::invariant(breaks.app_break >= breaks.high_water_mark)]
#[flux_rs::invariant(breaks.app_break <= breaks.kernel_break)]
#[flux_rs::invariant(breaks.high_water_mark >= breaks.memory_start)]
pub(crate) struct AppMemoryAllocator {
    #[field(AppBreaks[breaks])]
    pub breaks: AppBreaks,
    #[field(RArray<CortexMRegion>[regions])]
    pub regions: RArray<CortexMRegion>,
}

impl Display for AppMemoryAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\r\n MPU")?;
        for region in self.regions.iter() {
            write!(f, "{}", region)?;
        }
        write!(f, "\r\n")
    }
}

impl AppMemoryAllocator {
    #[flux_rs::sig(fn () -> RArray<CortexMRegion>{regions: app_regions_cant_access_at_all(regions, 0, 0xFFFF_FFFF)})]
    fn new_regions() -> RArray<CortexMRegion> {
        // let regions = core::array::from_fn(|i| CortexMRegion::default(i));
        let regions = [CortexMRegion::empty(0); 8];
        let mut regions = RArray::new(regions);

        regions.set(0, CortexMRegion::empty(0));
        regions.set(1, CortexMRegion::empty(1));
        regions.set(2, CortexMRegion::empty(2));
        regions.set(3, CortexMRegion::empty(3));
        regions.set(4, CortexMRegion::empty(4));
        regions.set(5, CortexMRegion::empty(5));
        regions.set(6, CortexMRegion::empty(6));
        regions.set(7, CortexMRegion::empty(7));
        // let mut i = 0;
        // while i < regions.len() {
        //     regions.set(i, CortexMRegion::empty(i));
        //     i += 1;
        // }
        regions
    }

    // #[flux_rs::sig(fn (self: &strg Self) ensures self: Self)]
    // pub(crate) fn reset(&mut self) {
    //     // for (i, r) in self.regions.iter_mut().enumerate() {
    //     //     *r = <M as mpu::MPU>::Region::default(i)
    //     // }
    //     let mut i = 0;
    //     while i < self.regions.len() {
    //         self.regions.set(i, CortexMRegion::empty(i));
    //         i += 1;
    //     }
    // }

    #[flux_rs::sig(fn (&Self[@b]) -> FluxPtrU8[b.breaks.flash_start])]
    pub(crate) fn flash_start(&self) -> FluxPtrU8 {
        self.breaks.flash_start
    }

    #[flux_rs::sig(fn (&Self[@b]) -> FluxPtrU8[b.breaks.flash_start + b.breaks.flash_size])]
    pub(crate) fn flash_end(&self) -> FluxPtrU8 {
        self.breaks.flash_start.wrapping_add(self.breaks.flash_size)
    }

    #[flux_rs::sig(fn (&Self[@b]) -> FluxPtrU8[b.breaks.memory_start])]
    pub(crate) fn memory_start(&self) -> FluxPtrU8 {
        self.breaks.memory_start
    }

    #[flux_rs::sig(fn (&Self[@b]) -> usize[b.breaks.memory_size])]
    pub(crate) fn memory_size(&self) -> usize {
        self.breaks.memory_size
    }
    
    #[flux_rs::sig(fn (&Self[@b]) -> FluxPtrU8[b.breaks.memory_start + b.breaks.memory_size])]
    pub(crate) fn memory_end(&self) -> FluxPtrU8 {
        self.breaks.memory_start.wrapping_add(self.breaks.memory_size)
    }

    #[flux_rs::sig(fn (&Self[@b]) -> FluxPtrU8[b.breaks.app_break])]
    pub(crate) fn app_break(&self) -> FluxPtrU8 {
        self.breaks.app_break
    }

    #[flux_rs::sig(fn (&Self[@b]) -> FluxPtrU8[b.breaks.kernel_break])]
    pub(crate) fn kernel_break(&self) -> FluxPtrU8 {
        self.breaks.kernel_break
    }

    #[flux_rs::sig(fn (&Self[@b], start: FluxPtrU8, end: FluxPtrU8) -> bool[end >= start && start >= b.breaks.memory_start && end <= b.breaks.app_break])]
    pub(crate) fn in_app_ram_memory(&self, start: FluxPtrU8, end: FluxPtrU8) -> bool {
        end >= start && start >= self.breaks.memory_start && end <= self.breaks.app_break
    }

    #[flux_rs::sig(fn (&Self[@b], start: FluxPtrU8, end: FluxPtrU8) -> bool[end >= start && start >= b.breaks.flash_start && end <= b.breaks.flash_start + b.breaks.flash_size])]
    pub(crate) fn in_app_flash_memory(&self, start: FluxPtrU8, end: FluxPtrU8) -> bool {
        end >= start
            && start >= self.breaks.flash_start
            && end <= self.breaks.flash_start.wrapping_add(self.breaks.flash_size)
    }

    #[flux_rs::sig(fn (self: &strg Self, _, _) -> Result<(), ()> ensures self: Self)]
    pub(crate) fn add_shared_readonly_buffer(
        &mut self,
        buf_start_addr: FluxPtrU8Mut,
        size: usize,
    ) -> Result<(), ()> {
        let buf_end_addr = buf_start_addr.wrapping_add(size);
        if self.in_app_ram_memory(buf_start_addr, buf_end_addr) {
            // TODO: Check for buffer aliasing here
            // Valid buffer, we need to adjust the app's watermark
            // note: `in_app_owned_memory` ensures this offset does not wrap
            let new_water_mark = max_ptr(self.breaks.high_water_mark, buf_end_addr);
            self.breaks.high_water_mark = new_water_mark;
            Ok(())
        } else if self.in_app_flash_memory(buf_start_addr, buf_end_addr) {
            Ok(())
        } else {
            Err(())
        }
    }

    #[flux_rs::sig(fn (self: &strg Self, _, _) -> Result<(), ()> ensures self: Self)]
    pub(crate) fn add_shared_readwrite_buffer(
        &mut self,
        buf_start_addr: FluxPtrU8Mut,
        size: usize,
    ) -> Result<(), ()> {
        // let breaks = &mut self.breaks.ok_or(())?;
        let buf_end_addr = buf_start_addr.wrapping_add(size);
        if self.in_app_ram_memory(buf_start_addr, buf_end_addr) {
            // TODO: Check for buffer aliasing here
            // Valid buffer, we need to adjust the app's watermark
            // note: `in_app_owned_memory` ensures this offset does not wrap
            let new_water_mark = max_ptr(self.breaks.high_water_mark, buf_end_addr);
            self.breaks.high_water_mark = new_water_mark;
            Ok(())
        } else {
            Err(())
        }
    }

    #[flux_rs::sig(fn (self: &strg Self, _, _) -> Result<_, _> ensures self: Self)]
    pub(crate) fn allocate_custom_grant(&mut self, size: usize, align: usize) -> Result<(ProcessCustomGrantIdentifier, NonNull<u8>), ()> {
        let ptr = self.allocate_in_grant_region_internal(size, align).ok_or(())?;
        let custom_grant_address = ptr.as_usize();
        let process_memory_end = self.memory_end().as_usize();

        Ok(
            (
                ProcessCustomGrantIdentifier {
                    offset: process_memory_end - custom_grant_address,
                }, 
                ptr.into())
        )
    }

    #[flux_rs::sig(
        fn (self: &strg Self[@old_bc], usize, usize) -> Option<{p. FluxPtrU8[p] | p < bc.breaks.memory_start + bc.breaks.memory_size}>[#opt] 
            ensures self: Self[#bc],
            (opt => bc.breaks.kernel_break >= bc.breaks.app_break) &&
            (!opt => bc == old_bc)
    )]
    pub(crate) fn allocate_in_grant_region_internal(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<FluxPtrU8> {
        // First, compute the candidate new pointer. Note that at this point
        // we have not yet checked whether there is space for this
        // allocation or that it meets alignment requirements.
        let new_break_unaligned = self.kernel_break().wrapping_sub(size).as_usize();

        // Our minimum alignment requirement is two bytes, so that the
        // lowest bit of the address will always be zero and we can use it
        // as a flag. It doesn't hurt to increase the alignment (except for
        // potentially a wasted byte) so we make sure `align` is at least
        // two.
        let align = max_usize(align, 2);

        // The alignment must be a power of two, 2^a. The expression
        // `!(align - 1)` then returns a mask with leading ones, followed by
        // `a` trailing zeros.
        let alignment_mask = !(align - 1);
        let new_break = (new_break_unaligned & alignment_mask).as_fluxptr();

        // Verify there is space for this allocation
        if new_break < self.app_break() || new_break > self.kernel_break() {
            None
        } else {
            // Allocation is valid.
            // The app break is precisely the end of the process
            // accessible memory so we don't need to ask the MPU
            // anything

            flux_rs::assert(new_break <= self.kernel_break());
            flux_rs::assert(self.kernel_break() < self.memory_end());
            // We always allocate down, so we must lower the
            // kernel_memory_break.
            self.set_kernel_break(new_break);

            // ### Safety
            //
            // Here we are guaranteeing that `grant_ptr` is not null. We can
            // ensure this because we just created `grant_ptr` based on the
            // process's allocated memory, and we know it cannot be null.
            Some(new_break)
        }
    }

    #[flux_rs::sig(
        fn (
            self: &strg Self[@old_app],
            { FluxPtrU8[@new_break] | new_break >= old_app.breaks.app_break && new_break < old_app.breaks.memory_start + old_app.breaks.memory_size }
        ) ensures self: Self[{breaks: AppBreaks { kernel_break: new_break, ..old_app.breaks }, ..old_app}] 
    )]
    fn set_kernel_break(&mut self, new_break: FluxPtrU8) {
        self.breaks.kernel_break = new_break;
    }

    #[flux_rs::trusted] // needs a spec for iter
    #[flux_rs::sig(fn (&Self) -> Option<{idx. usize[idx] | idx > 1 && idx < 8}>)]
    fn next_available_ipc_idx(&self) -> Option<usize> {
        // let mut i = 0;
        // while i < self.regions.len() {
        //     let region = self.regions.get(i);
        //     if i != FLASH_REGION_NUMBER && i != RAM_REGION_NUMBER && !region.is_set() {
        //         return Some(i);
        //     }
        //     i += 1;
        // }

        for (i, region) in self.regions.iter().enumerate() {
            if i != FLASH_REGION_NUMBER && i != RAM_REGION_NUMBER && !region.is_set() {
                return Some(i);
            }
        }
        None
    }

    #[flux_rs::sig(fn (&Self[@app], &CortexMRegion[@region]) -> bool{b: !b => !(forall i in 0..8 { regions_overlap(map_select(app.regions, i), region) }) })]
    fn any_overlaps(&self, region: &CortexMRegion) -> bool {
        region.region_overlaps(&self.regions.get(0)) ||
        region.region_overlaps(&self.regions.get(1)) ||
        region.region_overlaps(&self.regions.get(2)) ||
        region.region_overlaps(&self.regions.get(3)) ||
        region.region_overlaps(&self.regions.get(4)) ||
        region.region_overlaps(&self.regions.get(5)) ||
        region.region_overlaps(&self.regions.get(6)) ||
        region.region_overlaps(&self.regions.get(7))
    }

    #[flux_rs::trusted] // IPC is entirely broken, being rewritten and not worth verifying
    pub(crate) fn allocate_ipc_region(
        &mut self,
        start: FluxPtrU8,
        size: usize,
        permissions: mpu::Permissions,
    ) -> Result<mpu::Region, ()> {
        let region_idx = self.next_available_ipc_idx().ok_or(())?;
        let region = CortexMRegion::create_exact_region(
                region_idx, 
                start,
                size,
                permissions,
            )
            .ok_or(())?;

        // make sure new region doesn't overlap
        if self.any_overlaps(&region) {
            return Err(())
        }

        self.regions.set(region_idx, region);
        let start = region.accessible_start().ok_or(())?;
        let size = region.accessible_size().ok_or(())?;
        Ok(mpu::Region::new(start, size))
    }

    #[flux_rs::sig(
        fn (
            mem_start: FluxPtrU8,
            mem_size: usize, 
            min_mem_size: usize,
            app_mem_size: usize, 
            kernel_mem_size: usize,
            flash_start: FluxPtrU8,
            flash_size: usize, 
        ) -> Result<{app. Self[app] | 
            // b.app_break <= b.memory_start + b.memory_size - kernel_mem_size &&
            // b.app_break >= b.memory_start + app_mem_size &&
            app.breaks.memory_start >= mem_start &&
            app.breaks.memory_start + app.breaks.memory_size <= u32::MAX &&
            app.breaks.memory_start > 0 &&
            app.breaks.kernel_break == app.breaks.memory_start + app.breaks.memory_size - kernel_mem_size
            // app_can_access_flash(app, flash_start, flash_size) &&
            // app_can_access_ram(app, b.memory_start, b.app_break) &&
            // app_cant_access_at_all(app, 0, flash_start - 1) &&
            // app_cant_access_at_all(app, flash_start + flash_size, b.memory_start - 1) &&
            // app_cant_access_at_all(app, b.app_break + 1, u32::MAX)
        }
        , AllocateAppMemoryError>
        requires flash_start + flash_size < mem_start && min_mem_size > 0 
    )]
    #[flux_rs::trusted] // for now
    pub(crate) fn new_app_alloc(
        unallocated_memory_start: FluxPtrU8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        flash_start: FluxPtrU8,
        flash_size: usize,
    ) -> Result<Self, AllocateAppMemoryError> {
        let mut app_regions = Self::new_regions();

        // get our flash region
        let flash_region = CortexMRegion::create_exact_region(
                FLASH_REGION_NUMBER,
                flash_start,
                flash_size,
                mpu::Permissions::ReadExecuteOnly,
            )
            .ok_or(AllocateAppMemoryError::FlashError)?;

        app_regions.set(FLASH_REGION_NUMBER, flash_region);

        // set our stack, data, and heap up
        let ideal_region_size = cmp::max(
            min_memory_size,
            initial_app_memory_size 
        );
        let region = CortexMRegion::create_bounded_region(
                RAM_REGION_NUMBER,
                unallocated_memory_start,
                unallocated_memory_size,
                ideal_region_size,
                mpu::Permissions::ReadWriteOnly,
            )
            .ok_or(AllocateAppMemoryError::HeapError)?;

        let memory_start = region
            .accessible_start()
            .ok_or(AllocateAppMemoryError::HeapError)?;
        let app_memory_size = region.accessible_size().ok_or(AllocateAppMemoryError::HeapError)?;
        let app_break = memory_start.wrapping_add(app_memory_size);

        // compute the total block size: 
        // if the process block size is too big fail
        if app_memory_size + initial_kernel_memory_size > (u32::MAX / 2 + 1) as usize {
            return Err(AllocateAppMemoryError::HeapError);
        }
        // make it a power of two to add some space between the app and the kernel regions of memory
        let total_block_size = math::closest_power_of_two_usize(app_memory_size + initial_kernel_memory_size);

        // make sure we can actually fit everything into te RAM pool
        if memory_start.wrapping_add(total_block_size) > unallocated_memory_start.wrapping_add(unallocated_memory_size) {
            // We don't have enough memory left in the RAM pool to 
            // give this process memory
            return Err(AllocateAppMemoryError::HeapError)
        }

        app_regions.set(RAM_REGION_NUMBER, region);

        // compute breaks
        let high_water_mark = memory_start;
        let kernel_break = memory_start.wrapping_add(total_block_size).wrapping_sub(initial_kernel_memory_size);
        let breaks = AppBreaks {
            memory_start,
            memory_size: total_block_size,
            app_break,
            high_water_mark,
            kernel_break,
            flash_start,
            flash_size,
        };

        Ok(
            Self {
                breaks,
                regions: app_regions
            }
        )
    }

    #[flux_rs::sig(fn (self: &strg Self[@old_app], new_app_break: FluxPtrU8) -> Result<(), Error>[#res]
        ensures self: Self { app: 
            res => (
                app_regions_can_access_flash(app.regions, app.breaks.flash_start, app.breaks.flash_start + app.breaks.flash_size) &&
                app_regions_can_access_ram(app.regions, app.breaks.memory_start, app.breaks.app_break)
            )
        // app_regions_cant_access_at_all(app.regions, 0, app.breaks.flash_start - 1)
        // app_regions_cant_access_at_all(app.regions, app.breaks.flash_start + app.breaks.flash_size, app.breaks.memory_start - 1)
        // app_regions_cant_access_at_all(app.regions, app.breaks.app_break + 1, u32::MAX)
        // no_app_regions_overlap(app.regions)
            // )
    })]
    #[flux_rs::trusted] // for now
    pub(crate) fn update_app_memory(
        &mut self,
        new_app_break: FluxPtrU8,
    ) -> Result<(), Error> {
        let memory_start = self.memory_start();
        let memory_end = self.memory_end();
        let high_water_mark = self.breaks.high_water_mark;
        let kernel_break = self.kernel_break();
        if new_app_break.as_usize() > kernel_break.as_usize() {
            return Err(Error::OutOfMemory)
        }
        if new_app_break.as_usize() <= memory_start.as_usize()
            || new_app_break.as_usize() > memory_end.as_usize()
            || new_app_break.as_usize() < high_water_mark.as_usize()
        {
            return Err(Error::AddressOutOfBounds);
        }
        let new_region_size = new_app_break.as_usize() - memory_start.as_usize();
        let new_region = CortexMRegion::adjust_region_fixed_start(
            self.memory_start(),
            self.memory_size(),
            new_region_size,
            RAM_REGION_NUMBER,
            mpu::Permissions::ReadWriteOnly,
        ).ok_or(Error::OutOfMemory)?;

        // no need to check overlap because the current region is bounded?

        let new_app_break = new_region
            .accessible_start()
            .ok_or(Error::KernelError)?
            .as_usize()
            + new_region.accessible_size().ok_or(Error::KernelError)?;
        if new_app_break > kernel_break.as_usize() {
            return Err(Error::OutOfMemory);
        }
        self.breaks.app_break = FluxPtrU8::from(new_app_break);
        self.regions.set(RAM_REGION_NUMBER, new_region);
        Ok(())
    }
}
