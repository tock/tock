use crate::gpio;
use crate::i2c;
use crate::peripheral_interrupts::NVIC_IRQ;
use crate::rtc;
use crate::uart;
use cortexm4::{self, nvic};
use enum_primitive::cast::FromPrimitive;

pub struct Cc26X2 {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl Cc26X2 {
    pub unsafe fn new() -> Cc26X2 {
        Cc26X2 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            // The systick clocks with 48MHz by default
            systick: cortexm4::systick::SysTick::new_with_calibration(48 * 1000000),
        }
    }
}

impl kernel::Chip for Cc26X2 {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                let irq = NVIC_IRQ::from_u32(interrupt)
                    .expect("Pending IRQ flag not enumerated in NVIQ_IRQ");
                match irq {
                    NVIC_IRQ::GPIO => gpio::PORT.handle_interrupt(),
                    NVIC_IRQ::AON_RTC => rtc::RTC.handle_interrupt(),
                    NVIC_IRQ::UART0 => uart::UART0.handle_interrupt(),
                    NVIC_IRQ::I2C0 => i2c::I2C0.handle_interrupt(),
                    // We need to ignore JTAG events since some debuggers emit these
                    NVIC_IRQ::AON_PROG => (),
                    _ => panic!("Unhandled interrupt {:?}", irq),
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
