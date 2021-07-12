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
        let uart = esp32::uart::Uart::new(esp32::uart::UART0_BASE);
        uart.disable_tx_interrupt();
        uart.disable_rx_interrupt();
        uart.transmit_sync(buf);
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    debug::panic_banner(writer, pi);
    debug::panic_cpu_state(&CHIP, writer);
    debug::panic_process_info(&PROCESSES, writer);

    loop {
        rv32i::support::nop();
    }
}

#[cfg(test)]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;

    debug::panic_print(writer, pi, &rv32i::support::nop, &PROCESSES, &CHIP);

    let _ = writeln!(writer, "{}", pi);
    loop {}
}
