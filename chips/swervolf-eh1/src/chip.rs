//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use kernel;
use kernel::platform::chip::{Chip, InterruptService};
use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::StaticRef;
use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};
use rv32i::syscall::SysCall;
use swerv::eh1_pic::{Pic, PicRegisters};

pub const PIC_BASE: StaticRef<PicRegisters> =
    unsafe { StaticRef::new(0xF00C_0000 as *const PicRegisters) };

pub static mut PIC: Pic = Pic::new(PIC_BASE);

static mut TIMER0_IRQ: VolatileCell<bool> = VolatileCell::new(false);
static mut TIMER1_IRQ: VolatileCell<bool> = VolatileCell::new(false);

/// The UART interrupt line
pub const IRQ_UART: u32 = 1;
/// This is a fake value used to indicate a timer1 interrupt
pub const IRQ_TIMER1: u32 = 0xFFFF_FFFF;

pub struct SweRVolf<'a, I: InterruptService<()> + 'a> {
    userspace_kernel_boundary: SysCall,
    pic: &'a Pic,
    scheduler_timer: swerv::eh1_timer::Timer<'static>,
    mtimer: &'static crate::syscon::SysCon<'static>,
    pic_interrupt_service: &'a I,
}

pub struct SweRVolfDefaultPeripherals<'a> {
    pub uart: crate::uart::Uart<'a>,
    pub timer1: swerv::eh1_timer::Timer<'a>,
}

impl<'a> SweRVolfDefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            uart: crate::uart::Uart::new(crate::uart::UART_BASE),
            timer1: swerv::eh1_timer::Timer::new(swerv::eh1_timer::TimerNumber::ONE),
        }
    }
}

impl<'a> InterruptService<()> for SweRVolfDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            IRQ_UART => {
                self.uart.handle_interrupt();
            }
            IRQ_TIMER1 => {
                // This is a fake value to indiate a timer1
                // interrupt occured.
                self.timer1.handle_interrupt();
            }
            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, I: InterruptService<()> + 'a> SweRVolf<'a, I> {
    pub unsafe fn new(
        pic_interrupt_service: &'a I,
        mtimer: &'static crate::syscon::SysCon,
    ) -> Self {
        Self {
            userspace_kernel_boundary: SysCall::new(),
            pic: &PIC,
            scheduler_timer: swerv::eh1_timer::Timer::new(swerv::eh1_timer::TimerNumber::ZERO),
            mtimer,
            pic_interrupt_service,
        }
    }

    pub fn get_scheduler_timer(&self) -> &swerv::eh1_timer::Timer<'static> {
        &self.scheduler_timer
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

impl<'a, I: InterruptService<()> + 'a> kernel::platform::chip::Chip for SweRVolf<'a, I> {
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        loop {
            let mip = CSR.mip.extract();

            // Check if the timer interrupt is pending
            if mip.is_set(mip::mtimer) {
                self.mtimer.handle_interrupt();
            }
            // timer0/timer1 pending bits in MIP are NOT sticky
            // This means Tock never sees the pending bits in MIP.
            // Instead we have mutable statics that tell us.
            if unsafe { TIMER0_IRQ.get() } {
                // timer0
                self.scheduler_timer.handle_interrupt();
                unsafe {
                    TIMER0_IRQ.set(false);
                }
            }
            if unsafe { TIMER1_IRQ.get() } {
                // timer1
                unsafe {
                    self.pic_interrupt_service.service_interrupt(IRQ_TIMER1);
                    TIMER1_IRQ.set(false);
                }
            }

            if self.pic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_pic_interrupts();
                }
            }

            if !mip.matches_any(mip::mtimer::SET)
                && !unsafe { TIMER0_IRQ.get() }
                && !unsafe { TIMER1_IRQ.get() }
                && self.pic.get_saved_interrupts().is_none()
            {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie
            .modify(mie::mext::SET + mie::mtimer::SET + mie::BIT28::SET + mie::BIT29::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        let mip = CSR.mip.extract();
        self.pic.get_saved_interrupts().is_some()
            || mip.matches_any(mip::mtimer::SET)
            || unsafe { TIMER0_IRQ.get() }
            || unsafe { TIMER1_IRQ.get() }
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
            if CSR.mcause.get() == 0x8000_001D {
                // Timer0
                CSR.mie.modify(mie::BIT29::CLEAR);
                TIMER0_IRQ.set(true);
                return;
            } else if CSR.mcause.get() == 0x8000_001C {
                // Timer1
                CSR.mie.modify(mie::BIT28::CLEAR);
                TIMER1_IRQ.set(true);
                return;
            }
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
