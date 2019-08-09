use core::fmt::Write;

use kernel;
use kernel::debug;
use rv32i;
use rv32i::plic;

use crate::gpio;
use crate::interrupts;
use crate::uart;

#[derive(Copy, Clone, Default)]
pub struct RvStoredState {}

pub struct NullSysCall();

impl NullSysCall {
    pub const unsafe fn new() -> NullSysCall {
        NullSysCall()
    }
}

impl kernel::syscall::UserspaceKernelBoundary for NullSysCall {
    type StoredState = RvStoredState;

    unsafe fn initialize_new_process(
        &self,
        _stack_pointer: *const usize,
        _stack_size: usize,
        _state: &mut Self::StoredState,
    ) -> Result<*const usize, ()> {
        Err(())
    }

    unsafe fn set_syscall_return_value(
        &self,
        _stack_pointer: *const usize,
        _state: &mut RvStoredState,
        _return_value: isize,
    ) {
    }

    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        _remaining_stack_memory: usize,
        _state: &mut RvStoredState,
        _callback: kernel::procs::FunctionCall,
    ) -> Result<*mut usize, *mut usize> {
        Err(stack_pointer as *mut usize)
    }

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        _state: &mut RvStoredState,
    ) -> (*mut usize, kernel::syscall::ContextSwitchReason) {
        (
            stack_pointer as *mut usize,
            kernel::syscall::ContextSwitchReason::Fault,
        )
    }

    unsafe fn fault_fmt(&self, _writer: &mut Write) {}

    unsafe fn process_detail_fmt(
        &self,
        _stack_pointer: *const usize,
        _state: &RvStoredState,
        _writer: &mut Write,
    ) {
    }
}

pub struct E310x {
    userspace_kernel_boundary: NullSysCall,
}

impl E310x {
    pub unsafe fn new() -> E310x {
        E310x {
            userspace_kernel_boundary: NullSysCall::new(),
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        rv32i::plic::disable_all();
        rv32i::plic::clear_all_pending();
        rv32i::plic::enable_all();
    }
}

impl kernel::Chip for E310x {
    type MPU = ();
    type UserspaceKernelBoundary = NullSysCall;
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn systick(&self) -> &Self::SysTick {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &NullSysCall {
        &(self.userspace_kernel_boundary)
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = plic::next_pending() {
                match interrupt {
                    interrupts::UART0 => uart::UART0.handle_interrupt(),
                    index @ interrupts::GPIO0..interrupts::GPIO31 => {
                        gpio::PORT[index as usize].handle_interrupt()
                    }
                    _ => debug!("Pidx {}", interrupt),
                }

                // Mark that we are done with this interrupt and the hardware
                // can clear it.
                plic::complete(interrupt);
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { plic::has_pending() }
    }

    fn sleep(&self) {
        // unsafe {
        // riscv32i::support::wfi();
        rv32i::support::nop();
        // }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::atomic(f)
    }
}

/// Trap handler for board/chip specific code.
///
/// For the e310 this gets called when an interrupt occurs while the chip is
/// in kernel mode. All we need to do is check which interrupt occurred and
/// disable it.
#[export_name = "_start_trap_rust"]
pub extern "C" fn start_trap_rust() {}

/// Function that gets called if an interrupt occurs while an app was running.
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_handler"]
pub extern "C" fn disable_interrupt_trap_handler(_mcause: u32) {}
