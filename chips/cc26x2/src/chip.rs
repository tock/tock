use cc26xx::gpio;
use cc26xx::peripheral_interrupts;
use cc26xx::uart;
use cortexm4::{self, nvic};
use kernel;
use i2c;
use rfc;
use rtc;

pub struct Cc26X2 {
    mpu: cortexm4::mpu::MPU,
    systick: cortexm4::systick::SysTick,
    // pub rf_core: rfc::RFCore,
}

impl Cc26X2 {
    pub unsafe fn new() -> Cc26X2 {
        Cc26X2 {
            mpu: cortexm4::mpu::MPU::new(),
            // The systick clocks with 48MHz by default
            systick: cortexm4::systick::SysTick::new_with_calibration(48 * 1000000),
            // rf_core: rfc::RFCore::new(),
        }
    }
}

impl kernel::Chip for Cc26X2 {
    type MPU = cortexm4::mpu::MPU;
    type SysTick = cortexm4::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }
    fn service_pending_interrupts(&mut self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                match interrupt {
                    peripheral_interrupts::GPIO => gpio::PORT.handle_interrupt(),
                    peripheral_interrupts::AON_RTC => rtc::RTC.handle_interrupt(),
                    peripheral_interrupts::UART0 => uart::UART0.handle_interrupt(),
                    peripheral_interrupts::I2C => i2c::I2C0.handle_interrupt(),
                    peripheral_interrupts::RF_CORE_HW => {
                        rfc::RFC.handle_interrupt(rfc::RfcInterrupt::Hardware)
                    }
                    peripheral_interrupts::RF_CMD_ACK => {
                        rfc::RFC.handle_interrupt(rfc::RfcInterrupt::CmdAck)
                    }
                    peripheral_interrupts::RF_CORE_PE1 => {
                        rfc::RFC.handle_interrupt(rfc::RfcInterrupt::Cpe0)
                    }
                    peripheral_interrupts::RF_CORE_PE2 => {
                        rfc::RFC.handle_interrupt(rfc::RfcInterrupt::Cpe1)
                    }
                    // AON Programmable interrupt
                    // We need to ignore JTAG events since some debuggers emit these
                    peripheral_interrupts::AON_PROG => (),
                    _ => panic!("unhandled interrupt {}", interrupt),
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
