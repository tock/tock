use cortexm4::{self, nvic};
use gpio;
use i2c;
use kernel;
use peripheral_interrupts;
use radio;
use rtc;
use uart;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum SleepMode {
    DeepSleep = 0,
    Sleep = 1,
    Active = 2,
}

impl From<u32> for SleepMode {
    fn from(n: u32) -> Self {
        match n {
            0 => SleepMode::DeepSleep,
            1 => SleepMode::Sleep,
            2 => SleepMode::Active,
            _ => unimplemented!(),
        }
    }
}

pub struct Cc26X2 {
    mpu: cortexm4::mpu::MPU,
    systick: cortexm4::systick::SysTick,
}

impl Cc26X2 {
    pub unsafe fn new() -> Cc26X2 {
        Cc26X2 {
            mpu: cortexm4::mpu::MPU::new(),
            // The systick clocks with 48MHz by default
            systick: cortexm4::systick::SysTick::new_with_calibration(48 * 1000000),
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
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Hardware)
                    },
                    peripheral_interrupts::RF_CMD_ACK => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::CmdAck)
                    },
                    peripheral_interrupts::RF_CORE_CPE0 => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe0)
                    },
                    peripheral_interrupts::RF_CORE_CPE1 => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe1)
                    },
                    // AON Programmable interrupt
                    // We need to ignore JTAG events since some debuggers emit these
                    peripheral_interrupts::AON_PROG => (),
                    // peripheral_interrupts::OSC => prcm::handle_osc_interrupt(),
                    _ => panic!("unhandled interrupt {}", interrupt),
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
        /*
        unsafe {
            radio::RFC.ack_nvic.disable();
            if radio::RFC.ack_event.get() {
                radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::CmdAck);
                radio::RFC.ack_event.set(false);
            };
            radio::RFC.ack_nvic.enable();
        }

        unsafe {
            radio::RFC.cpe0_nvic.disable();
            if radio::RFC.cpe0_event.get() {
                radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe0);
                radio::RFC.cpe0_event.set(false);
            };
            radio::RFC.cpe0_nvic.enable();
        }
        */
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
