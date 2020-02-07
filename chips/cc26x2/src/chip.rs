use crate::gpio;
use crate::i2c;
use crate::peripheral_interrupts::NvicIrq;
use crate::rtc;
use crate::uart;
use core::fmt::Write;
use cortexm4::{self, nvic};
use enum_primitive::cast::FromPrimitive;

pub struct Cc26X2 {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl Cc26X2 {
    // internal HFREQ is 40_000_000 Hz
    // but if you are using an external HFREQ to derive systick, you will want to input value here (in Hz)
    pub unsafe fn new(hfreq: u32) -> Cc26X2 {
        Cc26X2 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            // The systick clocks with 48MHz by default
            systick: cortexm4::systick::SysTick::new_with_calibration(hfreq),
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
                let irq = NvicIrq::from_u32(interrupt)
                    .expect("Pending IRQ flag not enumerated in NviqIrq");
                match irq {
                    NvicIrq::Gpio => gpio::PORT.handle_interrupt(),
                    NvicIrq::AonRtc => rtc::RTC.handle_interrupt(),
                    NvicIrq::Uart0 => uart::UART0.handle_interrupt(),
                    NvicIrq::I2c0 => i2c::I2C0.handle_interrupt(),
                    // We need to ignore JTAG events since some debuggers emit these
                    NvicIrq::AonProg => (),
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

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        cortexm4::print_cortexm4_state(writer);
    }
}
