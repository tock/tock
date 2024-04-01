// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::core_local::CoreLocal;
use kernel::debug::{self, IoWrite};
use kernel::hil::led::LedHigh;
use kernel::hil::uart::{Configure, Parameters, Parity, StopBits, Width};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::StaticSlice;

use rp2040::chip::{Rp2040, Rp2040DefaultPeripherals};
use rp2040::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2040::uart::Uart;

pub(crate) struct DebugInfo {
    pub chip: &'static Rp2040<'static, Rp2040DefaultPeripherals<'static>>,
    pub processes:
        &'static CoreLocal<MapCell<StaticSlice<Option<&'static dyn kernel::process::Process>>>>,
    pub process_printer: &'static capsules_system::process_printer::ProcessPrinterText,
}

pub(crate) static mut DEBUG_INFO: CoreLocal<MapCell<DebugInfo>> =
    unsafe { CoreLocal::new_single_core(MapCell::empty()) };

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    uart: OptionalCell<&'static Uart<'static>>,
}

pub(crate) static WRITER: CoreLocal<MapCell<Writer>> = unsafe {
    CoreLocal::new_single_core(MapCell::new(Writer {
        uart: OptionalCell::empty(),
    }))
};

impl Writer {
    pub fn set_uart(&self, uart: &'static Uart) {
        self.uart.set(uart);
    }

    fn configure_uart(&self, uart: &Uart) {
        if !uart.is_configured() {
            let parameters = Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                parity: Parity::None,
                stop_bits: StopBits::One,
                hw_flow_control: false,
            };
            //configure parameters of uart for sending bytes
            let _ = uart.configure(parameters);
            //set RX and TX pins in UART mode
            let gpio_tx = RPGpioPin::new(RPGpio::GPIO0);
            let gpio_rx = RPGpioPin::new(RPGpio::GPIO1);
            gpio_rx.set_function(GpioFunction::UART);
            gpio_tx.set_function(GpioFunction::UART);
        }
    }

    fn write_to_uart(&self, uart: &Uart, buf: &[u8]) {
        for &c in buf {
            uart.send_byte(c);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.uart.map_or_else(
            || {
                let uart = Uart::new_uart0();
                self.configure_uart(&uart);
                self.write_to_uart(&uart, buf);
            },
            |uart| {
                self.configure_uart(uart);
                self.write_to_uart(uart, buf);
            },
        );
        buf.len()
    }
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
/// Panic handler
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // LED is connected to GPIO 25

    let led_kernel_pin = &RPGpioPin::new(RPGpio::GPIO25);
    let led = &mut LedHigh::new(led_kernel_pin);
    let mut writer = WRITER.with(|w| w.take()).unwrap();

    DEBUG_INFO.with(|di| {
        di.map(|debug_info| {
            let processes = debug_info
                .processes
                .with(|processes| processes.take())
                .unwrap_or(StaticSlice::new(&mut []));
            debug::panic(
                &mut [led],
                &mut writer,
                pi,
                &cortexm0p::support::nop,
                &processes[..],
                debug_info.chip,
                debug_info.process_printer,
            )
        })
        .unwrap_or_else(|| loop {})
    })
}
