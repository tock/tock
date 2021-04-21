//! Interface for Tock kernel schedulers.

pub mod cooperative;
pub mod mlfq;
pub mod priority;
pub mod round_robin;

use crate::dynamic_deferred_call::DynamicDeferredCall;
use crate::kernel::StoppedExecutingReason;
use crate::platform::chip::Chip;
use crate::process::ProcessId;
use crate::Kernel;

/// Trait which any scheduler must implement.
pub trait Scheduler<C: Chip> {
    /// Decide which process to run next.
    ///
    /// The scheduler must decide whether to run a process, and if so, which
    /// one. If the scheduler chooses not to run a process, it can request that
    /// the chip enter sleep mode.
    ///
    /// If the scheduler selects a process to run it must provide its `ProcessId`
    /// and an optional timeslice length in microseconds to provide to that
    /// process. If the timeslice is `None`, the process will be run
    /// cooperatively (i.e. without preemption). Otherwise the process will run
    /// with a timeslice set to the specified length.
    fn next(&self, kernel: &Kernel) -> SchedulingDecision;

    /// Inform the scheduler of why the last process stopped executing, and how
    /// long it executed for. Notably, `execution_time_us` will be `None`
    /// if the the scheduler requested this process be run cooperatively.
    fn result(&self, result: StoppedExecutingReason, execution_time_us: Option<u32>);

    /// Tell the scheduler to execute kernel work such as interrupt bottom
    /// halves and dynamic deferred calls. Most schedulers will use this default
    /// implementation, but schedulers which at times wish to defer interrupt
    /// handling will reimplement it.
    ///
    /// Providing this interface allows schedulers to fully manage how the main
    /// kernel loop executes. For example, a more advanced scheduler that
    /// attempts to help processes meet their deadlines may need to defer bottom
    /// half interrupt handling or to selectively service certain interrupts.
    /// Or, a power aware scheduler may want to selectively choose what work to
    /// complete at any time to meet power requirements.
    ///
    /// Custom implementations of this function must be very careful, however,
    /// as this function is called in the core kernel loop.
    unsafe fn execute_kernel_work(&self, chip: &C) {
        chip.service_pending_interrupts();
        DynamicDeferredCall::call_global_instance_while(|| !chip.has_pending_interrupts());
    }

    /// Ask the scheduler whether to take a break from executing userspace
    /// processes to handle kernel tasks. Most schedulers will use this default
    /// implementation, which always prioritizes kernel work, but schedulers
    /// that wish to defer interrupt handling may reimplement it.
    unsafe fn do_kernel_work_now(&self, chip: &C) -> bool {
        chip.has_pending_interrupts()
            || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
    }

    /// Ask the scheduler whether to continue trying to execute a process.
    ///
    /// Once a process is scheduled the kernel will try to execute it until it
    /// has no more work to do or exhausts its timeslice. The kernel will call
    /// this function before every loop to check with the scheduler if it wants
    /// to continue trying to execute this process.
    ///
    /// Most schedulers will use this default implementation, which causes the
    /// `do_process()` loop to return if there are interrupts or deferred calls
    /// that need to be serviced. However, schedulers which wish to defer
    /// interrupt handling may change this, or priority schedulers which wish to
    /// check if the execution of the current process has caused a higher
    /// priority process to become ready (such as in the case of IPC). If this
    /// returns `false`, then `do_process` will exit with a `KernelPreemption`.
    ///
    /// `id` is the identifier of the currently active process.
    unsafe fn continue_process(&self, _id: ProcessId, chip: &C) -> bool {
        !(chip.has_pending_interrupts()
            || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false))
    }
}

/// Enum representing the actions the scheduler can request in each call to
/// `scheduler.next()`.
#[derive(Copy, Clone)]
pub enum SchedulingDecision {
    /// Tell the kernel to run the specified process with the passed timeslice.
    /// If `None` is passed as a timeslice, the process will be run
    /// cooperatively.
    RunProcess((ProcessId, Option<u32>)),

    /// Tell the kernel to go to sleep. Notably, if the scheduler asks the
    /// kernel to sleep when kernel tasks are ready, the kernel will not sleep,
    /// and will instead restart the main loop and call `next()` again.
    TrySleep,
}
