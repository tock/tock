//! High-level setup and interrupt mapping for the chip.

use crate::intc::{Intc, IntcRegisters};
use core::fmt::Write;
use kernel;
use kernel::common::registers::interfaces::{ReadWriteable, Readable};
use kernel::common::StaticRef;
use kernel::{Chip, InterruptService};
use rv32i::csr::{self, mcause, CSR};
use rv32i::syscall::SysCall;

pub const INTC_BASE: StaticRef<IntcRegisters> =
    unsafe { StaticRef::new(0x600C_2000 as *const IntcRegisters) };

pub static mut INTC: Intc = Intc::new(INTC_BASE);

pub struct Esp32C3<'a, I: InterruptService<()> + 'a> {
    userspace_kernel_boundary: SysCall,
    intc: &'a Intc,
    scheduler_timer: esp32::timg::TimG<'a>,
    pic_interrupt_service: &'a I,
}

pub struct Esp32C3DefaultPeripherals<'a> {
    pub uart0: esp32::uart::Uart<'a>,
    pub timg0: esp32::timg::TimG<'a>,
    pub rtc_cntl: esp32::rtc_cntl::RtcCntl,
}

impl<'a> Esp32C3DefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            uart0: esp32::uart::Uart::new(esp32::uart::UART0_BASE),
            timg0: esp32::timg::TimG::new(esp32::timg::TIMG0_BASE),
            rtc_cntl: esp32::rtc_cntl::RtcCntl::new(esp32::rtc_cntl::RTC_CNTL_BASE),
        }
    }
}

impl<'a> InterruptService<()> for Esp32C3DefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            5 => {
                self.uart0.handle_interrupt();
            }
            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, I: InterruptService<()> + 'a> Esp32C3<'a, I> {
    pub unsafe fn new(pic_interrupt_service: &'a I) -> Self {
        Self {
            userspace_kernel_boundary: SysCall::new(),
            intc: &INTC,
            scheduler_timer: esp32::timg::TimG::new(esp32::timg::TIMG1_BASE),
            pic_interrupt_service,
        }
    }

    pub unsafe fn enable_pic_interrupts(&self) {
        self.intc.enable_all();
    }

    unsafe fn handle_pic_interrupts(&self) {
        while let Some(interrupt) = self.intc.get_saved_interrupts() {
            if !self.pic_interrupt_service.service_interrupt(interrupt) {
                panic!("Unhandled interrupt {}", interrupt);
            }
            self.atomic(|| {
                // Safe as interrupts are disabled
                self.intc.complete(interrupt);
            });
        }
    }
}

impl<'a, I: InterruptService<()> + 'a> kernel::Chip for Esp32C3<'a, I> {
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;
    type SchedulerTimer = esp32::timg::TimG<'a>;
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
            if self.intc.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_pic_interrupts();
                }
            }

            if self.intc.get_saved_interrupts().is_none() {
                break;
            }
        }

        self.intc.enable_all();
    }

    fn has_pending_interrupts(&self) -> bool {
        self.intc.get_saved_interrupts().is_some()
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

unsafe fn handle_interrupt(_intr: mcause::Interrupt) {
    CSR.mstatus.modify(csr::mstatus::mstatus::mie::CLEAR);

    // Claim the interrupt, unwrap() as we know an interrupt exists
    // Once claimed this interrupt won't fire until it's completed
    // NOTE: The interrupt is no longer pending in the PLIC
    loop {
        let interrupt = INTC.next_pending();

        match interrupt {
            Some(irq) => {
                // Safe as interrupts are disabled
                INTC.save_interrupt(irq);
                INTC.disable(irq);
            }
            None => {
                // Enable generic interrupts
                CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);
                break;
            }
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
