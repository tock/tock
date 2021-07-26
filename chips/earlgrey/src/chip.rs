//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use kernel;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::platform::chip::{Chip, InterruptService};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use rv32i::csr::{mcause, mie::mie, mip::mip, mtvec::mtvec, CSR};
use rv32i::epmp::PMP;
use rv32i::syscall::SysCall;

use crate::chip_config::CONFIG;
use crate::interrupts;
use crate::plic::Plic;
use crate::plic::PLIC;

pub struct EarlGrey<'a, I: InterruptService<()> + 'a> {
    userspace_kernel_boundary: SysCall,
    pub pmp: PMP<8>,
    plic: &'a Plic,
    timer: &'static crate::timer::RvTimer<'static>,
    pwrmgr: lowrisc::pwrmgr::PwrMgr,
    plic_interrupt_service: &'a I,
}

pub struct EarlGreyDefaultPeripherals<'a> {
    pub aes: crate::aes::Aes<'a>,
    pub hmac: lowrisc::hmac::Hmac<'a>,
    pub usb: lowrisc::usbdev::Usb<'a>,
    pub uart0: lowrisc::uart::Uart<'a>,
    pub otbn: lowrisc::otbn::Otbn<'a>,
    pub gpio_port: crate::gpio::Port<'a>,
    pub i2c0: lowrisc::i2c::I2c<'a>,
    pub flash_ctrl: lowrisc::flash_ctrl::FlashCtrl<'a>,
}

impl<'a> EarlGreyDefaultPeripherals<'a> {
    pub fn new(deferred_caller: &'static DynamicDeferredCall) -> Self {
        Self {
            aes: crate::aes::Aes::new(deferred_caller),
            hmac: lowrisc::hmac::Hmac::new(crate::hmac::HMAC0_BASE),
            usb: lowrisc::usbdev::Usb::new(crate::usbdev::USB0_BASE),
            uart0: lowrisc::uart::Uart::new(crate::uart::UART0_BASE, CONFIG.peripheral_freq),
            otbn: lowrisc::otbn::Otbn::new(crate::otbn::OTBN_BASE, deferred_caller),
            gpio_port: crate::gpio::Port::new(),
            i2c0: lowrisc::i2c::I2c::new(
                crate::i2c::I2C0_BASE,
                (1 / CONFIG.cpu_freq) * 1000 * 1000,
            ),
            flash_ctrl: lowrisc::flash_ctrl::FlashCtrl::new(
                crate::flash_ctrl::FLASH_CTRL_BASE,
                lowrisc::flash_ctrl::FlashRegion::REGION0,
            ),
        }
    }
}

impl<'a> InterruptService<()> for EarlGreyDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART0_TX_WATERMARK..=interrupts::UART0_RX_PARITYERR => {
                self.uart0.handle_interrupt();
            }
            int_pin @ interrupts::GPIO_PIN0..=interrupts::GPIO_PIN31 => {
                let pin = &self.gpio_port[(int_pin - interrupts::GPIO_PIN0) as usize];
                pin.handle_interrupt();
            }
            interrupts::HMAC_HMACDONE..=interrupts::HMAC_HMACERR => {
                self.hmac.handle_interrupt();
            }
            interrupts::USBDEV_PKTRECEIVED..=interrupts::USBDEV_LINKOUTERR => {
                self.usb.handle_interrupt();
            }
            interrupts::FLASHCTRL_PROGEMPTY..=interrupts::FLASHCTRL_OPDONE => {
                self.flash_ctrl.handle_interrupt()
            }
            interrupts::I2C0_FMTWATERMARK..=interrupts::I2C0_HOSTTIMEOUT => {
                self.i2c0.handle_interrupt()
            }
            interrupts::OTBN_DONE => self.otbn.handle_interrupt(),
            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, I: InterruptService<()> + 'a> EarlGrey<'a, I> {
    pub unsafe fn new(
        plic_interrupt_service: &'a I,
        timer: &'static crate::timer::RvTimer,
    ) -> Self {
        Self {
            userspace_kernel_boundary: SysCall::new(),
            pmp: PMP::new(),
            plic: &PLIC,
            pwrmgr: lowrisc::pwrmgr::PwrMgr::new(crate::pwrmgr::PWRMGR_BASE),
            timer,
            plic_interrupt_service,
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        self.plic.disable_all();
        self.plic.enable_all();
    }

    unsafe fn handle_plic_interrupts(&self) {
        while let Some(interrupt) = self.plic.get_saved_interrupts() {
            match interrupt {
                interrupts::PWRMGRAONWAKEUP => {
                    self.pwrmgr.handle_interrupt();
                    self.check_until_true_or_interrupt(
                        || self.pwrmgr.check_clock_propagation(),
                        None,
                    );
                }
                interrupts::RVTIMERTIMEREXPIRED0_0 => self.timer.service_interrupt(),
                _ => {
                    if interrupt >= interrupts::HMAC_HMACDONE
                        && interrupt <= interrupts::HMAC_HMACERR
                    {
                        // Claim the interrupt before we handle it.
                        // Currently the interrupt has been claimed but not completed.
                        // This means that if the interrupt re-asserts we will loose the
                        // re-assertion. Generally this isn't a problem, but some of the
                        // interrupt handlers expect that interrupts could occur.
                        // For example the HMAC interrupt handler will write data to the
                        // HMAC buffer. We then rely on an interrupt triggering when that
                        // buffer becomes empty. This can happen while we are still in the
                        // interrupt handler. To ensure we don't loose the interrupt we
                        // claim it here.
                        // In order to stop an interrupt loop, we first disable the
                        // interrupt. `service_pending_interrupts()` will re-enable
                        // interrupts once they are all handled.
                        self.atomic(|| {
                            // Safe as interrupts are disabled
                            self.plic.disable(interrupt);
                            self.plic.complete(interrupt);
                        });
                    }
                    if !self.plic_interrupt_service.service_interrupt(interrupt) {
                        panic!("Unknown interrupt: {}", interrupt);
                    }
                }
            }

            match interrupt {
                interrupts::HMAC_HMACDONE..=interrupts::HMAC_HMACERR => {}
                _ => {
                    self.atomic(|| {
                        self.plic.complete(interrupt);
                    });
                }
            }
        }
    }

    /// Run a function in an interruptable loop.
    ///
    /// The function will run until it returns true, an interrupt occurs or if
    /// `max_tries` is not `None` and that limit is reached.
    /// If the function returns true this call will also return true. If an
    /// interrupt occurs or `max_tries` is reached this call will return false.
    fn check_until_true_or_interrupt<F>(&self, f: F, max_tries: Option<usize>) -> bool
    where
        F: Fn() -> bool,
    {
        match max_tries {
            Some(t) => {
                for _i in 0..t {
                    if self.has_pending_interrupts() {
                        return false;
                    }
                    if f() {
                        return true;
                    }
                }
            }
            None => {
                while !self.has_pending_interrupts() {
                    if f() {
                        return true;
                    }
                }
            }
        }

        false
    }
}

impl<'a, I: InterruptService<()> + 'a> kernel::platform::chip::Chip for EarlGrey<'a, I> {
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
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.service_interrupt();
            }
            if self.plic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if !mip.matches_any(mip::mtimer::SET) && self.plic.get_saved_interrupts().is_none() {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie.modify(mie::mext::SET + mie::mtimer::SET);
        self.plic.enable_all();
    }

    fn has_pending_interrupts(&self) -> bool {
        let mip = CSR.mip.extract();
        self.plic.get_saved_interrupts().is_some() || mip.matches_any(mip::mtimer::SET)
    }

    fn sleep(&self) {
        unsafe {
            self.pwrmgr.enable_low_power();
            self.check_until_true_or_interrupt(|| self.pwrmgr.check_clock_propagation(), None);
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
            "\r\n---| EarlGrey configuration for {} |---",
            CONFIG.name
        ));
        rv32i::print_riscv_state(writer);
        let _ = writer.write_fmt(format_args!("{}", self.pmp));
    }
}

fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall | mcause::Exception::SupervisorEnvCall => (),

        // Breakpoints occur from the tests running on hardware
        mcause::Exception::Breakpoint => loop {},

        mcause::Exception::InstructionMisaligned
        | mcause::Exception::InstructionFault
        | mcause::Exception::IllegalInstruction
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
            // NOTE: The interrupt is no longer pending in the PLIC
            loop {
                let interrupt = PLIC.next_pending();

                match interrupt {
                    Some(irq) => {
                        // Safe as interrupts are disabled
                        PLIC.save_interrupt(irq);
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
/// For the Ibex this gets called when an interrupt occurs while the chip is
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

pub unsafe fn configure_trap_handler() {
    // The Ibex CPU does not support non-vectored trap entries.
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
        // According to the Ibex user manual:
        // [NMI] has interrupt ID 31, i.e., it has the highest priority of all
        // interrupts and the core jumps to the trap-handler base address (in
        // mtvec) plus 0x7C to handle the NMI.
        //
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
            j _start_trap
        ",
            options(noreturn)
        );
    }
}
