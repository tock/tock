use core::fmt::{write, Arguments, Write};
use kernel::hil::uart::{self, UART};
use nrf51;
use nrf5x;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut nrf51::uart::UART0 };
        if !self.initialized {
            self.initialized = true;
            uart.init(uart::UARTParams {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
        }
        for c in s.bytes() {
            unsafe {
                uart.send_byte(c);
            }
            while !uart.tx_ready() {}
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
        ($($arg:tt)*) => (
            {
                use core::fmt::write;
                let writer = &mut $crate::io::WRITER;
                let _ = write(writer, format_args!($($arg)*));
            }
        );
}

#[macro_export]
macro_rules! println {
        ($fmt:expr) => (print!(concat!($fmt, "\n")));
            ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(
    _args: Arguments,
    _file: &'static str,
    _line: usize,
) -> ! {
    use kernel::hil::gpio::Pin;
    use kernel::process;
    // The nRF51 DK LEDs (see back of board)
    const LED1_PIN: usize = 21;
    const LED2_PIN: usize = 22;

    let writer = &mut WRITER;
    let _ = writer.write_fmt(format_args!(
        "\r\nKernel panic at {}:{}:\r\n\t\"",
        _file, _line
    ));
    let _ = write(writer, _args);
    let _ = writer.write_str("\"\r\n");

    // Print version of the kernel
    let _ = writer.write_fmt(format_args!(
        "\tKernel version {}\r\n",
        env!("TOCK_KERNEL_VERSION")
    ));

    // Print fault status once
    let procs = &mut process::PROCS;
    if procs.len() > 0 {
        procs[0].as_mut().map(|process| {
            process.fault_str(writer);
        });
    }

    // print data about each process
    let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
    let procs = &mut process::PROCS;
    for idx in 0..procs.len() {
        procs[idx].as_mut().map(|process| {
            process.statistics_str(writer);
        });
    }
    let led0 = &nrf5x::gpio::PORT[LED1_PIN];
    let led1 = &nrf5x::gpio::PORT[LED2_PIN];

    led0.make_output();
    led1.make_output();
    loop {
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..100000 {
            led0.set();
            led1.set();
        }
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..500000 {
            led0.set();
            led1.set();
        }
    }
}
