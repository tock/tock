use core::fmt::Write;
use kernel;
use kernel::debug;
use riscv;

use crate::gpio;
use crate::interrupts;
use crate::timer;
use crate::uart;

extern "C" {
    fn _start_trap();
}

pub struct ArtyExx {
    pmp: riscv::pmp::PMPConfig,
    userspace_kernel_boundary: riscv::syscall::SysCall,
    clic: riscv::clic::Clic,
}

impl ArtyExx {
    pub unsafe fn new() -> ArtyExx {
        // Make a bit-vector of all interrupt locations that we actually intend
        // to use on this chip.
        // 0001 1111 1111 1111 1111 0000 0000 1000 0000
        let in_use_interrupts: u64 = 0x1FFFF0080;

        ArtyExx {
            pmp: riscv::pmp::PMPConfig::new(4),
            userspace_kernel_boundary: riscv::syscall::SysCall::new(),
            clic: riscv::clic::Clic::new(in_use_interrupts),
        }
    }

    pub fn enable_all_interrupts(&self) {
        self.clic.enable_all();
    }

    /// By default the machine timer is enabled and will trigger interrupts. To
    /// prevent that we can make the compare register very large to effectively
    /// stop the interrupt from triggering, and then the machine timer can be
    /// used later as needed.
    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    pub unsafe fn disable_machine_timer(&self) {
        llvm_asm!("
            // Initialize machine timer mtimecmp to disable the machine timer
            // interrupt.
            li   t0, -1       // Set mtimecmp to 0xFFFFFFFF
            lui  t1, %hi(0x02004000)     // Load the address of mtimecmp to t1
            addi t1, t1, %lo(0x02004000) // Load the address of mtimecmp to t1
            sw   t0, 0(t1)    // mtimecmp is 64 bits, set to all ones
            sw   t0, 4(t1)    // mtimecmp is 64 bits, set to all ones
        "
        :
        :
        :
        : "volatile");
    }

    // Mock implementation for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub unsafe fn disable_machine_timer(&self) {
        unimplemented!()
    }

    /// Setup the function that should run when a trap happens.
    ///
    /// This needs to be chip specific because how the CLIC works is configured
    /// when the trap handler address is specified in mtvec, and that is only
    /// valid for platforms with a CLIC.
    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    pub unsafe fn configure_trap_handler(&self) {
        llvm_asm!("
            // The csrw instruction writes a Control and Status Register (CSR)
            // with a new value.
            //
            // CSR 0x305 (mtvec, 'Machine trap-handler base address.') sets the
            // address of the trap handler. We do not care about its old value,
            // so we don't bother reading it. We want to enable direct CLIC mode
            // so we set the second lowest bit.
            lui  t0, %hi(_start_trap)
            addi t0, t0, %lo(_start_trap)
            ori  t0, t0, 0x02 // Set CLIC direct mode
            csrw 0x305, t0    // Write the mtvec CSR.
        "
        :
        :
        :
        : "volatile");
    }

    // Mock implementation for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub unsafe fn configure_trap_handler(&self) {
        unimplemented!()
    }

    /// Generic helper initialize function to setup all of the chip specific
    /// operations. Different boards can call the functions that `initialize()`
    /// calls directly if it needs to use a custom setup operation.
    pub unsafe fn initialize(&self) {
        self.disable_machine_timer();
        self.configure_trap_handler();
    }
}

impl kernel::Chip for ArtyExx {
    type MPU = riscv::pmp::PMPConfig;
    type UserspaceKernelBoundary = riscv::syscall::SysCall;
    type SchedulerTimer = ();
    type WatchDog = ();

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &()
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &riscv::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = self.clic.next_pending() {
                match interrupt {
                    interrupts::MTIP => timer::MACHINETIMER.handle_interrupt(),

                    interrupts::GPIO0 => gpio::PORT[0].handle_interrupt(),
                    interrupts::GPIO1 => gpio::PORT[1].handle_interrupt(),
                    interrupts::GPIO2 => gpio::PORT[2].handle_interrupt(),
                    interrupts::GPIO3 => gpio::PORT[3].handle_interrupt(),
                    interrupts::GPIO4 => gpio::PORT[4].handle_interrupt(),
                    interrupts::GPIO5 => gpio::PORT[5].handle_interrupt(),
                    interrupts::GPIO6 => gpio::PORT[6].handle_interrupt(),
                    interrupts::GPIO7 => gpio::PORT[7].handle_interrupt(),
                    interrupts::GPIO8 => gpio::PORT[8].handle_interrupt(),
                    interrupts::GPIO9 => gpio::PORT[9].handle_interrupt(),
                    interrupts::GPIO10 => gpio::PORT[10].handle_interrupt(),
                    interrupts::GPIO11 => gpio::PORT[11].handle_interrupt(),
                    interrupts::GPIO12 => gpio::PORT[12].handle_interrupt(),
                    interrupts::GPIO13 => gpio::PORT[13].handle_interrupt(),
                    interrupts::GPIO14 => gpio::PORT[14].handle_interrupt(),
                    interrupts::GPIO15 => gpio::PORT[15].handle_interrupt(),

                    interrupts::UART0 => uart::UART0.handle_interrupt(),

                    _ => debug!("Pidx {}", interrupt),
                }

                // Mark that we are done with this interrupt and the hardware
                // can clear it.
                self.clic.complete(interrupt);
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        self.clic.has_pending()
    }

    fn sleep(&self) {
        unsafe {
            riscv::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        riscv::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        riscv::print_riscv_state(write);
    }
}

/// Trap handler for board/chip specific code.
///
/// For the arty-e21 this gets called when an interrupt occurs while the chip is
/// in kernel mode. All we need to do is check which interrupt occurred and
/// disable it.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[export_name = "_start_trap_rust"]
pub extern "C" fn start_trap_rust() {
    let mut mcause: i32;

    unsafe {
        llvm_asm!("
            // Read the mcause CSR to determine why we entered the trap handler.
            // Since we are using the CLIC, the hardware includes the interrupt
            // index in the mcause register.
            csrr $0, 0x342    // CSR=0x342=mcause
        "
        : "=r"(mcause)
        :
        :
        : "volatile");
    }

    // Check if the trap was from an interrupt or some other exception.
    if mcause < 0 {
        // If the most significant bit is set (i.e. mcause is negative) then
        // this was an interrupt. The interrupt number is then the lowest 8
        // bits.
        let interrupt_index = mcause & 0xFF;
        unsafe {
            riscv::clic::disable_interrupt(interrupt_index as u32);
        }
    } else {
        // Otherwise, the kernel encountered a fault...so panic!()?
        panic!("kernel exception");
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_handler"]
pub extern "C" fn disable_interrupt_trap_handler(mcause: u32) {
    // The interrupt number is then the lowest 8
    // bits.
    let interrupt_index = mcause & 0xFF;
    unsafe {
        riscv::clic::disable_interrupt(interrupt_index as u32);
    }
}
