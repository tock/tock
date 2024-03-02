Scheduling
==========

This describes how processes are scheduled by the Tock kernel.

<!-- npm i -g markdown-toc; markdown-toc -i Scheduling.md -->

<!-- toc -->

- [Tock Scheduling](#tock-scheduling)
- [Process State](#process-state)

<!-- tocstop -->

## Tock Scheduling

The kernel defines a `Scheduler` trait that the main kernel loop uses to
determine which process to execute next. Here is a simplified view of that
trait:

```rust
pub trait Scheduler {
    /// Decide which process to run next.
    fn next(&self) -> SchedulingDecision;

    /// Inform the scheduler of why the last process stopped executing, and how
    /// long it executed for.
    fn result(&self, result: StoppedExecutingReason, execution_time_us: Option<u32>);

    /// Tell the scheduler to execute kernel work such as interrupt bottom
    /// halves and dynamic deferred calls. Most schedulers will use the default
    /// implementation.
    unsafe fn execute_kernel_work(&self, chip: &C) {...}

    /// Ask the scheduler whether to take a break from executing userspace
    /// processes to handle kernel tasks.
    unsafe fn do_kernel_work_now(&self, chip: &C) -> bool {...}

    /// Ask the scheduler whether to continue trying to execute a process.
    /// Most schedulers will use this default implementation.
    unsafe fn continue_process(&self, _id: ProcessId, chip: &C) -> bool {...}
}
```

Individual boards can choose which scheduler to use, and implementing new
schedulers just requires implementing this trait.

## Process State

In Tock, a process can be in one of seven states:

- **Running**: Normal operation. A Running process is eligible to be scheduled
  for execution, although is subject to being paused by Tock to allow interrupt
  handlers or other processes to run. During normal operation, a process remains
  in the Running state until it explicitly yields. Upcalls from other kernel
  operations are not delivered to Running processes (i.e. upcalls do not
  interrupt processes), rather they are enqueued until the process yields.
- **Yielded**: Suspended operation. A Yielded process will not be scheduled by
  Tock. Processes often yield while they are waiting for I/O or other operations
  to complete and have no immediately useful work to do. Whenever the kernel
  issues an upcall to a Yielded process, the process is transitioned to the
  Running state.
- **Fault**: Erroneous operation. A Fault-ed process will not be scheduled by
  Tock. Processes enter the Fault state by performing an illegal operation, such
  as accessing memory outside of their address space.
- **Terminated**: The process ended itself by calling the `Exit` system call and
  the kernel has not restarted it.
- **Unstarted**: The process has not yet started; this state is typically very
  short-lived, between process loading and it started. However, in cases when
  processes might be loaded for a long time without running, this state might be
  long-lived.
- **StoppedRunning**, **StoppedYielded**: These states correspond to a process
  that was in either the Running or Yielded state but was then explicitly
  stopped by the kernel (e.g., by the process console). A process in these
  states will not be made runnable until it is restarted, at which point it will
  continue execution where it was stopped.
