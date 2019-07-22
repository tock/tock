use core::fmt::Write;

use kernel;
use kernel::debug;
use rv32i;
use rv32i::plic;

use crate::gpio;
use crate::interrupts;
use crate::uart;

pub struct E310x {
    userspace_kernel_boundary: rv32i::syscall::SysCall,
}

impl E310x {
    pub unsafe fn new() -> E310x {
        E310x {
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
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
    type UserspaceKernelBoundary = rv32i::syscall::SysCall;
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn systick(&self) -> &Self::SysTick {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &rv32i::syscall::SysCall {
        &self.userspace_kernel_boundary
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
        unsafe {
            rv32i::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::atomic(f)
    }
}

pub unsafe fn handle_trap() {
    // TODO THIS IS TEMPORARY FOR DEBUGGING ONLY
    let val = rv32i::riscvregs::register::mtval::read();
    let cause = rv32i::riscvregs::register::mcause::read();
    // if most sig bit is set, is interrupt
    if cause.is_interrupt() {
        // strip off the msb
        match rv32i::riscvregs::register::mcause::Interrupt::from(cause.code()) {
            rv32i::riscvregs::register::mcause::Interrupt::MachineSoft => {
                debug!("encountered machine mode software interrupt");
            }
            // should never occur
            rv32i::riscvregs::register::mcause::Interrupt::UserSoft => (),
            rv32i::riscvregs::register::mcause::Interrupt::SupervisorSoft => (),

            rv32i::riscvregs::register::mcause::Interrupt::MachineTimer => (),

            // should never occur
            rv32i::riscvregs::register::mcause::Interrupt::UserTimer => (),
            rv32i::riscvregs::register::mcause::Interrupt::SupervisorTimer => (),

            // this includes UART, GPIO pins, etc
            rv32i::riscvregs::register::mcause::Interrupt::MachineExternal => {
                // just send out a message that the interrupt occurred and complete it
                let ext_interrupt_wrapper = plic::next_pending();
                match ext_interrupt_wrapper {
                    None => (),
                    Some(ext_interrupt_id) => {
                        debug!("interrupt triggered {}\n", ext_interrupt_id);
                        plic::complete(ext_interrupt_id);
                        plic::surpress_all();
                    }
                }
            }
            // should never occur
            rv32i::riscvregs::register::mcause::Interrupt::UserExternal => (),
            rv32i::riscvregs::register::mcause::Interrupt::SupervisorExternal => (),

            rv32i::riscvregs::register::mcause::Interrupt::Unknown => {
                debug!("interrupt of unknown cause");
            }
        }
    } else {
        // strip off the msb, pattern match
        match rv32i::riscvregs::register::mcause::Exception::from(cause.code()) {
            rv32i::riscvregs::register::mcause::Exception::InstructionMisaligned => {
                panic!(
                    "misaligned instruction {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::InstructionFault => {
                panic!(
                    "instruction fault {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::IllegalInstruction => {
                panic!(
                    "illegal instruction {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::Breakpoint => {
                debug!("breakpoint\n");
            }
            rv32i::riscvregs::register::mcause::Exception::LoadMisaligned => {
                panic!(
                    "misaligned load {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::LoadFault => {
                panic!(
                    "load fault {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::StoreMisaligned => {
                panic!(
                    "misaligned store {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::StoreFault => {
                panic!(
                    "store fault {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::UserEnvCall => (),
            rv32i::riscvregs::register::mcause::Exception::SupervisorEnvCall => (),
            rv32i::riscvregs::register::mcause::Exception::MachineEnvCall => {
                // GENERATED BY ECALL; should never happen....
                panic!("machine mode environment call\n");
            }
            rv32i::riscvregs::register::mcause::Exception::InstructionPageFault => {
                panic!(
                    "instruction page fault {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::LoadPageFault => {
                panic!(
                    "load page fault {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::StorePageFault => {
                panic!(
                    "store page fault {:x}\n",
                    rv32i::riscvregs::register::mtval::read()
                );
            }
            rv32i::riscvregs::register::mcause::Exception::Unknown => {
                panic!("exception type unknown");
            }
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// For the e310 this gets called when an interrupt occurs while the chip is
/// in kernel mode. All we need to do is check which interrupt occurred and
/// disable it.
#[export_name = "_start_trap_rust"]
pub unsafe extern "C" fn start_trap_rust() {
    unsafe {
        handle_trap();
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_handler"]
pub extern "C" fn disable_interrupt_trap_handler(_mcause: u32) {
    unsafe {
        handle_trap();
    }
}
