use std::cell::RefCell;
use std::collections::vec_deque::VecDeque;
use std::collections::{hash_map::Entry, HashMap};
use std::path::Path;
use std::process::{self, Stdio};

use tock_cells::map_cell::MapCell;

use crate::ipc_syscalls as ipc;

use kernel::capabilities::{ExternalProcessCapability, ProcessManagementCapability};
use kernel::mpu;
use kernel::procs::{
    Error, FaultResponse, FunctionCall, FunctionCallSource, ProcessLoadError, ProcessType, State,
    Task,
};
use kernel::syscall::{self, Syscall, UserspaceKernelBoundary};
use kernel::AppId;
use kernel::AppSlice;
use kernel::CallbackId;
use kernel::Chip;
use kernel::Kernel;
use kernel::ReturnCode;
use kernel::Shared;

use core::cell::Cell;
use core::fmt::Write;
use core::ptr::NonNull;
use tock_cells::numeric_cell_ext::NumericCellExt;

use crate::emulation_config::Config;
use crate::syscall_transport::SyscallTransport;
use crate::Result;
use crate::{log, log_info};

pub struct UnixProcess<'a> {
    id: usize,
    proc_path: &'a Path,
    process: MapCell<process::Child>,
    allow_map: RefCell<HashMap<*const u8, AllowSlice>>,
    allow_count: Cell<usize>,
}

struct AllowSlice {
    slice: Vec<u8>,
}

impl AllowSlice {
    fn new(slice: Vec<u8>) -> AllowSlice {
        AllowSlice { slice: slice }
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn get(&self) -> &Vec<u8> {
        &self.slice
    }
}

impl<'a> UnixProcess<'a> {
    pub fn new(exec: &'a Path, id: usize) -> UnixProcess<'a> {
        UnixProcess {
            id: id,
            proc_path: exec,
            process: MapCell::empty(),
            allow_map: RefCell::new(HashMap::new()),
            allow_count: Cell::new(0),
        }
    }

    /// Starts the process and supplies necessary command line arguments.
    pub fn start(&self, socket_rx: &Path, socket_tx: &Path) -> Result<()> {
        let config = Config::get();
        let mut proc = process::Command::new(self.proc_path);
        proc.arg("--id")
            .arg(self.id.to_string())
            .arg("--socket_send")
            .arg(socket_rx)
            .arg("--socket_recv")
            .arg(socket_tx)
            .arg("--log")
            .arg(config.app_log_level.to_string());

        if config.app_log_level != 0 {
            proc.env("RUST_BACKTRACE", "1");
        } else {
            proc.stdout(Stdio::null());
        }
        let child = proc.spawn()?;
        self.process.put(child);
        Ok(())
    }

    /// Checks if the process has yet been started.
    pub fn was_started(&self) -> bool {
        self.process.is_some()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Iterates through all known allow'ed slices and transfers the contents of
    /// each slice to the app. First kernel sends information about number of
    /// allowed slices, then each meta data of slice `AllowsInfo` after which
    /// raw slice bytes follow.
    ///
    /// The apps do not maintain any information about what slices they have
    /// ALLOWED TO THE KERNEL. iT is the kernel's sole responsibility to track
    /// that information and only request valid slices from the apps.
    pub fn send_allows(&self, transport: &SyscallTransport) {
        if self.allow_count.get() == 0 {
            return;
        }

        let app_id = self.get_id();
        let allows_number = ipc::AllowsInfo {
            number_of_slices: self.allow_count.get(),
        };
        transport.send_msg(app_id, &allows_number);

        for (addr, slice) in self.allow_map.borrow().iter() {
            let allow_slice = ipc::AllowSliceInfo::new(*addr as usize, slice.len());
            transport.send_msg(app_id, &allow_slice);
            transport.send_bytes(app_id, slice.get());
        }
    }

    pub fn recv_allow_data(
        &self,
        app_allow_address: *const u8,
        transport: &SyscallTransport,
    ) -> *mut u8 {
        let slices_count: ipc::AllowsInfo = transport.recv_msg();
        if slices_count.number_of_slices != 1 {
            unsafe {
                panic!(
                    "Received incorrect number of slices got {} expected 1",
                    slices_count.number_of_slices
                );
            }
        }
        let allow_slice: ipc::AllowSliceInfo = transport.recv_msg();

        let mut buf: Vec<u8> = Vec::new();
        buf.resize_with(allow_slice.length, Default::default);
        let rx_len = transport.recv_bytes(buf.as_mut_slice());
        if rx_len != allow_slice.length {
            unsafe {
                panic!(
                    "Slice length mismatch, expected {}, but got {}",
                    allow_slice.length, rx_len
                );
            }
        }
        let app_slice_addr = allow_slice.address as *const u8;
        if app_slice_addr != app_allow_address {
            panic!(
                "Received incorrect allow address, expected {:p}, but got {:p}",
                app_allow_address, app_slice_addr
            );
        }
        let mut allow_map = self.allow_map.borrow_mut();
        let entry = allow_map.entry(app_slice_addr);
        let ret = match entry {
            Entry::Vacant(_) => {
                self.allow_count.increment();
                entry.or_insert(AllowSlice::new(buf))
            }
            Entry::Occupied(_) => entry.or_insert(AllowSlice::new(buf)),
        };

        ret.slice.as_mut_ptr()
    }
}

#[derive(Default)]
struct ProcessDebug {
    timeslice_expiration_count: usize,
    dropped_callback_count: usize,
    syscall_count: usize,
    last_syscall: Option<Syscall>,
}

pub struct EmulatedProcess<C: 'static + Chip> {
    app_id: Cell<AppId>,
    name: &'static str,
    chip: &'static C,
    kernel: &'static Kernel,
    state: Cell<State>,
    tasks: MapCell<VecDeque<Task>>,
    stored_state: MapCell<
        &'static mut <<C as Chip>::UserspaceKernelBoundary as UserspaceKernelBoundary>::StoredState,
    >,
    grant_region: Cell<*mut *mut u8>,
    restart_count: Cell<usize>,
    debug: MapCell<ProcessDebug>,
    external_process_cap: &'static dyn ExternalProcessCapability,
}

impl<C: 'static + Chip> EmulatedProcess<C> {
    pub fn create(
        app_id: AppId,
        name: &'static str,
        chip: &'static C,
        kernel: &'static Kernel,
        start_state: &'static mut <<C as Chip>::UserspaceKernelBoundary as UserspaceKernelBoundary>::StoredState,
        external_process_cap: &'static dyn ExternalProcessCapability,
    ) -> core::result::Result<EmulatedProcess<C>, ()> {
        let process = EmulatedProcess {
            app_id: Cell::new(app_id),
            name: name,
            chip: chip,
            kernel: kernel,
            state: Cell::new(State::Unstarted),
            tasks: MapCell::new(VecDeque::with_capacity(10)),
            stored_state: MapCell::new(start_state),
            grant_region: Cell::new(0 as *mut *mut u8),
            restart_count: Cell::new(0),
            debug: MapCell::new(ProcessDebug::default()),
            external_process_cap: external_process_cap,
        };

        let _ = process
            .stored_state
            .map(|stored_state| unsafe {
                chip.userspace_kernel_boundary().initialize_process(
                    0 as *const usize,
                    0,
                    stored_state,
                )
            })
            .ok_or(())?;

        // Use a special pc of 0 to indicate that we need to exec the process.
        process.tasks.map(|tasks| {
            tasks.push_back(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: 0,
                argument0: 0,
                argument1: 0,
                argument2: 0,
                argument3: 0,
            }));
        });

        kernel.increment_work_external(external_process_cap);
        Ok(process)
    }
}

impl<C: 'static + Chip> EmulatedProcess<C> {
    #[allow(dead_code)]
    pub fn load_processes(
        _kernel: &'static Kernel,
        _chip: &'static C,
        _app_flash: &'static [u8],
        _app_memory: &mut [u8],
        _procs: &'static mut [Option<&'static dyn ProcessType>],
        _fault_response: FaultResponse,
        _capability: &dyn ProcessManagementCapability,
    ) -> std::result::Result<(), ProcessLoadError> {
        return Ok(());
    }
}

impl<C: 'static + Chip> EmulatedProcess<C> {
    fn is_active(&self) -> bool {
        let state = self.state.get();
        state != State::StoppedFaulted && state != State::Fault
    }
}

impl<C: 'static + Chip> ProcessType for EmulatedProcess<C> {
    fn appid(&self) -> AppId {
        self.app_id.get()
    }

    fn enqueue_task(&self, task: Task) -> bool {
        if !self.is_active() {
            return false;
        }

        self.kernel
            .increment_work_external(self.external_process_cap);

        let ret = self.tasks.map_or(false, |tasks| {
            tasks.push_back(task);
            true
        });

        if !ret {
            self.debug.map(|debug| {
                debug.dropped_callback_count += 1;
            });
        }

        ret
    }

    fn dequeue_task(&self) -> Option<Task> {
        self.tasks.map_or(None, |tasks| {
            tasks.pop_front().map(|cb| {
                self.kernel
                    .decrement_work_external(self.external_process_cap);
                cb
            })
        })
    }

    fn remove_pending_callbacks(&self, callback_id: CallbackId) {
        self.tasks.map(|tasks| {
            tasks.retain(|task| match task {
                Task::FunctionCall(call) => match call.source {
                    FunctionCallSource::Kernel => true,
                    FunctionCallSource::Driver(id) => id != callback_id,
                },
                _ => true,
            })
        });
    }

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_yielded_state(&self) {
        if self.state.get() == State::Running {
            self.state.set(State::Yielded);
            self.kernel
                .decrement_work_external(self.external_process_cap);
        }
    }

    fn stop(&self) {
        match self.state.get() {
            State::Running => self.state.set(State::StoppedRunning),
            State::Yielded => self.state.set(State::StoppedYielded),
            _ => {}
        }
    }

    fn resume(&self) {
        match self.state.get() {
            State::StoppedRunning => self.state.set(State::Running),
            State::StoppedYielded => self.state.set(State::Yielded),
            _ => {}
        }
    }

    fn set_fault_state(&self) {
        self.state.set(State::Fault);

        // TODO Handle based on `FaultResponse`
        panic!("Process {} has a fault", self.get_process_name());
    }

    fn get_restart_count(&self) -> usize {
        self.restart_count.get()
    }

    fn get_process_name(&self) -> &'static str {
        self.name
    }

    fn allow(
        &self,
        buf_start_addr: *const u8,
        size: usize,
    ) -> core::result::Result<Option<AppSlice<Shared, u8>>, ReturnCode> {
        // The work has already been done, and |buf_start_addr| ponts to a
        // buffer in this process's heap. We just need to manipulate types here.

        match NonNull::new(buf_start_addr as *mut u8) {
            None => Ok(None),
            Some(buf_start) => {
                let slice = unsafe {
                    AppSlice::new_external(buf_start, size, self.appid(), self.external_process_cap)
                };
                Ok(Some(slice))
            }
        }
    }

    fn flash_non_protected_start(&self) -> *const u8 {
        0 as *const u8
    }

    fn setup_mpu(&self) {}

    fn add_mpu_region(
        &self,
        _unallocated_memory_start: *const u8,
        _unallocated_memory_size: usize,
        _min_region_size: usize,
    ) -> Option<mpu::Region> {
        None
    }

    fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        if !self.is_active() {
            return None;
        }
        let layout = match std::alloc::Layout::from_size_align(size, align) {
            Ok(l) => l,
            Err(e) => {
                log_info!("Failed to alloc region: {}", e);
                return None;
            }
        };
        unsafe {
            let region = std::alloc::alloc(layout);
            Some(NonNull::new_unchecked(region as *mut u8))
        }
    }

    unsafe fn free(&self, _: *mut u8) {
        // Tock processes don't support free yet.
    }

    fn get_grant_ptr(&self, grant_num: usize) -> Option<*mut u8> {
        if !self.is_active() {
            return None;
        }

        if grant_num
            >= self
                .kernel
                .get_grant_count_and_finalize_external(self.external_process_cap)
        {
            return None;
        }

        Some(self.grant_region.get() as *mut u8)
    }

    unsafe fn set_grant_ptr(&self, _grant_num: usize, grant_ptr: *mut u8) {
        self.grant_region.set(grant_ptr as *mut *mut u8);
    }

    unsafe fn set_syscall_return_value(&self, return_value: isize) {
        self.stored_state.map(|stored_state| {
            self.chip
                .userspace_kernel_boundary()
                .set_syscall_return_value(0 as *const usize, stored_state, return_value);
        });
    }

    unsafe fn set_process_function(&self, callback: FunctionCall) {
        let res = self.stored_state.map(|stored_state| {
            self.chip.userspace_kernel_boundary().set_process_function(
                0 as *const usize,
                0,
                stored_state,
                callback,
            )
        });

        match res {
            Some(Ok(_)) => {
                self.kernel
                    .increment_work_external(self.external_process_cap);
                self.state.set(State::Running);
            }

            None | Some(Err(_)) => {
                self.set_fault_state();
            }
        }
    }

    unsafe fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        let res = self.stored_state.map(|stored_state| {
            self.chip
                .userspace_kernel_boundary()
                .switch_to_process(0 as *const usize, stored_state)
                .1
        })?;

        self.debug.map(|debug| {
            if res == syscall::ContextSwitchReason::TimesliceExpired {
                debug.timeslice_expiration_count += 1;
            }
        });

        Some(res)
    }

    fn debug_syscall_count(&self) -> usize {
        self.debug.map_or(0, |debug| debug.syscall_count)
    }

    fn debug_dropped_callback_count(&self) -> usize {
        self.debug.map_or(0, |debug| debug.dropped_callback_count)
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

    // *************************************************************************
    // Functions bellow are required by the `ProcessType` trait but are either
    // unused or do not translate to this framework and are treated as NO-OPs.
    // *************************************************************************
    unsafe fn print_memory_map(&self, _writer: &mut dyn Write) {}

    unsafe fn print_full_process(&self, _writer: &mut dyn Write) {}

    fn brk(&self, _new_break: *const u8) -> core::result::Result<*const u8, Error> {
        Ok(0 as *const u8)
    }

    fn sbrk(&self, _increment: isize) -> core::result::Result<*const u8, Error> {
        Ok(0 as *const u8)
    }

    fn mem_start(&self) -> *const u8 {
        0 as *const u8
    }

    fn mem_end(&self) -> *const u8 {
        0 as *const u8
    }

    fn flash_start(&self) -> *const u8 {
        0 as *const u8
    }

    fn flash_end(&self) -> *const u8 {
        0 as *const u8
    }

    fn kernel_memory_break(&self) -> *const u8 {
        0 as *const u8
    }

    fn number_writeable_flash_regions(&self) -> usize {
        0
    }

    fn get_writeable_flash_region(&self, _region_index: usize) -> (u32, u32) {
        (0, 0)
    }

    fn update_stack_start_pointer(&self, _stack_pointer: *const u8) {}

    fn update_heap_start_pointer(&self, _heap_pointer: *const u8) {}
}
