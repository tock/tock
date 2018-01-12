use core::fmt::*;
use kernel::hil::uart::{self, UART};
use kernel::process;
use sam4l;

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
#[no_mangle]
#[lang = "panic_fmt"]
pub unsafe extern "C" fn panic_fmt(args: Arguments, file: &'static str, line: u32) -> ! {
    // XXX Replace with something like kernel::begin_panic()
    // XXX Maybe place that call at panic_fmt, as it's called first
    // XXX Better to cancel the transaction rather than hope we wait long enough
    // Let any outstanding uart DMA's finish
    asm!("nop");
    asm!("nop");
    for _ in 0..200000 {
        asm!("nop");
    }
    asm!("nop");
    asm!("nop");

    let writer = &mut WRITER;
    let _ = writer.write_fmt(format_args!(
        "\r\n\nKernel panic at {}:{}:\r\n\t\"",
        file, line
    ));
    let _ = write(writer, args);
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

    // turn off the non panic leds, just in case
    let ledg = &sam4l::gpio::PA[14];
    ledg.enable_output();
    ledg.set();
    let ledb = &sam4l::gpio::PA[15];
    ledb.enable_output();
    ledb.set();

    // blink the panic signal
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
