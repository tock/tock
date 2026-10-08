// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use core::ptr::addr_of;

use kernel::platform::chip::{Chip, InterruptService};
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};

use rv32i::csr::{self, CSR, mcause, mtvec::mtvec};
use rv32i::pmp::{PMPUserMPU, simple::SimplePMP};
use rv32i::syscall::SysCall;

use crate::intc::{Intc, IntcRegisters};
use crate::interrupts;
use crate::rng;
use crate::sysreg;
use crate::timg;

pub const INTC_BASE: StaticRef<IntcRegisters> =
    unsafe { StaticRef::new(0x600C_2000 as *const IntcRegisters) };

pub static mut INTC: Intc = Intc::new(INTC_BASE);

/// Number of PMP entries available for the "User MPU" implementation.
///
/// Tock uses TOR regions, so each region actually occupies two of these
/// entries; see [`USER_PMP_ENTRIES`].
///
/// The ESP32-C3 chip actually has 16 entries, but it's got a bug: when taking a
/// trap from user-mode, it faults. It seems like this fault only occurs if the
/// trap vector table is not accessible to user-mode in the PMP. This behavior
/// is not compliant with the RISC-V PMP spec. We configure the bottom 2 entries
/// of the PMP to grant execute-only access to user-mode, which fixes this
/// issue.
///
/// This does not provide execute permissions for the rest of the kernel text,
/// and the trap vector only contains jumps to the trap handler.
const USER_PMP_ENTRIES: usize = 14;

/// Process MPU regions, two (TOR) PMP entries each.
///
/// See [`USER_MPU_REGIONS`].
const USER_MPU_REGIONS: usize = USER_PMP_ENTRIES / 2;

/// Size of the vector table at `mtvec`.
///
/// See [`_start_trap_vectored`]. We set up one jump per trap, 32 different
/// traps in total, all non-compressed RISC-V instructions that are 4 byte long.
///
/// We use this value to work around a CPU bug in the ESP32-C3, to allow apps to
/// execute the trap vector. We don't give execute permissions to the rest of
/// kernel text, and the trap vector itself just contains jumps to a single
/// unified handler, so this should be OK.
const TRAP_VECTOR_TABLE_LEN: usize = 32 * 4;

pub struct Esp32C3<'a, I: InterruptService + 'a> {
    userspace_kernel_boundary: SysCall,
    pub pmp: PMPUserMPU<USER_MPU_REGIONS, SimplePMP<USER_PMP_ENTRIES>>,
    intc: &'a Intc,
    pic_interrupt_service: &'a I,
}

pub struct Esp32C3DefaultPeripherals<'a> {
    pub uart0: esp32::uart::Uart<'a>,
    pub timg0: timg::TimG<'a>,
    pub timg1: timg::TimG<'a>,
    pub gpio: esp32::gpio::Port<'a>,
    pub rtc_cntl: esp32::rtc_cntl::RtcCntl,
    pub sysreg: sysreg::SysReg,
    pub rng: rng::Rng<'a>,
}

impl Esp32C3DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            uart0: esp32::uart::Uart::new(esp32::uart::UART0_BASE),
            timg0: timg::TimG::new(timg::TIMG0_BASE, timg::ClockSource::Pll),
            timg1: timg::TimG::new(timg::TIMG1_BASE, timg::ClockSource::Pll),
            gpio: esp32::gpio::Port::new(),
            rtc_cntl: esp32::rtc_cntl::RtcCntl::new(esp32::rtc_cntl::RTC_CNTL_BASE),
            sysreg: sysreg::SysReg::new(),
            rng: rng::Rng::new(),
        }
    }

    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.rng);
    }
}

impl InterruptService for Esp32C3DefaultPeripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::IRQ_UART0 => self.uart0.handle_interrupt(),

            interrupts::IRQ_TIMER1 => self.timg0.handle_interrupt(),
            interrupts::IRQ_TIMER2 => self.timg1.handle_interrupt(),

            interrupts::IRQ_GPIO | interrupts::IRQ_GPIO_NMI => self.gpio.handle_interrupt(),

            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> Esp32C3<'a, I> {
    pub unsafe fn new(pic_interrupt_service: &'a I) -> Self {
        let pmp = SimplePMP::new().unwrap();

        // The ESP32-C3 seems to have a CPU bug where it requires the PMP to be
        // configured to give user-mode execute permissions on the trap vector;
        // otherwise it just faults. See the comment on [`USER_PMP_ENTRIES`].
        // This dedicates the last two regions (which aren't accessible through
        // `SimplePMP`) to that purpose.
        let mtvec_addr = _start_trap_vectored as extern "C" fn() -> ! as usize;
        let (bottom, top) = (USER_PMP_ENTRIES, USER_PMP_ENTRIES + 1);
        CSR.pmpaddr_set(bottom, mtvec_addr >> 2);
        CSR.pmpaddr_set(top, (mtvec_addr + TRAP_VECTOR_TABLE_LEN) >> 2);

        // Configure the last two entries' pmpcfg. Second to last will remain
        // off, as the start address of a TOR region.
        let pmpcfg = CSR.pmpconfig_get(3) & 0x0000_ffff;
        // pmpcfg[4:3] = 0b01 -> TOR
        // pmpcfg[2] = 1 -> execute
        const TOR_EXECUTE: usize = 0b01 << 3 | 1 << 2;
        CSR.pmpconfig_set(3, pmpcfg | TOR_EXECUTE << 24);

        Self {
            userspace_kernel_boundary: SysCall::new(),
            pmp: PMPUserMPU::new(pmp),
            intc: &*addr_of!(INTC),
            pic_interrupt_service,
        }
    }

    pub fn map_pic_interrupts(&self) {
        // As per the ESP32-C3 reference manual, performing any operations on
        // the interrupt controller register may cause it to go into a
        // "transient" state where its effects on the CPU's MIP interrupt line
        // become unpredictable. It is advised that we disable CPU traps on the
        // interrupt line being asserted by setting MIE = 0, and run a fence
        // instruction after every `intc` register write (which we do in the
        // Intc driver).
        self.with_interrupts_disabled(|| self.intc.map_interrupts());
    }

    pub unsafe fn enable_pic_interrupts(&self) {
        // As per the ESP32-C3 reference manual, performing any operations on
        // the interrupt controller register may cause it to go into a
        // "transient" state where its effects on the CPU's MIP interrupt line
        // become unpredictable. It is advised that we disable CPU traps on the
        // interrupt line being asserted by setting MIE = 0, and run a fence
        // instruction after every `intc` register write (which we do in the
        // Intc driver).
        self.with_interrupts_disabled(|| self.intc.enable_all());
    }
}

impl<'a, I: InterruptService + 'a> Chip for Esp32C3<'a, I> {
    type MPU = PMPUserMPU<USER_MPU_REGIONS, SimplePMP<USER_PMP_ENTRIES>>;
    type UserspaceKernelBoundary = SysCall;
    type ThreadIdProvider = rv32i::thread_id::RiscvThreadIdProvider;

    fn init() {}

    // # Safety (TODO: this is not an unsafe function!)
    //
    // This function will unconditionally enable interrupts by setting
    // `mstatus::MIE`, and as such it must not be run in a
    // `with_interrupts_disabled` closure body.
    fn service_pending_interrupts(&self) {
        // As per the ESP32-C3 reference manual, performing any operations on
        // the interrupt controller register may cause it to go into a
        // "transient" state where its effects on the CPU's MIP interrupt line
        // become unpredictable. It is advised that we disable CPU traps on the
        // interrupt line being asserted by setting MIE = 0, and run a fence
        // instruction after every `intc` register write (which we do in the
        // Intc driver), before re-enabling interrupts (which we do at the
        // bottom of this function). Even though [`start_trap_from_rust`] and
        // [`disable_interrupt_trap_handler`] both disable `MIE`, we must still
        // do this because we can enter this function multiple times without
        // receiving an interrupt and enable `MIE` below.
        CSR.mstatus.modify(csr::mstatus::mstatus::mie::CLEAR);

        while let Some(pending_interrupt) = self.intc.next_pending() {
            // Clear the pending interrupt flag for this IRQ, to catch any new
            // edge-triggered ones being raised while handling this one. This is
            // a no-op for level-triggered interrupts, which need to be cleared
            // by the implementation of [`I::service_interrupt`].
            self.intc.clear_interrupt(pending_interrupt);

            // Run the handler for this particular IRQ source:
            if !self
                .pic_interrupt_service
                .service_interrupt(pending_interrupt)
            {
                panic!("Unhandled interrupt at IRQ {pending_interrupt}");
            }
        }

        // [`start_trap_rust`] and [`disable_interrupt_trap_handler`] both clear
        // `MIE` when receiving an interrupt. Now that we've handled all of
        // them, we re-enable CPU interrupts, which ensures that the kernel gets
        // woken from, e.g., `wfi` on a new pending interrupt.
        //
        // SAFETY (TODO: this is not an unsafe function)
        //
        // This operation re-enables interrupts. Callers must ensure that this
        // function only runs in contexts where that is a legal operation (not
        // in an atomic / `with_interrupts_disabled` context). An interrupt, in
        // turn, will simply clear this flag (we rely only on the side-effect of
        // waking the CPU and returning to the kernel).
        CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        self.intc.next_pending().is_some()
    }

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            rv32i::support::wfi();
        }
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, writer: &mut dyn Write) {
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

/// Trap handler for board/chip specific code.
///
/// This gets called when an interrupt occurs while the chip is
/// in kernel mode.
#[export_name = "_start_trap_rust_from_kernel"]
pub unsafe extern "C" fn start_trap_rust() {
    match mcause::Trap::from(CSR.mcause.extract()) {
        mcause::Trap::Interrupt(_) => {
            // We don't handle interrupts synchronously here, and instead just
            // clear the `MPIE` flag, which upon trap return will clear the
            // `MIE` flag, allowing the kernel to continue executing.
            //
            // We were already in kernel mode, and the kernel's loop will
            // eventually get to calling [`Chip::service_pending_interrupts`],
            // which will then actually service the interrupt that caused this
            // code to run (and potentially others that have been raised in the
            // meantime), and raising more traps due to interrupts along the way
            // isn't useful work. [`Chip::service_pending_interrupts`] will
            // re-enable `MIE`, to make us aware of the next interrupt.
            //
            // Deferring interrupts here has another useful benefit: if we were
            // to mask the actual `intc` interrupt line here instead, it's
            // possible for an interrupt to be triggered _after_ the last kernel
            // loop interrupt check before switching to a process. The process
            // can then run until its timeslice expires, because `MIP` (machine
            // interrupt pending) would be de-asserted (possibly delaying the
            // interrupt for a long while). Clearing `MIE` instead has the
            // benefit of `switch_to_process` re-enabling MIE, which causes an
            // immediate return to the kernel.
            CSR.mstatus.modify(csr::mstatus::mstatus::mpie::CLEAR);
        }
        mcause::Trap::Exception(exception) => {
            handle_exception(exception);
        }
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
#[export_name = "_disable_interrupt_trap_rust_from_app"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: u32) {
    match mcause::Trap::from(mcause_val as usize) {
        mcause::Trap::Interrupt(_) => {
            // We don't handle interrupts synchronously here, and instead just
            // clear the `MPIE` flag (which will clear `MIE` upon trap return).
            // This handler we're running now is specific to switches coming
            // from an application context, and we only rely on the side-effect
            // of it causing a switch to machine mode. The kernel loop will
            // eventually handle these pending interrupts through
            // [`Chip::service_pending_interrupts`], which will re-enable `MIE`.
            CSR.mstatus.modify(csr::mstatus::mstatus::mpie::CLEAR);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}

/// The ESP32C3 should support non-vectored and vectored interrupts, but
/// vectored interrupts seem more reliable so let's use that.
pub unsafe fn configure_trap_handler() {
    CSR.mtvec.write(
        mtvec::trap_addr.val(_start_trap_vectored as extern "C" fn() -> ! as usize >> 2)
            + mtvec::mode::Vectored,
    )
}

// Mock implementation for crate tests that does not include the section
// specifier, as the test will not use our linker script, and the host
// compilation environment may not allow the section name.
#[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
pub extern "C" fn _start_trap_vectored() -> ! {
    use core::hint::unreachable_unchecked;
    unsafe {
        unreachable_unchecked();
    }
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
// Only apply the `link_section` attribute when actually targeting bare-metal
// RISC-V (`target_os = "none"`). Non-bare-metal object formats (Mach-O, PE,
// ...) reject a bare section name like this, yielding errors such as:
// `mach-o section specifier requires a segment and section separated by a
// comma`.
#[cfg_attr(
    all(target_arch = "riscv32", target_os = "none"),
    link_section = ".riscv.trap_vectored"
)]
#[unsafe(naked)]
pub extern "C" fn _start_trap_vectored() -> ! {
    use core::arch::naked_asm;
    // Below are 32 (non-compressed) jumps to cover the entire possible
    // range of vectored traps.
    naked_asm!(
        "
      .option push
      .option norvc
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
        j {start_trap}
      .option pop
        ",
        start_trap = sym rv32i::_start_trap,
    );
}

/// Array used to track the "trap handler active" state per hart.
///
/// The `riscv` crate requires chip crates to allocate an array to
/// track whether any given hart is currently in a trap handler. The
/// array must be zero-initialized.
#[export_name = "_trap_handler_active"]
static mut TRAP_HANDLER_ACTIVE: [usize; 1] = [0; 1];
