//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;

use kernel;
use kernel::platform::chip::{Chip, InterruptService};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;

use rv32i::csr::{self, mcause, mtvec::mtvec, CSR};
use rv32i::pmp::PMP;
use rv32i::syscall::SysCall;

use crate::intc::{Intc, IntcRegisters};
use crate::interrupts;

pub const INTC_BASE: StaticRef<IntcRegisters> =
    unsafe { StaticRef::new(0x600C_2000 as *const IntcRegisters) };

pub static mut INTC: Intc = Intc::new(INTC_BASE);

pub struct Esp32C3<'a, I: InterruptService<()> + 'a> {
    userspace_kernel_boundary: SysCall,
    pub pmp: PMP<8>,
    intc: &'a Intc,
    pic_interrupt_service: &'a I,
}

pub struct Esp32C3DefaultPeripherals<'a> {
    pub uart0: esp32::uart::Uart<'a>,
    pub timg0: esp32::timg::TimG<'a>,
    pub gpio: esp32::gpio::Port<'a>,
    pub rtc_cntl: esp32::rtc_cntl::RtcCntl,
}

impl<'a> Esp32C3DefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            uart0: esp32::uart::Uart::new(esp32::uart::UART0_BASE),
            timg0: esp32::timg::TimG::new(esp32::timg::TIMG0_BASE),
            gpio: esp32::gpio::Port::new(),
            rtc_cntl: esp32::rtc_cntl::RtcCntl::new(esp32::rtc_cntl::RTC_CNTL_BASE),
        }
    }
}

impl<'a> InterruptService<()> for Esp32C3DefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::IRQ_UART0 => {
                self.uart0.handle_interrupt();
            }
            interrupts::IRQ_GPIO | interrupts::IRQ_GPIO_NMI => {
                self.gpio.handle_interrupt();
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
            pmp: PMP::new(),
            intc: &INTC,
            pic_interrupt_service,
        }
    }

    pub fn map_pic_interrupts(&self) {
        self.intc.map_interrupts();
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

impl<'a, I: InterruptService<()> + 'a> Chip for Esp32C3<'a, I> {
    type MPU = PMP<8>;
    type UserspaceKernelBoundary = SysCall;

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
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
        let mcval: csr::mcause::Trap = core::convert::From::from(csr::CSR.mcause.extract());
        let _ = writer.write_fmt(format_args!("\r\n---| RISC-V Machine State |---\r\n"));
        let _ = writer.write_fmt(format_args!("Last cause (mcause): "));
        rv32i::print_mcause(mcval, writer);
        let interrupt = csr::CSR.mcause.read(csr::mcause::mcause::is_interrupt);
        let code = csr::CSR.mcause.read(csr::mcause::mcause::reason);
        let _ = writer.write_fmt(format_args!(
            " (interrupt={}, exception code={:#010X})",
            interrupt, code
        ));
        let _ = writer.write_fmt(format_args!(
            "\r\nLast value (mtval):  {:#010X}\
         \r\n\
         \r\nSystem register dump:\
         \r\n mepc:    {:#010X}    mstatus:     {:#010X}\
         \r\n mtvec:   {:#010X}",
            csr::CSR.mtval.get(),
            csr::CSR.mepc.get(),
            csr::CSR.mstatus.get(),
            csr::CSR.mtvec.get()
        ));
        let mstatus = csr::CSR.mstatus.extract();
        let uie = mstatus.is_set(csr::mstatus::mstatus::uie);
        let sie = mstatus.is_set(csr::mstatus::mstatus::sie);
        let mie = mstatus.is_set(csr::mstatus::mstatus::mie);
        let upie = mstatus.is_set(csr::mstatus::mstatus::upie);
        let spie = mstatus.is_set(csr::mstatus::mstatus::spie);
        let mpie = mstatus.is_set(csr::mstatus::mstatus::mpie);
        let spp = mstatus.is_set(csr::mstatus::mstatus::spp);
        let _ = writer.write_fmt(format_args!(
            "\r\n mstatus: {:#010X}\
         \r\n  uie:    {:5}  upie:   {}\
         \r\n  sie:    {:5}  spie:   {}\
         \r\n  mie:    {:5}  mpie:   {}\
         \r\n  spp:    {}",
            mstatus.get(),
            uie,
            upie,
            sie,
            spie,
            mie,
            mpie,
            spp
        ));
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

/// The ESP32C3 should support non-vectored and vectored interrupts, but
/// vectored interrupts seem more reliable so let's use that.
pub unsafe fn configure_trap_handler() {
    CSR.mtvec
        .write(mtvec::trap_addr.val(_start_trap_vectored as usize >> 2) + mtvec::mode::Vectored)
}

// Mock implementation for crate tests that does not include the section
// specifier, as the test will not use our linker script, and the host
// compilation environment may not allow the section name.
#[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
pub extern "C" fn _start_trap_vectored() {
    use core::hint::unreachable_unchecked;
    unsafe {
        unreachable_unchecked();
    }
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[link_section = ".riscv.trap_vectored"]
#[export_name = "_start_trap_vectored"]
#[naked]
pub extern "C" fn _start_trap_vectored() -> ! {
    unsafe {
        // Below are 32 (non-compressed) jumps to cover the entire possible
        // range of vectored traps.
        asm!(
            "
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
        ",
            options(noreturn)
        );
    }
}
