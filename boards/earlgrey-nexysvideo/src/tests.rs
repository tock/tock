use crate::debug::IoWrite;
use core::fmt::Write;
use core::panic::PanicInfo;

extern "C" {
    pub(crate) fn semihost_command(command: usize, arg0: usize, arg1: usize) -> !;
}

#[cfg(test)]
pub(crate) fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }

    // Make sure all the messages are printed
    for _ in 0..200000 {
        rv32i::support::nop();
    }

    // Exit QEMU with a return code of 0
    unsafe {
        semihost_command(0x18, 0x20026, 0);
    }
}

struct Writer<'a> {
    uart: earlgrey::uart::Uart<'a>,
}

// This creates a second UART peripheral.
// This should ONLY be used by test cases running on QEMU.
static mut WRITER: Writer = Writer {
    uart: earlgrey::uart::Uart::new(
        earlgrey::uart::UART0_BASE,
        earlgrey::chip_config::CONFIG.peripheral_freq,
    ),
};

impl Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer<'_> {
    fn write(&mut self, buf: &[u8]) {
        self.uart.transmit_sync(buf);
    }
}

macro_rules! print {
    () => ({
        // Allow an empty debug!() to print the location when hit
        print!("")
    });
    ($msg:expr $(,)?) => ({
        #[allow(unused_unsafe)]
        unsafe { write!(&mut WRITER, $msg).unwrap() }
    });
    ($fmt:expr, $($arg:tt)+) => ({
        #[allow(unused_unsafe)]
        unsafe { write!(&mut WRITER, $fmt, $($arg)+).unwrap() }
    });
}

#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    print!("{}", pi);
    // Exit QEMU with a return code of 1
    crate::tests::semihost_command(0x18, 1, 0);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    print!("    [ok]\r\n");
}
