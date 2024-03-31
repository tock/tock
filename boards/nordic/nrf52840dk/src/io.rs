// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use kernel::hil::uart;
use kernel::hil::uart::Configure;
use kernel::utilities::cells::MapCell;
use kernel::StaticSlice;
use kernel::{core_local::CoreLocal, debug::IoWrite};

use nrf52840::uart::{Uarte, UARTE0_BASE};

enum Writer {
    WriterUart(/* initialized */ bool),
    WriterRtt(&'static segger::rtt::SeggerRttMemory<'static>),
}

static WRITER: CoreLocal<MapCell<Writer>> = unsafe { CoreLocal::new_single_core(MapCell::empty()) };

/// Set the RTT memory buffer used to output panic messages.
pub unsafe fn set_rtt_memory(rtt_memory: &'static segger::rtt::SeggerRttMemory<'static>) {
    WRITER.with(|w| w.put(Writer::WriterRtt(rtt_memory)));
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        match self {
            Writer::WriterUart(ref mut initialized) => {
                // Here, we create a second instance of the Uarte struct.
                // This is okay because we only call this during a panic, and
                // we will never actually process the interrupts
                let uart = Uarte::new(UARTE0_BASE);
                if !*initialized {
                    *initialized = true;
                    let _ = uart.configure(uart::Parameters {
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
            Writer::WriterRtt(rtt_memory) => {
                rtt_memory.write_sync(buf);
            }
        };
        buf.len()
    }
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &core::panic::PanicInfo) -> ! {
    use kernel::debug;
    use kernel::hil::led;
    use nrf52840::gpio::Pin;

    // The nRF52840DK LEDs (see back of board)
    let led_kernel_pin = &nrf52840::gpio::GPIOPin::new(Pin::P0_13);
    let led = &mut led::LedLow::new(led_kernel_pin);
    let mut writer = WRITER
        .with(|w| w.take())
        .unwrap_or(Writer::WriterUart(false));
    crate::DEBUG_INFO
        .with(|di| {
            di.map(|di| {
                let processes = di
                    .processes
                    .with(|processes| processes.take())
                    .unwrap_or(StaticSlice::new(&mut []));
                debug::panic(
                    &mut [led],
                    &mut writer,
                    pi,
                    &cortexm4::support::nop,
                    &processes[..],
                    di.chip,
                    di.process_printer,
                )
            })
        })
        .unwrap_or_else(|| loop {})
}
