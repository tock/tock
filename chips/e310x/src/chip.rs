//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use kernel;
use kernel::common::registers::FieldValue;
use kernel::debug;
use kernel::hil::time::Alarm;
use rv32i;
use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};

use crate::gpio;
use crate::interrupts;
use crate::plic;
use crate::timer;
use crate::uart;

pub struct E310x<A: 'static + Alarm<'static>> {
    userspace_kernel_boundary: rv32i::syscall::SysCall,
    pmp: rv32i::pmp::PMPConfig,
    scheduler_timer: kernel::VirtualSchedulerTimer<A>,
}

impl<A: 'static + Alarm<'static>> E310x<A> {
    pub unsafe fn new(alarm: &'static A) -> Self {
        Self {
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
            pmp: rv32i::pmp::PMPConfig::new(8),
            scheduler_timer: kernel::VirtualSchedulerTimer::new(alarm),
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        plic::disable_all();
        plic::clear_all_pending();
        plic::enable_all();
    }

    unsafe fn handle_plic_interrupts() {
        while let Some(interrupt) = plic::next_pending() {
            match interrupt {
                interrupts::UART0 => uart::UART0.handle_interrupt(),
                int_pin @ interrupts::GPIO0..=interrupts::GPIO31 => {
                    let pin = &gpio::PORT[(int_pin - interrupts::GPIO0) as usize];
                    pin.handle_interrupt();
                }
                _ => debug!("Pidx {}", interrupt),
            }
            plic::complete(interrupt);
        }
    }
}

impl<A: 'static + Alarm<'static>> kernel::Chip for E310x<A> {
    type MPU = rv32i::pmp::PMPConfig;
    type UserspaceKernelBoundary = rv32i::syscall::SysCall;
    type SchedulerTimer = kernel::VirtualSchedulerTimer<A>;
    type WatchDog = ();

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &rv32i::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        let mut reenable_intr = FieldValue::<usize, mie::Register>::new(0, 0, 0);

        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                unsafe {
                    timer::MACHINETIMER.handle_interrupt();
                }
                reenable_intr += mie::mtimer::SET;
            }
            if mip.is_set(mip::mext) {
                unsafe {
                    Self::handle_plic_interrupts();
                }
                reenable_intr += mie::mext::SET;
            }

            if !mip.matches_any(mip::mext::SET + mip::mtimer::SET) {
                break;
            }
        }

        // re-enable any interrupt classes which we handled
        CSR.mie.modify(reenable_intr);
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

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        rv32i::print_riscv_state(writer);
    }
}

fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall | mcause::Exception::SupervisorEnvCall => (),

        mcause::Exception::InstructionMisaligned
        | mcause::Exception::InstructionFault
        | mcause::Exception::IllegalInstruction
        | mcause::Exception::Breakpoint
        | mcause::Exception::LoadMisaligned
        | mcause::Exception::LoadFault
        | mcause::Exception::StoreMisaligned
        | mcause::Exception::StoreFault
        | mcause::Exception::MachineEnvCall
        | mcause::Exception::InstructionPageFault
        | mcause::Exception::LoadPageFault
        | mcause::Exception::StorePageFault
        | mcause::Exception::Unknown => {
            panic!("fatal exception");
        }
    }
}

unsafe fn handle_interrupt(intr: mcause::Interrupt) {
    match intr {
        mcause::Interrupt::UserSoft
        | mcause::Interrupt::UserTimer
        | mcause::Interrupt::UserExternal => {
            debug!("unexpected user-mode interrupt");
        }
        mcause::Interrupt::SupervisorExternal
        | mcause::Interrupt::SupervisorTimer
        | mcause::Interrupt::SupervisorSoft => {
            debug!("unexpected supervisor-mode interrupt");
        }

        mcause::Interrupt::MachineSoft => {
            CSR.mie.modify(mie::msoft::CLEAR);
        }
        mcause::Interrupt::MachineTimer => {
            CSR.mie.modify(mie::mtimer::CLEAR);
        }
        mcause::Interrupt::MachineExternal => {
            CSR.mie.modify(mie::mext::CLEAR);
        }

        mcause::Interrupt::Unknown => {
            debug!("interrupt of unknown cause");
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
    match mcause::Trap::from(CSR.mcause.extract()) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        mcause::Trap::Exception(exception) => {
            handle_exception(exception);
        }
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_handler"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: usize) {
    match mcause::Trap::from(mcause_val) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}
