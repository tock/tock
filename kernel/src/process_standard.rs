//! Tock default Process implementation.
//!
//! `ProcessStandard` is an implementation for a userspace process running on
//! the Tock kernel.

use core::cell::Cell;
use core::cmp;
use core::fmt::Write;
use core::ptr::NonNull;
use core::{mem, ptr, slice, str};

use crate::collections::queue::Queue;
use crate::collections::ring_buffer::RingBuffer;
use crate::config;
use crate::debug;
use crate::errorcode::ErrorCode;
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::platform::mpu::{self, MPU};
use crate::process::{Error, FunctionCall, FunctionCallSource, Process, State, Task};
use crate::process::{FaultAction, ProcessCustomGrantIdentifer, ProcessId, ProcessStateCell};
use crate::process_policies::ProcessFaultPolicy;
use crate::process_utilities::ProcessLoadError;
use crate::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use crate::syscall::{self, Syscall, SyscallReturn, UserspaceKernelBoundary};
use crate::upcall::UpcallId;
use crate::utilities::cells::{MapCell, NumericCellExt};

// The completion code for a process if it faulted.
const COMPLETION_FAULT: u32 = 0xffffffff;

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
    app_heap_start_pointer: Option<*const u8>,

    /// Where the start of the stack is for the process. If the kernel does the
    /// PIC setup for this app then we know this, otherwise we need the app to
    /// tell us where it put its stack.
    app_stack_start_pointer: Option<*const u8>,

    /// How low have we ever seen the stack pointer.
    app_stack_min_pointer: Option<*const u8>,

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
    grant_ptr: *mut u8,
}

/// A type for userspace processes in Tock.
pub struct ProcessStandard<'a, C: 'static + Chip> {
    /// Identifier of this process and the index of the process in the process
    /// table.
    process_id: Cell<ProcessId>,

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
    memory_start: *const u8,
    /// Number of bytes of memory allocated to this process.
    memory_len: usize,

    /// Reference to the slice of `GrantPointerEntry`s stored in the process's
    /// memory reserved for the kernel. These driver numbers are zero and
    /// pointers are null if the grant region has not been allocated. When the
    /// grant region is allocated these pointers are updated to point to the
    /// allocated memory and the driver number is set to match the driver that
    /// owns the grant. No other reference to these pointers exists in the Tock
    /// kernel.
    grant_pointers: MapCell<&'static mut [GrantPointerEntry]>,

    /// Pointer to the end of the allocated (and MPU protected) grant region.
    kernel_memory_break: Cell<*const u8>,

    /// Pointer to the end of process RAM that has been sbrk'd to the process.
    app_break: Cell<*const u8>,

    /// Pointer to high water mark for process buffers shared through `allow`
    allow_high_water_mark: Cell<*const u8>,

    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: &'static [u8],

    /// Collection of pointers to the TBF header in flash.
    header: tock_tbf::types::TbfHeader,

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
    state: ProcessStateCell<'static>,

    /// How to respond if this process faults.
    fault_policy: &'a dyn ProcessFaultPolicy,

    /// Configuration data for the MPU
    mpu_config: MapCell<<<C as Chip>::MPU as MPU>::MpuConfig>,

    /// MPU regions are saved as a pointer-size pair.
    mpu_regions: [Cell<Option<mpu::Region>>; 6],

    /// Essentially a list of upcalls that want to call functions in the
    /// process.
    tasks: MapCell<RingBuffer<'a, Task>>,

    /// Count of how many times this process has entered the fault condition and
    /// been restarted. This is used by some `ProcessRestartPolicy`s to
    /// determine if the process should be restarted or not.
    restart_count: Cell<usize>,

    /// Name of the app.
    process_name: &'static str,

    /// Values kept so that we can print useful debug messages when apps fault.
    debug: MapCell<ProcessStandardDebug>,
}

impl<C: Chip> Process for ProcessStandard<'_, C> {
    fn processid(&self) -> ProcessId {
        self.process_id.get()
    }

    fn enqueue_task(&self, task: Task) -> Result<(), ErrorCode> {
        // If this app is in a `Fault` state then we shouldn't schedule
        // any work for it.
        if !self.is_active() {
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

        if ret.is_ok() {
            self.kernel.increment_work();
        } else {
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
            tasks.retain(|task| match task {
                // Remove only tasks that are function calls with an id equal
                // to `upcall_id`.
                Task::FunctionCall(function_call) => match function_call.source {
                    FunctionCallSource::Kernel => true,
                    FunctionCallSource::Driver(id) => {
                        if id != upcall_id {
                            true
                        } else {
                            self.kernel.decrement_work();
                            false
                        }
                    }
                },
                _ => true,
            });
            if config::CONFIG.trace_syscalls {
                let count_after = tasks.len();
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

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_yielded_state(&self) {
        if self.state.get() == State::Running {
            self.state.update(State::Yielded);
        }
    }

    fn stop(&self) {
        match self.state.get() {
            State::Running => self.state.update(State::StoppedRunning),
            State::Yielded => self.state.update(State::StoppedYielded),
            _ => {} // Do nothing
        }
    }

    fn resume(&self) {
        match self.state.get() {
            State::StoppedRunning => self.state.update(State::Running),
            State::StoppedYielded => self.state.update(State::Yielded),
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
                self.state.update(State::Faulted);
                panic!("Process {} had a fault", self.process_name);
            }
            FaultAction::Restart => {
                self.try_restart(COMPLETION_FAULT);
            }
            FaultAction::Stop => {
                // This looks a lot like restart, except we just leave the app
                // how it faulted and mark it as `Faulted`. By clearing
                // all of the app's todo work it will not be scheduled, and
                // clearing all of the grant regions will cause capsules to drop
                // this app as well.
                self.terminate(COMPLETION_FAULT);
                self.state.update(State::Faulted);
            }
        }
    }

    fn try_restart(&self, completion_code: u32) {
        // Terminate the process, freeing its state and removing any
        // pending tasks from the scheduler's queue.
        self.terminate(completion_code);

        // If there is a kernel policy that controls restarts, it should be
        // implemented here. For now, always restart.
        let _res = self.restart();

        // Decide what to do with res later. E.g., if we can't restart
        // want to reclaim the process resources.
    }

    fn terminate(&self, _completion_code: u32) {
        // Remove the tasks that were scheduled for the app from the
        // amount of work queue.
        let tasks_len = self.tasks.map_or(0, |tasks| tasks.len());
        for _ in 0..tasks_len {
            self.kernel.decrement_work();
        }

        // And remove those tasks
        self.tasks.map(|tasks| {
            tasks.empty();
        });

        // Clear any grant regions this app has setup with any capsules.
        unsafe {
            self.grant_ptrs_reset();
        }

        // Mark the app as stopped so the scheduler won't try to run it.
        self.state.update(State::Terminated);
    }

    fn get_restart_count(&self) -> usize {
        self.restart_count.get()
    }

    fn has_tasks(&self) -> bool {
        self.tasks.map_or(false, |tasks| tasks.has_elements())
    }

    fn dequeue_task(&self) -> Option<Task> {
        self.tasks.map_or(None, |tasks| {
            tasks.dequeue().map(|cb| {
                self.kernel.decrement_work();
                cb
            })
        })
    }

    fn mem_start(&self) -> *const u8 {
        self.memory_start
    }

    fn mem_end(&self) -> *const u8 {
        self.memory_start.wrapping_add(self.memory_len)
    }

    fn flash_start(&self) -> *const u8 {
        self.flash.as_ptr()
    }

    fn flash_non_protected_start(&self) -> *const u8 {
        ((self.flash.as_ptr() as usize) + self.header.get_protected_size() as usize) as *const u8
    }

    fn flash_end(&self) -> *const u8 {
        self.flash.as_ptr().wrapping_add(self.flash.len())
    }

    fn kernel_memory_break(&self) -> *const u8 {
        self.kernel_memory_break.get()
    }

    fn number_writeable_flash_regions(&self) -> usize {
        self.header.number_writeable_flash_regions()
    }

    fn get_writeable_flash_region(&self, region_index: usize) -> (u32, u32) {
        self.header.get_writeable_flash_region(region_index)
    }

    fn update_stack_start_pointer(&self, stack_pointer: *const u8) {
        if stack_pointer >= self.mem_start() && stack_pointer < self.mem_end() {
            self.debug.map(|debug| {
                debug.app_stack_start_pointer = Some(stack_pointer);

                // We also reset the minimum stack pointer because whatever value
                // we had could be entirely wrong by now.
                debug.app_stack_min_pointer = Some(stack_pointer);
            });
        }
    }

    fn update_heap_start_pointer(&self, heap_pointer: *const u8) {
        if heap_pointer >= self.mem_start() && heap_pointer < self.mem_end() {
            self.debug.map(|debug| {
                debug.app_heap_start_pointer = Some(heap_pointer);
            });
        }
    }

    fn app_memory_break(&self) -> *const u8 {
        self.app_break.get()
    }

    fn setup_mpu(&self) {
        self.mpu_config.map(|config| {
            self.chip.mpu().configure_mpu(&config, &self.processid());
        });
    }

    fn add_mpu_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
    ) -> Option<mpu::Region> {
        self.mpu_config.and_then(|mut config| {
            let new_region = self.chip.mpu().allocate_region(
                unallocated_memory_start,
                unallocated_memory_size,
                min_region_size,
                mpu::Permissions::ReadWriteOnly,
                &mut config,
            );

            if new_region.is_none() {
                return None;
            }

            for region in self.mpu_regions.iter() {
                if region.get().is_none() {
                    region.set(new_region);
                    return new_region;
                }
            }

            // Not enough room in Process struct to store the MPU region.
            None
        })
    }

    fn sbrk(&self, increment: isize) -> Result<*const u8, Error> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return Err(Error::InactiveApp);
        }

        let new_break = unsafe { self.app_break.get().offset(increment) };
        self.brk(new_break)
    }

    fn brk(&self, new_break: *const u8) -> Result<*const u8, Error> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return Err(Error::InactiveApp);
        }

        self.mpu_config
            .map_or(Err(Error::KernelError), |mut config| {
                if new_break < self.allow_high_water_mark.get() || new_break >= self.mem_end() {
                    Err(Error::AddressOutOfBounds)
                } else if new_break > self.kernel_memory_break.get() {
                    Err(Error::OutOfMemory)
                } else if let Err(_) = self.chip.mpu().update_app_memory_region(
                    new_break,
                    self.kernel_memory_break.get(),
                    mpu::Permissions::ReadWriteOnly,
                    &mut config,
                ) {
                    Err(Error::OutOfMemory)
                } else {
                    let old_break = self.app_break.get();
                    self.app_break.set(new_break);
                    self.chip.mpu().configure_mpu(&config, &self.processid());
                    Ok(old_break)
                }
            })
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn build_readwrite_process_buffer(
        &self,
        buf_start_addr: *mut u8,
        size: usize,
    ) -> Result<ReadWriteProcessBuffer, ErrorCode> {
        if !self.is_active() {
            // Do not operate on an inactive process
            return Err(ErrorCode::FAIL);
        }

        // A process is allowed to pass any pointer if the buffer
        // length is 0, as to revoke kernel access to a memory region
        // without granting access to another one
        if size == 0 {
            // Clippy complains that we're deferencing a pointer in a
            // public and safe function here. While we are not
            // dereferencing the pointer here, we pass it along to an
            // unsafe function, which is as dangerous (as it is likely
            // to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as a buffer
            // of length 0 will never allow dereferencing any memory
            // in a safe manner.
            //
            // ### Safety
            //
            // We specific a zero-length buffer, so the implementation of
            // `ReadWriteProcessBuffer` will handle any safety issues. Therefore, we
            // can encapsulate the unsafe.
            Ok(unsafe { ReadWriteProcessBuffer::new(buf_start_addr, 0, self.processid()) })
        } else if self.in_app_owned_memory(buf_start_addr, size) {
            // TODO: Check for buffer aliasing here

            // Valid buffer, we need to adjust the app's watermark
            // note: in_app_owned_memory ensures this offset does not wrap
            let buf_end_addr = buf_start_addr.wrapping_add(size);
            let new_water_mark = cmp::max(self.allow_high_water_mark.get(), buf_end_addr);
            self.allow_high_water_mark.set(new_water_mark);

            // Clippy complains that we're deferencing a pointer in a
            // public and safe function here. While we are not
            // deferencing the pointer here, we pass it along to an
            // unsafe function, which is as dangerous (as it is likely
            // to be deferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as long as
            // we make sure that we're pointing towards userspace
            // memory (verified using `in_app_owned_memory`) and
            // respect alignment and other constraints of the Rust
            // references created by ReadWriteProcessBuffer.
            //
            // ### Safety
            //
            // We encapsulate the unsafe here on the condition in the TODO
            // above, as we must ensure that this `ReadWriteProcessBuffer` will be
            // the only reference to this memory.
            Ok(unsafe { ReadWriteProcessBuffer::new(buf_start_addr, size, self.processid()) })
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn build_readonly_process_buffer(
        &self,
        buf_start_addr: *const u8,
        size: usize,
    ) -> Result<ReadOnlyProcessBuffer, ErrorCode> {
        if !self.is_active() {
            // Do not operate on an inactive process
            return Err(ErrorCode::FAIL);
        }

        // A process is allowed to pass any pointer if the buffer
        // length is 0, as to revoke kernel access to a memory region
        // without granting access to another one
        if size == 0 {
            // Clippy complains that we're deferencing a pointer in a
            // public and safe function here. While we are not
            // deferencing the pointer here, we pass it along to an
            // unsafe function, which is as dangerous (as it is likely
            // to be deferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as a buffer
            // of length 0 will never allow dereferencing any memory
            // in a safe manner.
            //
            // ### Safety
            //
            // We specific a zero-length buffer, so the implementation of
            // `ReadOnlyProcessBuffer` will handle any safety issues. Therefore, we
            // can encapsulate the unsafe.
            Ok(unsafe { ReadOnlyProcessBuffer::new(buf_start_addr, 0, self.processid()) })
        } else if self.in_app_owned_memory(buf_start_addr, size)
            || self.in_app_flash_memory(buf_start_addr, size)
        {
            // TODO: Check for buffer aliasing here

            // Valid buffer, we need to adjust the app's watermark
            // note: in_app_owned_memory ensures this offset does not wrap
            let buf_end_addr = buf_start_addr.wrapping_add(size);
            let new_water_mark = cmp::max(self.allow_high_water_mark.get(), buf_end_addr);
            self.allow_high_water_mark.set(new_water_mark);

            // Clippy complains that we're deferencing a pointer in a
            // public and safe function here. While we are not
            // deferencing the pointer here, we pass it along to an
            // unsafe function, which is as dangerous (as it is likely
            // to be deferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as long as
            // we make sure that we're pointing towards userspace
            // memory (verified using `in_app_owned_memory` or
            // `in_app_flash_memory`) and respect alignment and other
            // constraints of the Rust references created by
            // ReadWriteProcessBuffer.
            //
            // ### Safety
            //
            // We encapsulate the unsafe here on the condition in the TODO
            // above, as we must ensure that this `ReadOnlyProcessBuffer` will be
            // the only reference to this memory.
            Ok(unsafe { ReadOnlyProcessBuffer::new(buf_start_addr, size, self.processid()) })
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    unsafe fn set_byte(&self, addr: *mut u8, value: u8) -> bool {
        if self.in_app_owned_memory(addr, 1) {
            // We verify that this will only write process-accessible memory,
            // but this can still be undefined behavior if something else holds
            // a reference to this memory.
            *addr = value;
            true
        } else {
            false
        }
    }

    fn grant_is_allocated(&self, grant_num: usize) -> Option<bool> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return None;
        }

        // Update the grant pointer to the address of the new allocation.
        self.grant_pointers.map_or(None, |grant_pointers| {
            // Implement `grant_pointers[grant_num]` without a
            // chance of a panic.
            grant_pointers
                .get(grant_num)
                .map_or(None, |grant_entry| Some(!grant_entry.grant_ptr.is_null()))
        })
    }

    fn allocate_grant(
        &self,
        grant_num: usize,
        driver_num: usize,
        size: usize,
        align: usize,
    ) -> Option<NonNull<u8>> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return None;
        }

        // Verify the grant_num is valid.
        if grant_num >= self.kernel.get_grant_count_and_finalize() {
            return None;
        }

        // Verify that the grant is not already allocated. If the pointer is not
        // null then the grant is already allocated.
        if let Some(is_allocated) = self.grant_is_allocated(grant_num) {
            if is_allocated {
                return None;
            }
        }

        // Verify that there is not already a grant allocated with the same
        // driver_num.
        let exists = self.grant_pointers.map_or(false, |grant_pointers| {
            // Check our list of grant pointers if the driver number is used.
            grant_pointers.iter().any(|grant_entry| {
                // Check if the grant is both allocated (its grant pointer
                // is non null) and the driver number matches.
                (!grant_entry.grant_ptr.is_null()) && grant_entry.driver_num == driver_num
            })
        });
        // If we find a match, then the driver_num must already be used and the
        // grant allocation fails.
        if exists {
            return None;
        }

        // Use the shared grant allocator function to actually allocate memory.
        // Returns `None` if the allocation cannot be created.
        if let Some(grant_ptr) = self.allocate_in_grant_region_internal(size, align) {
            // Update the grant pointer to the address of the new allocation.
            self.grant_pointers.map_or(None, |grant_pointers| {
                // Implement `grant_pointers[grant_num] = grant_ptr` without a
                // chance of a panic.
                grant_pointers
                    .get_mut(grant_num)
                    .map_or(None, |grant_entry| {
                        // Actually set the driver num and grant pointer.
                        grant_entry.driver_num = driver_num;
                        grant_entry.grant_ptr = grant_ptr.as_ptr() as *mut u8;

                        // If all of this worked, return the allocated pointer.
                        Some(grant_ptr)
                    })
            })
        } else {
            // Could not allocate the memory for the grant region.
            None
        }
    }

    fn allocate_custom_grant(
        &self,
        size: usize,
        align: usize,
    ) -> Option<(ProcessCustomGrantIdentifer, NonNull<u8>)> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return None;
        }

        // Use the shared grant allocator function to actually allocate memory.
        // Returns `None` if the allocation cannot be created.
        if let Some(ptr) = self.allocate_in_grant_region_internal(size, align) {
            // Create the identifier that the caller will use to get access to
            // this custom grant in the future.
            let identifier = self.create_custom_grant_identifier(ptr);

            Some((identifier, ptr))
        } else {
            // Could not allocate memory for the custom grant.
            None
        }
    }

    fn enter_grant(&self, grant_num: usize) -> Result<*mut u8, Error> {
        // Do not try to access the grant region of inactive process.
        if !self.is_active() {
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
                        if (grant_ptr as usize) & 0x1 == 0x1 {
                            // Lowest bit is one, meaning this grant has been
                            // entered.
                            Err(Error::AlreadyInUse)
                        } else {
                            // Now, to mark that the grant has been entered, we
                            // set the lowest bit to one and save this as the
                            // grant pointer.
                            grant_entry.grant_ptr = (grant_ptr as usize | 0x1) as *mut u8;

                            // And we return the grant pointer to the entered
                            // grant.
                            Ok(grant_ptr)
                        }
                    }
                    None => Err(Error::AddressOutOfBounds),
                }
            })
    }

    fn enter_custom_grant(
        &self,
        identifier: ProcessCustomGrantIdentifer,
    ) -> Result<*mut u8, Error> {
        // Do not try to access the grant region of inactive process.
        if !self.is_active() {
            return Err(Error::InactiveApp);
        }

        // Get the address of the custom grant based on the identifier.
        let custom_grant_address = self.get_custom_grant_address(identifier);

        // We never deallocate custom grants and only we can change the
        // `identifier` so we know this is a valid address for the custom grant.
        Ok(custom_grant_address as *mut u8)
    }

    fn leave_grant(&self, grant_num: usize) {
        // Do not modify an inactive process.
        if !self.is_active() {
            return;
        }

        self.grant_pointers.map(|grant_pointers| {
            // Implement `grant_pointers[grant_num]` without a chance of a
            // panic.
            match grant_pointers.get_mut(grant_num) {
                Some(grant_entry) => {
                    // Get a copy of the actual grant pointer.
                    let grant_ptr = grant_entry.grant_ptr;

                    // Now, to mark that the grant has been released, we set the
                    // lowest bit back to zero and save this as the grant
                    // pointer.
                    grant_entry.grant_ptr = (grant_ptr as usize & !0x1) as *mut u8;
                }
                None => {}
            }
        });
    }

    fn grant_allocated_count(&self) -> Option<usize> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return None;
        }

        self.grant_pointers.map(|grant_pointers| {
            // Filter our list of grant pointers into just the non null ones,
            // and count those. A grant is allocated if its grant pointer is non
            // null.
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
                // pointer is non null.
                match grant_pointers.iter().position(|grant_entry| {
                    // Only consider allocated grants.
                    (!grant_entry.grant_ptr.is_null()) && grant_entry.driver_num == driver_num
                }) {
                    Some(idx) => Ok(idx),
                    None => Err(Error::OutOfMemory),
                }
            })
    }

    fn is_valid_upcall_function_pointer(&self, upcall_fn: NonNull<()>) -> bool {
        let ptr = upcall_fn.as_ptr() as *const u8;
        let size = mem::size_of::<*const u8>();

        // It is ok if this function is in memory or flash.
        self.in_app_flash_memory(ptr, size) || self.in_app_owned_memory(ptr, size)
    }

    fn get_process_name(&self) -> &'static str {
        self.process_name
    }

    fn set_syscall_return_value(&self, return_value: SyscallReturn) {
        match self.stored_state.map(|stored_state| unsafe {
            // Actually set the return value for a particular process.
            //
            // The UKB implementation uses the bounds of process-accessible
            // memory to verify that any memory changes are valid. Here, the
            // unsafe promise we are making is that the bounds passed to the UKB
            // are correct.
            self.chip
                .userspace_kernel_boundary()
                .set_syscall_return_value(
                    self.mem_start(),
                    self.app_break.get(),
                    stored_state,
                    return_value,
                )
        }) {
            Some(Ok(())) => {
                // If we get an `Ok` we are all set.
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
            unsafe {
                self.chip.userspace_kernel_boundary().set_process_function(
                    self.mem_start(),
                    self.app_break.get(),
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
                self.state.update(State::Running);
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

    fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        // Cannot switch to an invalid process
        if !self.is_active() {
            return None;
        }

        let (switch_reason, stack_pointer) =
            self.stored_state.map_or((None, None), |stored_state| {
                // Switch to the process. We guarantee that the memory pointers
                // we pass are valid, ensuring this context switch is safe.
                // Therefore we encapsulate the `unsafe`.
                unsafe {
                    let (switch_reason, optional_stack_pointer) = self
                        .chip
                        .userspace_kernel_boundary()
                        .switch_to_process(self.mem_start(), self.app_break.get(), stored_state);
                    (Some(switch_reason), optional_stack_pointer)
                }
            });

        // If the UKB implementation passed us a stack pointer, update our
        // debugging state. This is completely optional.
        stack_pointer.map(|sp| {
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
        });

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

    fn debug_heap_start(&self) -> Option<*const u8> {
        self.debug
            .map_or(None, |debug| debug.app_heap_start_pointer.map(|p| p))
    }

    fn debug_stack_start(&self) -> Option<*const u8> {
        self.debug
            .map_or(None, |debug| debug.app_stack_start_pointer.map(|p| p))
    }

    fn debug_stack_end(&self) -> Option<*const u8> {
        self.debug
            .map_or(None, |debug| debug.app_stack_min_pointer.map(|p| p))
    }

    fn print_memory_map(&self, writer: &mut dyn Write) {
        if !config::CONFIG.debug_panics {
            return;
        }
        // Flash
        let flash_end = self.flash.as_ptr().wrapping_add(self.flash.len()) as usize;
        let flash_start = self.flash.as_ptr() as usize;
        let flash_protected_size = self.header.get_protected_size() as usize;
        let flash_app_start = flash_start + flash_protected_size;
        let flash_app_size = flash_end - flash_app_start;

        // SRAM addresses
        let sram_end = self.mem_end() as usize;
        let sram_grant_start = self.kernel_memory_break.get() as usize;
        let sram_heap_end = self.app_break.get() as usize;
        let sram_heap_start: Option<usize> = self.debug.map_or(None, |debug| {
            debug.app_heap_start_pointer.map(|p| p as usize)
        });
        let sram_stack_start: Option<usize> = self.debug.map_or(None, |debug| {
            debug.app_stack_start_pointer.map(|p| p as usize)
        });
        let sram_stack_bottom: Option<usize> = self.debug.map_or(None, |debug| {
            debug.app_stack_min_pointer.map(|p| p as usize)
        });
        let sram_start = self.mem_start() as usize;

        // SRAM sizes
        let sram_grant_size = sram_end - sram_grant_start;
        let sram_grant_allocated = sram_end - sram_grant_start;

        // application statistics
        let events_queued = self.tasks.map_or(0, |tasks| tasks.len());
        let syscall_count = self.debug.map_or(0, |debug| debug.syscall_count);
        let last_syscall = self.debug.map(|debug| debug.last_syscall);
        let dropped_upcall_count = self.debug.map_or(0, |debug| debug.dropped_upcall_count);
        let restart_count = self.restart_count.get();

        let _ = writer.write_fmt(format_args!(
            "\
             𝐀𝐩𝐩: {}   -   [{:?}]\
             \r\n Events Queued: {}   Syscall Count: {}   Dropped Upcall Count: {}\
             \r\n Restart Count: {}\r\n",
            self.process_name,
            self.state.get(),
            events_queued,
            syscall_count,
            dropped_upcall_count,
            restart_count,
        ));

        let _ = match last_syscall {
            Some(syscall) => writer.write_fmt(format_args!(" Last Syscall: {:?}\r\n", syscall)),
            None => writer.write_str(" Last Syscall: None\r\n"),
        };

        let _ = writer.write_fmt(format_args!(
            "\
             \r\n\
             \r\n ╔═══════════╤══════════════════════════════════════════╗\
             \r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
             \r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
             \r\n             │ ▼ Grant      {:6} | {:6}{}\
             \r\n  {:#010X} ┼───────────────────────────────────────────\
             \r\n             │ Unused\
             \r\n  {:#010X} ┼───────────────────────────────────────────",
            sram_end,
            sram_grant_size,
            sram_grant_allocated,
            exceeded_check(sram_grant_size, sram_grant_allocated),
            sram_grant_start,
            sram_heap_end,
        ));

        match sram_heap_start {
            Some(sram_heap_start) => {
                let sram_heap_size = sram_heap_end - sram_heap_start;
                let sram_heap_allocated = sram_grant_start - sram_heap_start;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\n             │ ▲ Heap       {:6} | {:6}{}     S\
                     \r\n  {:#010X} ┼─────────────────────────────────────────── R",
                    sram_heap_size,
                    sram_heap_allocated,
                    exceeded_check(sram_heap_size, sram_heap_allocated),
                    sram_heap_start,
                ));
            }
            None => {
                let _ = writer.write_str(
                    "\
                     \r\n             │ ▲ Heap            ? |      ?               S\
                     \r\n  ?????????? ┼─────────────────────────────────────────── R",
                );
            }
        }

        match (sram_heap_start, sram_stack_start) {
            (Some(sram_heap_start), Some(sram_stack_start)) => {
                let sram_data_size = sram_heap_start - sram_stack_start;
                let sram_data_allocated = sram_data_size as usize;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\n             │ Data         {:6} | {:6}               A",
                    sram_data_size, sram_data_allocated,
                ));
            }
            _ => {
                let _ = writer.write_str(
                    "\
                     \r\n             │ Data              ? |      ?               A",
                );
            }
        }

        match (sram_stack_start, sram_stack_bottom) {
            (Some(sram_stack_start), Some(sram_stack_bottom)) => {
                let sram_stack_size = sram_stack_start - sram_stack_bottom;
                let sram_stack_allocated = sram_stack_start - sram_start;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\n  {:#010X} ┼─────────────────────────────────────────── M\
                     \r\n             │ ▼ Stack      {:6} | {:6}{}",
                    sram_stack_start,
                    sram_stack_size,
                    sram_stack_allocated,
                    exceeded_check(sram_stack_size, sram_stack_allocated),
                ));
            }
            _ => {
                let _ = writer.write_str(
                    "\
                     \r\n  ?????????? ┼─────────────────────────────────────────── M\
                     \r\n             │ ▼ Stack           ? |      ?",
                );
            }
        }

        let _ = writer.write_fmt(format_args!(
            "\
             \r\n  {:#010X} ┼───────────────────────────────────────────\
             \r\n             │ Unused\
             \r\n  {:#010X} ┴───────────────────────────────────────────\
             \r\n             .....\
             \r\n  {:#010X} ┬─────────────────────────────────────────── F\
             \r\n             │ App Flash    {:6}                        L\
             \r\n  {:#010X} ┼─────────────────────────────────────────── A\
             \r\n             │ Protected    {:6}                        S\
             \r\n  {:#010X} ┴─────────────────────────────────────────── H\
             \r\n",
            sram_stack_bottom.unwrap_or(0),
            sram_start,
            flash_end,
            flash_app_size,
            flash_app_start,
            flash_protected_size,
            flash_start
        ));
    }

    fn print_full_process(&self, writer: &mut dyn Write) {
        if !config::CONFIG.debug_panics {
            return;
        }
        self.print_memory_map(writer);

        self.stored_state.map(|stored_state| {
            // We guarantee the memory bounds pointers provided to the UKB are
            // correct.
            unsafe {
                self.chip.userspace_kernel_boundary().print_context(
                    self.mem_start(),
                    self.app_break.get(),
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
                                "  Grant {:>2} {:#x}: {:p}",
                                index, grant_entry.driver_num, grant_entry.grant_ptr
                            ));
                        }
                    });
                }
                let _ = writer.write_fmt(format_args!("\r\n"));
            }
        });

        // Display the current state of the MPU for this process.
        self.mpu_config.map(|config| {
            let _ = writer.write_fmt(format_args!("{}", config));
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
                     \r\nTo debug, run `make lst` in the app's folder\
                     \r\nand open the arch.{:#x}.{:#x}.lst file.\r\n\r\n",
                    debug.fixed_address_flash.unwrap_or(0),
                    debug.fixed_address_ram.unwrap_or(0)
                ));
            } else {
                // PIC, need to specify the addresses.
                let sram_start = self.mem_start() as usize;
                let flash_start = self.flash.as_ptr() as usize;
                let flash_init_fn = flash_start + self.header.get_init_function_offset() as usize;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\nTo debug, run `make debug RAM_START={:#x} FLASH_INIT={:#x}`\
                     \r\nin the app's folder and open the .lst file.\r\n\r\n",
                    sram_start, flash_init_fn
                ));
            }
        });
    }
}

// Only used if debug_panics == true
#[allow(unused)]
fn exceeded_check(size: usize, allocated: usize) -> &'static str {
    if size > allocated {
        " EXCEEDED!"
    } else {
        "          "
    }
}

impl<C: 'static + Chip> ProcessStandard<'_, C> {
    // Memory offset for upcall ring buffer (10 element length).
    const CALLBACK_LEN: usize = 10;
    const CALLBACKS_OFFSET: usize = mem::size_of::<Task>() * Self::CALLBACK_LEN;

    // Memory offset to make room for this process's metadata.
    const PROCESS_STRUCT_OFFSET: usize = mem::size_of::<ProcessStandard<C>>();

    pub(crate) unsafe fn create<'a>(
        kernel: &'static Kernel,
        chip: &'static C,
        app_flash: &'static [u8],
        header_length: usize,
        app_version: u16,
        remaining_memory: &'a mut [u8],
        fault_policy: &'static dyn ProcessFaultPolicy,
        require_kernel_version: bool,
        index: usize,
    ) -> Result<(Option<&'static dyn Process>, &'a mut [u8]), ProcessLoadError> {
        // Get a slice for just the app header.
        let header_flash = app_flash
            .get(0..header_length as usize)
            .ok_or(ProcessLoadError::NotEnoughFlash)?;

        // Parse the full TBF header to see if this is a valid app. If the
        // header can't parse, we will error right here.
        let tbf_header = tock_tbf::parse::parse_tbf_header(header_flash, app_version)?;

        // First thing: check that the process is at the correct location in
        // flash if the TBF header specified a fixed address. If there is a
        // mismatch we catch that early.
        if let Some(fixed_flash_start) = tbf_header.get_fixed_address_flash() {
            // The flash address in the header is based on the app binary,
            // so we need to take into account the header length.
            let actual_address = app_flash.as_ptr() as u32 + tbf_header.get_protected_size();
            let expected_address = fixed_flash_start;
            if actual_address != expected_address {
                return Err(ProcessLoadError::IncorrectFlashAddress {
                    actual_address,
                    expected_address,
                });
            }
        }

        let process_name = tbf_header.get_package_name();

        // If this isn't an app (i.e. it is padding) or it is an app but it
        // isn't enabled, then we can skip it and do not create a `Process`
        // object.
        if !tbf_header.is_app() || !tbf_header.enabled() {
            if config::CONFIG.debug_load_processes {
                if !tbf_header.is_app() {
                    debug!(
                        "Padding in flash={:#010X}-{:#010X}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1
                    );
                }
                if !tbf_header.enabled() {
                    debug!(
                        "Process not enabled flash={:#010X}-{:#010X} process={:?}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1,
                        process_name.unwrap_or("(no name)")
                    );
                }
            }
            // Return no process and the full memory slice we were given.
            return Ok((None, remaining_memory));
        }

        if let Some((major, minor)) = tbf_header.get_kernel_version() {
            // If the `KernelVersion` header is present, we read the requested kernel version and compare it to
            // the running kernel version.
            if crate::MAJOR != major || crate::MINOR < minor {
                // If the kernel major version is different, we prevent the process from being loaded.
                //
                // If the kernel major version is the same, we compare the kernel minor version. The current
                // running kernel minor version has to be greater or equal to the one that the process
                // has requested. If not, we prevent the process from loading.
                if config::CONFIG.debug_load_processes {
                    debug!("WARN process {:?} not loaded as it requires kernel version >= {}.{} and < {}.0, (running kernel {}.{})", process_name.unwrap_or("(no name)"), major, minor, (major+1), crate::MAJOR, crate::MINOR);
                }
                return Err(ProcessLoadError::IncompatibleKernelVersion {
                    version: Some((major, minor)),
                });
            }
        } else {
            if require_kernel_version {
                // If enforcing the kernel version is requested, and the `KernelVersion` header is not present,
                // we prevent the process from loading.
                if config::CONFIG.debug_load_processes {
                    debug!(
                            "WARN process {:?} not loaded as it has no kernel version header, please upgrade to elf2tab >= 0.8.0",
                            process_name.unwrap_or ("(no name")
                        );
                }
                return Err(ProcessLoadError::IncompatibleKernelVersion { version: None });
            }
        }

        // Otherwise, actually load the app.
        let process_ram_requested_size = tbf_header.get_minimum_app_ram_size() as usize;
        let init_fn = app_flash
            .as_ptr()
            .offset(tbf_header.get_init_function_offset() as isize) as usize;

        // Initialize MPU region configuration.
        let mut mpu_config: <<C as Chip>::MPU as MPU>::MpuConfig = Default::default();

        // Allocate MPU region for flash.
        if chip
            .mpu()
            .allocate_region(
                app_flash.as_ptr(),
                app_flash.len(),
                app_flash.len(),
                mpu::Permissions::ReadExecuteOnly,
                &mut mpu_config,
            )
            .is_none()
        {
            if config::CONFIG.debug_load_processes {
                debug!(
                    "[!] flash={:#010X}-{:#010X} process={:?} - couldn't allocate MPU region for flash",
                    app_flash.as_ptr() as usize,
                    app_flash.as_ptr() as usize + app_flash.len() - 1,
                    process_name
                );
            }
            return Err(ProcessLoadError::MpuInvalidFlashLength);
        }

        // Determine how much space we need in the application's
        // memory space just for kernel and grant state. We need to make
        // sure we allocate enough memory just for that.

        // Make room for grant pointers.
        let grant_ptr_size = mem::size_of::<GrantPointerEntry>();
        let grant_ptrs_num = kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        // Initial size of the kernel-owned part of process memory can be
        // calculated directly based on the initial size of all kernel-owned
        // data structures.
        let initial_kernel_memory_size =
            grant_ptrs_offset + Self::CALLBACKS_OFFSET + Self::PROCESS_STRUCT_OFFSET;

        // By default we start with the initial size of process-accessible
        // memory set to 0. This maximizes the flexibility that processes have
        // to allocate their memory as they see fit. If a process needs more
        // accessible memory it must use the `brk` memop syscalls to request more
        // memory.
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
        let remaining_memory = if let Some(fixed_memory_start) = tbf_header.get_fixed_address_ram()
        {
            // The process does have a fixed address.
            if fixed_memory_start == remaining_memory.as_ptr() as u32 {
                // Address already matches.
                remaining_memory
            } else if fixed_memory_start > remaining_memory.as_ptr() as u32 {
                // Process wants a memory address farther in memory. Try to
                // advance the memory region to make the address match.
                let diff = (fixed_memory_start - remaining_memory.as_ptr() as u32) as usize;
                if diff > remaining_memory.len() {
                    // We ran out of memory.
                    let actual_address =
                        remaining_memory.as_ptr() as u32 + remaining_memory.len() as u32 - 1;
                    let expected_address = fixed_memory_start;
                    return Err(ProcessLoadError::MemoryAddressMismatch {
                        actual_address,
                        expected_address,
                    });
                } else {
                    // Change the memory range to start where the process
                    // requested it.
                    remaining_memory
                        .get_mut(diff..)
                        .ok_or(ProcessLoadError::InternalError)?
                }
            } else {
                // Address is earlier in memory, nothing we can do.
                let actual_address = remaining_memory.as_ptr() as u32;
                let expected_address = fixed_memory_start;
                return Err(ProcessLoadError::MemoryAddressMismatch {
                    actual_address,
                    expected_address,
                });
            }
        } else {
            remaining_memory
        };

        // Determine where process memory will go and allocate MPU region for
        // app-owned memory.
        let (app_memory_start, app_memory_size) = match chip.mpu().allocate_app_memory_region(
            remaining_memory.as_ptr() as *const u8,
            remaining_memory.len(),
            min_total_memory_size,
            min_process_memory_size,
            initial_kernel_memory_size,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        ) {
            Some((memory_start, memory_size)) => (memory_start, memory_size),
            None => {
                // Failed to load process. Insufficient memory.
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash={:#010X}-{:#010X} process={:?} - couldn't allocate memory region of size >= {:#X}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1,
                        process_name,
                        min_total_memory_size
                    );
                }
                return Err(ProcessLoadError::NotEnoughMemory);
            }
        };

        // Get a slice for the memory dedicated to the process. This can fail if
        // the MPU returns a region of memory that is not inside of the
        // `remaining_memory` slice passed to `create()` to allocate the
        // process's memory out of.
        let memory_start_offset = app_memory_start as usize - remaining_memory.as_ptr() as usize;
        // First split the remaining memory into a slice that contains the
        // process memory and a slice that will not be used by this process.
        let (app_memory_oversize, unused_memory) =
            remaining_memory.split_at_mut(memory_start_offset + app_memory_size);
        // Then since the process's memory need not start at the beginning of
        // the remaining slice given to create(), get a smaller slice as needed.
        let app_memory = app_memory_oversize
            .get_mut(memory_start_offset..)
            .ok_or(ProcessLoadError::InternalError)?;

        // Check if the memory region is valid for the process. If a process
        // included a fixed address for the start of RAM in its TBF header (this
        // field is optional, processes that are position independent do not
        // need a fixed address) then we check that we used the same address
        // when we allocated it in RAM.
        if let Some(fixed_memory_start) = tbf_header.get_fixed_address_ram() {
            let actual_address = app_memory.as_ptr() as u32;
            let expected_address = fixed_memory_start;
            if actual_address != expected_address {
                return Err(ProcessLoadError::MemoryAddressMismatch {
                    actual_address,
                    expected_address,
                });
            }
        }

        // Set the initial process-accessible memory to the amount specified by
        // the context switch implementation.
        let initial_app_brk = app_memory.as_ptr().add(min_process_memory_size);

        // Set the initial allow high water mark to the start of process memory
        // since no `allow` calls have been made yet.
        let initial_allow_high_water_mark = app_memory.as_ptr();

        // Set up initial grant region.
        let mut kernel_memory_break = app_memory.as_mut_ptr().add(app_memory.len());

        // Now that we know we have the space we can setup the grant
        // pointers.
        kernel_memory_break = kernel_memory_break.offset(-(grant_ptrs_offset as isize));

        // This is safe today, as MPU constraints ensure that `memory_start`
        // will always be aligned on at least a word boundary, and that
        // memory_size will be aligned on at least a word boundary, and
        // `grant_ptrs_offset` is a multiple of the word size. Thus,
        // `kernel_memory_break` must be word aligned. While this is unlikely to
        // change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set all grant pointers to null.
        let grant_pointers = slice::from_raw_parts_mut(
            kernel_memory_break as *mut GrantPointerEntry,
            grant_ptrs_num,
        );
        for grant_entry in grant_pointers.iter_mut() {
            grant_entry.driver_num = 0;
            grant_entry.grant_ptr = ptr::null_mut();
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
        let upcall_buf =
            slice::from_raw_parts_mut(kernel_memory_break as *mut Task, Self::CALLBACK_LEN);
        let tasks = RingBuffer::new(upcall_buf);

        // Last thing in the kernel region of process RAM is the process struct.
        kernel_memory_break = kernel_memory_break.offset(-(Self::PROCESS_STRUCT_OFFSET as isize));
        let process_struct_memory_location = kernel_memory_break;

        // Create the Process struct in the app grant region.
        let mut process: &mut ProcessStandard<C> =
            &mut *(process_struct_memory_location as *mut ProcessStandard<'static, C>);

        // Ask the kernel for a unique identifier for this process that is being
        // created.
        let unique_identifier = kernel.create_process_identifier();

        // Save copies of these in case the app was compiled for fixed addresses
        // for later debugging.
        let fixed_address_flash = tbf_header.get_fixed_address_flash();
        let fixed_address_ram = tbf_header.get_fixed_address_ram();

        process
            .process_id
            .set(ProcessId::new(kernel, unique_identifier, index));
        process.kernel = kernel;
        process.chip = chip;
        process.allow_high_water_mark = Cell::new(initial_allow_high_water_mark);
        process.memory_start = app_memory.as_ptr();
        process.memory_len = app_memory.len();
        process.header = tbf_header;
        process.kernel_memory_break = Cell::new(kernel_memory_break);
        process.app_break = Cell::new(initial_app_brk);
        process.grant_pointers = MapCell::new(grant_pointers);

        process.flash = app_flash;

        process.stored_state = MapCell::new(Default::default());
        // Mark this process as unstarted
        process.state = ProcessStateCell::new(process.kernel);
        process.fault_policy = fault_policy;
        process.restart_count = Cell::new(0);

        process.mpu_config = MapCell::new(mpu_config);
        process.mpu_regions = [
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
        ];
        process.tasks = MapCell::new(tasks);
        process.process_name = process_name.unwrap_or("");

        process.debug = MapCell::new(ProcessStandardDebug {
            fixed_address_flash: fixed_address_flash,
            fixed_address_ram: fixed_address_ram,
            app_heap_start_pointer: None,
            app_stack_start_pointer: None,
            app_stack_min_pointer: None,
            syscall_count: 0,
            last_syscall: None,
            dropped_upcall_count: 0,
            timeslice_expiration_count: 0,
        });

        let flash_protected_size = process.header.get_protected_size() as usize;
        let flash_app_start_addr = app_flash.as_ptr() as usize + flash_protected_size;

        process.tasks.map(|tasks| {
            tasks.enqueue(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: init_fn,
                argument0: flash_app_start_addr,
                argument1: process.memory_start as usize,
                argument2: process.memory_len,
                argument3: process.app_break.get() as usize,
            }));
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
                app_memory_start,
                initial_app_brk,
                stored_state,
            )
        }) {
            Some(Ok(())) => {}
            _ => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash={:#010X}-{:#010X} process={:?} - couldn't initialize process",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1,
                        process_name
                    );
                }
                return Err(ProcessLoadError::InternalError);
            }
        };

        kernel.increment_work();

        // Return the process object and a remaining memory for processes slice.
        Ok((Some(process), unused_memory))
    }

    /// Restart the process, resetting all of its state and re-initializing
    /// it to start running.  Assumes the process is not running but is still in flash
    /// and still has its memory region allocated to it. This implements
    /// the mechanism of restart.
    fn restart(&self) -> Result<(), ErrorCode> {
        // We need a new process identifier for this process since the restarted
        // version is in effect a new process. This is also necessary to
        // invalidate any stored `ProcessId`s that point to the old version of the
        // process. However, the process has not moved locations in the
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

        // FLASH

        // We are going to start this process over again, so need the init_fn
        // location.
        let app_flash_address = self.flash_start();
        let init_fn = unsafe {
            app_flash_address.offset(self.header.get_init_function_offset() as isize) as usize
        };

        // Reset MPU region configuration.
        // TODO: ideally, this would be moved into a helper function used by both
        // create() and reset(), but process load debugging complicates this.
        // We just want to create new config with only flash and memory regions.
        let mut mpu_config: <<C as Chip>::MPU as MPU>::MpuConfig = Default::default();
        // Allocate MPU region for flash.
        let app_mpu_flash = self.chip.mpu().allocate_region(
            self.flash.as_ptr(),
            self.flash.len(),
            self.flash.len(),
            mpu::Permissions::ReadExecuteOnly,
            &mut mpu_config,
        );
        if app_mpu_flash.is_none() {
            // We were unable to allocate an MPU region for flash. This is very
            // unexpected since we previously ran this process. However, we
            // return now and leave the process faulted and it will not be
            // scheduled.
            return Err(ErrorCode::FAIL);
        }

        // RAM

        // Re-determine the minimum amount of RAM the kernel must allocate to the process
        // based on the specific requirements of the syscall implementation.
        let min_process_memory_size = self
            .chip
            .userspace_kernel_boundary()
            .initial_process_app_brk_size();

        // Recalculate initial_kernel_memory_size as was done in create()
        let grant_ptr_size = mem::size_of::<(usize, *mut u8)>();
        let grant_ptrs_num = self.kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        let initial_kernel_memory_size =
            grant_ptrs_offset + Self::CALLBACKS_OFFSET + Self::PROCESS_STRUCT_OFFSET;

        let app_mpu_mem = self.chip.mpu().allocate_app_memory_region(
            self.mem_start(),
            self.memory_len,
            self.memory_len, //we want exactly as much as we had before restart
            min_process_memory_size,
            initial_kernel_memory_size,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        );
        let (app_mpu_mem_start, app_mpu_mem_len) = match app_mpu_mem {
            Some((start, len)) => (start, len),
            None => {
                // We couldn't configure the MPU for the process. This shouldn't
                // happen since we were able to start the process before, but at
                // this point it is better to leave the app faulted and not
                // schedule it.
                return Err(ErrorCode::NOMEM);
            }
        };

        // Reset memory pointers now that we know the layout of the process
        // memory and know that we can configure the MPU.

        // app_brk is set based on minimum syscall size above the start of
        // memory.
        let app_brk = app_mpu_mem_start.wrapping_add(min_process_memory_size);
        self.app_break.set(app_brk);
        // kernel_brk is calculated backwards from the end of memory the size of
        // the initial kernel data structures.
        let kernel_brk = app_mpu_mem_start
            .wrapping_add(app_mpu_mem_len)
            .wrapping_sub(initial_kernel_memory_size);
        self.kernel_memory_break.set(kernel_brk);
        // High water mark for `allow`ed memory is reset to the start of the
        // process's memory region.
        self.allow_high_water_mark.set(app_mpu_mem_start);

        // Drop the old config and use the clean one
        self.mpu_config.replace(mpu_config);

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
            Err(_) => {
                // We couldn't initialize the architecture-specific
                // state for this process. This shouldn't happen since
                // the app was able to be started before, but at this
                // point the app is no longer valid. The best thing we
                // can do now is leave the app as still faulted and not
                // schedule it.
                return Err(ErrorCode::RESERVE);
            }
        };

        // And queue up this app to be restarted.
        let flash_protected_size = self.header.get_protected_size() as usize;
        let flash_app_start = app_flash_address as usize + flash_protected_size;

        // Mark the state as `Unstarted` for the scheduler.
        self.state.update(State::Unstarted);

        // Mark that we restarted this process.
        self.restart_count.increment();

        // Enqueue the initial function.
        self.tasks.map(|tasks| {
            tasks.enqueue(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: init_fn,
                argument0: flash_app_start,
                argument1: self.mem_start() as usize,
                argument2: self.memory_len,
                argument3: self.app_break.get() as usize,
            }));
        });

        // Mark that the process is ready to run.
        self.kernel.increment_work();

        Ok(())
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// is within the RAM bounds currently exposed to the processes (i.e.
    /// ending at `app_break`). If this method returns `true`, the buffer
    /// is guaranteed to be accessible to the process and to not overlap with
    /// the grant region.
    fn in_app_owned_memory(&self, buf_start_addr: *const u8, size: usize) -> bool {
        let buf_end_addr = buf_start_addr.wrapping_add(size);

        buf_end_addr >= buf_start_addr
            && buf_start_addr >= self.mem_start()
            && buf_end_addr <= self.app_break.get()
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// are within the readable region of an application's flash
    /// memory.  If this method returns true, the buffer
    /// is guaranteed to be readable to the process.
    fn in_app_flash_memory(&self, buf_start_addr: *const u8, size: usize) -> bool {
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
                grant_entry.grant_ptr = ptr::null_mut();
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
    fn allocate_in_grant_region_internal(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        self.mpu_config.and_then(|mut config| {
            // First, compute the candidate new pointer. Note that at this
            // point we have not yet checked whether there is space for
            // this allocation or that it meets alignment requirements.
            let new_break_unaligned = self.kernel_memory_break.get().wrapping_sub(size);

            // Our minimum alignment requirement is two bytes, so that the
            // lowest bit of the address will always be zero and we can use it
            // as a flag. It doesn't hurt to increase the alignment (except for
            // potentially a wasted byte) so we make sure `align` is at least
            // two.
            let align = cmp::max(align, 2);

            // The alignment must be a power of two, 2^a. The expression
            // `!(align - 1)` then returns a mask with leading ones,
            // followed by `a` trailing zeros.
            let alignment_mask = !(align - 1);
            let new_break = (new_break_unaligned as usize & alignment_mask) as *const u8;

            // Verify there is space for this allocation
            if new_break < self.app_break.get() {
                None
            // Verify it didn't wrap around
            } else if new_break > self.kernel_memory_break.get() {
                None
            // Verify this is compatible with the MPU.
            } else if let Err(_) = self.chip.mpu().update_app_memory_region(
                self.app_break.get(),
                new_break,
                mpu::Permissions::ReadWriteOnly,
                &mut config,
            ) {
                None
            } else {
                // Allocation is valid.

                // We always allocate down, so we must lower the
                // kernel_memory_break.
                self.kernel_memory_break.set(new_break);

                // We need `grant_ptr` as a mutable pointer.
                let grant_ptr = new_break as *mut u8;

                // ### Safety
                //
                // Here we are guaranteeing that `grant_ptr` is not null. We can
                // ensure this because we just created `grant_ptr` based on the
                // process's allocated memory, and we know it cannot be null.
                unsafe { Some(NonNull::new_unchecked(grant_ptr)) }
            }
        })
    }

    /// Create the identifier for a custom grant that grant.rs uses to access
    /// the custom grant.
    ///
    /// We create this identifier by calculating the number of bytes between
    /// where the custom grant starts and the end of the process memory.
    fn create_custom_grant_identifier(&self, ptr: NonNull<u8>) -> ProcessCustomGrantIdentifer {
        let custom_grant_address = ptr.as_ptr() as usize;
        let process_memory_end = self.mem_end() as usize;

        ProcessCustomGrantIdentifer {
            offset: process_memory_end - custom_grant_address,
        }
    }

    /// Use a ProcessCustomGrantIdentifer to find the address of the custom
    /// grant.
    ///
    /// This reverses `create_custom_grant_identifier()`.
    fn get_custom_grant_address(&self, identifier: ProcessCustomGrantIdentifer) -> usize {
        let process_memory_end = self.mem_end() as usize;

        // Subtract the offset in the identifier from the end of the process
        // memory to get the address of the custom grant.
        process_memory_end - identifier.offset
    }

    /// Check if the process is active.
    ///
    /// "Active" is defined as the process can resume executing in the future.
    /// This means its state in the `Process` struct is still valid, and that
    /// the kernel could resume its execution without completely restarting and
    /// resetting its state.
    ///
    /// A process is inactive if the kernel cannot resume its execution, such as
    /// if the process faults and is in an invalid state, or if the process
    /// explicitly exits.
    fn is_active(&self) -> bool {
        let current_state = self.state.get();
        current_state != State::Terminated && current_state != State::Faulted
    }
}
