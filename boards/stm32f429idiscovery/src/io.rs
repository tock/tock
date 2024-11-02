// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::core_local::CoreLocal;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;

use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::StaticSlice;
use stm32f429zi::chip::Stm32f4xx;
use stm32f429zi::chip_specs::Stm32f429Specs;
use stm32f429zi::gpio::PinId;
use stm32f429zi::interrupt_service::Stm32f429ziDefaultPeripherals;

pub(crate) struct DebugInfo {
    pub chip: &'static Stm32f4xx<'static, Stm32f429ziDefaultPeripherals<'static>>,
    pub processes:
        &'static CoreLocal<MapCell<StaticSlice<Option<&'static dyn kernel::process::Process>>>>,
    pub process_printer: &'static capsules_system::process_printer::ProcessPrinterText,
}

pub(crate) static mut DEBUG_INFO: CoreLocal<MapCell<DebugInfo>> =
    unsafe { CoreLocal::new_single_core(MapCell::empty()) };

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    uart:
        OptionalCell<&'static stm32f429zi::usart::Usart<'static, stm32f429zi::dma::Dma2<'static>>>,
}

/// Global static for debug writer
pub(crate) static WRITER: CoreLocal<MapCell<Writer>> = unsafe {
    CoreLocal::new_single_core(MapCell::new(Writer {
        uart: OptionalCell::empty(),
    }))
};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.uart.map(|uart| {
            for &c in buf {
                uart.send_byte(c);
            }
        });

        buf.len()
    }
}

/// Panic handler.
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    // User LD4 is connected to PG14
    // Have to reinitialize several peripherals because otherwise can't access them here.
    let rcc = stm32f429zi::rcc::Rcc::new();
    let clocks: stm32f429zi::clocks::Clocks<Stm32f429Specs> =
        stm32f429zi::clocks::Clocks::new(&rcc);
    let syscfg = stm32f429zi::syscfg::Syscfg::new(&clocks);
    let exti = stm32f429zi::exti::Exti::new(&syscfg);
    let pin = stm32f429zi::gpio::Pin::new(PinId::PG14, &exti);
    let gpio_ports = stm32f429zi::gpio::GpioPorts::new(&clocks, &exti);
    pin.set_ports_ref(&gpio_ports);
    let led = &mut led::LedHigh::new(&pin);

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
                info,
                &cortexm4::support::nop,
                &processes[..],
                debug_info.chip,
                debug_info.process_printer,
            )
        })
        .unwrap_or_else(|| loop {})
    })
}
