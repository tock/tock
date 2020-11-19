//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use kernel;
use kernel::common::registers::FieldValue;
use kernel::debug;
use kernel::hil::time::Alarm;
use kernel::InterruptService;
use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};
use rv32i::syscall::SysCall;
use rv32i::PMPConfigMacro;

use crate::interrupt_controller;

// TODO: Actually implement the PMP
PMPConfigMacro!(4);

pub struct LiteXVexRiscv<A: 'static + Alarm<'static>, I: 'static + InterruptService<()>> {
    soc_identifier: &'static str,
    userspace_kernel_boundary: SysCall,
    _pmp: PMP,
    scheduler_timer: kernel::VirtualSchedulerTimer<A>,
    interrupt_service: &'static I,
}

impl<A: 'static + Alarm<'static>, I: 'static + InterruptService<()>> LiteXVexRiscv<A, I> {
    pub unsafe fn new(
        soc_identifier: &'static str,
        alarm: &'static A,
        interrupt_service: &'static I,
    ) -> Self {
        Self {
            soc_identifier,
            userspace_kernel_boundary: SysCall::new(),
            _pmp: PMP::new(),
            scheduler_timer: kernel::VirtualSchedulerTimer::new(alarm),
            interrupt_service,
        }
    }

    pub unsafe fn enable_interrupts(&self) {
        interrupt_controller::enable_interrupts();
        interrupt_controller::unmask_all_interrupts();
    }

    unsafe fn handle_interrupts(&self) {
        while let Some(interrupt) = interrupt_controller::next_pending() {
            if !self.interrupt_service.service_interrupt(interrupt as u32) {
                debug!("Unknown interrupt: {}", interrupt);
            }
        }
    }
}

impl<A: 'static + Alarm<'static>, I: 'static + InterruptService<()>> kernel::Chip
    for LiteXVexRiscv<A, I>
{
    // type MPU = PMP;
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;
    type SchedulerTimer = kernel::VirtualSchedulerTimer<A>;
    type WatchDog = ();

    fn mpu(&self) -> &Self::MPU {
        //&self.pmp
        &()
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        let mut reenable_intr = FieldValue::<u32, mie::Register>::new(0, 0, 0);

        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                // TODO: Actually implement the riscv machine timer with VexRiscv
                // unsafe {
                //     timer::TIMER.service_interrupt();
                // }
                reenable_intr += mie::mtimer::SET;
            }
            if mip.is_set(mip::mext) {
                unsafe {
                    self.handle_interrupts();
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
        let mip = CSR.mip.extract();
        mip.matches_any(mip::mext::SET + mip::mtimer::SET)
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
        let _ = writer.write_fmt(format_args!(
            "\r\n---| LiteX configuration for {} |---",
            self.soc_identifier,
        ));
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
/// For the Ibex this gets called when an interrupt occurs while the chip is
/// in kernel mode. All we need to do is check which interrupt occurred and
/// disable it.
///
/// TODO: Only for ibex?
#[export_name = "_start_trap_rust_from_kernel"]
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
///
/// TODO
#[export_name = "_disable_interrupt_trap_rust_from_app"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: u32) {
    match mcause::Trap::from(mcause_val) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}
