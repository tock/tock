// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::fmt::{Display, Write};
use core::marker::PhantomData;
use core::ptr::addr_of;
use kernel::platform::chip::{Chip, InterruptService};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use rv32i::csr::{mcause, mie::mie, mtvec::mtvec, CSR};
use rv32i::pmp::{PMPUserMPU, TORUserPMP};
use rv32i::syscall::SysCall;

use crate::chip_config::EarlGreyConfig;
use crate::interrupts;
use crate::pinmux_config::EarlGreyPinmuxConfig;
use crate::plic::Plic;
use crate::plic::PLIC;

pub struct EarlGrey<
    'a,
    const MPU_REGIONS: usize,
    I: InterruptService + 'a,
    CFG: EarlGreyConfig + 'static,
    PINMUX: EarlGreyPinmuxConfig,
    PMP: TORUserPMP<{ MPU_REGIONS }> + Display + 'static,
> {
    userspace_kernel_boundary: SysCall,
    pub mpu: PMPUserMPU<MPU_REGIONS, PMP>,
    plic: &'a Plic,
    timer: &'static crate::timer::RvTimer<'static, CFG>,
    pwrmgr: lowrisc::pwrmgr::PwrMgr,
    plic_interrupt_service: &'a I,
    _cfg: PhantomData<CFG>,
    _pinmux: PhantomData<PINMUX>,
}

pub struct EarlGreyDefaultPeripherals<'a, CFG: EarlGreyConfig, PINMUX: EarlGreyPinmuxConfig> {
    pub aes: crate::aes::Aes<'a>,
    pub hmac: lowrisc::hmac::Hmac<'a>,
    pub usb: lowrisc::usbdev::Usb<'a>,
    pub uart0: lowrisc::uart::Uart<'a>,
    pub otbn: lowrisc::otbn::Otbn<'a>,
    pub gpio_port: crate::gpio::Port<'a>,
    pub i2c0: lowrisc::i2c::I2c<'a>,
    pub spi_host0: lowrisc::spi_host::SpiHost<'a>,
    pub spi_host1: lowrisc::spi_host::SpiHost<'a>,
    pub flash_ctrl: lowrisc::flash_ctrl::FlashCtrl<'a>,
    pub rng: lowrisc::csrng::CsRng<'a>,
    pub watchdog: lowrisc::aon_timer::AonTimer,
    _cfg: PhantomData<CFG>,
    _pinmux: PhantomData<PINMUX>,
}

impl<CFG: EarlGreyConfig, PINMUX: EarlGreyPinmuxConfig>
    EarlGreyDefaultPeripherals<'_, CFG, PINMUX>
{
    pub fn new() -> Self {
        Self {
            aes: crate::aes::Aes::new(),
            hmac: lowrisc::hmac::Hmac::new(crate::hmac::HMAC0_BASE),
            usb: lowrisc::usbdev::Usb::new(crate::usbdev::USB0_BASE),
            uart0: lowrisc::uart::Uart::new(crate::uart::UART0_BASE, CFG::PERIPHERAL_FREQ),
            otbn: lowrisc::otbn::Otbn::new(crate::otbn::OTBN_BASE),
            gpio_port: crate::gpio::Port::new::<PINMUX>(),
            i2c0: lowrisc::i2c::I2c::new(crate::i2c::I2C0_BASE, (1 / CFG::CPU_FREQ) * 1000 * 1000),
            spi_host0: lowrisc::spi_host::SpiHost::new(
                crate::spi_host::SPIHOST0_BASE,
                CFG::CPU_FREQ,
            ),
            spi_host1: lowrisc::spi_host::SpiHost::new(
                crate::spi_host::SPIHOST1_BASE,
                CFG::CPU_FREQ,
            ),
            flash_ctrl: lowrisc::flash_ctrl::FlashCtrl::new(
                crate::flash_ctrl::FLASH_CTRL_BASE,
                lowrisc::flash_ctrl::FlashRegion::REGION0,
            ),

            rng: lowrisc::csrng::CsRng::new(crate::csrng::CSRNG_BASE),
            watchdog: lowrisc::aon_timer::AonTimer::new(
                crate::aon_timer::AON_TIMER_BASE,
                CFG::CPU_FREQ,
            ),
            _cfg: PhantomData,
            _pinmux: PhantomData,
        }
    }

    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.aes);
        kernel::deferred_call::DeferredCallClient::register(&self.uart0);
    }
}

impl<CFG: EarlGreyConfig, PINMUX: EarlGreyPinmuxConfig> InterruptService
    for EarlGreyDefaultPeripherals<'_, CFG, PINMUX>
{
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
            interrupts::CSRNG_CSCMDREQDONE..=interrupts::CSRNG_CSFATALERR => {
                self.rng.handle_interrupt()
            }
            interrupts::SPIHOST0_ERROR..=interrupts::SPIHOST0_SPIEVENT => {
                self.spi_host0.handle_interrupt()
            }
            interrupts::SPIHOST1_ERROR..=interrupts::SPIHOST1_SPIEVENT => {
                self.spi_host1.handle_interrupt()
            }
            interrupts::AON_TIMER_AON_WKUP_TIMER_EXPIRED
                ..=interrupts::AON_TIMER_AON_WDOG_TIMER_BARK => self.watchdog.handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<
        'a,
        const MPU_REGIONS: usize,
        I: InterruptService + 'a,
        CFG: EarlGreyConfig,
        PINMUX: EarlGreyPinmuxConfig,
        PMP: TORUserPMP<{ MPU_REGIONS }> + Display + 'static,
    > EarlGrey<'a, MPU_REGIONS, I, CFG, PINMUX, PMP>
{
    pub unsafe fn new(
        plic_interrupt_service: &'a I,
        timer: &'static crate::timer::RvTimer<CFG>,
        pmp: PMP,
    ) -> Self {
        Self {
            userspace_kernel_boundary: SysCall::new(),
            mpu: PMPUserMPU::new(pmp),
            plic: &*addr_of!(PLIC),
            pwrmgr: lowrisc::pwrmgr::PwrMgr::new(crate::pwrmgr::PWRMGR_BASE),
            timer,
            plic_interrupt_service,
            _cfg: PhantomData,
            _pinmux: PhantomData,
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
                        self.with_interrupts_disabled(|| {
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
                    self.with_interrupts_disabled(|| {
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

impl<
        'a,
        const MPU_REGIONS: usize,
        I: InterruptService + 'a,
        CFG: EarlGreyConfig,
        PINMUX: EarlGreyPinmuxConfig,
        PMP: TORUserPMP<{ MPU_REGIONS }> + Display + 'static,
    > kernel::platform::chip::Chip for EarlGrey<'a, MPU_REGIONS, I, CFG, PINMUX, PMP>
{
    type MPU = PMPUserMPU<MPU_REGIONS, PMP>;
    type UserspaceKernelBoundary = SysCall;
    type ThreadIdProvider = rv32i::thread_id::RiscvThreadIdProvider;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        loop {
            if self.plic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if self.plic.get_saved_interrupts().is_none() {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie.modify(mie::mext::SET + mie::mtimer::CLEAR);
        self.plic.enable_all();
    }

    fn has_pending_interrupts(&self) -> bool {
        self.plic.get_saved_interrupts().is_some()
    }

    fn sleep(&self) {
        unsafe {
            self.pwrmgr.enable_low_power();
            self.check_until_true_or_interrupt(|| self.pwrmgr.check_clock_propagation(), None);
            rv32i::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        let _ = writer.write_fmt(format_args!(
            "\r\n---| OpenTitan Earlgrey configuration for {} |---",
            CFG::NAME
        ));
        rv32i::print_riscv_state(writer);
        let _ = writer.write_fmt(format_args!("{}", self.mpu.pmp));
    }
}

fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall | mcause::Exception::SupervisorEnvCall => (),

        // Breakpoints occur from the tests running on hardware
        mcause::Exception::Breakpoint => loop {
            unsafe { rv32i::support::wfi() }
        },

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
                let interrupt = (*addr_of!(PLIC)).next_pending();

                match interrupt {
                    Some(irq) => {
                        // Safe as interrupts are disabled
                        (*addr_of!(PLIC)).save_interrupt(irq);
                    }
                    None => {
                        // Enable generic interrupts
                        CSR.mie.modify(mie::mext::SET);
                        break;
                    }
                }
            }
        }

        mcause::Interrupt::Unknown(_) => {
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
///
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
    // The common _start_trap handler uses mscratch to determine
    // whether we are executing kernel or process code. Set to `0` to
    // indicate we're in the kernel right now.
    CSR.mscratch.set(0);

    // The Ibex CPU does not support non-vectored trap entries.
    CSR.mtvec.write(
        mtvec::trap_addr.val(_earlgrey_start_trap_vectored as usize >> 2) + mtvec::mode::Vectored,
    );
}

// Mock implementation for crate tests that does not include the section
// specifier, as the test will not use our linker script, and the host
// compilation environment may not allow the section name.
#[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
pub extern "C" fn _earlgrey_start_trap_vectored() {
    use core::hint::unreachable_unchecked;
    unsafe {
        unreachable_unchecked();
    }
}

#[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
#[link_section = ".riscv.trap_vectored"]
#[unsafe(naked)]
pub extern "C" fn _earlgrey_start_trap_vectored() -> ! {
    use core::arch::naked_asm;
    // According to the Ibex user manual:
    // [NMI] has interrupt ID 31, i.e., it has the highest priority of all
    // interrupts and the core jumps to the trap-handler base address (in
    // mtvec) plus 0x7C to handle the NMI.
    //
    // Below are 32 (non-compressed) jumps to cover the entire possible
    // range of vectored traps.
    naked_asm!(
        "
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
