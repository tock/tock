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

    unsafe fn get_syscall(&self, _stack_pointer: *const usize) -> Option<kernel::syscall::Syscall> {
        None
    }

    unsafe fn set_syscall_return_value(&self, _stack_pointer: *const usize, _return_value: isize) {}

    unsafe fn pop_syscall_stack_frame(
        &self,
        stack_pointer: *const usize,
        _state: &mut RvStoredState,
    ) -> *mut usize {
        stack_pointer as *mut usize
    }

    unsafe fn push_function_call(
        &self,
        stack_pointer: *const usize,
        _remaining_stack_memory: usize,
        _callback: kernel::procs::FunctionCall,
        _state: &RvStoredState,
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
