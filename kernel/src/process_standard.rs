// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock default Process implementation.
//!
//! `ProcessStandard` is an implementation for a userspace process running on
//! the Tock kernel.

use core::cell::Cell;
use core::cmp;
use core::fmt::Write;
use core::num::NonZeroU32;
use core::ptr::NonNull;
use core::{mem, slice, str};
#[allow(clippy::wildcard_imports)]
use flux_support::*;

use crate::collections::queue::Queue;
use crate::collections::ring_buffer::RingBuffer;
use crate::config;
use crate::debug;
use crate::errorcode::ErrorCode;
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::platform::mpu::{self, MPU};
use crate::process::BinaryVersion;
use crate::process::ProcessBinary;
use crate::process::{Error, FunctionCall, FunctionCallSource, Process, Task};
use crate::process::{FaultAction, ProcessCustomGrantIdentifier, ProcessId};
use crate::process::{ProcessAddresses, ProcessSizes, ShortId};
use crate::process::{State, StoppedState};
use crate::process_checker::AcceptedCredential;
use crate::process_loading::ProcessLoadError;
use crate::process_policies::ProcessFaultPolicy;
use crate::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use crate::storage_permissions;
use crate::syscall::{self, Syscall, SyscallReturn, UserspaceKernelBoundary};
use crate::upcall::UpcallId;
use crate::utilities::cells::{MapCell, NumericCellExt, OptionalCell};

use tock_tbf::types::CommandPermissions;

/// State for helping with debugging apps.
///
/// These pointers and counters are not strictly required for kernel operation,
/// but provide helpful information when an app crashes.
struct ProcessStandardDebug {
    /// If this process was compiled for fixed addresses, save the address
    /// it must be at in flash. This is useful for debugging and saves having
    /// to re-parse the entire TBF header.
    fixed_address_flash: Option<u32>,

    /// If this process was compiled for fixed addresses, save the address
    /// it must be at in RAM. This is useful for debugging and saves having
    /// to re-parse the entire TBF header.
    fixed_address_ram: Option<u32>,

    /// Where the process has started its heap in RAM.
    app_heap_start_pointer: Option<FluxPtrU8Mut>,

    /// Where the start of the stack is for the process. If the kernel does the
    /// PIC setup for this app then we know this, otherwise we need the app to
    /// tell us where it put its stack.
    app_stack_start_pointer: Option<FluxPtrU8Mut>,

    /// How low have we ever seen the stack pointer.
    app_stack_min_pointer: Option<FluxPtrU8Mut>,

    /// How many syscalls have occurred since the process started.
    syscall_count: usize,

    /// What was the most recent syscall.
    last_syscall: Option<Syscall>,

    /// How many upcalls were dropped because the queue was insufficiently
    /// long.
    dropped_upcall_count: usize,

    /// How many times this process has been paused because it exceeded its
    /// timeslice.
    timeslice_expiration_count: usize,
}

/// Entry that is stored in the grant pointer table at the top of process
/// memory.
///
/// One copy of this entry struct is stored per grant region defined in the
/// kernel. This type allows the core kernel to lookup a grant based on the
/// driver_num associated with the grant, and also holds the pointer to the
/// memory allocated for the particular grant.
#[repr(C)]
struct GrantPointerEntry {
    /// The syscall driver number associated with the allocated grant.
    ///
    /// This defaults to 0 if the grant has not been allocated. Note, however,
    /// that 0 is a valid driver_num, and therefore cannot be used to check if a
    /// grant is allocated or not.
    driver_num: usize,

    /// The start of the memory location where the grant has been allocated, or
    /// null if the grant has not been allocated.
    grant_ptr: FluxPtrU8Mut,
}

// VTOCK-TODO: break up this struct when we have a better solution for interior mutability
// VTOCK-TODO: is it ok for app_break == kernel_break?
// kernel_memory_break > app_break && app_break <= high_water_mark
#[flux_rs::refined_by(kernel_break: int, app_break: int, allow_high_water_mark: int, mem_start: int, mem_len: int, flash_start: int, flash_len: int)]
#[flux_rs::invariant(kernel_break >= app_break)]
#[flux_rs::invariant(kernel_break < mem_start + mem_len)]
#[flux_rs::invariant(app_break >= allow_high_water_mark)]
#[flux_rs::invariant(allow_high_water_mark >= mem_start)]
#[derive(Clone, Copy)]
struct ProcessBreaks {
    /// Pointer to the end of the allocated (and MPU protected) grant region.
    #[field({FluxPtrU8Mut[kernel_break] | kernel_break >= app_break && kernel_break < mem_start + mem_len})]
    pub kernel_memory_break: FluxPtrU8Mut,
    /// Pointer to the end of process RAM that has been sbrk'd to the process.
    #[field(FluxPtrU8Mut[app_break])]
    pub app_break: FluxPtrU8Mut,
    /// Pointer to high water mark for process buffers shared through `allow`
    #[field({FluxPtrU8Mut[allow_high_water_mark] | app_break >= allow_high_water_mark && allow_high_water_mark >= mem_start})]
    pub allow_high_water_mark: FluxPtrU8Mut,
    // start of process heap (where stack ends)
    #[field(FluxPtrU8[mem_start])]
    pub mem_start: FluxPtrU8,
    // length of process memory block
    #[field(usize[mem_len])]
    pub mem_len: usize,
    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: &'static [u8],
    #[field(FlashGhostState[flash_start, flash_len])]
    _flash_ghost: FlashGhostState,
}

impl ProcessBreaks {
    #[flux_rs::sig(
        fn (self: &strg Self[@pb], FluxPtrU8Mut[@new_break]) 
            requires pb.kernel_break >= new_break && new_break >= pb.allow_high_water_mark
            ensures self: Self[{app_break: new_break, ..pb}]
    )]
    pub(crate) fn set_app_break(&mut self, new_break: FluxPtrU8Mut) {
        self.app_break = new_break;
    }

    #[flux_rs::sig(
        fn (self: &strg Self[@pb], FluxPtrU8Mut[@new_hwm]) 
            requires pb.app_break >= new_hwm && new_hwm >= pb.mem_start
            ensures self: Self[{allow_high_water_mark: new_hwm, ..pb}]
    )]
    pub(crate) fn set_high_water_mark(&mut self, new_high_water_mark: FluxPtrU8Mut) {
        self.allow_high_water_mark = new_high_water_mark;
    }

    #[flux_rs::sig(
        fn (self: &strg Self[@pb], FluxPtrU8Mut[@new_break]) 
            requires new_break >= pb.app_break && new_break <= pb.kernel_break
            ensures self: Self[{ kernel_break: new_break, ..pb }]
    )]
    pub(crate) fn set_kernel_break(&mut self, new_kernel_break: FluxPtrU8Mut) {
        self.kernel_memory_break = new_kernel_break;
    }
}

#[flux_rs::refined_by(
    kernel_break: int,
    app_break: int,
    allow_high_water_mark: int,
    mem_start: int,
    mem_len: int,
    flash_start: int, 
    flash_len: int,
    mpu_config: <<C as Chip>::MPU as MPU>::MpuConfig
)]
#[flux_rs::invariant(mem_start + mem_len <= usize::MAX)]
struct BreaksAndMPUConfig<C: 'static + Chip> {
    /// Configuration data for the MPU
    #[field({<<C as Chip>::MPU as MPU>::MpuConfig[mpu_config] | 
        app_break >= mem_start &&
        kernel_break <= mem_start + mem_len &&
        <<C as Chip>::MPU as MPU>::config_can_access_heap(mpu_config, mem_start, app_break) &&
        <<C as Chip>::MPU as MPU>::config_can_access_flash(mpu_config, flash_start, flash_len) &&
        <<C as Chip>::MPU as MPU>::config_cant_access_at_all(mpu_config, 0, flash_start) &&
        <<C as Chip>::MPU as MPU>::config_cant_access_at_all(mpu_config, flash_start + flash_len, mem_start - (flash_start + flash_len)) &&
        <<C as Chip>::MPU as MPU>::config_cant_access_at_all(mpu_config, app_break, 0xffff_ffff)
    })]
    pub mpu_config: <<C as Chip>::MPU as MPU>::MpuConfig,

    // MPU regions are saved as a pointer-size pair.
    mpu_regions: [Cell<Option<mpu::Region>>; 6], // VTOCK TODO: Need to get rid of these cells

    /// Pointers that demarcate kernel-managed regions and userspace-managed regions.
    #[field(ProcessBreaks[kernel_break, app_break, allow_high_water_mark, mem_start, mem_len, flash_start, flash_len])]
    breaks: ProcessBreaks,
}

impl<C: 'static + Chip> BreaksAndMPUConfig<C> {
    #[flux_rs::sig(
        fn (
            self: &strg Self[@bc],
            FluxPtrU8Mut,
            &mut <C as Chip>::MPU
        ) -> Result<FluxPtrU8Mut[bc.app_break], Error>[#res]
            ensures self: Self {new_bc: 
                new_bc.mem_start == bc.mem_start &&
                new_bc.mem_len == bc.mem_len &&
                new_bc.flash_start == bc.flash_start &&
                new_bc.flash_len == bc.flash_len
                // new_bc.kernel_break == bc.kernel_break
                // &&
                // new_bc.app_break >= new_bc.allow_high_water_mark &&
                // new_bc.app_break <= new_bc.kernel_break  &&
                // new_bc.kernel_break < new_bc.mem_start + new_bc.mem_len &&
                // new_bc.allow_high_water_mark >= new_bc.mem_start &&
                // (res => 
                //     <<C as Chip>::MPU as MPU>::config_can_access_heap(new_bc.mpu_config, new_bc.mem_start, new_bc.app_break) &&
                //     <<C as Chip>::MPU as MPU>::config_can_access_flash(new_bc.mpu_config, new_bc.flash_start, new_bc.flash_len) &&
                //     <<C as Chip>::MPU as MPU>::config_cant_access_at_all(new_bc.mpu_config, 0, new_bc.flash_start) &&
                //     <<C as Chip>::MPU as MPU>::config_cant_access_at_all(new_bc.mpu_config, new_bc.flash_start + new_bc.flash_len, new_bc.mem_start - (new_bc.flash_start + new_bc.flash_len)) &&
                //     <<C as Chip>::MPU as MPU>::config_cant_access_at_all(new_bc.mpu_config, new_bc.app_break, 0xffff_ffff)
                // ) &&
                // (!res => new_bc == bc) // WTF :(
            }
    )]
    pub(crate) fn brk(
        &mut self,
        new_break: FluxPtrU8Mut,
        mpu: &mut <C as Chip>::MPU,
    ) -> Result<FluxPtrU8Mut, Error> {
        if new_break < self.breaks.allow_high_water_mark || new_break >= self.mem_end() {
            Err(Error::AddressOutOfBounds)
        } else if new_break > self.breaks.kernel_memory_break {
            Err(Error::OutOfMemory)
        } else if let Ok(mpu_breaks) = mpu.update_app_memory_regions(
            self.breaks.mem_start,
            new_break,
            self.breaks.kernel_memory_break,
            self.flash_start(),
            self.flash_size(),
            &mut self.mpu_config,
        ) {
            let old_break = self.breaks.app_break;
            self.breaks.set_app_break(mpu_breaks.app_break);
            mpu.configure_mpu(&self.mpu_config);
            Ok(old_break)
        } else {
            // MPU could not allocate a region without overlapping kernel memory
            Err(Error::OutOfMemory)
        }
    }

    #[flux_rs::sig(fn (&Self[@pb]) -> FluxPtrU8[pb.mem_start + pb.mem_len])]
    fn mem_end(&self) -> FluxPtrU8 {
        self.breaks.mem_start.wrapping_add(self.breaks.mem_len)
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn (&Self[@pb]) -> FluxPtrU8[pb.flash_start])]
    fn flash_start(&self) -> FluxPtrU8 {
        self.breaks.flash.as_fluxptr()
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn (&Self[@pb]) -> usize[pb.flash_len])]
    fn flash_size(&self) -> usize {
        self.breaks.flash.len()
    }

    #[flux_rs::sig(
        fn (self: &strg Self[@bc], FluxPtrU8Mut[@buf_start], usize[@size]) -> Result<(), ()>[#res]
            ensures self: Self {new_bc: 
                new_bc.mem_start == bc.mem_start &&
                new_bc.mem_len == bc.mem_len &&
                new_bc.flash_start == bc.flash_start &&
                new_bc.flash_len == bc.flash_len 
            }
    )]
    pub(crate) fn build_readwrite_process_buffer(
        &mut self,
        buf_start_addr: FluxPtrU8Mut,
        size: usize,
    ) -> Result<(), ()> {
        // Check that buffer is in app owned memory
        let buf_end_addr = buf_start_addr.wrapping_add(size);
        if self.in_app_owned_memory(buf_start_addr, buf_end_addr) {
            // TODO: Check for buffer aliasing here
            // Valid buffer, we need to adjust the app's watermark
            // note: `in_app_owned_memory` ensures this offset does not wrap
            let new_water_mark = max_ptr(self.breaks.allow_high_water_mark, buf_end_addr);

            self.breaks.set_high_water_mark(new_water_mark);
            Ok(())
        } else {
            Err(())
        }
    }

    #[flux_rs::sig(
        fn (self: &strg Self[@old_bc], usize, usize) -> Option<NonNull<u8>>[#opt] 
            ensures self: Self {bc: 
                (opt => bc.kernel_break >= bc.app_break) &&
                (!opt => bc.kernel_break == old_bc.kernel_break) &&
                bc.mem_start == old_bc.mem_start &&
                bc.mem_len == old_bc.mem_len &&
                bc.flash_start == old_bc.flash_start &&
                bc.flash_len == old_bc.flash_len
            }
    )]
    pub(crate) fn allocate_in_grant_region_internal(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<NonNull<u8>> {
        // First, compute the candidate new pointer. Note that at this point
        // we have not yet checked whether there is space for this
        // allocation or that it meets alignment requirements.
        let new_break_unaligned = self.breaks.kernel_memory_break.wrapping_sub(size);

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
        let new_break = (new_break_unaligned.as_usize() & alignment_mask).as_fluxptr();

        // Verify there is space for this allocation
        if new_break < self.breaks.app_break {
            None
            // Verify it didn't wrap around
        } else if new_break > self.breaks.kernel_memory_break {
            None
            // Verify this is compatible with the MPU.
        } else {
            // Allocation is valid.
            // The app break is precisely the end of the process
            // accessible memory so we don't need to ask the MPU
            // anything

            // We always allocate down, so we must lower the
            // kernel_memory_break.
            self.breaks.set_kernel_break(new_break);

            // We need `grant_ptr` as a mutable pointer.
            let grant_ptr = new_break;

            // ### Safety
            //
            // Here we are guaranteeing that `grant_ptr` is not null. We can
            // ensure this because we just created `grant_ptr` based on the
            // process's allocated memory, and we know it cannot be null.
            unsafe { Some(NonNull::new_unchecked(grant_ptr.unsafe_as_ptr())) }
        }
    }

    #[flux_rs::sig(
        fn(
            &Self[@c],
            FluxPtr[@buf_start],
            FluxPtr[@buf_end],
        ) -> bool[buf_end >= buf_start && buf_start >= c.mem_start && buf_end <= c.app_break]
    )]
    pub(crate) fn in_app_owned_memory(
        &self,
        buf_start_addr: FluxPtr,
        buf_end_addr: FluxPtr,
    ) -> bool {
        buf_end_addr >= buf_start_addr
            && buf_start_addr >= self.breaks.mem_start
            && buf_end_addr <= self.breaks.app_break
    }
}

#[derive(Clone, Copy)]
#[flux_rs::refined_by(start: int, len: int)]
#[flux_rs::opaque]
struct FlashGhostState {}

impl FlashGhostState {
    #[flux_rs::trusted]
    #[flux_rs::sig(fn (start: FluxPtr, len: usize) -> Self[start, len])]
    pub(crate) fn new(_start: FluxPtr, _len: usize) -> Self {
        Self {}
    }
}

/// A type for userspace processes in Tock.
#[flux_rs::refined_by(mem_start: int, mem_len: int, flash_start: int, flash_len: int)]
#[flux_rs::invariant(mem_start + mem_len <= usize::MAX)]
pub struct ProcessStandard<'a, C: 'static + Chip> {
    /// Identifier of this process and the index of the process in the process
    /// table.
    process_id: Cell<ProcessId>,

    /// An application ShortId, generated from process loading and
    /// checking, which denotes the security identity of this process.
    app_id: ShortId,

    /// Pointer to the main Kernel struct.
    kernel: &'static Kernel,

    /// Pointer to the struct that defines the actual chip the kernel is running
    /// on. This is used because processes have subtle hardware-based
    /// differences. Specifically, the actual syscall interface and how
    /// processes are switched to is architecture-specific, and how memory must
    /// be allocated for memory protection units is also hardware-specific.
    chip: &'static C,

    /// Application memory layout:
    ///
    /// ```text
    ///     ╒════════ ← memory_start + memory_len
    ///  ╔═ │ Grant Pointers
    ///  ║  │ ──────
    ///     │ Process Control Block
    ///  D  │ ──────
    ///  Y  │ Grant Regions
    ///  N  │
    ///  A  │   ↓
    ///  M  │ ──────  ← kernel_memory_break
    ///  I  │
    ///  C  │ ──────  ← app_break               ═╗
    ///     │                                    ║
    ///  ║  │   ↑                                  A
    ///  ║  │  Heap                              P C
    ///  ╠═ │ ──────  ← app_heap_start           R C
    ///     │  Data                              O E
    ///  F  │ ──────  ← data_start_pointer       C S
    ///  I  │ Stack                              E S
    ///  X  │   ↓                                S I
    ///  E  │                                    S B
    ///  D  │ ──────  ← current_stack_pointer      L
    ///     │                                    ║ E
    ///  ╚═ ╘════════ ← memory_start            ═╝
    /// ```
    ///
    /// The start of process memory. We store this as a pointer and length and
    /// not a slice due to Rust aliasing rules. If we were to store a slice,
    /// then any time another slice to the same memory or an ProcessBuffer is
    /// used in the kernel would be undefined behavior.
    #[field({FluxPtrU8Mut[mem_start] | mem_start + mem_len <= usize::MAX})]
    memory_start: FluxPtrU8Mut,
    /// Number of bytes of memory allocated to this process.
    #[field(usize[mem_len])]
    memory_len: usize,

    // breaks and corresponding configuration
    // refinement says that these are the same
    #[field(MapCell<BreaksAndMPUConfig<C>{bc: 
        bc.mem_start == mem_start &&
        bc.mem_len == mem_len &&
        bc.flash_start == flash_start &&
        bc.flash_len == flash_len
    }>)]
    breaks_and_config: MapCell<BreaksAndMPUConfig<C>>,

    /// Reference to the slice of `GrantPointerEntry`s stored in the process's
    /// memory reserved for the kernel. These driver numbers are zero and
    /// pointers are null if the grant region has not been allocated. When the
    /// grant region is allocated these pointers are updated to point to the
    /// allocated memory and the driver number is set to match the driver that
    /// owns the grant. No other reference to these pointers exists in the Tock
    /// kernel.
    grant_pointers: MapCell<&'static mut [GrantPointerEntry]>,

    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: &'static [u8],
    #[field(FlashGhostState[flash_start, flash_len])]
    _flash_ghost: FlashGhostState,

    /// The footers of the process binary (may be zero-sized), which are metadata
    /// about the process not covered by integrity. Used, among other things, to
    /// store signatures.
    footers: &'static [u8],

    /// Collection of pointers to the TBF header in flash.
    header: tock_tbf::types::TbfHeader,

    /// Credential that was approved for this process, or `None` if the
    /// credential was permitted to run without an accepted credential.
    credential: Option<AcceptedCredential>,

    /// State saved on behalf of the process each time the app switches to the
    /// kernel.
    stored_state:
        MapCell<<<C as Chip>::UserspaceKernelBoundary as UserspaceKernelBoundary>::StoredState>,

    /// The current state of the app. The scheduler uses this to determine
    /// whether it can schedule this app to execute.
    ///
    /// The `state` is used both for bookkeeping for the scheduler as well as
    /// for enabling control by other parts of the system. The scheduler keeps
    /// track of if a process is ready to run or not by switching between the
    /// `Running` and `Yielded` states. The system can control the process by
    /// switching it to a "stopped" state to prevent the scheduler from
    /// scheduling it.
    state: Cell<State>,

    /// How to respond if this process faults.
    fault_policy: &'a dyn ProcessFaultPolicy,

    /// Essentially a list of upcalls that want to call functions in the
    /// process.
    tasks: MapCell<RingBuffer<'a, Task>>,

    /// Count of how many times this process has entered the fault condition and
    /// been restarted. This is used by some `ProcessRestartPolicy`s to
    /// determine if the process should be restarted or not.
    restart_count: Cell<usize>,

    /// The completion code set by the process when it last exited, restarted,
    /// or was terminated. If the process is has never terminated, then the
    /// `OptionalCell` will be empty (i.e. `None`). If the process has exited,
    /// restarted, or terminated, the `OptionalCell` will contain an optional 32
    /// bit value. The option will be `None` if the process crashed or was
    /// stopped by the kernel and there is no provided completion code. If the
    /// process called the exit syscall then the provided completion code will
    /// be stored as `Some(completion code)`.
    completion_code: OptionalCell<Option<u32>>,

    /// Values kept so that we can print useful debug messages when apps fault.
    debug: MapCell<ProcessStandardDebug>,
}

impl<C: Chip> Process for ProcessStandard<'_, C> {
    fn processid(&self) -> ProcessId {
        self.process_id.get()
    }

    fn short_app_id(&self) -> ShortId {
        self.app_id
    }

    fn binary_version(&self) -> Option<BinaryVersion> {
        let version = self.header.get_binary_version();
        match NonZeroU32::new(version) {
            Some(version_nonzero) => Some(BinaryVersion::new(version_nonzero)),
            None => None,
        }
    }

    fn get_credential(&self) -> Option<AcceptedCredential> {
        self.credential
    }

    fn enqueue_task(&self, task: Task) -> Result<(), ErrorCode> {
        // If this app is in a `Fault` state then we shouldn't schedule
        // any work for it.
        if !self.is_running() {
            return Err(ErrorCode::NODEVICE);
        }

        let ret = self.tasks.map_or(Err(ErrorCode::FAIL), |tasks| {
            match tasks.enqueue(task) {
                true => {
                    // The task has been successfully enqueued.
                    Ok(())
                }
                false => {
                    // The task could not be enqueued as there is
                    // insufficient space in the ring buffer.
                    Err(ErrorCode::NOMEM)
                }
            }
        });

        if ret.is_err() {
            // On any error we were unable to enqueue the task. Record the
            // error, but importantly do _not_ increment kernel work.
            self.debug.map(|debug| {
                debug.dropped_upcall_count += 1;
            });
        }

        ret
    }

    fn ready(&self) -> bool {
        self.tasks.map_or(false, |ring_buf| ring_buf.has_elements())
            || self.state.get() == State::Running
    }

    fn remove_pending_upcalls(&self, upcall_id: UpcallId) {
        self.tasks.map(|tasks| {
            let count_before = tasks.len();
            // VTOCK-TODO: prove tasks.retain() reduces number of tasks
            tasks.retain(|task| match task {
                // Remove only tasks that are function calls with an id equal
                // to `upcall_id`.
                Task::FunctionCall(function_call) => match function_call.source {
                    FunctionCallSource::Kernel => true,
                    FunctionCallSource::Driver(id) => id != upcall_id,
                },
                _ => true,
            });
            if config::CONFIG.trace_syscalls {
                let count_after = tasks.len();
                assume(count_before >= count_after); // requires refined ringbuffer
                debug!(
                    "[{:?}] remove_pending_upcalls[{:#x}:{}] = {} upcall(s) removed",
                    self.processid(),
                    upcall_id.driver_num,
                    upcall_id.subscribe_num,
                    count_before - count_after,
                );
            }
        });
    }

    fn is_running(&self) -> bool {
        match self.state.get() {
            State::Running | State::Yielded | State::YieldedFor(_) | State::Stopped(_) => true,
            _ => false,
        }
    }

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_yielded_state(&self) {
        if self.state.get() == State::Running {
            self.state.set(State::Yielded);
        }
    }

    fn set_yielded_for_state(&self, upcall_id: UpcallId) {
        if self.state.get() == State::Running {
            self.state.set(State::YieldedFor(upcall_id));
        }
    }

    fn stop(&self) {
        match self.state.get() {
            State::Running => self.state.set(State::Stopped(StoppedState::Running)),
            State::Yielded => self.state.set(State::Stopped(StoppedState::Yielded)),
            State::YieldedFor(upcall_id) => self
                .state
                .set(State::Stopped(StoppedState::YieldedFor(upcall_id))),
            State::Stopped(_stopped_state) => {
                // Already stopped, nothing to do.
            }
            State::Faulted | State::Terminated => {
                // Stop has no meaning on a inactive process.
            }
        }
    }

    fn resume(&self) {
        match self.state.get() {
            State::Stopped(stopped_state) => match stopped_state {
                StoppedState::Running => self.state.set(State::Running),
                StoppedState::Yielded => self.state.set(State::Yielded),
                StoppedState::YieldedFor(upcall_id) => self.state.set(State::YieldedFor(upcall_id)),
            },
            _ => {} // Do nothing
        }
    }

    fn set_fault_state(&self) {
        // Use the per-process fault policy to determine what action the kernel
        // should take since the process faulted.
        let action = self.fault_policy.action(self);
        match action {
            FaultAction::Panic => {
                // process faulted. Panic and print status
                self.state.set(State::Faulted);
                panic!("Process {} had a fault", self.get_process_name());
            }
            FaultAction::Restart => {
                self.try_restart(None);
            }
            FaultAction::Stop => {
                // This looks a lot like restart, except we just leave the app
                // how it faulted and mark it as `Faulted`. By clearing
                // all of the app's todo work it will not be scheduled, and
                // clearing all of the grant regions will cause capsules to drop
                // this app as well.
                self.terminate(None);
                self.state.set(State::Faulted);
            }
        }
    }

    fn start(&self, _cap: &dyn crate::capabilities::ProcessStartCapability) {
        // `start()` can only be called on a terminated process.
        if self.get_state() != State::Terminated {
            return;
        }

        // Reset to start the process.
        if let Ok(()) = self.reset() {
            self.state.set(State::Yielded);
        }
    }

    fn try_restart(&self, completion_code: Option<u32>) {
        // `try_restart()` cannot be called if the process is terminated. Only
        // `start()` can start a terminated process.
        if self.get_state() == State::Terminated {
            return;
        }

        // Terminate the process, freeing its state and removing any
        // pending tasks from the scheduler's queue.
        self.terminate(completion_code);

        // If there is a kernel policy that controls restarts, it should be
        // implemented here. For now, always restart.
        if let Ok(()) = self.reset() {
            self.state.set(State::Yielded);
        }

        // Decide what to do with res later. E.g., if we can't restart
        // want to reclaim the process resources.
    }

    fn terminate(&self, completion_code: Option<u32>) {
        // A process can be terminated if it is running or in the `Faulted`
        // state. Otherwise, you cannot terminate it and this method return
        // early.
        //
        // The kernel can terminate in the `Faulted` state to return the process
        // to a state in which it can run again (e.g., reset it).
        if !self.is_running() && self.get_state() != State::Faulted {
            return;
        }

        // And remove those tasks
        self.tasks.map(|tasks| {
            tasks.empty();
        });

        // Clear any grant regions this app has setup with any capsules.
        unsafe {
            self.grant_ptrs_reset();
        }

        // Save the completion code.
        self.completion_code.set(completion_code);

        // Mark the app as stopped so the scheduler won't try to run it.
        self.state.set(State::Terminated);
    }

    fn get_restart_count(&self) -> usize {
        self.restart_count.get()
    }

    fn has_tasks(&self) -> bool {
        self.tasks.map_or(false, |tasks| tasks.has_elements())
    }

    fn dequeue_task(&self) -> Option<Task> {
        self.tasks.map_or(None, |tasks| tasks.dequeue())
    }

    fn remove_upcall(&self, upcall_id: UpcallId) -> Option<Task> {
        self.tasks.map_or(None, |tasks| {
            tasks.remove_first_matching(|task| match task {
                Task::FunctionCall(fc) => match fc.source {
                    FunctionCallSource::Driver(upid) => upid == upcall_id,
                    _ => false,
                },
                Task::ReturnValue(rv) => rv.upcall_id == upcall_id,
                Task::IPC(_) => false,
            })
        })
    }

    fn pending_tasks(&self) -> usize {
        self.tasks.map_or(0, |tasks| tasks.len())
    }

    fn get_command_permissions(&self, driver_num: usize, offset: usize) -> CommandPermissions {
        self.header.get_command_permissions(driver_num, offset)
    }

    fn get_storage_permissions(&self) -> Option<storage_permissions::StoragePermissions> {
        let (read_count, read_ids) = self.header.get_storage_read_ids().unwrap_or((0, [0; 8]));

        let (modify_count, modify_ids) =
            self.header.get_storage_modify_ids().unwrap_or((0, [0; 8]));

        let write_id = self.header.get_storage_write_id();

        Some(storage_permissions::StoragePermissions::new(
            read_count,
            read_ids,
            modify_count,
            modify_ids,
            write_id,
        ))
    }

    fn number_writeable_flash_regions(&self) -> usize {
        self.header.number_writeable_flash_regions()
    }

    fn get_writeable_flash_region(&self, region_index: usize) -> (u32, u32) {
        self.header.get_writeable_flash_region(region_index)
    }

    fn update_stack_start_pointer(&self, stack_pointer: FluxPtrU8Mut) {
        if stack_pointer >= self.mem_start() && stack_pointer < self.mem_end() {
            self.debug.map(|debug| {
                debug.app_stack_start_pointer = Some(stack_pointer);

                // We also reset the minimum stack pointer because whatever
                // value we had could be entirely wrong by now.
                debug.app_stack_min_pointer = Some(stack_pointer);
            });
        }
    }

    fn update_heap_start_pointer(&self, heap_pointer: FluxPtrU8Mut) {
        if heap_pointer >= self.mem_start() && heap_pointer < self.mem_end() {
            self.debug.map(|debug| {
                debug.app_heap_start_pointer = Some(heap_pointer);
            });
        }
    }

    fn setup_mpu(&self) {
        self.breaks_and_config.map(|breaks_and_config| {
            self.chip.mpu().configure_mpu(&breaks_and_config.mpu_config);
        });
    }

    #[flux_rs::trusted] // VTOCK: This is problematic and deals with IPC
    fn add_mpu_region(
        &self,
        unallocated_memory_start: FluxPtrU8Mut,
        unallocated_memory_size: usize,
        min_region_size: usize,
    ) -> Option<mpu::Region> {
        self.breaks_and_config.and_then(|breaks_and_config| {
            let new_region = self.chip.mpu().allocate_region(
                unallocated_memory_start,
                unallocated_memory_size,
                min_region_size,
                mpu::Permissions::ReadWriteOnly,
                &mut breaks_and_config.mpu_config,
            )?;

            // VTOCK TODO: Oh boy - seems like they're iterating over the old configuration?
            for region in breaks_and_config.mpu_regions.iter() {
                if region.get().is_none() {
                    region.set(Some(new_region));
                    return Some(new_region);
                }
            }

            // Not enough room in Process struct to store the MPU region.
            None
        })
    }

    #[flux_rs::trusted] // VTOCK TODO: This is problematic and deals with IPC
    fn remove_mpu_region(&self, region: mpu::Region) -> Result<(), ErrorCode> {
        self.breaks_and_config
            .map_or(Err(ErrorCode::INVAL), |breaks_and_config| {
                // Find the existing mpu region that we are removing; it needs to match exactly.
                if let Some(internal_region) = breaks_and_config
                    .mpu_regions
                    .iter()
                    .find(|r| r.get().map_or(false, |r| r == region))
                {
                    self.chip
                        .mpu()
                        .remove_memory_region(region, &mut breaks_and_config.mpu_config)
                        .or(Err(ErrorCode::FAIL))?;

                    // Remove this region from the tracking cache of mpu_regions
                    internal_region.set(None);
                    Ok(())
                } else {
                    Err(ErrorCode::INVAL)
                }
            })
    }

    fn sbrk(&self, increment: isize) -> Result<FluxPtrU8Mut, Error> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }
        let app_break = self.app_memory_break().map_err(|_| Error::KernelError)?;
        let new_break = unsafe { app_break.offset(increment) };
        self.brk(new_break)
    }

    fn brk(&self, new_break: FluxPtrU8Mut) -> Result<FluxPtrU8Mut, Error> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        self.breaks_and_config
            .map_or(Err(Error::KernelError), |breaks_and_config| {
                breaks_and_config.brk(new_break, self.chip.mpu())
            })
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn build_readwrite_process_buffer(
        &self,
        buf_start_addr: FluxPtrU8Mut,
        size: usize,
    ) -> Result<ReadWriteProcessBuffer, ErrorCode> {
        if !self.is_running() {
            // Do not operate on an inactive process
            return Err(ErrorCode::FAIL);
        }

        // A process is allowed to pass any pointer if the buffer length is 0,
        // as to revoke kernel access to a memory region without granting access
        // to another one
        if size == 0 {
            // Clippy complains that we're dereferencing a pointer in a public
            // and safe function here. While we are not dereferencing the
            // pointer here, we pass it along to an unsafe function, which is as
            // dangerous (as it is likely to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as a buffer of length
            // 0 will never allow dereferencing any memory in a safe manner.
            //
            // ### Safety
            //
            // We specify a zero-length buffer, so the implementation of
            // `ReadWriteProcessBuffer` will handle any safety issues.
            // Therefore, we can encapsulate the unsafe.
            Ok(unsafe { ReadWriteProcessBuffer::new(buf_start_addr, 0, self.processid()) })
        } else {
            let _ = self
                .breaks_and_config
                .map_or(Err(ErrorCode::INVAL), |breaks_and_config| {
                    Ok(breaks_and_config.build_readwrite_process_buffer(buf_start_addr, size))
                })
                .map_err(|_| ErrorCode::INVAL)?;
            // Clippy complains that we're dereferencing a pointer in a public
            // and safe function here. While we are not dereferencing the
            // pointer here, we pass it along to an unsafe function, which is as
            // dangerous (as it is likely to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as long as we make
            // sure that we're pointing towards userspace memory (verified using
            // `in_app_owned_memory`) and respect alignment and other
            // constraints of the Rust references created by
            // `ReadWriteProcessBuffer`.
            //
            // ### Safety
            //
            // We encapsulate the unsafe here on the condition in the TODO
            // above, as we must ensure that this `ReadWriteProcessBuffer` will
            // be the only reference to this memory.
            Ok(unsafe { ReadWriteProcessBuffer::new(buf_start_addr, size, self.processid()) })
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[flux_rs::trusted] // refinement error
    fn build_readonly_process_buffer(
        &self,
        buf_start_addr: FluxPtrU8Mut,
        size: usize,
    ) -> Result<ReadOnlyProcessBuffer, ErrorCode> {
        if !self.is_running() {
            // Do not operate on an inactive process
            return Err(ErrorCode::FAIL);
        }

        // A process is allowed to pass any pointer if the buffer length is 0,
        // as to revoke kernel access to a memory region without granting access
        // to another one
        if size == 0 {
            // Clippy complains that we're dereferencing a pointer in a public
            // and safe function here. While we are not dereferencing the
            // pointer here, we pass it along to an unsafe function, which is as
            // dangerous (as it is likely to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as a buffer of length
            // 0 will never allow dereferencing any memory in a safe manner.
            //
            // ### Safety
            //
            // We specify a zero-length buffer, so the implementation of
            // `ReadOnlyProcessBuffer` will handle any safety issues. Therefore,
            // we can encapsulate the unsafe.
            Ok(unsafe { ReadOnlyProcessBuffer::new(buf_start_addr, 0, self.processid()) })
        } else if self
            .in_app_owned_memory(buf_start_addr, size)
            .map_err(|_| ErrorCode::FAIL)?
            || self.in_app_flash_memory(buf_start_addr, size)
        {
            // TODO: Check for buffer aliasing here

            if self
                .in_app_owned_memory(buf_start_addr, size)
                .map_err(|_| ErrorCode::FAIL)?
            {
                // Valid buffer, and since this is in read-write memory (i.e.
                // not flash), we need to adjust the process's watermark. Note:
                // `in_app_owned_memory()` ensures this offset does not wrap.
                let buf_end_addr = buf_start_addr.wrapping_add(size);

                self.breaks_and_config
                    .map_or(Err(ErrorCode::FAIL), |breaks_and_config| {
                        let breaks = &mut breaks_and_config.breaks;
                        let new_water_mark = cmp::max(breaks.allow_high_water_mark, buf_end_addr);
                        breaks.allow_high_water_mark = new_water_mark;
                        Ok(())
                    })?;
            }

            // Clippy complains that we're dereferencing a pointer in a public
            // and safe function here. While we are not dereferencing the
            // pointer here, we pass it along to an unsafe function, which is as
            // dangerous (as it is likely to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as long as we make
            // sure that we're pointing towards userspace memory (verified using
            // `in_app_owned_memory` or `in_app_flash_memory`) and respect
            // alignment and other constraints of the Rust references created by
            // `ReadWriteProcessBuffer`.
            //
            // ### Safety
            //
            // We encapsulate the unsafe here on the condition in the TODO
            // above, as we must ensure that this `ReadOnlyProcessBuffer` will
            // be the only reference to this memory.
            Ok(unsafe { ReadOnlyProcessBuffer::new(buf_start_addr, size, self.processid()) })
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    unsafe fn set_byte(&self, mut addr: FluxPtrU8Mut, value: u8) -> Result<bool, ()> {
        if self.in_app_owned_memory(addr, 1)? {
            // We verify that this will only write process-accessible memory,
            // but this can still be undefined behavior if something else holds
            // a reference to this memory.
            *addr = value;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn grant_is_allocated(&self, grant_num: usize) -> Option<bool> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return None;
        }

        // Update the grant pointer to the address of the new allocation.
        self.grant_pointers.map_or(None, |grant_pointers| {
            // Implement `grant_pointers[grant_num]` without a chance of a
            // panic.
            grant_pointers
                .get(grant_num)
                .map(|grant_entry| !grant_entry.grant_ptr.is_null())
        })
    }

    fn allocate_grant(
        &self,
        grant_num: usize,
        driver_num: usize,
        size: usize,
        align: usize,
    ) -> Result<(), ()> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(());
        }

        // Verify the grant_num is valid.
        if grant_num >= self.kernel.get_grant_count_and_finalize() {
            return Err(());
        }

        // Verify that the grant is not already allocated. If the pointer is not
        // null then the grant is already allocated.
        if let Some(is_allocated) = self.grant_is_allocated(grant_num) {
            if is_allocated {
                return Err(());
            }
        }

        // Verify that there is not already a grant allocated with the same
        // `driver_num`.
        let exists = self.grant_pointers.map_or(false, |grant_pointers| {
            // Check our list of grant pointers if the driver number is used.
            grant_pointers.iter().any(|grant_entry| {
                // Check if the grant is both allocated (its grant pointer is
                // non null) and the driver number matches.
                (!grant_entry.grant_ptr.is_null()) && grant_entry.driver_num == driver_num
            })
        });
        // If we find a match, then the `driver_num` must already be used and
        // the grant allocation fails.
        if exists {
            return Err(());
        }

        // Use the shared grant allocator function to actually allocate memory.
        // Returns `None` if the allocation cannot be created.
        if let Some(grant_ptr) = self.allocate_in_grant_region_internal(size, align) {
            // Update the grant pointer to the address of the new allocation.
            self.grant_pointers.map_or(Err(()), |grant_pointers| {
                // Implement `grant_pointers[grant_num] = grant_ptr` without a
                // chance of a panic.
                grant_pointers
                    .get_mut(grant_num)
                    .map_or(Err(()), |grant_entry| {
                        // Actually set the driver num and grant pointer.
                        grant_entry.driver_num = driver_num;
                        grant_entry.grant_ptr = grant_ptr.as_fluxptr();

                        // If all of this worked, return true.
                        Ok(())
                    })
            })
        } else {
            // Could not allocate the memory for the grant region.
            Err(())
        }
    }

    #[flux_rs::trusted] // refinement error
    fn allocate_custom_grant(
        &self,
        size: usize,
        align: usize,
    ) -> Result<(ProcessCustomGrantIdentifier, NonNull<u8>), ()> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(());
        }

        // Use the shared grant allocator function to actually allocate memory.
        // Returns `None` if the allocation cannot be created.
        if let Some(ptr) = self.allocate_in_grant_region_internal(size, align) {
            // Create the identifier that the caller will use to get access to
            // this custom grant in the future.
            let identifier = self.create_custom_grant_identifier(ptr);

            Ok((identifier, ptr))
        } else {
            // Could not allocate memory for the custom grant.
            Err(())
        }
    }

    fn enter_grant(&self, grant_num: usize) -> Result<NonNull<u8>, Error> {
        // Do not try to access the grant region of an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        // Retrieve the grant pointer from the `grant_pointers` slice. We use
        // `[slice].get()` so that if the grant number is invalid this will
        // return `Err` and not panic.
        self.grant_pointers
            .map_or(Err(Error::KernelError), |grant_pointers| {
                // Implement `grant_pointers[grant_num]` without a chance of a
                // panic.
                match grant_pointers.get_mut(grant_num) {
                    Some(grant_entry) => {
                        // Get a copy of the actual grant pointer.
                        let grant_ptr = grant_entry.grant_ptr;

                        // Check if the grant pointer is marked that the grant
                        // has already been entered. If so, return an error.
                        if (grant_ptr.as_usize()) & 0x1 == 0x1 {
                            // Lowest bit is one, meaning this grant has been
                            // entered.
                            Err(Error::AlreadyInUse)
                        } else {
                            // Now, to mark that the grant has been entered, we
                            // set the lowest bit to one and save this as the
                            // grant pointer.
                            grant_entry.grant_ptr = (grant_ptr.as_usize() | 0x1).as_fluxptr();

                            // And we return the grant pointer to the entered
                            // grant.
                            Ok(unsafe { NonNull::new_unchecked(grant_ptr.unsafe_as_ptr()) })
                        }
                    }
                    None => Err(Error::AddressOutOfBounds),
                }
            })
    }

    fn enter_custom_grant(
        &self,
        identifier: ProcessCustomGrantIdentifier,
    ) -> Result<FluxPtrU8Mut, Error> {
        // Do not try to access the grant region of an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        // Get the address of the custom grant based on the identifier.
        let custom_grant_address = self.get_custom_grant_address(identifier);

        // We never deallocate custom grants and only we can change the
        // `identifier` so we know this is a valid address for the custom grant.
        Ok(custom_grant_address.as_fluxptr())
    }

    unsafe fn leave_grant(&self, grant_num: usize) {
        // Do not modify an inactive process.
        if !self.is_running() {
            return;
        }

        self.grant_pointers.map(|grant_pointers| {
            // Implement `grant_pointers[grant_num]` without a chance of a
            // panic.
            if let Some(grant_entry) = grant_pointers.get_mut(grant_num) {
                // Get a copy of the actual grant pointer.
                let grant_ptr = grant_entry.grant_ptr;

                // Now, to mark that the grant has been released, we set the
                // lowest bit back to zero and save this as the grant
                // pointer.
                grant_entry.grant_ptr = (grant_ptr.as_usize() & !0x1).as_fluxptr();
            }
        });
    }

    fn grant_allocated_count(&self) -> Option<usize> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return None;
        }

        self.grant_pointers.map(|grant_pointers| {
            // Filter our list of grant pointers into just the non-null ones,
            // and count those. A grant is allocated if its grant pointer is
            // non-null.
            grant_pointers
                .iter()
                .filter(|grant_entry| !grant_entry.grant_ptr.is_null())
                .count()
        })
    }

    fn lookup_grant_from_driver_num(&self, driver_num: usize) -> Result<usize, Error> {
        self.grant_pointers
            .map_or(Err(Error::KernelError), |grant_pointers| {
                // Filter our list of grant pointers into just the non null
                // ones, and count those. A grant is allocated if its grant
                // pointer is non-null.
                match grant_pointers.iter().position(|grant_entry| {
                    // Only consider allocated grants.
                    (!grant_entry.grant_ptr.is_null()) && grant_entry.driver_num == driver_num
                }) {
                    Some(idx) => Ok(idx),
                    None => Err(Error::OutOfMemory),
                }
            })
    }

    fn is_valid_upcall_function_pointer(&self, upcall_fn: NonNull<()>) -> Result<bool, ()> {
        let ptr = upcall_fn.as_fluxptr();
        let size = mem::size_of::<FluxPtrU8Mut>();

        // It is okay if this function is in memory or flash.
        Ok(self.in_app_flash_memory(ptr, size) || self.in_app_owned_memory(ptr, size)?)
    }

    fn get_process_name(&self) -> &'static str {
        self.header.get_package_name().unwrap_or("")
    }

    fn get_completion_code(&self) -> Option<Option<u32>> {
        self.completion_code.get()
    }

    fn set_syscall_return_value(&self, return_value: SyscallReturn) {
        match self.stored_state.map(|stored_state| unsafe {
            // Actually set the return value for a particular process.
            //
            // The UKB implementation uses the bounds of process-accessible
            // memory to verify that any memory changes are valid. Here, the
            // unsafe promise we are making is that the bounds passed to the UKB
            // are correct.
            let app_break = self.app_memory_break()?;
            self.chip
                .userspace_kernel_boundary()
                .set_syscall_return_value(self.mem_start(), app_break, stored_state, return_value)
        }) {
            Some(Ok(())) => {
                // If we get an `Ok` we are all set.

                // The process is either already in the running state (having
                // just called a nonblocking syscall like command) or needs to
                // be moved to the running state having called Yield-WaitFor and
                // now needing to be resumed. Either way we can set the state to
                // running.
                self.state.set(State::Running);
            }

            Some(Err(())) => {
                // If we get an `Err`, then the UKB implementation could not set
                // the return value, likely because the process's stack is no
                // longer accessible to it. All we can do is fault.
                self.set_fault_state();
            }

            None => {
                // We should never be here since `stored_state` should always be
                // occupied.
                self.set_fault_state();
            }
        }
    }

    fn set_process_function(&self, callback: FunctionCall) {
        // See if we can actually enqueue this function for this process.
        // Architecture-specific code handles actually doing this since the
        // exact method is both architecture- and implementation-specific.
        //
        // This can fail, for example if the process does not have enough memory
        // remaining.
        match self.stored_state.map(|stored_state| {
            // Let the UKB implementation handle setting the process's PC so
            // that the process executes the upcall function. We encapsulate
            // unsafe here because we are guaranteeing that the memory bounds
            // passed to `set_process_function` are correct.
            let app_break = self.app_memory_break()?;
            unsafe {
                self.chip.userspace_kernel_boundary().set_process_function(
                    self.mem_start(),
                    app_break,
                    stored_state,
                    callback,
                )
            }
        }) {
            Some(Ok(())) => {
                // If we got an `Ok` we are all set and should mark that this
                // process is ready to be scheduled.

                // Move this process to the "running" state so the scheduler
                // will schedule it.
                self.state.set(State::Running);
            }

            Some(Err(())) => {
                // If we got an Error, then there was likely not enough room on
                // the stack to allow the process to execute this function given
                // the details of the particular architecture this is running
                // on. This process has essentially faulted, so we mark it as
                // such.
                self.set_fault_state();
            }

            None => {
                // We should never be here since `stored_state` should always be
                // occupied.
                self.set_fault_state();
            }
        }
    }

    #[flux_rs::trusted] // https://github.com/flux-rs/flux/issues/782
    fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        // Cannot switch to an invalid process
        if !self.is_running() {
            return None;
        }

        let (switch_reason, stack_pointer) =
            self.stored_state.map_or((None, None), |stored_state| {
                // Switch to the process. We guarantee that the memory pointers
                // we pass are valid, ensuring this context switch is safe.
                // Therefore we encapsulate the `unsafe`.
                match self.app_memory_break().ok() {
                    None => (None, None),
                    Some(app_break) => unsafe {
                        let (switch_reason, optional_stack_pointer) = self
                            .chip
                            .userspace_kernel_boundary()
                            .switch_to_process(self.mem_start(), app_break, stored_state);
                        (Some(switch_reason), optional_stack_pointer)
                    },
                }
            });

        // If the UKB implementation passed us a stack pointer, update our
        // debugging state. This is completely optional.
        if let Some(sp) = stack_pointer {
            self.debug.map(|debug| {
                match debug.app_stack_min_pointer {
                    None => debug.app_stack_min_pointer = Some(sp),
                    Some(asmp) => {
                        // Update max stack depth if needed.
                        if sp < asmp {
                            debug.app_stack_min_pointer = Some(sp);
                        }
                    }
                }
            });
        }

        switch_reason
    }

    fn debug_syscall_count(&self) -> usize {
        self.debug.map_or(0, |debug| debug.syscall_count)
    }

    fn debug_dropped_upcall_count(&self) -> usize {
        self.debug.map_or(0, |debug| debug.dropped_upcall_count)
    }

    fn debug_timeslice_expiration_count(&self) -> usize {
        self.debug
            .map_or(0, |debug| debug.timeslice_expiration_count)
    }

    fn debug_timeslice_expired(&self) {
        self.debug
            .map(|debug| debug.timeslice_expiration_count += 1);
    }

    fn debug_syscall_called(&self, last_syscall: Syscall) {
        self.debug.map(|debug| {
            debug.syscall_count += 1;
            debug.last_syscall = Some(last_syscall);
        });
    }

    fn debug_syscall_last(&self) -> Option<Syscall> {
        self.debug.map_or(None, |debug| debug.last_syscall)
    }

    fn get_addresses(&self) -> Result<ProcessAddresses, ()> {
        Ok(ProcessAddresses {
            flash_start: self.flash_start().as_usize(),
            flash_non_protected_start: self.flash_non_protected_start().as_usize(),
            flash_integrity_end: ((self.flash.as_fluxptr().as_usize())
                + (self.header.get_binary_end() as usize))
                .as_fluxptr(),
            flash_end: self.flash_end().as_usize(),
            sram_start: self.mem_start().as_usize(),
            sram_app_brk: self.app_memory_break()?.as_usize(),
            sram_grant_start: self.kernel_memory_break()?.as_usize(),
            sram_end: self.mem_end().as_usize(),
            sram_heap_start: self.debug.map_or(None, |debug| {
                debug.app_heap_start_pointer.map(|p| p.as_usize())
            }),
            sram_stack_top: self.debug.map_or(None, |debug| {
                debug.app_stack_start_pointer.map(|p| p.as_usize())
            }),
            sram_stack_bottom: self.debug.map_or(None, |debug| {
                debug.app_stack_min_pointer.map(|p| p.as_usize())
            }),
        })
    }

    fn get_sizes(&self) -> ProcessSizes {
        ProcessSizes {
            grant_pointers: mem::size_of::<GrantPointerEntry>()
                * self.kernel.get_grant_count_and_finalize(),
            upcall_list: Self::CALLBACKS_OFFSET,
            process_control_block: Self::PROCESS_STRUCT_OFFSET,
        }
    }

    fn print_full_process(&self, writer: &mut dyn Write) {
        if !config::CONFIG.debug_panics {
            return;
        }

        self.stored_state.map(|stored_state| {
            // We guarantee the memory bounds pointers provided to the UKB are
            // correct.
            let maybe_app_break = self.app_memory_break();
            if maybe_app_break.is_err() {
                let _ = writer.write_str(
                    "Uh oh. Somehow the app_memory_break behind a map cell returned an error",
                );
                return;
            }
            unsafe {
                self.chip.userspace_kernel_boundary().print_context(
                    self.mem_start(),
                    maybe_app_break.unwrap(),
                    stored_state,
                    writer,
                );
            }
        });

        // Display grant information.
        let number_grants = self.kernel.get_grant_count_and_finalize();
        let _ = writer.write_fmt(format_args!(
            "\
            \r\n Total number of grant regions defined: {}\r\n",
            self.kernel.get_grant_count_and_finalize()
        ));
        let rows = (number_grants + 2) / 3;

        // Access our array of grant pointers.
        self.grant_pointers.map(|grant_pointers| {
            // Iterate each grant and show its address.
            for i in 0..rows {
                for j in 0..3 {
                    let index = i + (rows * j);
                    if index >= number_grants {
                        break;
                    }

                    // Implement `grant_pointers[grant_num]` without a chance of
                    // a panic.
                    grant_pointers.get(index).map(|grant_entry| {
                        if grant_entry.grant_ptr.is_null() {
                            let _ =
                                writer.write_fmt(format_args!("  Grant {:>2} : --        ", index));
                        } else {
                            let _ = writer.write_fmt(format_args!(
                                "  Grant {:>2} {:#x}: {:?}",
                                index, grant_entry.driver_num, grant_entry.grant_ptr
                            ));
                        }
                    });
                }
                let _ = writer.write_fmt(format_args!("\r\n"));
            }
        });

        // Display the current state of the MPU for this process.
        self.breaks_and_config.map(|breaks_and_config| {
            let _ = writer.write_fmt(format_args!("{}", breaks_and_config.mpu_config));
        });

        // Print a helpful message on how to re-compile a process to view the
        // listing file. If a process is PIC, then we also need to print the
        // actual addresses the process executed at so that the .lst file can be
        // generated for those addresses. If the process was already compiled
        // for a fixed address, then just generating a .lst file is fine.

        self.debug.map(|debug| {
            if debug.fixed_address_flash.is_some() {
                // Fixed addresses, can just run `make lst`.
                let _ = writer.write_fmt(format_args!(
                    "\
                    \r\nTo debug libtock-c apps, run `make lst` in the app's\
                    \r\nfolder and open the arch.{:#x}.{:#x}.lst file.\r\n\r\n",
                    debug.fixed_address_flash.unwrap_or(0),
                    debug.fixed_address_ram.unwrap_or(0)
                ));
            } else {
                // PIC, need to specify the addresses.
                let sram_start = self.mem_start().as_usize();
                let flash_start = self.flash.as_fluxptr().as_usize();
                let flash_init_fn = flash_start + self.header.get_init_function_offset() as usize;

                let _ = writer.write_fmt(format_args!(
                    "\
                    \r\nTo debug libtock-c apps, run\
                    \r\n`make debug RAM_START={:#x} FLASH_INIT={:#x}`\
                    \r\nin the app's folder and open the .lst file.\r\n\r\n",
                    sram_start, flash_init_fn
                ));
            }
        });
    }

    fn get_stored_state(&self, out: &mut [u8]) -> Result<usize, ErrorCode> {
        self.stored_state
            .map(|stored_state| {
                self.chip
                    .userspace_kernel_boundary()
                    .store_context(stored_state, out)
            })
            .unwrap_or(Err(ErrorCode::FAIL))
    }

    fn get_flash_start(&self) -> usize {
        self.flash_start().as_usize()
    }

    fn get_flash_end(&self) -> usize {
        self.flash_end().as_usize()
    }

    fn get_sram_start(&self) -> usize {
        self.mem_start().as_usize()
    }

    fn get_sram_end(&self) -> usize {
        self.mem_end().as_usize()
    }
}

impl<C: 'static + Chip> ProcessStandard<'_, C> {
    // Memory offset for upcall ring buffer (10 element length).
    const CALLBACK_LEN: usize = 10;
    const CALLBACKS_OFFSET: usize = mem::size_of::<Task>() * Self::CALLBACK_LEN;

    // Memory offset to make room for this process's metadata.
    const PROCESS_STRUCT_OFFSET: usize = mem::size_of::<ProcessStandard<C>>();

    /// Create a `ProcessStandard` object based on the found `ProcessBinary`.
    #[flux_rs::sig(
        fn (
            _,
            _,
            ProcessBinary, 
            &mut [u8],
            _,
            ShortId,
            usize
        )-> Result<(Option<&_>, &mut [u8]), (ProcessLoadError, &mut [u8])>
    )]
    #[flux_rs::trusted] // VTock TODO: There is a place_ty issue here
    pub(crate) unsafe fn create<'a>(
        kernel: &'static Kernel,
        chip: &'static C,
        pb: ProcessBinary,
        remaining_memory: &'a mut [u8],
        fault_policy: &'static dyn ProcessFaultPolicy,
        app_id: ShortId,
        index: usize,
    ) -> Result<(Option<&'static dyn Process>, &'a mut [u8]), (ProcessLoadError, &'a mut [u8])>
    {
        let process_name = pb.header.get_package_name();
        let process_ram_requested_size = pb.header.get_minimum_app_ram_size() as usize;

        // Initialize MPU region configuration.
        let mut mpu_config = match chip.mpu().new_config() {
            Some(mpu_config) => mpu_config,
            None => return Err((ProcessLoadError::MpuConfigurationError, remaining_memory)),
        };

        // Determine how much space we need in the application's memory space
        // just for kernel and grant state. We need to make sure we allocate
        // enough memory just for that.

        // Make room for grant pointers.
        let grant_ptr_size = mem::size_of::<GrantPointerEntry>();
        let grant_ptrs_num = kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        // Initial size of the kernel-owned part of process memory can be
        // calculated directly based on the initial size of all kernel-owned
        // data structures.
        //
        // We require our kernel memory break (located at the end of the
        // MPU-returned allocated memory region) to be word-aligned. However, we
        // don't have any explicit alignment constraints from the MPU. To ensure
        // that the below kernel-owned data structures still fit into the
        // kernel-owned memory even with padding for alignment, add an extra
        // `sizeof(usize)` bytes.
        let initial_kernel_memory_size = grant_ptrs_offset
            + Self::CALLBACKS_OFFSET
            + Self::PROCESS_STRUCT_OFFSET
            + core::mem::size_of::<usize>();

        // By default we start with the initial size of process-accessible
        // memory set to 0. This maximizes the flexibility that processes have
        // to allocate their memory as they see fit. If a process needs more
        // accessible memory it must use the `brk` memop syscalls to request
        // more memory.
        //
        // We must take into account any process-accessible memory required by
        // the context switching implementation and allocate at least that much
        // memory so that we can successfully switch to the process. This is
        // architecture and implementation specific, so we query that now.
        let min_process_memory_size = chip
            .userspace_kernel_boundary()
            .initial_process_app_brk_size();

        // We have to ensure that we at least ask the MPU for
        // `min_process_memory_size` so that we can be sure that `app_brk` is
        // not set inside the kernel-owned memory region. Now, in practice,
        // processes should not request 0 (or very few) bytes of memory in their
        // TBF header (i.e. `process_ram_requested_size` will almost always be
        // much larger than `min_process_memory_size`), as they are unlikely to
        // work with essentially no available memory. But, we still must protect
        // for that case.
        let min_process_ram_size = cmp::max(process_ram_requested_size, min_process_memory_size);

        // Minimum memory size for the process.
        let min_total_memory_size = min_process_ram_size + initial_kernel_memory_size;

        // Check if this process requires a fixed memory start address. If so,
        // try to adjust the memory region to work for this process.
        //
        // Right now, we only support skipping some RAM and leaving a chunk
        // unused so that the memory region starts where the process needs it
        // to.
        let remaining_memory = if let Some(fixed_memory_start) = pb.header.get_fixed_address_ram() {
            // The process does have a fixed address.
            if fixed_memory_start == remaining_memory.as_fluxptr().as_u32() {
                // Address already matches.
                remaining_memory
            } else if fixed_memory_start > remaining_memory.as_fluxptr().as_u32() {
                // Process wants a memory address farther in memory. Try to
                // advance the memory region to make the address match.
                let diff = (fixed_memory_start - remaining_memory.as_fluxptr().as_u32()) as usize;
                if diff > remaining_memory.len() {
                    // We ran out of memory.
                    let actual_address =
                        remaining_memory.as_fluxptr().as_u32() + remaining_memory.len() as u32 - 1;
                    let expected_address = fixed_memory_start;
                    return Err((
                        ProcessLoadError::MemoryAddressMismatch {
                            actual_address,
                            expected_address,
                        },
                        remaining_memory,
                    ));
                } else {
                    // Change the memory range to start where the process
                    // requested it. Because of the if statement above we know this should
                    // work. Doing it more cleanly would be good but was a bit beyond my borrow
                    // ken; calling get_mut has a mutable borrow.-pal
                    &mut remaining_memory[diff..]
                }
            } else {
                // Address is earlier in memory, nothing we can do.
                let actual_address = remaining_memory.as_fluxptr().as_u32();
                let expected_address = fixed_memory_start;
                return Err((
                    ProcessLoadError::MemoryAddressMismatch {
                        actual_address,
                        expected_address,
                    },
                    remaining_memory,
                ));
            }
        } else {
            remaining_memory
        };

        // Determine where process memory will go and allocate an MPU region.
        //
        // `[allocation_start, allocation_size)` will cover both
        //
        // - the app-owned `min_process_memory_size`-long part of memory (at
        //   some offset within `remaining_memory`), as well as
        //
        // - the kernel-owned allocation growing downward starting at the end
        //   of this allocation, `initial_kernel_memory_size` bytes long.
        //
        let breaks_and_size = match chip.mpu().allocate_app_memory_regions(
            remaining_memory.as_fluxptr(),
            remaining_memory.len(),
            min_total_memory_size,
            min_process_memory_size,
            initial_kernel_memory_size,
            pb.flash.as_fluxptr(),
            pb.flash.len(),
            &mut mpu_config,
        ) {
            Ok(bnsz) => bnsz,
            Err(mpu::AllocateAppMemoryError::FlashError) => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                            "[!] flash={:#010X}-{:#010X} process={:?} - couldn't allocate MPU region for flash",
                            pb.flash.as_fluxptr().as_usize(),
                            pb.flash.as_fluxptr().as_usize() + pb.flash.len() - 1,
                            process_name
                        );
                }
                return Err((ProcessLoadError::MpuInvalidFlashLength, remaining_memory));
            }
            Err(mpu::AllocateAppMemoryError::HeapError) => {
                // Failed to load process. Insufficient memory.
                if config::CONFIG.debug_load_processes {
                    debug!(
                            "[!] flash={:#010X}-{:#010X} process={:?} - couldn't allocate memory region of size >= {:#X}",
                            pb.flash.as_fluxptr().as_usize(),
                            pb.flash.as_fluxptr().as_usize() + pb.flash.len() - 1,
                            process_name,
                            min_total_memory_size
                        );
                }
                return Err((ProcessLoadError::NotEnoughMemory, remaining_memory));
            }
        };

        let allocation_start = breaks_and_size.breaks.memory_start;
        let allocation_size = breaks_and_size.memory_size;

        // Determine the offset of the app-owned part of the above memory
        // allocation. An MPU may not place it at the very start of
        // `remaining_memory` for internal alignment constraints. This can only
        // overflow if the MPU implementation is incorrect; a compliant
        // implementation must return a memory allocation within the
        // `remaining_memory` slice.
        let app_memory_start_offset =
            allocation_start.as_usize() - remaining_memory.as_fluxptr().as_usize();

        // Check if the memory region is valid for the process. If a process
        // included a fixed address for the start of RAM in its TBF header (this
        // field is optional, processes that are position independent do not
        // need a fixed address) then we check that we used the same address
        // when we allocated it in RAM.
        if let Some(fixed_memory_start) = pb.header.get_fixed_address_ram() {
            let actual_address =
                remaining_memory.as_fluxptr().as_u32() + app_memory_start_offset as u32;
            let expected_address = fixed_memory_start;
            if actual_address != expected_address {
                return Err((
                    ProcessLoadError::MemoryAddressMismatch {
                        actual_address,
                        expected_address,
                    },
                    remaining_memory,
                ));
            }
        }

        // With our MPU allocation, we can begin to divide up the
        // `remaining_memory` slice into individual regions for the process and
        // kernel, as follows:
        //
        //
        //  +-----------------------------------------------------------------
        //  | remaining_memory
        //  +----------------------------------------------------+------------
        //  v                                                    v
        //  +----------------------------------------------------+
        //  | allocated_padded_memory                            |
        //  +--+-------------------------------------------------+
        //     v                                                 v
        //     +-------------------------------------------------+
        //     | allocated_memory                                |
        //     +-------------------------------------------------+
        //     v                                                 v
        //     +-----------------------+-------------------------+
        //     | app_accessible_memory | allocated_kernel_memory |
        //     +-----------------------+-------------------+-----+
        //                                                 v
        //                               kernel memory break
        //                                                  \---+/
        //                                                      v
        //                                        optional padding
        //
        //
        // First split the `remaining_memory` into two slices:
        //
        // - `allocated_padded_memory`: the allocated memory region, containing
        //
        //   1. optional padding at the start of the memory region of
        //      `app_memory_start_offset` bytes,
        //
        //   2. the app accessible memory region of `process_allocated_size` (the size given by the MPU region),
        //
        //   3. optional unallocated memory, and
        //
        //   4. kernel-reserved memory, growing downward starting at
        //      `app_memory_padding`.
        //
        // - `unused_memory`: the rest of the `remaining_memory`, not assigned
        //   to this app.
        //
        let (allocated_padded_memory, unused_memory) =
            remaining_memory.split_at_mut(app_memory_start_offset + allocation_size);

        // Now, slice off the (optional) padding at the start:
        let (_padding, allocated_memory) =
            allocated_padded_memory.split_at_mut(app_memory_start_offset);

        // We continue to sub-slice the `allocated_memory` into
        // process-accessible and kernel-owned memory. Prior to that, store the
        // start and length ofthe overall allocation:
        let allocated_memory_start = allocated_memory.as_fluxptr();
        let allocated_memory_len = allocated_memory.len();

        // Set the initial process-accessible memory:
        let initial_app_brk = breaks_and_size.breaks.app_break;
        // Slice off the process-accessible memory:
        // use the size of the accessible region given to us by the MPU since
        // a process should not be able to access anything past it's app break
        let process_allocated_size =
            initial_app_brk.as_usize() - breaks_and_size.breaks.memory_start.as_usize();
        let (app_accessible_memory, allocated_kernel_memory) =
            allocated_memory.split_at_mut(process_allocated_size);

        // Set the initial allow high water mark to the start of process memory
        // since no `allow` calls have been made yet.
        let initial_allow_high_water_mark = app_accessible_memory.as_fluxptr();

        // Set up initial grant region.
        //
        // `kernel_memory_break` is set to the end of kernel-accessible memory
        // and grows downward.
        //
        // We require the `kernel_memory_break` to be aligned to a
        // word-boundary, as we rely on this during offset calculations to
        // kernel-accessed structs (e.g. the grant pointer table) below. As it
        // moves downward in the address space, we can't use the `align_offset`
        // convenience functions.
        //
        // Calling `wrapping_sub` is safe here, as we've factored in an optional
        // padding of at most `sizeof(usize)` bytes in the calculation of
        // `initial_kernel_memory_size` above.
        let mut kernel_memory_break = allocated_kernel_memory
            .as_fluxptr()
            .add(allocated_kernel_memory.len());

        kernel_memory_break = kernel_memory_break
            .wrapping_sub(kernel_memory_break.as_usize() % core::mem::size_of::<usize>());

        // Now that we know we have the space we can setup the grant pointers.
        kernel_memory_break = kernel_memory_break.offset(-(grant_ptrs_offset as isize));

        // This is safe, `kernel_memory_break` is aligned to a word-boundary,
        // and `grant_ptrs_offset` is a multiple of the word size.
        #[allow(clippy::cast_ptr_alignment)]
        // Set all grant pointers to null.
        let grant_pointers = slice::from_raw_parts_mut(
            kernel_memory_break.unsafe_as_ptr() as *mut GrantPointerEntry,
            grant_ptrs_num,
        );
        for grant_entry in grant_pointers.iter_mut() {
            grant_entry.driver_num = 0;
            grant_entry.grant_ptr = FluxPtr::null_mut();
        }

        // Now that we know we have the space we can setup the memory for the
        // upcalls.
        kernel_memory_break = kernel_memory_break.offset(-(Self::CALLBACKS_OFFSET as isize));

        // This is safe today, as MPU constraints ensure that `memory_start`
        // will always be aligned on at least a word boundary, and that
        // memory_size will be aligned on at least a word boundary, and
        // `grant_ptrs_offset` is a multiple of the word size. Thus,
        // `kernel_memory_break` must be word aligned. While this is unlikely to
        // change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set up ring buffer for upcalls to the process.
        let upcall_buf = slice::from_raw_parts_mut(
            kernel_memory_break.unsafe_as_ptr() as *mut Task,
            Self::CALLBACK_LEN,
        );
        let tasks = RingBuffer::new(upcall_buf);

        // Last thing in the kernel region of process RAM is the process struct.
        kernel_memory_break = kernel_memory_break.offset(-(Self::PROCESS_STRUCT_OFFSET as isize));
        let process_struct_memory_location = kernel_memory_break;

        // Create the Process struct in the app grant region.
        // Note that this requires every field be explicitly initialized, as
        // we are just transforming a pointer into a structure.
        let process: &mut ProcessStandard<C> = &mut *(process_struct_memory_location.unsafe_as_ptr()
            as *mut ProcessStandard<'static, C>);

        // Ask the kernel for a unique identifier for this process that is being
        // created.
        let unique_identifier = kernel.create_process_identifier();

        // Save copies of these in case the app was compiled for fixed addresses
        // for later debugging.
        let fixed_address_flash = pb.header.get_fixed_address_flash();
        let fixed_address_ram = pb.header.get_fixed_address_ram();

        process
            .process_id
            .set(ProcessId::new(kernel, unique_identifier, index));
        process.app_id = app_id;
        process.kernel = kernel;
        process.chip = chip;
        // process.allow_high_water_mark = Cell::new(initial_allow_high_water_mark);
        process.memory_start = allocated_memory_start;
        process.memory_len = allocated_memory_len;
        process.header = pb.header;
        // process.kernel_memory_break = Cell::new(kernel_memory_break);
        // process.app_break = Cell::new(initial_app_brk);
        let breaks = ProcessBreaks {
            mem_start: allocated_memory_start,
            mem_len: allocated_memory_len,
            flash: pb.flash,
            _flash_ghost: FlashGhostState::new(pb.flash.as_fluxptr(), pb.flash.len()),
            kernel_memory_break,
            app_break: initial_app_brk,
            allow_high_water_mark: initial_allow_high_water_mark,
        };
        process.grant_pointers = MapCell::new(grant_pointers);

        process.credential = pb.credential.get();
        process.footers = pb.footers;
        process.flash = pb.flash;

        process.stored_state = MapCell::new(Default::default());
        // Mark this process as approved and leave it to the kernel to start it.
        process.state = Cell::new(State::Yielded);
        process.fault_policy = fault_policy;
        process.restart_count = Cell::new(0);
        process.completion_code = OptionalCell::empty();

        let breaks_and_config = BreaksAndMPUConfig {
            mpu_config,
            breaks,
            mpu_regions: [
                Cell::new(None),
                Cell::new(None),
                Cell::new(None),
                Cell::new(None),
                Cell::new(None),
                Cell::new(None),
            ],
        };

        process.breaks_and_config = MapCell::new(breaks_and_config);
        process.tasks = MapCell::new(tasks);

        process.debug = MapCell::new(ProcessStandardDebug {
            fixed_address_flash,
            fixed_address_ram,
            app_heap_start_pointer: None,
            app_stack_start_pointer: None,
            app_stack_min_pointer: None,
            syscall_count: 0,
            last_syscall: None,
            dropped_upcall_count: 0,
            timeslice_expiration_count: 0,
        });

        // Handle any architecture-specific requirements for a new process.
        //
        // NOTE! We have to ensure that the start of process-accessible memory
        // (`app_memory_start`) is word-aligned. Since we currently start
        // process-accessible memory at the beginning of the allocated memory
        // region, we trust the MPU to give us a word-aligned starting address.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        match process.stored_state.map(|stored_state| {
            chip.userspace_kernel_boundary().initialize_process(
                app_accessible_memory.as_fluxptr(),
                initial_app_brk,
                stored_state,
            )
        }) {
            Some(Ok(())) => {}
            _ => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash={:#010X}-{:#010X} process={:?} - couldn't initialize process",
                        pb.flash.as_fluxptr().as_usize(),
                        pb.flash.as_fluxptr().as_usize() + pb.flash.len() - 1,
                        process_name
                    );
                }
                // Note that since remaining_memory was split by split_at_mut into
                // application memory and unused_memory, a failure here will leak
                // the application memory. Not leaking it requires being able to
                // reconstitute the original memory slice.
                return Err((ProcessLoadError::InternalError, unused_memory));
            }
        };

        let flash_start = process.flash.as_fluxptr();
        let app_start = flash_start
            .wrapping_add(process.header.get_app_start_offset() as usize)
            .as_usize();
        let init_fn = flash_start
            .wrapping_add(process.header.get_init_function_offset() as usize)
            .as_usize();

        process.tasks.map(|tasks| {
            let app_break = process
                .app_memory_break()
                .map_err(|_| ProcessLoadError::MpuConfigurationError)?
                .as_usize();
            tasks.enqueue(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: init_fn,
                argument0: app_start,
                argument1: process.memory_start.as_usize(),
                argument2: process.memory_len,
                argument3: app_break,
            }));
            Ok::<(), ProcessLoadError>(())
        });

        // Return the process object and a remaining memory for processes slice.
        Ok((Some(process), unused_memory))
    }

    /// Reset the process, resetting all of its state and re-initializing it so
    /// it can start running. Assumes the process is not running but is still in
    /// flash and still has its memory region allocated to it.
    #[flux_rs::trusted] // refinement error
    fn reset(&self) -> Result<(), ErrorCode> {
        // We need a new process identifier for this process since the restarted
        // version is in effect a new process. This is also necessary to
        // invalidate any stored `ProcessId`s that point to the old version of
        // the process. However, the process has not moved locations in the
        // processes array, so we copy the existing index.
        let old_index = self.process_id.get().index;
        let new_identifier = self.kernel.create_process_identifier();
        self.process_id
            .set(ProcessId::new(self.kernel, new_identifier, old_index));

        // Reset debug information that is per-execution and not per-process.
        self.debug.map(|debug| {
            debug.syscall_count = 0;
            debug.last_syscall = None;
            debug.dropped_upcall_count = 0;
            debug.timeslice_expiration_count = 0;
        });

        // Reset MPU region configuration.
        //
        // TODO: ideally, this would be moved into a helper function used by
        // both create() and reset(), but process load debugging complicates
        // this. We just want to create new config with only flash and memory
        // regions.
        //
        // We must have a previous MPU configuration stored, fault the
        // process if this invariant is violated. We avoid allocating
        // a new MPU configuration, as this may eventually exhaust the
        // number of available MPU configurations.
        let mut breaks_and_mpu_config = self.breaks_and_config.take().ok_or(ErrorCode::FAIL)?;
        self.chip
            .mpu()
            .reset_config(&mut breaks_and_mpu_config.mpu_config);

        // RAM and Flash

        // Re-determine the minimum amount of RAM the kernel must allocate to
        // the process based on the specific requirements of the syscall
        // implementation.
        let min_process_memory_size = self
            .chip
            .userspace_kernel_boundary()
            .initial_process_app_brk_size();

        // Recalculate initial_kernel_memory_size as was done in create()
        let grant_ptr_size = mem::size_of::<(usize, FluxPtrU8Mut)>();
        let grant_ptrs_num = self.kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        let initial_kernel_memory_size =
            grant_ptrs_offset + Self::CALLBACKS_OFFSET + Self::PROCESS_STRUCT_OFFSET;

        // allocate mpu regions for app flash and ram
        let app_mpu_mem = self.chip.mpu().allocate_app_memory_regions(
            self.mem_start(),
            self.memory_len,
            self.memory_len, //we want exactly as much as we had before restart
            min_process_memory_size,
            initial_kernel_memory_size,
            self.flash.as_fluxptr(),
            self.flash.len(),
            &mut breaks_and_mpu_config.mpu_config,
        );
        let breaks_and_size = match app_mpu_mem {
            Ok(breaks_and_size) => breaks_and_size,
            Err(mpu::AllocateAppMemoryError::FlashError) => {
                return Err(ErrorCode::FAIL);
            }
            Err(mpu::AllocateAppMemoryError::HeapError) => {
                // We couldn't configure the MPU for the process. This shouldn't
                // happen since we were able to start the process before, but at
                // this point it is better to leave the app faulted and not
                // schedule it.
                return Err(ErrorCode::NOMEM);
            }
        };

        // Reset memory pointers now that we know the layout of the process
        // memory and know that we can configure the MPU.

        let app_mpu_mem_start = breaks_and_size.breaks.memory_start;

        // app_brk is set based on minimum syscall size above the start of
        // memory.
        let app_brk = breaks_and_size.breaks.app_break;
        // self.app_break.set(app_brk);
        // kernel_brk is calculated backwards from the end of memory the size of
        // the initial kernel data structures.
        let kernel_brk =
            app_mpu_mem_start.as_usize() + breaks_and_size.memory_size - initial_kernel_memory_size;
        // self.kernel_memory_break.set(kernel_brk);
        // High water mark for `allow`ed memory is reset to the start of the
        // process's memory region.
        // self.allow_high_water_mark.set(app_mpu_mem_start);
        let breaks = ProcessBreaks {
            mem_start: self.memory_start,
            mem_len: self.memory_len,
            flash: self.flash,
            _flash_ghost: FlashGhostState::new(self.flash_start(), self.flash_size()),
            kernel_memory_break: FluxPtr::from(kernel_brk),
            app_break: app_brk,
            allow_high_water_mark: app_mpu_mem_start,
        };
        let new_breaks_and_mpu_config = BreaksAndMPUConfig {
            breaks,
            mpu_regions: breaks_and_mpu_config.mpu_regions,
            mpu_config: breaks_and_mpu_config.mpu_config,
        };
        self.breaks_and_config.replace(new_breaks_and_mpu_config);
        // self.breaks.set(breaks);
        // Store the adjusted MPU configuration:
        // self.mpu_config.replace(mpu_config);

        // Handle any architecture-specific requirements for a process when it
        // first starts (as it would when it is new).
        let ukb_init_process = self.stored_state.map_or(Err(()), |stored_state| unsafe {
            self.chip.userspace_kernel_boundary().initialize_process(
                app_mpu_mem_start,
                app_brk,
                stored_state,
            )
        });
        match ukb_init_process {
            Ok(()) => {}
            Err(()) => {
                // We couldn't initialize the architecture-specific state for
                // this process. This shouldn't happen since the app was able to
                // be started before, but at this point the app is no longer
                // valid. The best thing we can do now is leave the app as still
                // faulted and not schedule it.
                return Err(ErrorCode::RESERVE);
            }
        };

        self.restart_count.increment();

        // Mark the state as `Yielded` for the scheduler.
        self.state.set(State::Yielded);

        // And queue up this app to be restarted.
        let flash_start = self.flash_start();
        let app_start = flash_start
            .wrapping_add(self.header.get_app_start_offset() as usize)
            .as_usize();
        let init_fn = flash_start
            .wrapping_add(self.header.get_init_function_offset() as usize)
            .as_usize();

        self.enqueue_task(Task::FunctionCall(FunctionCall {
            source: FunctionCallSource::Kernel,
            pc: init_fn,
            argument0: app_start,
            argument1: self.memory_start.as_usize(),
            argument2: self.memory_len,
            argument3: self
                .app_memory_break()
                .map_err(|_| ErrorCode::FAIL)?
                .as_usize(),
        }))
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// is within the RAM bounds currently exposed to the processes (i.e. ending
    /// at `app_break`). If this method returns `true`, the buffer is guaranteed
    /// to be accessible to the process and to not overlap with the grant
    /// region.
    #[flux_rs::sig(fn(&Self[@p], FluxPtr[@ptr], usize[@sz]) -> Result<bool{b: b == true => ptr >= p.mem_start}, ()>)]
    fn in_app_owned_memory(&self, buf_start_addr: FluxPtrU8Mut, size: usize) -> Result<bool, ()> {
        let buf_end_addr = buf_start_addr.wrapping_add(size);
        self.breaks_and_config.map_or(Err(()), |breaks_and_config| {
            Ok(breaks_and_config.in_app_owned_memory(buf_start_addr, buf_end_addr))
        })
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// are within the readable region of an application's flash memory.  If
    /// this method returns true, the buffer is guaranteed to be readable to the
    /// process.
    fn in_app_flash_memory(&self, buf_start_addr: FluxPtrU8Mut, size: usize) -> bool {
        let buf_end_addr = buf_start_addr.wrapping_add(size);
        buf_end_addr >= buf_start_addr
            && buf_start_addr >= self.flash_non_protected_start()
            && buf_end_addr <= self.flash_end()
    }

    /// Reset all `grant_ptr`s to NULL.
    unsafe fn grant_ptrs_reset(&self) {
        self.grant_pointers.map(|grant_pointers| {
            for grant_entry in grant_pointers.iter_mut() {
                grant_entry.driver_num = 0;
                grant_entry.grant_ptr = FluxPtr::null_mut();
            }
        });
    }

    /// Allocate memory in a process's grant region.
    ///
    /// Ensures that the allocation is of `size` bytes and aligned to `align`
    /// bytes.
    ///
    /// If there is not enough memory, or the MPU cannot isolate the process
    /// accessible region from the new kernel memory break after doing the
    /// allocation, then this will return `None`.
    // #[flux_rs::sig(fn(&Self, usize, usize{align: align > 0}) -> Option<NonNull<u8>>)]
    fn allocate_in_grant_region_internal(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        self.breaks_and_config.and_then(|breaks_and_config| {
            breaks_and_config.allocate_in_grant_region_internal(size, align)
        })
    }

    /// Create the identifier for a custom grant that grant.rs uses to access
    /// the custom grant.
    ///
    /// We create this identifier by calculating the number of bytes between
    /// where the custom grant starts and the end of the process memory.
    #[flux_rs::sig(fn(self: &Self[@proc], ptr: NonNull<u8>{ptr: ptr < proc.mem_start + proc.mem_len}) -> ProcessCustomGrantIdentifier)]
    fn create_custom_grant_identifier(&self, ptr: NonNull<u8>) -> ProcessCustomGrantIdentifier {
        let custom_grant_address = ptr.as_fluxptr().as_usize();
        let process_memory_end = self.mem_end().as_usize();

        assume(process_memory_end > custom_grant_address); // refine ProcessStandard by mem_end + add precondition for input

        ProcessCustomGrantIdentifier {
            offset: process_memory_end - custom_grant_address,
        }
    }

    /// Use a `ProcessCustomGrantIdentifier` to find the address of the
    /// custom grant.
    ///
    /// This reverses `create_custom_grant_identifier()`.
    fn get_custom_grant_address(&self, identifier: ProcessCustomGrantIdentifier) -> usize {
        let process_memory_end = self.mem_end().as_usize();
        assume(process_memory_end > identifier.offset);
        // Subtract the offset in the identifier from the end of the process
        // memory to get the address of the custom grant.
        process_memory_end - identifier.offset
    }

    /// The start address of allocated RAM for this process.
    #[flux_rs::sig(fn(self: &Self[@p]) -> FluxPtrU8Mut[p.mem_start])]
    fn mem_start(&self) -> FluxPtrU8Mut {
        self.memory_start
    }

    /// The first address after the end of the allocated RAM for this process.
    #[flux_rs::sig(fn(self: &Self[@p]) -> FluxPtrU8Mut[p.mem_start + p.mem_len])]
    fn mem_end(&self) -> FluxPtrU8Mut {
        self.memory_start.wrapping_add(self.memory_len)
    }

    /// The start address of the flash region allocated for this process.
    #[flux_rs::trusted]
    #[flux_rs::sig(fn (&Self[@f]) -> FluxPtrU8Mut[f.flash_start])]
    fn flash_start(&self) -> FluxPtrU8Mut {
        self.flash.as_fluxptr()
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn (&Self[@f]) -> usize[f.flash_len])]
    fn flash_size(&self) -> usize {
        self.flash.len()
    }

    /// Get the first address of process's flash that isn't protected by the
    /// kernel. The protected range of flash contains the TBF header and
    /// potentially other state the kernel is storing on behalf of the process,
    /// and cannot be edited by the process.
    fn flash_non_protected_start(&self) -> FluxPtrU8Mut {
        ((self.flash.as_fluxptr().as_usize()) + self.header.get_protected_size() as usize)
            .as_fluxptr()
    }

    /// The first address after the end of the flash region allocated for this
    /// process.
    #[flux_rs::trusted]
    #[flux_rs::sig(fn (&Self[@f]) -> FluxPtrU8Mut[f.flash_start + f.flash_len])]
    fn flash_end(&self) -> FluxPtrU8Mut {
        self.flash.as_fluxptr().wrapping_add(self.flash.len())
    }

    /// The lowest address of the grant region for the process.
    fn kernel_memory_break(&self) -> Result<FluxPtrU8Mut, ()> {
        self.breaks_and_config.map_or(Err(()), |breaks_and_config| {
            Ok(breaks_and_config.breaks.kernel_memory_break)
        })
    }

    /// Return the highest address the process has access to, or the current
    /// process memory brk.
    fn app_memory_break(&self) -> Result<FluxPtrU8Mut, ()> {
        self.breaks_and_config.map_or(Err(()), |breaks_and_config| {
            Ok(breaks_and_config.breaks.app_break)
        })
    }
}
