//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use core::hint::unreachable_unchecked;

use kernel;
use kernel::debug;
use rv32i::csr::{mcause, mie::mie, mip::mip, mstatus, mtvec::mtvec, CSR};
use rv32i::syscall::SysCall;

use crate::gpio;
use crate::interrupts;
use crate::plic;
use crate::timer;
use crate::uart;

pub const CHIP_FREQ: u32 = 50_000_000;

pub struct Ibex {
    userspace_kernel_boundary: SysCall,
}

impl Ibex {
    pub unsafe fn new() -> Ibex {
        Ibex {
            userspace_kernel_boundary: SysCall::new(),
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        plic::disable_all();
        plic::clear_all_pending();
        plic::enable_all();
    }
}

impl kernel::Chip for Ibex {
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn systick(&self) -> &Self::SysTick {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        let mut handled_plic = false;

        unsafe {
            loop {
                // Any pending timer interrupts handled first
                let timer_fired = timer::TIMER.service_interrupts();

                let mut plic_fired = false;
                if let Some(interrupt) = plic::next_pending() {
                    match interrupt {
                        interrupts::UART_TX_WATERMARK..=interrupts::UART_RX_PARITY_ERR => {
                            uart::UART0.handle_interrupt()
                        }
                        int_pin @ interrupts::GPIO_PIN0..=interrupts::GPIO_PIN31 => {
                            let pin = &gpio::PORT[(int_pin - interrupts::GPIO_PIN0) as usize];
                            pin.handle_interrupt();
                        }
                        _ => debug!("Pidx {}", interrupt),
                    }
                    // Mark that we are done with this interrupt and the hardware
                    // can clear it.
                    plic::complete(interrupt);
                    handled_plic = true;
                    plic_fired = true;
                }

                if !timer_fired && !plic_fired {
                    // All pending interrupts have been handled
                    break;
                }
            }
        }

        if handled_plic {
            // If any interrupts from the PLIC were handled, then external interrupts must be
            // reenabled on the CPU.
            CSR.mie.modify(mie::mext::SET);
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { timer::TIMER.is_pending() || plic::has_pending() }
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

    unsafe fn write_state(&self, writer: &mut dyn Write) {
        let mcval: mcause::Trap = core::convert::From::from(CSR.mcause.extract());
        let _ = writer.write_fmt(format_args!("\r\n---| Machine State |---\r\n"));
        let _ = writer.write_fmt(format_args!("Cause (mcause): "));
        match mcval {
            mcause::Trap::Interrupt(interrupt) => {
                match interrupt {
                    mcause::Interrupt::UserSoft           => {let _ = writer.write_fmt(format_args!("User software interrupt"));},
                    mcause::Interrupt::SupervisorSoft     => {let _ = writer.write_fmt(format_args!("Supervisor software interrupt"));},
                    mcause::Interrupt::MachineSoft        => {let _ = writer.write_fmt(format_args!("Machine software interrupt"));},
                    mcause::Interrupt::UserTimer          => {let _ = writer.write_fmt(format_args!("User timer interrupt"));},
                    mcause::Interrupt::SupervisorTimer    => {let _ = writer.write_fmt(format_args!("Supervisor timer interrupt"));},
                    mcause::Interrupt::MachineTimer       => {let _ = writer.write_fmt(format_args!("Machine timer interrupt"));},
                    mcause::Interrupt::UserExternal       => {let _ = writer.write_fmt(format_args!("User external interrupt"));},
                    mcause::Interrupt::SupervisorExternal => {let _ = writer.write_fmt(format_args!("Supervisor external interrupt"));},
                    mcause::Interrupt::MachineExternal    => {let _ = writer.write_fmt(format_args!("Machine external interrupt"));},
                    mcause::Interrupt::Unknown            => {let _ = writer.write_fmt(format_args!("Reserved/Unknown"));},
                }
            },
            mcause::Trap::Exception(exception) => {
                match exception {
                    mcause::Exception::InstructionMisaligned => {let _ = writer.write_fmt(format_args!("Instruction access misaligned"));},
                    mcause::Exception::InstructionFault      => {let _ = writer.write_fmt(format_args!("Instruction access fault"));},
                    mcause::Exception::IllegalInstruction    => {let _ = writer.write_fmt(format_args!("Illegal instruction"));},
                    mcause::Exception::Breakpoint            => {let _ = writer.write_fmt(format_args!("Breakpoint"));},
                    mcause::Exception::LoadMisaligned        => {let _ = writer.write_fmt(format_args!("Load address misaligned"));},
                    mcause::Exception::LoadFault             => {let _ = writer.write_fmt(format_args!("Load access fault"));},
                    mcause::Exception::StoreMisaligned       => {let _ = writer.write_fmt(format_args!("Store/AMO address misaligned"));},
                    mcause::Exception::StoreFault            => {let _ = writer.write_fmt(format_args!("Store/AMO access fault"));},
                    mcause::Exception::UserEnvCall           => {let _ = writer.write_fmt(format_args!("Environment call from U-mode"));},
                    mcause::Exception::SupervisorEnvCall     => {let _ = writer.write_fmt(format_args!("Environment call from S-mode"));},
                    mcause::Exception::MachineEnvCall        => {let _ = writer.write_fmt(format_args!("Environment call from M-mode"));},
                    mcause::Exception::InstructionPageFault  => {let _ = writer.write_fmt(format_args!("Instruction page fault"));},
                    mcause::Exception::LoadPageFault         => {let _ = writer.write_fmt(format_args!("Load page fault"));},
                    mcause::Exception::StorePageFault        => {let _ = writer.write_fmt(format_args!("Store/AMO page fault"));},
                    mcause::Exception::Unknown               => {let _ = writer.write_fmt(format_args!("Reserved"));},
                }
            },
        }
        let mval = CSR.mcause.get();
        let interrupt = (mval & 0x80000000) == 0x80000000;
        let code = mval & 0x7fffffff;
        let _ = writer.write_fmt(format_args!(" (interrupt={}, exception code={})", interrupt, code));
        let _ = writer.write_fmt(format_args!(
            "\r\nValue (mtval):  {:#010X}\
             \r\n\
             \r\nSystem register dump:\
             \r\n mepc:    {:#010X}    mstatus:     {:#010X}\
             \r\n mcycle:  {:#010X}    minstret:    {:#010X}\
             \r\n mtvec:   {:#010X}",
            CSR.mtval.get(),
            CSR.mepc.get(), CSR.mstatus.get(),
            CSR.mcycle.get(),  CSR.minstret.get(),
            CSR.mtvec.get()));
        let mstatus = CSR.mstatus.extract();
        let uie = mstatus.is_set(mstatus::mstatus::uie);
        let sie = mstatus.is_set(mstatus::mstatus::sie);
        let mie = mstatus.is_set(mstatus::mstatus::mie);
        let upie = mstatus.is_set(mstatus::mstatus::upie);
        let spie = mstatus.is_set(mstatus::mstatus::spie);
        let mpie = mstatus.is_set(mstatus::mstatus::mpie);
        let spp = mstatus.is_set(mstatus::mstatus::spp);
        let _ = writer.write_fmt(format_args!(
            "\r\n mstatus:\
             \r\n  uie:    {}\
             \r\n  sie:    {}\
             \r\n  mie:    {}\
             \r\n  upie:   {}\
             \r\n  spie:   {}\
             \r\n  mpie:   {}\
             \r\n  spp:    {}",
            uie, sie, mie, upie, spie, mpie, spp));
        let e_usoft  = CSR.mie.is_set(mie::usoft);
        let e_ssoft  = CSR.mie.is_set(mie::ssoft);
        let e_msoft  = CSR.mie.is_set(mie::msoft);
        let e_utimer = CSR.mie.is_set(mie::utimer);
        let e_stimer = CSR.mie.is_set(mie::stimer);
        let e_mtimer = CSR.mie.is_set(mie::mtimer);
        let e_uext   = CSR.mie.is_set(mie::uext);
        let e_sext   = CSR.mie.is_set(mie::sext);
        let e_mext   = CSR.mie.is_set(mie::mext);

        let p_usoft  = CSR.mip.is_set(mip::usoft);
        let p_ssoft  = CSR.mip.is_set(mip::ssoft);
        let p_msoft  = CSR.mip.is_set(mip::msoft);
        let p_utimer = CSR.mip.is_set(mip::utimer);
        let p_stimer = CSR.mip.is_set(mip::stimer);
        let p_mtimer = CSR.mip.is_set(mip::mtimer);
        let p_uext   = CSR.mip.is_set(mip::uext);
        let p_sext   = CSR.mip.is_set(mip::sext);
        let p_mext   = CSR.mip.is_set(mip::mext);
        let _ = writer.write_fmt(format_args!(
            "\r\n mie:     {:#010X}    mip:      {:#010X}\
             \r\n  usoft:  {:6}                  {:6}\
             \r\n  ssoft:  {:6}                  {:6}\
             \r\n  msoft:  {:6}                  {:6}\
             \r\n  utimer: {:6}                  {:6}\
             \r\n  stimer: {:6}                  {:6}\
             \r\n  mtimer: {:6}                  {:6}\
             \r\n  uext:   {:6}                  {:6}\
             \r\n  sext:   {:6}                  {:6}\
             \r\n  mext:   {:6}                  {:6}\r\n",
            CSR.mie.get(),     CSR.mip.get(),
            e_usoft, p_usoft,
            e_ssoft, p_ssoft,
            e_msoft, p_msoft,
            e_utimer, p_utimer,
            e_stimer, p_stimer,
            e_mtimer, p_mtimer,
            e_uext, p_uext,
            e_sext, p_sext,
            e_mext, p_mext));
    }
}



fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall |
        mcause::Exception::SupervisorEnvCall => (),

        mcause::Exception::InstructionMisaligned |
        mcause::Exception::InstructionFault  |
        mcause::Exception::IllegalInstruction |
        mcause::Exception::Breakpoint |
        mcause::Exception::LoadMisaligned |
        mcause::Exception::LoadFault |
        mcause::Exception::StoreMisaligned |
        mcause::Exception::StoreFault |
        mcause::Exception::MachineEnvCall |
        mcause::Exception::InstructionPageFault |
        mcause::Exception::LoadPageFault |
        mcause::Exception::StorePageFault |
        mcause::Exception::Unknown => {
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
            timer::TIMER.handle_isr();
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

pub unsafe fn configure_trap_handler() {
    // The Ibex CPU does not support non-vectored trap entries.
    CSR.mtvec
        .write(mtvec::trap_addr.val(_start_trap_vectored as u32 >> 2) + mtvec::mode::Vectored)
}

#[link_section = ".riscv.trap_vectored"]
#[export_name = "_start_trap_vectored"]
#[naked]
pub extern "C" fn _start_trap_vectored() -> ! {
    unsafe {
        // According to the Ibex user manual:
        // [NMI] has interrupt ID 31, i.e., it has the highest priority of all
        // interrupts and the core jumps to the trap-handler base address (in
        // mtvec) plus 0x7C to handle the NMI.
        //
        // Below are 32 (non-compressed) jumps to cover the entire possible
        // range of vectored traps.
        #[cfg(all(target_arch = "riscv32", target_os = "none"))]
        asm!("
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
            j _start_trap
        "
        :
        :
        :
        : "volatile");
        unreachable_unchecked()
    }
}
