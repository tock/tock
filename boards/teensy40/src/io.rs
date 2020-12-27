use core::fmt::{self, Write};

use kernel::debug::{self, IoWrite};
use kernel::hil::{
    led,
    uart::{self, Configure},
};

use crate::imxrt1060::gpio;
use crate::imxrt1060::lpuart;

struct Writer<'a> {
    output: &'a mut lpuart::Lpuart<'static>,
}

const BAUD_RATE: u32 = 115_200;

impl<'a> Writer<'a> {
    pub unsafe fn new(output: &'a mut lpuart::Lpuart<'static>) -> Self {
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

impl IoWrite for Writer<'_> {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.output.send_byte(*byte);
        }
    }
}

impl Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

#[no_mangle]
#[panic_handler]
unsafe fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let ports = imxrt10xx::gpio::Ports::new();
    let led = &mut led::LedHigh::new(ports.pin(gpio::PinId::B0_03));
    let mut lpuart2 = lpuart::Lpuart::new_lpuart2();
    let mut writer = Writer::new(&mut lpuart2);
    debug::panic(
        &mut [led],
        &mut writer,
        panic_info,
        &cortexm7::support::nop,
        &crate::PROCESSES,
        &crate::CHIP,
    )
}
