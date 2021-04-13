use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
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
        for b in buf {
            // Print to a special address for simulation output
            unsafe {
                write_volatile(0x8000_1008 as *mut u8, *b as u8);
            }
        }
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    debug::panic_print(writer, pi, &rv32i::support::nop, &PROCESSES, &CHIP);

    // By writing to address 0x80001009 we can exit the simulation.
    // So instead of blinking in a loop let's exit the simulation.
    write_volatile(0x8000_1009 as *mut u8, 20);

    unreachable!()
}
