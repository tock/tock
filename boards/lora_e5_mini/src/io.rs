// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

use kernel::debug::IoWrite;
use kernel::hil::led;

use kernel::hil::led::Led;
use kernel::hil::uart;
use kernel::hil::uart::Configure;
use stm32wle5jc::chip_specs::Stm32wle5jcSpecs;
use stm32wle5jc::clocks;
use stm32wle5jc::gpio::PinId;
/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    initialized: bool,
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer { initialized: false };

impl Writer {
    /// Indicate that USART has already been initialized. Trying to double
    /// initialize USART1 causes stm32f429zi to go into in in-deterministic state.
    pub fn set_initialized(&mut self) {
        self.initialized = true;
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
        let rcc = stm32wle5jc::rcc::Rcc::new();
        let clocks: clocks::Clocks<Stm32wle5jcSpecs> = clocks::Clocks::new(&rcc);
        let usart = stm32wle5jc::usart::Usart::new_usart1(&clocks);

        if !self.initialized {
            self.initialized = true;

            let _ = usart.configure(uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            });
        }

        let mut written = 0;
        for byte in buf {
            usart.send_byte(*byte);
            written += 1;
        }
        written
    }
}

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // For now we add a loop to blink the LED to an interesting way.
    // To ensure that all dependencies are set up, we initialize all clocks
    // and GPIOs here in this function.
    //
    // Relying on `main.rs` to initialize clocks/gpios may result in the gpio
    // not being properly configured if the panic occurs early in `main.rs`.
    let rcc = stm32wle5jc::rcc::Rcc::new();
    let clocks: stm32wle5jc::clocks::Clocks<Stm32wle5jcSpecs> =
        stm32wle5jc::clocks::Clocks::new(&rcc);
    let syscfg = stm32wle5jc::syscfg::Syscfg::new();
    let exti = stm32wle5jc::exti::Exti::new(&syscfg);

    let gpio_ports = stm32wle5jc::gpio::GpioPorts::new(&clocks, &exti);
    gpio_ports.setup_circular_deps();
    gpio_ports
        .get_port_from_port_id(stm32wle5jc::gpio::PortId::B)
        .enable_clock();
    let pin = stm32wle5jc::gpio::Pin::new(PinId::PB05, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedLow::new(&pin);
    led.init();

    // USART1: PB6=TX , PB7=RX
    gpio_ports.get_pin(PinId::PB06).map(|pin| {
        pin.set_mode(stm32wle5jc::gpio::Mode::AlternateFunctionMode);
        pin.set_alternate_function(stm32wle5jc::gpio::AlternateFunction::AF7);
    });

    gpio_ports.get_pin(PinId::PB07).map(|pin| {
        pin.set_mode(stm32wle5jc::gpio::Mode::AlternateFunctionMode);
        pin.set_alternate_function(stm32wle5jc::gpio::AlternateFunction::AF7);
    });

    let usart = stm32wle5jc::usart::Usart::new_usart1(&clocks);
    usart.enable_clock();
    let _ = usart.configure(uart::Parameters {
        baud_rate: 115200,
        stop_bits: uart::StopBits::One,
        parity: uart::Parity::None,
        hw_flow_control: false,
        width: uart::Width::Eight,
    });

    kernel::debug::panic_print(
        &mut *addr_of_mut!(WRITER),
        info,
        &cortexm4::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // Unique LED blink pattern for panic
    loop {
        led.on();
        // Wait for LONG
        delay_long();

        led.off();
        delay_short();

        // SHORT
        led.on();
        delay_short();

        led.off();
        delay_short();

        // SHORT
        led.on();
        delay_short();

        led.off();
        delay_short();

        // LONG
        led.on();
        delay_long();

        led.off();
        delay_long();
    }
}

fn delay_long() {
    for _ in 0..1_000_000 {
        cortexm4::support::nop();
    }
}

fn delay_short() {
    for _ in 0..100_000 {
        cortexm4::support::nop();
    }
}
