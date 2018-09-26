//! Support for creating and running userspace applications.

use core::cell::Cell;
use core::fmt::Write;
use core::ptr::write_volatile;
use core::{mem, ptr, slice, str};

use callback::AppId;
use capabilities::ProcessManagementCapability;
use common::cells::MapCell;
use common::math;
use common::{Queue, RingBuffer};
use platform::mpu;
use returncode::ReturnCode;
use sched::Kernel;
use syscall::{self, Syscall, UserspaceKernelBoundary};
use tbfheader;

/// Helper function to load processes from flash into an array of active
/// processes. This is the default template for loading processes, but a board
/// is able to create its own `load_processes()` function and use that instead.
///
/// Processes are found in flash starting from the given address and iterating
/// through Tock Binary Format headers. Processes are given memory out of the
/// `app_memory` buffer until either the memory is exhausted or the allocated
/// number of processes are created, with process structures placed in the
/// provided array. How process faults are handled by the kernel is also
/// selected.
pub fn load_processes<S: UserspaceKernelBoundary>(
    kernel: &'static Kernel,
    syscall: &'static S,
    start_of_flash: *const u8,
    app_memory: &mut [u8],
    procs: &'static mut [Option<&'static ProcessType>],
    fault_response: FaultResponse,
    _capability: &ProcessManagementCapability,
) {
    let mut apps_in_flash_ptr = start_of_flash;
    let mut app_memory_ptr = app_memory.as_mut_ptr();
    let mut app_memory_size = app_memory.len();
    for i in 0..procs.len() {
        unsafe {
            let (process, flash_offset, memory_offset) = Process::create(
                kernel,
                syscall,
                apps_in_flash_ptr,
                app_memory_ptr,
                app_memory_size,
                fault_response,
            );

            if process.is_none() {
                // We did not get a valid process, but we may have gotten a disabled
                // process or padding. Therefore we want to skip this chunk of flash
                // and see if there is a valid app there. However, if we cannot
                // advance the flash pointer, then we are done.
                if flash_offset == 0 && memory_offset == 0 {
                    break;
                }
            } else {
                procs[i] = process;
            }

            apps_in_flash_ptr = apps_in_flash_ptr.offset(flash_offset as isize);
            app_memory_ptr = app_memory_ptr.offset(memory_offset as isize);
            app_memory_size -= memory_offset;
        }
    }
}

/// This trait is implemented by process structs.
pub trait ProcessType {
    /// Queue a `Task` for the process. This will be added to a per-process
    /// buffer and executed by the scheduler. `Task`s are some function the app
    /// should run, for example a callback or an IPC call.
    ///
    /// This function returns `true` if the `Task` was successfully enqueued,
    /// and `false` otherwise. This is represented as a simple `bool` because
    /// this is passed to the capsule that tried to schedule the `Task`.
    fn enqueue_task(&self, task: Task) -> bool;

    /// Remove the scheduled operation from the front of the queue and return it
    /// to be handled by the scheduler.
    ///
    /// If there are no `Task`s in the queue for this process this will return
    /// `None`.
    fn dequeue_task(&self) -> Option<Task>;

    /// Returns the current state the process is in. Common states are "running"
    /// or "yielded".
    fn get_state(&self) -> State;

    /// Move this process from the running state to the yielded state.
    fn set_yielded_state(&self);

    /// Put this process in the fault state. This will trigger the
    /// `FaultResponse` for this process to occur.
    fn set_fault_state(&self);

    /// Get the name of the process. Used for IPC.
    fn get_process_name(&self) -> &[u8];

    // memop operations

    /// Change the location of the program break.
    fn brk(&self, new_break: *const u8) -> Result<*const u8, Error>;

    /// Change the location of the program break and return the previous break
    /// address.
    fn sbrk(&self, increment: isize) -> Result<*const u8, Error>;

    /// The start address of allocated RAM for this process.
    fn mem_start(&self) -> *const u8;

    /// The first address after the end of the allocated RAM for this process.
    fn mem_end(&self) -> *const u8;

    /// The start address of the flash region allocated for this process.
    fn flash_start(&self) -> *const u8;

    /// The first address after the end of the flash region allocated for this
    /// process.
    fn flash_end(&self) -> *const u8;

    /// The lowest address of the grant region for the process.
    fn kernel_memory_break(&self) -> *const u8;

    /// How many writeable flash regions defined in the TBF header for this
    /// process.
    fn number_writeable_flash_regions(&self) -> usize;

    /// Get the offset from the beginning of flash and the size of the defined
    /// writeable flash region.
    fn get_writeable_flash_region(&self, region_index: usize) -> (u32, u32);

    /// Debug function to update the kernel on where the stack starts for this
    /// process. Processes are not required to call this through the memop
    /// system call, but it aids in debugging the process.
    fn update_stack_start_pointer(&self, stack_pointer: *const u8);

    /// Debug function to update the kernel on where the process heap starts.
    /// Also optional.
    fn update_heap_start_pointer(&self, heap_pointer: *const u8);

    // additional memop like functions

    /// Check if the buffer address and size is contained within the memory
    /// owned by this process. This isn't quite the same as the memory allocated
    /// to the process as this does not include the grant region which is owned
    /// by the kernel.
    fn in_app_owned_memory(&self, buf_start_addr: *const u8, size: usize) -> bool;

    /// Get the first address of process's flash that isn't protected by the
    /// kernel. The protected range of flash contains the TBF header and
    /// potentially other state the kernel is storing on behalf of the process,
    /// and cannot be edited by the process.
    fn flash_non_protected_start(&self) -> *const u8;

    // mpu

    fn setup_mpu(&self, mpu: &mpu::MPU);

    fn add_mpu_region(&self, base: *const u8, size: u32) -> bool;

    // grants

    /// Create new memory in the grant region.
    unsafe fn alloc(&self, size: usize) -> Option<&mut [u8]>;

    unsafe fn free(&self, _: *mut u8);

    /// Get a pointer to the grant pointer for this grant number.
    unsafe fn grant_ptr(&self, grant_num: usize) -> *mut *mut u8;

    // functions for processes that are architecture specific

    /// Get the syscall that the process called.
    unsafe fn get_syscall(&self) -> Option<Syscall>;

    /// Set the return value the process should see when it begins executing
    /// again after the syscall.
    unsafe fn set_syscall_return_value(&self, return_value: isize);

    /// Remove the last stack frame from the process.
    unsafe fn pop_syscall_stack_frame(&self);

    /// Replace the last stack frame with the new function call. This function
    /// is what should be executed when the process is resumed.
    unsafe fn push_function_call(&self, callback: FunctionCall);

    /// Context switch to a specific process.
    unsafe fn switch_to(&self) -> Option<syscall::ContextSwitchReason>;

    unsafe fn fault_fmt(&self, writer: &mut Write);
    unsafe fn process_detail_fmt(&self, writer: &mut Write);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    NoSuchApp,
    OutOfMemory,
    AddressOutOfBounds,
}

impl From<Error> for ReturnCode {
    fn from(err: Error) -> ReturnCode {
        match err {
            Error::OutOfMemory => ReturnCode::ENOMEM,
            Error::AddressOutOfBounds => ReturnCode::EINVAL,
            Error::NoSuchApp => ReturnCode::EINVAL,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Running,
    Yielded,
    Fault,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaultResponse {
    Panic,
    Restart,
}

#[derive(Copy, Clone, Debug)]
pub enum IPCType {
    Service,
    Client,
}

#[derive(Copy, Clone)]
pub enum Task {
    FunctionCall(FunctionCall),
    IPC((AppId, IPCType)),
}

/// Struct that defines a callback that can be passed to a process. The callback
/// takes four arguments that are `Driver` and callback specific, so they are
/// represented generically here.
///
/// Likely these four arguments will get passed as the first four register
/// values, but this is architecture-dependent.
#[derive(Copy, Clone, Debug)]
pub struct FunctionCall {
    pub argument0: usize,
    pub argument1: usize,
    pub argument2: usize,
    pub argument3: usize,
    pub pc: usize,
}

/// State for helping with debugging apps.
///
/// These pointers and counters are not strictly required for kernel operation,
/// but provide helpful information when an app crashes.
struct ProcessDebug {
    /// Where the process has started its heap in RAM.
    app_heap_start_pointer: Option<*const u8>,

    /// Where the start of the stack is for the process. If the kernel does the
    /// PIC setup for this app then we know this, otherwise we need the app to
    /// tell us where it put its stack.
    app_stack_start_pointer: Option<*const u8>,

    /// How low have we ever seen the stack pointer.
    min_stack_pointer: *const u8,

    /// How many syscalls have occurred since the process started.
    syscall_count: usize,

    /// What was the most recent syscall.
    last_syscall: Option<Syscall>,

    /// How many callbacks were dropped because the queue was insufficiently
    /// long.
    dropped_callback_count: usize,

    /// How many times this process has entered into a fault condition and the
    /// kernel has restarted it.
    restart_count: usize,

    /// How many times this process has been paused because it exceeded its
    /// timeslice.
    timeslice_expiration_count: usize,
}

pub struct Process<'a, S: 'static + UserspaceKernelBoundary> {
    /// Pointer to the main Kernel struct.
    kernel: &'static Kernel,

    /// Pointer to the struct that handles the architecture-specific syscall
    /// functions.
    syscall: &'static S,

    /// Application memory layout:
    ///
    /// ```text
    ///     ╒════════ ← memory[memory.len()]
    ///  ╔═ │ Grant
    ///     │   ↓
    ///  D  │ ──────  ← kernel_memory_break
    ///  Y  │
    ///  N  │ ──────  ← app_break
    ///  A  │
    ///  M  │   ↑
    ///     │  Heap
    ///  ╠═ │ ──────  ← app_heap_start
    ///     │  Data
    ///  F  │ ──────  ← data_start_pointer
    ///  I  │ Stack
    ///  X  │   ↓
    ///  E  │
    ///  D  │ ──────  ← current_stack_pointer
    ///     │
    ///  ╚═ ╘════════ ← memory[0]
    /// ```
    ///
    /// The process's memory.
    memory: &'static mut [u8],

    /// Pointer to the end of the allocated (and MPU protected) grant region.
    kernel_memory_break: Cell<*const u8>,

    /// Copy of where the kernel memory break is when the app is first started.
    /// This is handy if the app is restarted so we know where to reset
    /// the kernel_memory break to without having to recalculate it.
    original_kernel_memory_break: *const u8,

    /// Pointer to the end of process RAM that has been sbrk'd to the process.
    app_break: Cell<*const u8>,
    original_app_break: *const u8,

    /// Saved when the app switches to the kernel.
    current_stack_pointer: Cell<*const u8>,
    original_stack_pointer: *const u8,

    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: &'static [u8],

    /// Collection of pointers to the TBF header in flash.
    header: tbfheader::TbfHeader,

    /// State saved on behalf of the process each time the app switches to the
    /// kernel.
    stored_state: Cell<S::StoredState>,

    /// Whether the scheduler can schedule this app.
    state: Cell<State>,

    /// How to deal with Faults occurring in the process
    fault_response: FaultResponse,

    /// MPU regions are saved as a pointer-size pair.
    ///
    /// size is encoded as X where
    /// SIZE = 2<sup>(X + 1)</sup> and X >= 4.
    ///
    /// A null pointer represents an empty region.
    ///
    /// #### Invariants
    ///
    /// The pointer must be aligned to the size. E.g. if the size is 32 bytes, the pointer must be
    /// 32-byte aligned.
    mpu_regions: [Cell<(*const u8, math::PowerOfTwo)>; 5],

    /// Essentially a list of callbacks that want to call functions in the
    /// process.
    tasks: MapCell<RingBuffer<'a, Task>>,

    /// Name of the app.
    process_name: &'static str,

    /// Values kept so that we can print useful debug messages when apps fault.
    debug: MapCell<ProcessDebug>,
}

impl<S: UserspaceKernelBoundary> ProcessType for Process<'a, S> {
    fn enqueue_task(&self, task: Task) -> bool {
        // If this app is in the `Fault` state then we shouldn't schedule
        // any work for it.
        if self.state.get() == State::Fault {
            return false;
        }

        self.kernel.increment_work();

        let ret = self.tasks.map_or(false, |tasks| tasks.enqueue(task));

        // Make a note that we lost this callback if the enqueue function
        // fails.
        if ret == false {
            self.debug.map(|debug| {
                debug.dropped_callback_count += 1;
            });
        }

        ret
    }

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_yielded_state(&self) {
        if self.state.get() == State::Running {
            self.state.set(State::Yielded);
            self.kernel.decrement_work();
        }
    }

    fn set_fault_state(&self) {
        self.state.set(State::Fault);

        match self.fault_response {
            FaultResponse::Panic => {
                // process faulted. Panic and print status
                panic!("Process {} had a fault", self.process_name);
            }
            FaultResponse::Restart => {
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

                // Update debug information
                self.debug.map(|debug| {
                    // Mark that we restarted this process.
                    debug.restart_count += 1;

                    // Reset some state for the process.
                    debug.syscall_count = 0;
                    debug.last_syscall = None;
                    debug.dropped_callback_count = 0;
                });

                // We are going to start this process over again, so need
                // the init_fn location.
                let app_flash_address = self.flash_start();
                let init_fn = unsafe {
                    app_flash_address.offset(self.header.get_init_function_offset() as isize)
                        as usize
                };
                self.state.set(State::Yielded);

                // Need to reset the grant region.
                unsafe {
                    self.grant_ptrs_reset();
                }
                self.kernel_memory_break
                    .set(self.original_kernel_memory_break);

                // Reset other memory pointers.
                self.app_break.set(self.original_app_break);
                self.current_stack_pointer.set(self.original_stack_pointer);

                // And queue up this app to be restarted.
                let flash_protected_size = self.header.get_protected_size() as usize;
                let flash_app_start = app_flash_address as usize + flash_protected_size;

                self.tasks.map(|tasks| {
                    tasks.enqueue(Task::FunctionCall(FunctionCall {
                        pc: init_fn,
                        argument0: flash_app_start,
                        argument1: self.memory.as_ptr() as usize,
                        argument2: self.memory.len() as usize,
                        argument3: self.app_break.get() as usize,
                    }));
                });

                self.kernel.increment_work();
            }
        }
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
        self.memory.as_ptr()
    }

    fn mem_end(&self) -> *const u8 {
        unsafe { self.memory.as_ptr().offset(self.memory.len() as isize) }
    }

    fn flash_start(&self) -> *const u8 {
        self.flash.as_ptr()
    }

    fn flash_non_protected_start(&self) -> *const u8 {
        ((self.flash.as_ptr() as usize) + self.header.get_protected_size() as usize) as *const u8
    }

    fn flash_end(&self) -> *const u8 {
        unsafe { self.flash.as_ptr().offset(self.flash.len() as isize) }
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
                debug.min_stack_pointer = stack_pointer;
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

    fn setup_mpu(&self, mpu: &mpu::MPU) {
        // Flash segment read/execute (no write)
        let flash_start = self.flash.as_ptr() as usize;
        let flash_len = self.flash.len();

        match mpu.create_region(
            0,
            flash_start,
            flash_len,
            mpu::ExecutePermission::ExecutionPermitted,
            mpu::AccessPermission::ReadOnly,
        ) {
            None => panic!(
                "Infeasible MPU allocation. Base {:#x}, Length: {:#x}",
                flash_start, flash_len
            ),
            Some(region) => mpu.set_mpu(region),
        }

        let data_start = self.memory.as_ptr() as usize;
        let data_len = self.memory.len();

        match mpu.create_region(
            1,
            data_start,
            data_len,
            mpu::ExecutePermission::ExecutionPermitted,
            mpu::AccessPermission::ReadWrite,
        ) {
            None => panic!(
                "Infeasible MPU allocation. Base {:#x}, Length: {:#x}",
                data_start, data_len
            ),
            Some(region) => mpu.set_mpu(region),
        }

        // Disallow access to grant region
        let grant_len = unsafe {
            math::PowerOfTwo::ceiling(
                self.memory.as_ptr().offset(self.memory.len() as isize) as u32
                    - (self.kernel_memory_break.get() as u32),
            ).as_num::<u32>()
        };
        let grant_base = unsafe {
            self.memory
                .as_ptr()
                .offset(self.memory.len() as isize)
                .offset(-(grant_len as isize))
        };

        match mpu.create_region(
            2,
            grant_base as usize,
            grant_len as usize,
            mpu::ExecutePermission::ExecutionNotPermitted,
            mpu::AccessPermission::PrivilegedOnly,
        ) {
            None => panic!(
                "Infeasible MPU allocation. Base {:#x}, Length: {:#x}",
                grant_base as usize, grant_len
            ),
            Some(region) => mpu.set_mpu(region),
        }

        // Setup IPC MPU regions
        for (i, region) in self.mpu_regions.iter().enumerate() {
            if region.get().0.is_null() {
                mpu.set_mpu(mpu::Region::empty(i + 3));
                continue;
            }
            match mpu.create_region(
                i + 3,
                region.get().0 as usize,
                region.get().1.as_num::<u32>() as usize,
                mpu::ExecutePermission::ExecutionPermitted,
                mpu::AccessPermission::ReadWrite,
            ) {
                None => panic!(
                    "Unexpected: Infeasible MPU allocation: Num: {}, \
                     Base: {:#x}, Length: {:#x}",
                    i + 3,
                    region.get().0 as usize,
                    region.get().1.as_num::<u32>()
                ),
                Some(region) => mpu.set_mpu(region),
            }
        }
    }

    fn add_mpu_region(&self, base: *const u8, size: u32) -> bool {
        if size >= 16 && size.count_ones() == 1 && (base as u32) % size == 0 {
            let mpu_size = math::PowerOfTwo::floor(size);
            for region in self.mpu_regions.iter() {
                if region.get().0 == ptr::null() {
                    region.set((base, mpu_size));
                    return true;
                } else if region.get().0 == base {
                    if region.get().1 < mpu_size {
                        region.set((base, mpu_size));
                    }
                    return true;
                }
            }
        }
        return false;
    }

    fn sbrk(&self, increment: isize) -> Result<*const u8, Error> {
        let new_break = unsafe { self.app_break.get().offset(increment) };
        self.brk(new_break)
    }

    fn brk(&self, new_break: *const u8) -> Result<*const u8, Error> {
        if new_break < self.mem_start() || new_break >= self.mem_end() {
            Err(Error::AddressOutOfBounds)
        } else if new_break > self.kernel_memory_break.get() {
            Err(Error::OutOfMemory)
        } else {
            let old_break = self.app_break.get();
            self.app_break.set(new_break);
            Ok(old_break)
        }
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// are within the memory bounds currently exposed to the processes (i.e.
    /// ending at `kernel_memory_break`. If this method returns true, the buffer
    /// is guaranteed to be accessible to the process and to not overlap with
    /// the grant region.
    fn in_app_owned_memory(&self, buf_start_addr: *const u8, size: usize) -> bool {
        let buf_end_addr = buf_start_addr.wrapping_offset(size as isize);

        buf_end_addr >= buf_start_addr
            && buf_start_addr >= self.mem_start()
            && buf_end_addr <= self.mem_break()
    }

    unsafe fn alloc(&self, size: usize) -> Option<&mut [u8]> {
        let new_break = self.kernel_memory_break.get().offset(-(size as isize));
        if new_break < self.app_break.get() {
            None
        } else {
            self.kernel_memory_break.set(new_break);
            Some(slice::from_raw_parts_mut(new_break as *mut u8, size))
        }
    }

    unsafe fn free(&self, _: *mut u8) {}

    unsafe fn grant_ptr(&self, grant_num: usize) -> *mut *mut u8 {
        let grant_num = grant_num as isize;
        (self.mem_end() as *mut *mut u8).offset(-(grant_num + 1))
    }

    fn get_process_name(&self) -> &[u8] {
        self.process_name.as_bytes()
    }

    unsafe fn get_syscall(&self) -> Option<Syscall> {
        let last_syscall = self.syscall.get_syscall(self.sp());

        // Record this for debugging purposes.
        self.debug.map(|debug| {
            debug.syscall_count += 1;
            debug.last_syscall = last_syscall;
        });

        last_syscall
    }

    unsafe fn set_syscall_return_value(&self, return_value: isize) {
        self.syscall
            .set_syscall_return_value(self.sp(), return_value);
    }

    unsafe fn pop_syscall_stack_frame(&self) {
        let mut stored_state = self.stored_state.get();
        let new_stack_pointer = self
            .syscall
            .pop_syscall_stack_frame(self.sp(), &mut stored_state);
        self.current_stack_pointer
            .set(new_stack_pointer as *const u8);
        self.stored_state.set(stored_state);
    }

    unsafe fn push_function_call(&self, callback: FunctionCall) {
        // First we need to get how much memory is available for this app's
        // stack. Since the stack is at the bottom of the process's memory
        // region, this is straightforward.
        let remaining_stack_bytes = self.sp() as usize - self.memory.as_ptr() as usize;

        // Next we should see if we can actually add the frame to the process's
        // stack. Architecture-specific code handles actually doing the push
        // since we don't know the details of exactly what the stack frames look
        // like.
        let stored_state = self.stored_state.get();
        match self.syscall.push_function_call(
            self.sp(),
            remaining_stack_bytes,
            callback,
            &stored_state,
        ) {
            Ok(stack_bottom) => {
                // If we got an `Ok` with the new stack pointer we are all
                // set and should mark that this process is ready to be
                // scheduled.

                // We just setup up a new callback to do, which means this
                // process wants to execute, so we set that there is work to
                // be done.
                self.kernel.increment_work();

                // Move this process to the "running" state so the scheduler
                // will schedule it.
                self.state.set(State::Running);

                // Update helpful debugging metadata.
                self.current_stack_pointer.set(stack_bottom as *mut u8);
                self.debug_set_max_stack_depth();
            }

            Err(bad_stack_bottom) => {
                // If we got an Error, then there was no room to add this
                // stack frame. This process has essentially faulted, so we
                // mark it as such. We also update the debugging metadata so
                // that if the process fault message prints then it should
                // be easier to debug that the process exceeded its stack.
                self.debug.map(|debug| {
                    let bad_stack_bottom = bad_stack_bottom as *const u8;
                    if bad_stack_bottom < debug.min_stack_pointer {
                        debug.min_stack_pointer = bad_stack_bottom;
                    }
                });
                self.set_fault_state();
            }
        }
        self.stored_state.set(stored_state);
    }

    unsafe fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        let mut stored_state = self.stored_state.get();
        let (stack_pointer, switch_reason) =
            self.syscall.switch_to_process(self.sp(), &mut stored_state);
        self.current_stack_pointer.set(stack_pointer as *const u8);
        self.stored_state.set(stored_state);

        // Update debug state as needed after running this process.
        self.debug.map(|debug| {
            // Update max stack depth if needed.
            if self.current_stack_pointer.get() < debug.min_stack_pointer {
                debug.min_stack_pointer = self.current_stack_pointer.get();
            }

            // More debugging help. If this occurred because of a timeslice
            // expiration, mark that so we can check later if a process is
            // exceeding its timeslices too often.
            if switch_reason == syscall::ContextSwitchReason::TimesliceExpired {
                debug.timeslice_expiration_count += 1;
            }
        });

        Some(switch_reason)
    }

    unsafe fn fault_fmt(&self, writer: &mut Write) {
        self.syscall.fault_fmt(writer);
    }

    unsafe fn process_detail_fmt(&self, writer: &mut Write) {
        // Flash
        let flash_end = self.flash.as_ptr().offset(self.flash.len() as isize) as usize;
        let flash_start = self.flash.as_ptr() as usize;
        let flash_protected_size = self.header.get_protected_size() as usize;
        let flash_app_start = flash_start + flash_protected_size;
        let flash_app_size = flash_end - flash_app_start;
        let flash_init_fn = flash_start + self.header.get_init_function_offset() as usize;

        // SRAM addresses
        let sram_end = self.memory.as_ptr().offset(self.memory.len() as isize) as usize;
        let sram_grant_start = self.kernel_memory_break.get() as usize;
        let sram_heap_end = self.app_break.get() as usize;
        let sram_heap_start = self.debug.map_or(ptr::null(), |debug| {
            debug.app_heap_start_pointer.unwrap_or(ptr::null())
        }) as usize;
        let sram_stack_start = self.debug.map_or(ptr::null(), |debug| {
            debug.app_stack_start_pointer.unwrap_or(ptr::null())
        }) as usize;
        let sram_stack_bottom =
            self.debug
                .map_or(ptr::null(), |debug| debug.min_stack_pointer) as usize;
        let sram_start = self.memory.as_ptr() as usize;

        // SRAM sizes
        let sram_grant_size = sram_end - sram_grant_start;
        let sram_heap_size = sram_heap_end - sram_heap_start;
        let sram_data_size = sram_heap_start - sram_stack_start;
        let sram_stack_size = sram_stack_start - sram_stack_bottom;
        let sram_grant_allocated = sram_end - sram_grant_start;
        let sram_heap_allocated = sram_grant_start - sram_heap_start;
        let sram_stack_allocated = sram_stack_start - sram_start;
        let sram_data_allocated = sram_data_size as usize;

        // checking on sram
        let mut sram_grant_error_str = "          ";
        if sram_grant_size > sram_grant_allocated {
            sram_grant_error_str = " EXCEEDED!"
        }
        let mut sram_heap_error_str = "          ";
        if sram_heap_size > sram_heap_allocated {
            sram_heap_error_str = " EXCEEDED!"
        }
        let mut sram_stack_error_str = "          ";
        if sram_stack_size > sram_stack_allocated {
            sram_stack_error_str = " EXCEEDED!"
        }

        // application statistics
        let events_queued = self.tasks.map_or(0, |tasks| tasks.len());
        let syscall_count = self.debug.map_or(0, |debug| debug.syscall_count);
        let last_syscall = self.debug.map(|debug| debug.last_syscall);
        let dropped_callback_count = self.debug.map_or(0, |debug| debug.dropped_callback_count);
        let restart_count = self.debug.map_or(0, |debug| debug.restart_count);

        let _ = writer.write_fmt(format_args!(
            "\
             App: {}   -   [{:?}]\
             \r\n Events Queued: {}   Syscall Count: {}   Dropped Callback Count: {}\
             \n Restart Count: {}\n",
            self.process_name,
            self.state.get(),
            events_queued,
            syscall_count,
            dropped_callback_count,
            restart_count,
        ));

        let _ = match last_syscall {
            Some(syscall) => writer.write_fmt(format_args!(" Last Syscall: {:?}", syscall)),
            None => writer.write_fmt(format_args!(" Last Syscall: None")),
        };

        let _ = writer.write_fmt(format_args!("\
\r\n\
\r\n ╔═══════════╤══════════════════════════════════════════╗\
\r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
\r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
\r\n             │ ▼ Grant      {:6} | {:6}{}\
  \r\n  {:#010X} ┼───────────────────────────────────────────\
\r\n             │ Unused\
  \r\n  {:#010X} ┼───────────────────────────────────────────\
\r\n             │ ▲ Heap       {:6} | {:6}{}     S\
  \r\n  {:#010X} ┼─────────────────────────────────────────── R\
\r\n             │ Data         {:6} | {:6}               A\
  \r\n  {:#010X} ┼─────────────────────────────────────────── M\
\r\n             │ ▼ Stack      {:6} | {:6}{}\
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
  sram_end,
  sram_grant_size, sram_grant_allocated, sram_grant_error_str,
  sram_grant_start,
  sram_heap_end,
  sram_heap_size, sram_heap_allocated, sram_heap_error_str,
  sram_heap_start,
  sram_data_size, sram_data_allocated,
  sram_stack_start,
  sram_stack_size, sram_stack_allocated, sram_stack_error_str,
  sram_stack_bottom,
  sram_start,
  flash_end,
  flash_app_size,
  flash_app_start,
  flash_protected_size,
  flash_start));

        self.syscall
            .process_detail_fmt(self.sp(), &self.stored_state.get(), writer);

        let _ = writer.write_fmt(format_args!(
            "\
             \r\nTo debug, run `make debug RAM_START={:#x} FLASH_INIT={:#x}`\
             \r\nin the app's folder and open the .lst file.\r\n\r\n",
            sram_start, flash_init_fn
        ));
    }
}

impl<S: 'static + UserspaceKernelBoundary> Process<'a, S> {
    crate unsafe fn create(
        kernel: &'static Kernel,
        syscall: &'static S,
        app_flash_address: *const u8,
        remaining_app_memory: *mut u8,
        remaining_app_memory_size: usize,
        fault_response: FaultResponse,
    ) -> (Option<&'static ProcessType>, usize, usize) {
        if let Some(tbf_header) = tbfheader::parse_and_validate_tbf_header(app_flash_address) {
            let app_flash_size = tbf_header.get_total_size() as usize;

            // If this isn't an app (i.e. it is padding) or it is an app but it
            // isn't enabled, then we can skip it but increment past its flash.
            if !tbf_header.is_app() || !tbf_header.enabled() {
                return (None, app_flash_size, 0);
            }

            // Otherwise, actually load the app.
            let mut min_app_ram_size = tbf_header.get_minimum_app_ram_size();
            let process_name = tbf_header.get_package_name();
            let init_fn =
                app_flash_address.offset(tbf_header.get_init_function_offset() as isize) as usize;

            // Set the initial process stack and memory to 128 bytes.
            let initial_stack_pointer = remaining_app_memory.offset(128);
            let initial_sbrk_pointer = remaining_app_memory.offset(128);

            // First determine how much space we need in the application's
            // memory space just for kernel and grant state. We need to make
            // sure we allocate enough memory just for that.

            // Make room for grant pointers.
            let grant_ptr_size = mem::size_of::<*const usize>();
            let grant_ptrs_num = kernel.get_grant_count_and_finalize();
            let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

            // Allocate memory for callback ring buffer.
            let callback_size = mem::size_of::<Task>();
            let callback_len = 10;
            let callbacks_offset = callback_len * callback_size;

            // Make room to store this process's metadata.
            let process_struct_offset = mem::size_of::<Process<S>>();

            // Need to make sure that the amount of memory we allocate for
            // this process at least covers this state.
            if min_app_ram_size
                < (grant_ptrs_offset + callbacks_offset + process_struct_offset) as u32
            {
                min_app_ram_size =
                    (grant_ptrs_offset + callbacks_offset + process_struct_offset) as u32;
            }

            // TODO round app_ram_size up to a closer MPU unit.
            // This is a very conservative approach that rounds up to power of
            // two. We should be able to make this closer to what we actually need.
            let app_ram_size = math::closest_power_of_two(min_app_ram_size) as usize;

            // Check that we can actually give this app this much memory.
            if app_ram_size > remaining_app_memory_size {
                panic!(
                    "{:?} failed to load. Insufficient memory. Requested {} have {}",
                    process_name, app_ram_size, remaining_app_memory_size
                );
            }

            let app_memory = slice::from_raw_parts_mut(remaining_app_memory, app_ram_size);

            // Set up initial grant region.
            let mut kernel_memory_break = app_memory.as_mut_ptr().offset(app_memory.len() as isize);

            // Now that we know we have the space we can setup the grant
            // pointers.
            kernel_memory_break = kernel_memory_break.offset(-(grant_ptrs_offset as isize));

            // Set all pointers to null.
            let opts =
                slice::from_raw_parts_mut(kernel_memory_break as *mut *const usize, grant_ptrs_num);
            for opt in opts.iter_mut() {
                *opt = ptr::null()
            }

            // Now that we know we have the space we can setup the memory
            // for the callbacks.
            kernel_memory_break = kernel_memory_break.offset(-(callbacks_offset as isize));

            // Set up ring buffer.
            let callback_buf =
                slice::from_raw_parts_mut(kernel_memory_break as *mut Task, callback_len);
            let tasks = RingBuffer::new(callback_buf);

            // Last thing is the process struct.
            kernel_memory_break = kernel_memory_break.offset(-(process_struct_offset as isize));
            let process_struct_memory_location = kernel_memory_break;

            // Determine the debug information to the best of our
            // understanding. If the app is doing all of the PIC fixup and
            // memory management we don't know much.
            let mut app_heap_start_pointer = None;
            let mut app_stack_start_pointer = None;

            // Create the Process struct in the app grant region.
            let mut process: &mut Process<S> =
                &mut *(process_struct_memory_location as *mut Process<'static, S>);

            process.kernel = kernel;
            process.syscall = syscall;
            process.memory = app_memory;
            process.header = tbf_header;
            process.kernel_memory_break = Cell::new(kernel_memory_break);
            process.original_kernel_memory_break = kernel_memory_break;
            process.app_break = Cell::new(initial_sbrk_pointer);
            process.original_app_break = initial_sbrk_pointer;
            process.current_stack_pointer = Cell::new(initial_stack_pointer);
            process.original_stack_pointer = initial_stack_pointer;

            process.flash = slice::from_raw_parts(app_flash_address, app_flash_size);

            process.stored_state = Cell::new(Default::default());
            process.state = Cell::new(State::Yielded);
            process.fault_response = fault_response;

            process.mpu_regions = [
                Cell::new((ptr::null(), math::PowerOfTwo::zero())),
                Cell::new((ptr::null(), math::PowerOfTwo::zero())),
                Cell::new((ptr::null(), math::PowerOfTwo::zero())),
                Cell::new((ptr::null(), math::PowerOfTwo::zero())),
                Cell::new((ptr::null(), math::PowerOfTwo::zero())),
            ];
            process.tasks = MapCell::new(tasks);
            process.process_name = process_name;

            process.debug = MapCell::new(ProcessDebug {
                app_heap_start_pointer: app_heap_start_pointer,
                app_stack_start_pointer: app_stack_start_pointer,
                min_stack_pointer: initial_stack_pointer,
                syscall_count: 0,
                last_syscall: None,
                dropped_callback_count: 0,
                restart_count: 0,
                timeslice_expiration_count: 0,
            });

            if (init_fn & 0x1) != 1 {
                panic!(
                    "{:?} process image invalid. \
                     init_fn address must end in 1 to be Thumb, got {:#X}",
                    process_name, init_fn
                );
            }

            let flash_protected_size = process.header.get_protected_size() as usize;
            let flash_app_start = app_flash_address as usize + flash_protected_size;

            process.tasks.map(|tasks| {
                tasks.enqueue(Task::FunctionCall(FunctionCall {
                    pc: init_fn,
                    argument0: flash_app_start,
                    argument1: process.memory.as_ptr() as usize,
                    argument2: process.memory.len() as usize,
                    argument3: process.app_break.get() as usize,
                }));
            });

            kernel.increment_work();

            return (Some(process), app_flash_size, app_ram_size);
        }
        (None, 0, 0)
    }

    fn mem_break(&self) -> *const u8 {
        self.kernel_memory_break.get()
    }

    fn sp(&self) -> *const usize {
        self.current_stack_pointer.get() as *const usize
    }

    /// Reset all `grant_ptr`s to NULL.
    unsafe fn grant_ptrs_reset(&self) {
        let grant_ptrs_num = self.kernel.get_grant_count_and_finalize();
        for grant_num in 0..grant_ptrs_num {
            let grant_num = grant_num as isize;
            let ctr_ptr = (self.mem_end() as *mut *mut usize).offset(-(grant_num + 1));
            write_volatile(ctr_ptr, ptr::null_mut());
        }
    }

    fn debug_set_max_stack_depth(&self) {
        self.debug.map(|debug| {
            if self.current_stack_pointer.get() < debug.min_stack_pointer {
                debug.min_stack_pointer = self.current_stack_pointer.get();
            }
        });
    }
}
