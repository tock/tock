use riscv32i;
use riscv32i::plic;
use kernel;
use gpio;
use interrupts;
use uart;

pub struct E310x(());

impl E310x {
    pub unsafe fn new() -> E310x {
        E310x(())
    }
}

impl kernel::Chip for E310x {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &self.0
    }

    fn systick(&self) -> &Self::SysTick {
        &self.0
    }

    fn service_pending_interrupts(&mut self) {

        unsafe {
            while let Some(interrupt) = plic::next_pending() {

                match interrupt {
                    interrupts::UART0 => uart::UART0.handle_interrupt(),
                    index @ interrupts::GPIO0..interrupts::GPIO31 => gpio::PORT[index as usize].handle_interrupt(),
                    // _ => debug!("PLIC index not supported by Tock {}", interrupt),
                    _ => debug!("Pidx {}", interrupt),
                }

                // Mark that we are done with this interrupt and the hardware
                // can clear it.
                plic::complete(interrupt);
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { plic::has_pending() }
    }

    fn sleep(&self) {
        // unsafe {
            // riscv32i::support::wfi();
            riscv32i::support::nop();
        // }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        riscv32i::support::atomic(f)
    }
}
