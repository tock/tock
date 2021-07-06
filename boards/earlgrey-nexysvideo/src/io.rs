use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::debug::IoWrite;

use crate::CHIP;
use crate::PROCESSES;

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        // This creates a second instance of the UART peripheral, and should only be used
        // during panic.
        earlgrey::uart::Uart::new(
            earlgrey::uart::UART0_BASE,
            earlgrey::chip_config::CONFIG.peripheral_freq,
        )
        .transmit_sync(buf);
    }
}

#[cfg(not(test))]
use kernel::hil::gpio::Configure;
#[cfg(not(test))]
use kernel::hil::led;

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let first_led_pin = &mut earlgrey::gpio::GpioPin::new(
        earlgrey::gpio::GPIO0_BASE,
        earlgrey::gpio::PADCTRL_BASE,
        earlgrey::gpio::pins::pin7,
    );
    first_led_pin.make_output();
    let first_led = &mut led::LedLow::new(first_led_pin);

    let writer = &mut WRITER;

    debug::panic(
        &mut [first_led],
        writer,
        pi,
        &rv32i::support::nop,
        &PROCESSES,
        &CHIP,
    )
}

#[cfg(test)]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    debug::panic_print(writer, pi, &rv32i::support::nop, &PROCESSES, &CHIP);

    let _ = writeln!(writer, "{}", pi);
    // Exit QEMU with a return code of 1
    crate::tests::semihost_command_exit_failure();
}
