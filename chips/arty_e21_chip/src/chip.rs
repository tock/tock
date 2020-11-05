use core::fmt::Write;
use kernel;
use kernel::debug;
use kernel::InterruptService;
use rv32i;

use crate::interrupts;
use crate::pmp;
use crate::timer;

extern "C" {
    fn _start_trap();
}

pub struct ArtyExx<'a, I: InterruptService<()> + 'a> {
    pmp: pmp::PMP,
    userspace_kernel_boundary: rv32i::syscall::SysCall,
    clic: rv32i::clic::Clic,
    interrupt_service: &'a I,
}

pub struct ArtyExxDefaultPeripherals<'a> {
    pub machinetimer: rv32i::machine_timer::MachineTimer<'a>,
    pub gpio_port: crate::gpio::Port<'a>,
    pub uart0: sifive::uart::Uart<'a>,
}

impl<'a> ArtyExxDefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            machinetimer: rv32i::machine_timer::MachineTimer::new(timer::MTIME_BASE),
            gpio_port: crate::gpio::Port::new(),
            uart0: sifive::uart::Uart::new(crate::uart::UART0_BASE, 32_000_000),
        }
    }
}

impl<'a> InterruptService<()> for ArtyExxDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::MTIP => self.machinetimer.handle_interrupt(),

            interrupts::GPIO0 => self.gpio_port[0].handle_interrupt(),
            interrupts::GPIO1 => self.gpio_port[1].handle_interrupt(),
            interrupts::GPIO2 => self.gpio_port[2].handle_interrupt(),
            interrupts::GPIO3 => self.gpio_port[3].handle_interrupt(),
            interrupts::GPIO4 => self.gpio_port[4].handle_interrupt(),
            interrupts::GPIO5 => self.gpio_port[5].handle_interrupt(),
            interrupts::GPIO6 => self.gpio_port[6].handle_interrupt(),
            interrupts::GPIO7 => self.gpio_port[7].handle_interrupt(),
            interrupts::GPIO8 => self.gpio_port[8].handle_interrupt(),
            interrupts::GPIO9 => self.gpio_port[9].handle_interrupt(),
            interrupts::GPIO10 => self.gpio_port[10].handle_interrupt(),
            interrupts::GPIO11 => self.gpio_port[11].handle_interrupt(),
            interrupts::GPIO12 => self.gpio_port[12].handle_interrupt(),
            interrupts::GPIO13 => self.gpio_port[13].handle_interrupt(),
            interrupts::GPIO14 => self.gpio_port[14].handle_interrupt(),
            interrupts::GPIO15 => self.gpio_port[15].handle_interrupt(),

            interrupts::UART0 => self.uart0.handle_interrupt(),

            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, I: InterruptService<()> + 'a> ArtyExx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        // Make a bit-vector of all interrupt locations that we actually intend
        // to use on this chip.
        // 0001 1111 1111 1111 1111 0000 0000 1000 0000
        let in_use_interrupts: u64 = 0x1FFFF0080;

        Self {
            pmp: pmp::PMP::new(),
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
            clic: rv32i::clic::Clic::new(in_use_interrupts),
            interrupt_service,
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

impl<'a, I: InterruptService<()> + 'a> kernel::Chip for ArtyExx<'a, I> {
    type MPU = pmp::PMP;
    type UserspaceKernelBoundary = rv32i::syscall::SysCall;
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

    fn userspace_kernel_boundary(&self) -> &rv32i::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = self.clic.next_pending() {
                if !self.interrupt_service.service_interrupt(interrupt) {
                    debug!("unhandled interrupt: {:?}", interrupt);
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
            rv32i::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        rv32i::print_riscv_state(write);
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
            rv32i::clic::disable_interrupt(interrupt_index as u32);
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
        rv32i::clic::disable_interrupt(interrupt_index as u32);
    }
}
