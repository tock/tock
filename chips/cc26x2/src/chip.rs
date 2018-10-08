use cortexm4;
use events;
use events::EVENT_PRIORITY;
use gpio;
use i2c;
use kernel;
use prcm;
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

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(event) = events::next_pending() {
                events::clear_event_flag(event);
                match event {
                    EVENT_PRIORITY::GPIO => gpio::PORT.handle_events(),
                    EVENT_PRIORITY::AON_RTC => rtc::RTC.handle_events(),
                    EVENT_PRIORITY::I2C0 => i2c::I2C0.handle_events(),
                    EVENT_PRIORITY::UART0 => uart::UART0.handle_events(),
                    EVENT_PRIORITY::UART1 => uart::UART1.handle_events(),
                    EVENT_PRIORITY::RF_CORE_HW => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Hardware)
                    }
                    EVENT_PRIORITY::RF_CMD_ACK => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::CmdAck)
                    }
                    EVENT_PRIORITY::RF_CORE_CPE0 => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe0)
                    }
                    EVENT_PRIORITY::RF_CORE_CPE1 => {
                        radio::RFC.handle_interrupt(radio::rfc::RfcInterrupt::Cpe1)
                    }
                    EVENT_PRIORITY::OSC => prcm::handle_osc_interrupt(),
                    EVENT_PRIORITY::AON_PROG => (),
                    _ => panic!("unhandled event {:?} ", event),
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        events::has_event()
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
