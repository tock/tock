use core::fmt::*;
use kernel::hil::uart::{self, UART};
use sam4l;
use kernel::process;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut sam4l::usart::USART0 };
        if !self.initialized {
            self.initialized = true;
            uart.init(uart::UARTParams {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
            uart.reset();
            uart.enable_tx();

        }
        // XXX: I'd like to get this working the "right" way, but I'm not sure how
        for c in s.bytes() {
            uart.send_byte(c);
            while !uart.tx_ready() {}
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

    print_app_statistics(writer);

    let led = &sam4l::gpio::PA[13];
    led.enable_output();
    loop {
        for _ in 0..1000000 {
            led.clear();
        }
        for _ in 0..100000 {
            led.set();
        }
        for _ in 0..1000000 {
            led.clear();
        }
        for _ in 0..500000 {
            led.set();
        }
    }
}

unsafe fn print_app_statistics(writer: &mut Writer) {
    let _ = writer.write_fmt(format_args!("\r\n---| App Statistics |---\r\n"));

    // iterate through each process
    let procs = &mut process::PROCS;
    for idx in 0..procs.len() {
        procs[idx].as_ref().map(|process| {
            let _ = writer.write_fmt(format_args!("App {}\r\n", idx));
            process.statistics_str(writer);
        });
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
