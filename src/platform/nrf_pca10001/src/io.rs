use nrf51822::{self, PinCnf};
use core::fmt::*;
use support::nop;

pub struct Writer { initialized: bool }

pub static mut WRITER : Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut nrf51822::uart::UART::new() };
        if !self.initialized {
            self.initialized = true;
            unsafe {
                uart.init(PinCnf::new(9), PinCnf::new(11),
                          PinCnf::new(8), PinCnf::new(10));
            }
            uart.set_baudrate(115200);
        }
        uart.send_bytes(s.as_bytes());
        Ok(())
    }
}


#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern fn rust_begin_unwind(args: Arguments,
    file: &'static str, line: u32) -> ! {
    use hil::gpio::GPIOPin;

    let writer = &mut WRITER;
    let _ = writer.write_fmt(format_args!("Kernel panic at {}:{}:\r\n\t\"", file, line));
    let _ = write(writer, args);
    let _ = writer.write_str("\"\r\n");

    let led0 = &nrf51822::gpio::PA[21];
    let led1 = &nrf51822::gpio::PA[22];

    led0.enable_output();
    led1.enable_output();
    loop {
        for _ in 0..100000 {
            led0.set();
            led1.set();
            nop();
        }
        for _ in 0..100000 {
            led0.clear();
            led1.clear();
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

