// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright OxidOS Automotive 2025.

#![no_std]

use capsules_core::i2c_master::I2CMasterDriver;
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_extra::usb::cdc::CdcAcm;
use enum_primitive::cast::FromPrimitive;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::debug::PanicResources;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::FloatingState;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::usb::Client;
use kernel::platform::SyscallDriverLookup;
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::syscall::SyscallDriver;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::Kernel;
use kernel::{create_capability, static_init};
use rp2040::adc;
use rp2040::adc::Adc;
use rp2040::chip::{Rp2040, Rp2040DefaultPeripherals};
use rp2040::clocks::RtcAuxiliaryClockSource;
use rp2040::clocks::{AdcAuxiliaryClockSource, PeripheralAuxiliaryClockSource, PllClock};
use rp2040::clocks::{ReferenceAuxiliaryClockSource, ReferenceClockSource};
use rp2040::clocks::{SystemAuxiliaryClockSource, SystemClockSource, UsbAuxiliaryClockSource};
use rp2040::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2040::i2c::I2c;
use rp2040::resets::Peripheral;
use rp2040::sysinfo;
use rp2040::timer::RPTimer;
use rp2040::usb::UsbCtrl;

mod flash_bootloader;

// Manually setting the boot header section that contains the FCB header
//
// When compiling for a macOS host, the `link_section` attribute is elided as
// it yields the following error: `mach-o section specifier requires a segment
// and section separated by a comma`.
#[cfg_attr(not(target_os = "macos"), link_section = ".flash_bootloader")]
#[used]
static FLASH_BOOTLOADER: [u8; 256] = flash_bootloader::FLASH_BOOTLOADER;

// Number of concurrent processes this platform supports.
pub const NUM_PROCS: usize = 4;

pub type ChipHw = Rp2040<'static, Rp2040DefaultPeripherals<'static>>;
pub type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics.
pub static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new(PanicResources::new());

type TemperatureRp2040Sensor = components::temperature_rp2040::TemperatureRp2040ComponentType<
    capsules_core::virtualizers::virtual_adc::AdcDevice<'static, rp2040::adc::Adc<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<TemperatureRp2040Sensor>;

/// Base drivers for the Raspberry Pi Pico boards
pub struct Platform {
    pub systick: cortexm0p::systick::SysTick,
    pub ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    pub scheduler: &'static RoundRobinSched<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rp2040::timer::RPTimer<'static>>,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, RPGpioPin<'static>>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    temperature: &'static TemperatureDriver,
    i2c: &'static capsules_core::i2c_master::I2CMasterDriver<'static, I2c<'static, 'static>>,
    date_time:
        &'static capsules_extra::date_time::DateTimeCapsule<'static, rp2040::rtc::Rtc<'static>>,
    console: &'static capsules_core::console::Console<'static>,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_core::i2c_master::DRIVER_NUM => f(Some(self.i2c)),
            capsules_extra::date_time::DRIVER_NUM => f(Some(self.date_time)),
            _ => f(None),
        }
    }
}

/// Entry point used for debugger
///
/// When loaded using gdb, the Raspberry Pi Pico is not reset
/// by default. Without this function, gdb sets the PC to the
/// beginning of the flash. This is not correct, as the RP2040
/// has a more complex boot process.
///
/// This function is set to be the entry point for gdb and is used
/// to send the RP2040 back in the bootloader so that all the boot
/// sequence is performed.
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn jump_to_bootloader() {
    use core::arch::naked_asm;
    naked_asm!(
        "
    movs r0, #0
    ldr r1, =(0xe0000000 + 0x0000ed08)
    str r0, [r1]
    ldmia r0!, {{r1, r2}}
    msr msp, r1
    bx r2
        "
    );
}

fn init_clocks(peripherals: &Rp2040DefaultPeripherals) {
    // Start tick in watchdog
    peripherals.watchdog.start_tick(12);

    // Disable the Resus clock
    peripherals.clocks.disable_resus();

    // Setup the external Oscillator
    peripherals.xosc.init();

    // disable ref and sys clock aux sources
    peripherals.clocks.disable_sys_aux();
    peripherals.clocks.disable_ref_aux();

    peripherals
        .resets
        .reset(&[Peripheral::PllSys, Peripheral::PllUsb]);
    peripherals
        .resets
        .unreset(&[Peripheral::PllSys, Peripheral::PllUsb], true);

    // Configure PLLs (from Pico SDK)
    //                   REF     FBDIV VCO            POSTDIV
    // PLL SYS: 12 / 1 = 12MHz * 125 = 1500MHZ / 6 / 2 = 125MHz
    // PLL USB: 12 / 1 = 12MHz * 40  = 480 MHz / 5 / 2 =  48MHz

    // It seems that the external oscillator is clocked at 12 MHz

    peripherals
        .clocks
        .pll_init(PllClock::Sys, 12, 1, 1500 * 1000000, 6, 2);
    peripherals
        .clocks
        .pll_init(PllClock::Usb, 12, 1, 480 * 1000000, 5, 2);

    // pico-sdk: // CLK_REF = XOSC (12MHz) / 1 = 12MHz
    peripherals.clocks.configure_reference(
        ReferenceClockSource::Xosc,
        ReferenceAuxiliaryClockSource::PllUsb,
        12000000,
        12000000,
    );
    // pico-sdk: CLK SYS = PLL SYS (125MHz) / 1 = 125MHz
    peripherals.clocks.configure_system(
        SystemClockSource::Auxiliary,
        SystemAuxiliaryClockSource::PllSys,
        125000000,
        125000000,
    );
    // pico-sdk: CLK USB = PLL USB (48MHz) / 1 = 48MHz
    peripherals
        .clocks
        .configure_usb(UsbAuxiliaryClockSource::PllSys, 48000000, 48000000);
    // pico-sdk: CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
    peripherals
        .clocks
        .configure_adc(AdcAuxiliaryClockSource::PllUsb, 48000000, 48000000);
    // pico-sdk: CLK RTC = PLL USB (48MHz) / 1024 = 46875Hz
    peripherals
        .clocks
        .configure_rtc(RtcAuxiliaryClockSource::PllSys, 48000000, 46875);
    // pico-sdk:
    // CLK PERI = clk_sys. Used as reference clock for Peripherals. No dividers so just select and enable
    // Normally choose clk_sys or clk_usb
    peripherals
        .clocks
        .configure_peripheral(PeripheralAuxiliaryClockSource::System, 125000000);
}

/// Setup the CDC capsule
pub unsafe fn cdc_setup(
    peripherals: &'static Rp2040DefaultPeripherals,
    mux_alarm: &'static MuxAlarm<'static, RPTimer<'static>>,
) -> &'static CdcAcm<'static, UsbCtrl<'static>, VirtualMuxAlarm<'static, RPTimer<'static>>> {
    // CDC
    let strings = static_init!(
        [&str; 3],
        [
            "Raspberry Pi",      // Manufacturer
            "Pico - TockOS",     // Product
            "00000000000000000", // Serial number
        ]
    );

    components::cdc::CdcAcmComponent::new(
        &peripherals.usb,
        //capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_RP2040,
        64,
        peripherals.sysinfo.get_manufacturer_rp2040(),
        peripherals.sysinfo.get_part(),
        strings,
        mux_alarm,
        None,
    )
    .finalize(components::cdc_acm_component_static!(
        rp2040::usb::UsbCtrl,
        rp2040::timer::RPTimer
    ))
}

/// Use either CDC or UART as output
pub enum Output {
    Cdc,
    Uart,
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
///
/// This function takes the type of output it should use for console/debug writer/process console.
/// This usually depends on whether the user has the board connected to a debug probe or not.
pub unsafe fn setup(
    output: Output,
) -> (
    &'static Kernel,
    Platform,
    &'static Rp2040DefaultPeripherals<'static>,
    &'static MuxAlarm<'static, RPTimer<'static>>,
    &'static Rp2040<'static, Rp2040DefaultPeripherals<'static>>,
) {
    // Loads relocations and clears BSS
    rp2040::init();

    let peripherals = static_init!(Rp2040DefaultPeripherals, Rp2040DefaultPeripherals::new());
    peripherals.resolve_dependencies();

    // Reset all peripherals except QSPI (we might be booting from Flash), PLL USB and PLL SYS
    peripherals.resets.reset_all_except(&[
        Peripheral::IOQSpi,
        Peripheral::PadsQSpi,
        Peripheral::PllUsb,
        Peripheral::PllSys,
    ]);

    // Unreset all the peripherals that do not require clock setup as they run using the sys_clk or ref_clk
    // Wait for the peripherals to reset
    peripherals.resets.unreset_all_except(
        &[
            Peripheral::Adc,
            Peripheral::Rtc,
            Peripheral::Spi0,
            Peripheral::Spi1,
            Peripheral::Uart0,
            Peripheral::Uart1,
            Peripheral::UsbCtrl,
        ],
        true,
    );

    init_clocks(peripherals);

    // Unreset all peripherals
    peripherals.resets.unreset_all_except(&[], true);

    //set RX and TX pins in UART mode
    let gpio_tx = peripherals.pins.get_pin(RPGpio::GPIO0);
    let gpio_rx = peripherals.pins.get_pin(RPGpio::GPIO1);
    gpio_rx.set_function(GpioFunction::UART);
    gpio_tx.set_function(GpioFunction::UART);

    // Disable IE for pads 26-29 (the Pico SDK runtime does this, not sure why)
    for pin in 26..30 {
        peripherals
            .pins
            .get_pin(RPGpio::from_usize(pin).unwrap())
            .deactivate_pads();
    }

    let chip = static_init!(
        Rp2040<Rp2040DefaultPeripherals>,
        Rp2040::new(peripherals, &peripherals.sio)
    );
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.timer)
        .finalize(components::alarm_mux_component_static!(RPTimer));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(RPTimer));

    let uart_mux = match output {
        Output::Cdc => {
            let strings = static_init!(
                [&str; 3],
                [
                    "Raspberry Pi",      // Manufacturer
                    "Pico - TockOS",     // Product
                    "00000000000000000", // Serial number
                ]
            );

            let cdc = components::cdc::CdcAcmComponent::new(
                &peripherals.usb,
                //capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_RP2040,
                64,
                peripherals.sysinfo.get_manufacturer_rp2040(),
                peripherals.sysinfo.get_part(),
                strings,
                mux_alarm,
                None,
            )
            .finalize(components::cdc_acm_component_static!(
                rp2040::usb::UsbCtrl,
                rp2040::timer::RPTimer
            ));

            cdc.enable();
            cdc.attach();

            // UART
            // Create a shared UART channel for kernel debug.
            components::console::UartMuxComponent::new(cdc, 115200)
                .finalize(components::uart_mux_component_static!())
        }
        Output::Uart => components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
            .finalize(components::uart_mux_component_static!()),
    };

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new_unsafe(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
        || unsafe {
            kernel::debug::initialize_debug_writer_wrapper_unsafe::<
                <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
            >();
        },
    )
    .finalize(components::debug_writer_component_static!());

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            RPGpioPin,
            // Used for serial communication. Comment them in if you don't use serial.
            // 0 => peripherals.pins.get_pin(RPGpio::GPIO0),
            // 1 => peripherals.pins.get_pin(RPGpio::GPIO1),
            2 => peripherals.pins.get_pin(RPGpio::GPIO2),
            3 => peripherals.pins.get_pin(RPGpio::GPIO3),
            // Used for i2c. Comment them in if you don't use i2c.
            // 4 => peripherals.pins.get_pin(RPGpio::GPIO4),
            // 5 => peripherals.pins.get_pin(RPGpio::GPIO5),
            6 => peripherals.pins.get_pin(RPGpio::GPIO6),
            7 => peripherals.pins.get_pin(RPGpio::GPIO7),
            8 => peripherals.pins.get_pin(RPGpio::GPIO8),
            9 => peripherals.pins.get_pin(RPGpio::GPIO9),
            10 => peripherals.pins.get_pin(RPGpio::GPIO10),
            11 => peripherals.pins.get_pin(RPGpio::GPIO11),
            12 => peripherals.pins.get_pin(RPGpio::GPIO12),
            13 => peripherals.pins.get_pin(RPGpio::GPIO13),
            14 => peripherals.pins.get_pin(RPGpio::GPIO14),
            15 => peripherals.pins.get_pin(RPGpio::GPIO15),
            16 => peripherals.pins.get_pin(RPGpio::GPIO16),
            17 => peripherals.pins.get_pin(RPGpio::GPIO17),
            18 => peripherals.pins.get_pin(RPGpio::GPIO18),
            19 => peripherals.pins.get_pin(RPGpio::GPIO19),
            20 => peripherals.pins.get_pin(RPGpio::GPIO20),
            21 => peripherals.pins.get_pin(RPGpio::GPIO21),
            22 => peripherals.pins.get_pin(RPGpio::GPIO22),
            23 => peripherals.pins.get_pin(RPGpio::GPIO23),
            24 => peripherals.pins.get_pin(RPGpio::GPIO24),
            // LED pin
            // 25 => peripherals.pins.get_pin(RPGpio::GPIO25),

            // Uncomment to use these as GPIO pins instead of ADC pins
            // 26 => peripherals.pins.get_pin(RPGpio::GPIO26),
            // 27 => peripherals.pins.get_pin(RPGpio::GPIO27),
            // 28 => peripherals.pins.get_pin(RPGpio::GPIO28),
            // 29 => peripherals.pins.get_pin(RPGpio::GPIO29)
        ),
    )
    .finalize(components::gpio_component_static!(RPGpioPin<'static>));

    peripherals.adc.init();

    let adc_mux = components::adc::AdcMuxComponent::new(&peripherals.adc)
        .finalize(components::adc_mux_component_static!(Adc));

    let temp_sensor = components::temperature_rp2040::TemperatureRp2040Component::new(
        adc_mux,
        adc::Channel::Channel4,
        1.721,
        0.706,
    )
    .finalize(components::temperature_rp2040_adc_component_static!(
        rp2040::adc::Adc
    ));

    // RTC DATE TIME

    match peripherals.rtc.rtc_init() {
        Ok(()) => {}
        Err(e) => debug!("error starting rtc {:?}", e),
    }

    let date_time = components::date_time::DateTimeComponent::new(
        board_kernel,
        capsules_extra::date_time::DRIVER_NUM,
        &peripherals.rtc,
    )
    .finalize(components::date_time_component_static!(
        rp2040::rtc::Rtc<'static>
    ));

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        temp_sensor,
    )
    .finalize(components::temperature_component_static!(
        TemperatureRp2040Sensor
    ));

    let adc_channel_0 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel0)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_1 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel1)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_2 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel2)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_3 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel3)
        .finalize(components::adc_component_static!(Adc));

    let adc =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                adc_channel_0,
                adc_channel_1,
                adc_channel_2,
                adc_channel_3,
            ));

    // PROCESS CONSOLE
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm0p::support::reset),
    )
    .finalize(components::process_console_component_static!(RPTimer));
    let _ = process_console.start();

    let sda_pin = peripherals.pins.get_pin(RPGpio::GPIO4);
    let scl_pin = peripherals.pins.get_pin(RPGpio::GPIO5);

    sda_pin.set_function(GpioFunction::I2C);
    scl_pin.set_function(GpioFunction::I2C);

    sda_pin.set_floating_state(FloatingState::PullUp);
    scl_pin.set_floating_state(FloatingState::PullUp);

    let i2c_master_buffer = static_init!(
        [u8; capsules_core::i2c_master::BUFFER_LENGTH],
        [0; capsules_core::i2c_master::BUFFER_LENGTH]
    );
    let i2c0 = &peripherals.i2c0;
    let i2c = static_init!(
        I2CMasterDriver<I2c<'static, 'static>>,
        I2CMasterDriver::new(
            i2c0,
            i2c_master_buffer,
            board_kernel.create_grant(
                capsules_core::i2c_master::DRIVER_NUM,
                &memory_allocation_capability
            ),
        )
    );
    i2c0.init(10 * 1000);
    i2c0.set_master_client(i2c);

    let platform_type = match peripherals.sysinfo.get_platform() {
        sysinfo::Platform::Asic => "ASIC",
        sysinfo::Platform::Fpga => "FPGA",
    };

    debug!(
        "RP2040 Revision {} {}",
        peripherals.sysinfo.get_revision(),
        platform_type
    );

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        console,
        alarm,
        gpio,
        adc,
        temperature,
        i2c,
        date_time,
        systick: cortexm0p::systick::SysTick::new_with_calibration(125_000_000),
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        scheduler,
    };

    (board_kernel, platform, peripherals, mux_alarm, chip)
}
