//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use kernel;
use kernel::debug;
use kernel::hil::time::Alarm;
use rv32i;
use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};
use rv32i::PMPConfigMacro;

use crate::interrupts;
use crate::plic;
use kernel::InterruptService;

PMPConfigMacro!(8);

pub struct E310x<'a, A: 'static + Alarm<'static>, I: InterruptService<()> + 'a> {
    userspace_kernel_boundary: rv32i::syscall::SysCall,
    pmp: PMP,
    scheduler_timer: kernel::VirtualSchedulerTimer<A>,
    timer: &'a rv32i::machine_timer::MachineTimer<'a>,
    plic_interrupt_service: &'a I,
}

pub struct E310xDefaultPeripherals<'a> {
    pub uart0: sifive::uart::Uart<'a>,
    pub gpio_port: crate::gpio::Port<'a>,
    pub prci: sifive::prci::Prci,
    pub pwm0: sifive::pwm::Pwm,
    pub pwm1: sifive::pwm::Pwm,
    pub pwm2: sifive::pwm::Pwm,
    pub rtc: sifive::rtc::Rtc,
    pub watchdog: sifive::watchdog::Watchdog,
}

impl<'a> E310xDefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            uart0: sifive::uart::Uart::new(crate::uart::UART0_BASE, 16_000_000),
            gpio_port: crate::gpio::Port::new(),
            prci: sifive::prci::Prci::new(crate::prci::PRCI_BASE),
            pwm0: sifive::pwm::Pwm::new(crate::pwm::PWM0_BASE),
            pwm1: sifive::pwm::Pwm::new(crate::pwm::PWM1_BASE),
            pwm2: sifive::pwm::Pwm::new(crate::pwm::PWM2_BASE),
            rtc: sifive::rtc::Rtc::new(crate::rtc::RTC_BASE),
            watchdog: sifive::watchdog::Watchdog::new(crate::watchdog::WATCHDOG_BASE),
        }
    }
}

impl<'a> InterruptService<()> for E310xDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART0 => self.uart0.handle_interrupt(),
            int_pin @ interrupts::GPIO0..=interrupts::GPIO31 => {
                let pin = &self.gpio_port[(int_pin - interrupts::GPIO0) as usize];
                pin.handle_interrupt();
            }

            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, A: 'static + Alarm<'static>, I: InterruptService<()> + 'a> E310x<'a, A, I> {
    pub unsafe fn new(
        alarm: &'static A,
        plic_interrupt_service: &'a I,
        timer: &'a rv32i::machine_timer::MachineTimer<'a>,
    ) -> Self {
        Self {
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
            pmp: PMP::new(),
            scheduler_timer: kernel::VirtualSchedulerTimer::new(alarm),
            timer,
            plic_interrupt_service,
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        plic::disable_all();
        plic::clear_all_pending();
        plic::enable_all();
    }

    unsafe fn handle_plic_interrupts(&self) {
        while let Some(interrupt) = plic::next_pending() {
            if !self.plic_interrupt_service.service_interrupt(interrupt) {
                debug!("Pidx {}", interrupt);
            }
            plic::complete(interrupt);
        }
    }
}

impl<'a, A: 'static + Alarm<'static>, I: InterruptService<()> + 'a> kernel::Chip
    for E310x<'a, A, I>
{
    type MPU = PMP;
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
        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.handle_interrupt();
            }
            if mip.is_set(mip::mext) {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if !mip.matches_any(mip::mext::SET + mip::mtimer::SET) {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie.modify(mie::mext::SET + mie::mtimer::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        CSR.mip.matches_any(mip::mext::SET + mip::mtimer::SET)
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
            CSR.mie.modify(mie::mext::CLEAR);
        }

        mcause::Interrupt::Unknown => {
            panic!("interrupt of unknown cause");
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// For the e310 this gets called when an interrupt occurs while the chip is
/// in kernel mode. All we need to do is check which interrupt occurred and
/// disable it.
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
    match mcause::Trap::from(mcause_val) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}
