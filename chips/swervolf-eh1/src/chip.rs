//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use kernel;
use kernel::common::StaticRef;
use kernel::hil::time::Alarm;
use kernel::{Chip, InterruptService};
use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};
use rv32i::syscall::SysCall;
use swerv::eh1_pic::{Pic, PicRegisters};

pub const PIC_BASE: StaticRef<PicRegisters> =
    unsafe { StaticRef::new(0xF00C_0000 as *const PicRegisters) };

pub static mut PIC: Pic = Pic::new(PIC_BASE);

pub struct SweRVolf<'a, A: 'static + Alarm<'static>, I: InterruptService<()> + 'a> {
    userspace_kernel_boundary: SysCall,
    pic: &'a Pic,
    scheduler_timer: kernel::VirtualSchedulerTimer<A>,
    timer: &'static crate::syscon::SysCon<'static>,
    pic_interrupt_service: &'a I,
}

pub struct SweRVolfDefaultPeripherals<'a> {
    pub uart: crate::uart::Uart<'a>,
}

impl<'a> SweRVolfDefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            uart: crate::uart::Uart::new(crate::uart::UART_BASE),
        }
    }
}

impl<'a> InterruptService<()> for SweRVolfDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            1 => {
                self.uart.handle_interrupt();
            }
            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, A: 'static + Alarm<'static>, I: InterruptService<()> + 'a> SweRVolf<'a, A, I> {
    pub unsafe fn new(
        virtual_alarm: &'static A,
        pic_interrupt_service: &'a I,
        timer: &'static crate::syscon::SysCon,
    ) -> Self {
        Self {
            userspace_kernel_boundary: SysCall::new(),
            pic: &PIC,
            scheduler_timer: kernel::VirtualSchedulerTimer::new(virtual_alarm),
            timer,
            pic_interrupt_service,
        }
    }

    pub unsafe fn enable_pic_interrupts(&self) {
        self.pic.enable_all();
    }

    unsafe fn handle_pic_interrupts(&self) {
        while let Some(interrupt) = self.pic.get_saved_interrupts() {
            if !self.pic_interrupt_service.service_interrupt(interrupt) {
                panic!("Unhandled interrupt {}", interrupt);
            }
            self.atomic(|| {
                // Safe as interrupts are disabled
                self.pic.complete(interrupt);
            });
        }
    }
}

impl<'a, A: 'static + Alarm<'static>, I: InterruptService<()> + 'a> kernel::Chip
    for SweRVolf<'a, A, I>
{
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;
    type SchedulerTimer = kernel::VirtualSchedulerTimer<A>;
    type WatchDog = ();

    fn mpu(&self) -> &Self::MPU {
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
        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.handle_interrupt();
            }
            if self.pic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_pic_interrupts();
                }
            }

            if !mip.matches_any(mip::mtimer::SET) && self.pic.get_saved_interrupts().is_none() {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie.modify(mie::mext::SET + mie::mtimer::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        let mip = CSR.mip.extract();
        self.pic.get_saved_interrupts().is_some() || mip.matches_any(mip::mtimer::SET)
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
            panic!("fatal exception: {:?}: {:#x}", exception, CSR.mtval.get());
        }
    }
}

unsafe fn handle_interrupt(intr: mcause::Interrupt) {
    match intr {
        mcause::Interrupt::UserSoft
        | mcause::Interrupt::UserTimer
        | mcause::Interrupt::UserExternal => {
            panic!("unexpected user-mode interrupt");
        }
        mcause::Interrupt::SupervisorExternal
        | mcause::Interrupt::SupervisorTimer
        | mcause::Interrupt::SupervisorSoft => {
            panic!("unexpected supervisor-mode interrupt");
        }

        mcause::Interrupt::MachineSoft => {
            CSR.mie.modify(mie::msoft::CLEAR);
        }
        mcause::Interrupt::MachineTimer => {
            CSR.mie.modify(mie::mtimer::CLEAR);
        }
        mcause::Interrupt::MachineExternal => {
            // We received an interrupt, disable interrupts while we handle them
            CSR.mie.modify(mie::mext::CLEAR);

            // Claim the interrupt, unwrap() as we know an interrupt exists
            // Once claimed this interrupt won't fire until it's completed
            // NOTE: The interrupt is no longer pending in the PIC
            loop {
                let interrupt = PIC.next_pending();

                match interrupt {
                    Some(irq) => {
                        // Safe as interrupts are disabled
                        PIC.save_interrupt(irq);
                    }
                    None => {
                        // Enable generic interrupts
                        CSR.mie.modify(mie::mext::SET);
                        break;
                    }
                }
            }
        }

        mcause::Interrupt::Unknown => {
            panic!("interrupt of unknown cause");
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// This gets called when an interrupt occurs while the chip is
/// in kernel mode.
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
#[export_name = "_disable_interrupt_trap_rust_from_app"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: u32) {
    match mcause::Trap::from(mcause_val as usize) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}
