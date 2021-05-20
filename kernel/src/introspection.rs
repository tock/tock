//! Mechanism for inspecting the status of the kernel.
//!
//! In particular this provides functions for getting the status of processes
//! on the board. It potentially could be expanded to other kernel state.
//!
//! To restrict access on what can use this module, even though it is public (in
//! a Rust sense) so it is visible outside of this crate, the introspection
//! functions require the caller have the correct capability to call the
//! functions. This prevents arbitrary capsules from being able to use this
//! module, and only capsules that the board author has explicitly passed the
//! correct capabilities to can use it.

use core::cell::Cell;

use crate::capabilities::ProcessManagementCapability;
use crate::common::cells::NumericCellExt;
use crate::process;
use crate::process::ProcessId;
use crate::sched::Kernel;

/// This struct provides the inspection functions.
pub struct KernelInfo {
    kernel: &'static Kernel,
}

/// These constants are defined in the linker script.
extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    static _estack: u32;
    static _sstack: u32;
    static _stext: u32;
    static _etext: u32;
    static _srodata: u32;
    static _erodata: u32;
    static _srelocate: u32;
    static _erelocate: u32;
    static _szero: u32;
    static _ezero: u32;

}

impl KernelInfo {
    pub fn new(kernel: &'static Kernel) -> KernelInfo {
        KernelInfo { kernel: kernel }
    }

    /// Returns how many processes have been loaded on this platform. This is
    /// functionally equivalent to how many of the process slots have been used
    /// on the board. This does not consider what state the process is in, as
    /// long as it has been loaded.
    pub fn number_loaded_processes(&self, _capability: &dyn ProcessManagementCapability) -> usize {
        let count: Cell<usize> = Cell::new(0);
        self.kernel.process_each(|_| count.increment());
        count.get()
    }

    /// Returns how many processes are considered to be active. This includes
    /// processes in the `Running` and `Yield` states. This does not include
    /// processes which have faulted, or processes which the kernel is no longer
    /// scheduling because they have faulted too frequently or for some other
    /// reason.
    pub fn number_active_processes(&self, _capability: &dyn ProcessManagementCapability) -> usize {
        let count: Cell<usize> = Cell::new(0);
        self.kernel
            .process_each(|process| match process.get_state() {
                process::State::Running => count.increment(),
                process::State::Yielded => count.increment(),
                _ => {}
            });
        count.get()
    }

    /// Returns how many processes are considered to be inactive. This includes
    /// processes in the `Fault` state and processes which the kernel is not
    /// scheduling for any reason.
    pub fn number_inactive_processes(
        &self,
        _capability: &dyn ProcessManagementCapability,
    ) -> usize {
        let count: Cell<usize> = Cell::new(0);
        self.kernel
            .process_each(|process| match process.get_state() {
                process::State::Running => {}
                process::State::Yielded => {}
                _ => count.increment(),
            });
        count.get()
    }

    /// Get the name of the process.
    pub fn process_name(
        &self,
        app: ProcessId,
        _capability: &dyn ProcessManagementCapability,
    ) -> &'static str {
        self.kernel
            .process_map_or("unknown", app, |process| process.get_process_name())
    }

    /// Returns the number of syscalls the app has called.
    pub fn number_app_syscalls(
        &self,
        app: ProcessId,
        _capability: &dyn ProcessManagementCapability,
    ) -> usize {
        self.kernel
            .process_map_or(0, app, |process| process.debug_syscall_count())
    }

    /// Returns the number of dropped upcalls the app has experience.
    /// Upcalls can be dropped if the queue for the app is full when a capsule
    /// tries to schedule a upcall.
    pub fn number_app_dropped_upcalls(
        &self,
        app: ProcessId,
        _capability: &dyn ProcessManagementCapability,
    ) -> usize {
        self.kernel
            .process_map_or(0, app, |process| process.debug_dropped_upcall_count())
    }

    /// Returns the number of time this app has been restarted.
    pub fn number_app_restarts(
        &self,
        app: ProcessId,
        _capability: &dyn ProcessManagementCapability,
    ) -> usize {
        self.kernel
            .process_map_or(0, app, |process| process.get_restart_count())
    }

    /// Returns the number of time this app has exceeded its timeslice.
    pub fn number_app_timeslice_expirations(
        &self,
        app: ProcessId,
        _capability: &dyn ProcessManagementCapability,
    ) -> usize {
        self.kernel
            .process_map_or(0, app, |process| process.debug_timeslice_expiration_count())
    }

    /// Returns a tuple of the (the number of grants in the grant region this
    /// app has allocated, total number of grants that exist in the system).
    pub fn number_app_grant_uses(
        &self,
        app: ProcessId,
        _capability: &dyn ProcessManagementCapability,
    ) -> (usize, usize) {
        // Just need to get the number, this has already been finalized, but it
        // doesn't hurt to call this again.
        let number_of_grants = self.kernel.get_grant_count_and_finalize();
        let used = self.kernel.process_map_or(0, app, |process| {
            // Have process tell us the number of allocated grants. If this
            // process isn't valid then we can't count the grants and all we can
            // do is return 0.
            process.grant_allocated_count().unwrap_or(0)
        });

        (used, number_of_grants)
    }

    /// Returns the total number of times all processes have exceeded
    /// their timeslices.
    pub fn timeslice_expirations(&self, _capability: &dyn ProcessManagementCapability) -> usize {
        let count: Cell<usize> = Cell::new(0);
        self.kernel.process_each(|proc| {
            count.add(proc.debug_timeslice_expiration_count());
        });
        count.get()
    }

    pub fn get_kernel_stack_start(&self) -> u32 {
        unsafe { (&_sstack as *const u32) as u32 }
    }
    pub fn get_kernel_stack_end(&self) -> u32 {
        unsafe { (&_estack as *const u32) as u32 }
    }
    pub fn get_kernel_text_start(&self) -> u32 {
        unsafe { (&_stext as *const u32) as u32 }
    }
    pub fn get_kernel_text_end(&self) -> u32 {
        unsafe { (&_etext as *const u32) as u32 }
    }
    pub fn get_kernel_rodata_start(&self) -> u32 {
        unsafe { (&_srodata as *const u32) as u32 }
    }
    pub fn get_kernel_rodata_end(&self) -> u32 {
        unsafe { (&_erodata as *const u32) as u32 }
    }
    pub fn get_kernel_init_start(&self) -> u32 {
        unsafe { (&_srelocate as *const u32) as u32 }
    }
    pub fn get_kernel_init_end(&self) -> u32 {
        unsafe { (&_erelocate as *const u32) as u32 }
    }
    pub fn get_kernel_bss_start(&self) -> u32 {
        unsafe { (&_szero as *const u32) as u32 }
    }
    pub fn get_kernel_bss_end(&self) -> u32 {
        unsafe { (&_ezero as *const u32) as u32 }
    }
}
