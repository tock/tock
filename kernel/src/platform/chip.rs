// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interfaces for implementing microcontrollers in Tock.

use crate::platform::mpu;
use crate::syscall;
use core::fmt::Write;

/// Interface for individual MCUs.
///
/// The trait defines chip-specific properties of Tock's operation. These
/// include whether and which memory protection mechanism and scheduler_timer to
/// use, how to switch between the kernel and userland applications, and how to
/// handle hardware events.
///
/// Each microcontroller should define a struct and implement this trait.
pub trait Chip {
    /// The particular Memory Protection Unit (MPU) for this chip.
    type MPU: mpu::MPU;

    /// Provider to query the currently running thread ID.
    type ThreadIdProvider: ThreadIdProvider;

    /// The implementation of the interface between userspace and the kernel for
    /// this specific chip. Likely this is architecture specific, but individual
    /// chips may have various custom requirements.
    type UserspaceKernelBoundary: syscall::UserspaceKernelBoundary;

    /// The kernel calls this function to tell the chip to check for all pending
    /// interrupts and to correctly dispatch them to the peripheral drivers for
    /// the chip.
    ///
    /// This function should loop internally until all interrupts have been
    /// handled. It is ok, however, if an interrupt occurs after the last check
    /// but before this function returns. The kernel will handle this edge case.
    fn service_pending_interrupts(&self);

    /// Ask the chip to check if there are any pending interrupts.
    fn has_pending_interrupts(&self) -> bool;

    /// Returns a reference to the implementation for the MPU on this chip.
    fn mpu(&self) -> &Self::MPU;

    /// Returns a reference to the implementation for the interface between
    /// userspace and kernelspace.
    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary;

    /// Called when there is nothing left for the chip to do and it should enter
    /// a low power sleep state. This low power sleep state should allow
    /// interrupts to still be active so that the next interrupt event wakes the
    /// chip and resumes the scheduler.
    fn sleep(&self);

    /// Run a function in an atomic state w.r.t. to the current core. This
    /// means that interrupts are disabled so that an interrupt will not fire
    /// during the passed in function's execution, but *does not* make any
    /// guarantees about memory consistency on a multi-core system.
    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R;

    /// Print out debug information about the current chip state (system
    /// registers, MPU configuration, etc.) to a supplied writer.
    ///
    /// This function may be called across thread boundaries (such as from a
    /// panic handler). As implementors of `Chip` do not have to be `Send` or
    /// `Sync`, `&self` may not be available in these contexts. Therefore, this
    /// function instead accepts an `Option<&Self>` parameter named `this`. In
    /// contexts where `&self` is available, callers should invoke this function
    /// by passing `Some(&self)` to `this`. Otherwise, `this` will be set to
    /// `None`. The implementation of `print_state` may not print certain
    /// information if it depends on runtime-accessible state in `Self`, but
    /// that reference is not provided.
    unsafe fn print_state(this: Option<&Self>, writer: &mut dyn Write);
}

/// Interface for retrieving the currently executing thread.
///
/// This is used to enforce correctness with shared state that has access
/// restrictions (e.g., only a single thread can access a specific value).
///
/// Many embedded platforms are single-core and only permit a single execution
/// thread at a time. However, interrupts can typically occur at any time, and
/// the execution of an interrupt service routine (ISR) constitutes a second
/// thread. Implementations of this trait must be able to differentiate between
/// at minimum the main thread of execution and an ISR execution, but may also
/// consider multiple execution threads if available on a particular device.
///
/// # Safety
///
/// This thread is marked as `unsafe` as implementation must guarantee its
/// correctness. Users of this trait are allowed to make soundness guarantees
/// based on the implementation being correct. Failing to provide a correct
/// implementation can lead to unsound behavior. By implementing this trait,
/// providers are guaranteeing the implementations are always correct for the
/// given hardware platform.
pub unsafe trait ThreadIdProvider {
    /// Return a unique ID for the currently executing thread.
    ///
    /// The unique ID must fit in a `usize` and must be unique and consistent
    /// for the currently running thread. The actual value is opaque and there
    /// is no assumption about the meaning of the assigned IDs. Implementations
    /// are allowed to arbitrarily assign IDs to threads as long as the IDs are
    /// unique and consistent.
    fn running_thread_id() -> usize;
}

/// Interface for handling interrupts on a hardware chip.
///
/// Each board must construct an implementation of this trait to handle specific
/// interrupts. When an interrupt (identified by number) has triggered and
/// should be handled, the implementation of this trait will be called with the
/// interrupt number. The implementation can then handle the interrupt, or
/// return `false` to signify that it does not know how to handle the interrupt.
///
/// This functionality is given this `InterruptService` interface so that
/// multiple objects can be chained together to handle interrupts for a chip.
/// This is useful for code organization and removing the need for duplication
/// when multiple variations of a specific microcontroller exist. Then a shared,
/// base object can handle most interrupts, and variation-specific objects can
/// handle the variation-specific interrupts.
///
/// To simplify structuring the Rust code when using `InterruptService`, the
/// interrupt number should be passed "top-down". That is, an interrupt to be
/// handled will first be passed to the `InterruptService` object that is most
/// specific. If that object cannot handle the interrupt, then it should
/// maintain a reference to the second most specific object, and return by
/// calling to that object to handle the interrupt. This continues until the
/// base object handles the interrupt or decides that the chip does not know how
/// to handle the interrupt. For example, consider a `nRF52840` chip that
/// depends on the `nRF52` crate. If both have specific interrupts they know how
/// to handle, the flow would look like:
///
/// ```ignore
///           +---->nrf52840_peripherals
///           |        |
///           |        |
///           |        v
/// kernel-->nrf52     nrf52_peripherals
/// ```
/// where the kernel instructs the `nrf52` crate to handle interrupts, and if
/// there is an interrupt ready then that interrupt is passed through the
/// InterruptService objects until something can service it.
pub trait InterruptService {
    /// Service an interrupt, if supported by this chip. If this interrupt
    /// number is not supported, return false.
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool;
}

/// A default implementation of `InterruptService` that handles nothing and returns `false`.
impl InterruptService for () {
    unsafe fn service_interrupt(&self, _interrupt: u32) -> bool {
        false
    }
}

/// Generic operations that clock-like things are expected to support.
pub trait ClockInterface {
    fn is_enabled(&self) -> bool;
    fn enable(&self);
    fn disable(&self);
}

/// Helper struct for interfaces that expect clocks, but have no clock control.
pub struct NoClockControl {}
impl ClockInterface for NoClockControl {
    fn is_enabled(&self) -> bool {
        true
    }
    fn enable(&self) {}
    fn disable(&self) {}
}

/// Instance of NoClockControl for things that need references to
/// `ClockInterface` objects.
pub const NO_CLOCK_CONTROL: NoClockControl = NoClockControl {};
