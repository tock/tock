use core::fmt::{self, Write};

use kernel::debug::{self, IoWrite};
use kernel::hil::{
    led,
    uart::{self, Configure},
};

use imxrt10xx::gpio;
use imxrt10xx::lpuart;

struct Writer {
    output: &'static mut lpuart::Lpuart<'static>,
}

const BAUD_RATE: u32 = 115_200;

impl Writer {
    pub unsafe fn new(output: &'static mut lpuart::Lpuart) -> Self {
        output.configure(uart::Parameters {
            baud_rate: BAUD_RATE,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
            width: uart::Width::Eight,
        });

        Writer { output }
    }
}

impl IoWrite for Writer {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.output.send_byte(*byte);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
#[panic_handler]
unsafe fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let led = &mut led::LedHigh::new(gpio::PinId::B0_03.get_pin_mut().as_mut().unwrap());
    let mut writer = Writer::new(&mut lpuart::LPUART2);
    debug::panic(
        &mut [led],
        &mut writer,
        panic_info,
        &cortexm7::support::nop,
        &crate::PROCESSES,
        &crate::CHIP,
    )
}
