use core::fmt::Write;
use core::panic::PanicInfo;
use cortexm4;
#[cfg(feature = "usb_debugging")]
use kernel::common::cells::TakeCell;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
#[cfg(not(feature = "usb_debugging"))]
use kernel::hil::uart::{self, Configure};
use nrf52840::gpio::Pin;

use crate::PROCESSES;

struct Writer {
    #[cfg(not(feature = "usb_debugging"))]
    initialized: bool,
    #[cfg(feature = "usb_debugging")]
    rtt_memory: TakeCell<'static, capsules::segger_rtt::SeggerRttMemory<'static>>,
}

#[cfg(not(feature = "usb_debugging"))]
static mut WRITER: Writer = Writer { initialized: false };
#[cfg(feature = "usb_debugging")]
static mut WRITER: Writer = Writer {
    rtt_memory: TakeCell::empty(),
};

#[cfg(feature = "usb_debugging")]
fn wait() {
    let mut x = 0;
    for i in 0..5000 {
        unsafe { core::ptr::write_volatile(&mut x as *mut _, i) };
    }
}

/// Set the RTT memory buffer used to output panic messages.
#[cfg(feature = "usb_debugging")]
pub unsafe fn set_rtt_memory(
    rtt_memory: &'static mut capsules::segger_rtt::SeggerRttMemory<'static>,
) {
    WRITER.rtt_memory.replace(rtt_memory);
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    #[cfg(not(feature = "usb_debugging"))]
    fn write(&mut self, buf: &[u8]) {
        let uart = unsafe { &mut nrf52840::uart::UARTE0 };
        if !self.initialized {
            self.initialized = true;
            uart.configure(uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }
        for &c in buf {
            unsafe {
                uart.send_byte(c);
            }
            while !uart.tx_ready() {}
        }
    }

    #[cfg(feature = "usb_debugging")]
    fn write(&mut self, buf: &[u8]) {
        // TODO: initialize if needed.
        self.rtt_memory.map(|rtt_memory| {
            let up_buffer = &mut rtt_memory.up_buffer;
            let buffer_len = up_buffer.length.get();
            let buffer = unsafe {
                core::slice::from_raw_parts_mut(
                    up_buffer.buffer.get() as *mut u8,
                    buffer_len as usize,
                )
            };

            let mut write_position = up_buffer.write_position.get();

            for &c in buf {
                buffer[write_position as usize] = c;
                write_position = (write_position + 1) % buffer_len;
                up_buffer.write_position.set(write_position);
                wait();
            }
        });
    }
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
/// Panic handler
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // The nRF52840DK LEDs (see back of board)
    const LED1_PIN: Pin = Pin::P0_13;
    let led = &mut led::LedLow::new(&mut nrf52840::gpio::PORT[LED1_PIN]);
    let writer = &mut WRITER;
    debug::panic(&mut [led], writer, pi, &cortexm4::support::nop, &PROCESSES)
}
