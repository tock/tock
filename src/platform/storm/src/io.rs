use core::fmt::*;
use hil::Controller;
use hil::uart::{self, UART};
use sam4l;
use support::nop;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut sam4l::usart::USART3 };
        if !self.initialized {
            self.initialized = true;
            uart.configure(sam4l::usart::USARTParams {
                baud_rate: 115200,
                data_bits: 8,
                parity: uart::Parity::None,
                mode: uart::Mode::Normal,
            });
            uart.enable_tx();

        }
        for c in s.bytes() {
            uart.send_byte(c);
        }
        Ok(())
    }
}


#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(args: Arguments, file: &'static str, line: u32) -> ! {

    let writer = &mut WRITER;
    let _ = writer.write_fmt(format_args!("Kernel panic at {}:{}:\r\n\t\"", file, line));
    let _ = write(writer, args);
    let _ = writer.write_str("\"\r\n");

    let led = &sam4l::gpio::PC[10];
    led.enable_output();
    loop {
        for _ in 0..1000000 {
            led.set();
            nop();
        }
        for _ in 0..1000000 {
            led.clear();
            nop();
        }
    }
}

#[macro_export]
macro_rules! print {
        ($($arg:tt)*) => (
            {
                use core::fmt::write;
                let writer = unsafe { &mut $crate::io::WRITER };
                let _ = write(writer, format_args!($($arg)*));
            }
        );
}

#[macro_export]
macro_rules! println {
        ($fmt:expr) => (print!(concat!($fmt, "\n")));
            ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
