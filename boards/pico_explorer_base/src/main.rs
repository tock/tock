// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Raspberry Pi Pico.
//!
//! It is based on RP2040SoC SoC (Cortex M0+).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use components::led::LedsComponent;
use enum_primitive::cast::FromPrimitive;
use kernel::component::Component;
use kernel::hil::led::LedHigh;
use kernel::hil::usb::Client;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{capabilities, create_capability, static_init, Kernel};
use kernel::{debug, hil};

use rp2040::adc::{Adc, Channel};
use rp2040::chip::{Rp2040, Rp2040DefaultPeripherals};
use rp2040::clocks::{
    AdcAuxiliaryClockSource, PeripheralAuxiliaryClockSource, PllClock,
    ReferenceAuxiliaryClockSource, ReferenceClockSource, RtcAuxiliaryClockSource,
    SystemAuxiliaryClockSource, SystemClockSource, UsbAuxiliaryClockSource,
};
use rp2040::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2040::pio::Pio;
use rp2040::pio_pwm::PioPwm;
use rp2040::resets::Peripheral;
use rp2040::spi::Spi;
use rp2040::sysinfo;
use rp2040::timer::RPTimer;

mod io;

mod flash_bootloader;

/// Allocate memory for the stack
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1500] = [0; 0x1500];

// Manually setting the boot header section that contains the FCB header
#[used]
#[link_section = ".flash_bootloader"]
static FLASH_BOOTLOADER: [u8; 256] = flash_bootloader::FLASH_BOOTLOADER;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static Rp2040<Rp2040DefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

type TemperatureRp2040Sensor = components::temperature_rp2040::TemperatureRp2040ComponentType<
    capsules_core::virtualizers::virtual_adc::AdcDevice<'static, rp2040::adc::Adc<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<TemperatureRp2040Sensor>;

/// Supported drivers by the platform
pub struct PicoExplorerBase {
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    console: &'static capsules_core::console::Console<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rp2040::timer::RPTimer<'static>>,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, RPGpioPin<'static>>,
    led: &'static capsules_core::led::LedDriver<'static, LedHigh<'static, RPGpioPin<'static>>, 1>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    temperature: &'static TemperatureDriver,
    buzzer_driver: &'static capsules_extra::buzzer_driver::Buzzer<
        'static,
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                rp2040::timer::RPTimer<'static>,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<
                'static,
                rp2040::pwm::Pwm<'static>,
            >,
        >,
    >,
    button: &'static capsules_core::button::Button<'static, RPGpioPin<'static>>,
    screen: &'static capsules_extra::screen::Screen<'static>,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm0p::systick::SysTick,
}

impl SyscallDriverLookup for PicoExplorerBase {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::buzzer_driver::DRIVER_NUM => f(Some(self.buzzer_driver)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_extra::screen::DRIVER_NUM => f(Some(self.screen)),
            _ => f(None),
        }
    }
}

impl KernelResources<Rp2040<'static, Rp2040DefaultPeripherals<'static>>> for PicoExplorerBase {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm0p::systick::SysTick;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.systick
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

#[allow(dead_code)]
extern "C" {
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
    fn jump_to_bootloader();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
    .section .jump_to_bootloader, \"ax\"
    .global jump_to_bootloader
    .thumb_func
  jump_to_bootloader:
    movs r0, #0
    ldr r1, =(0xe0000000 + 0x0000ed08)
    str r0, [r1]
    ldmia r0!, {{r1, r2}}
    msr msp, r1
    bx r2
    "
);

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

    // It seems that the external osciallator is clocked at 12 MHz

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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    PicoExplorerBase,
    &'static rp2040::chip::Rp2040<'static, Rp2040DefaultPeripherals<'static>>,
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

    // Set the UART used for panic
    (*addr_of_mut!(io::WRITER)).set_uart(&peripherals.uart0);

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

    CHIP = Some(chip);

    let board_kernel = static_init!(Kernel, Kernel::new(&*addr_of!(PROCESSES)));

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.timer)
        .finalize(components::alarm_mux_component_static!(RPTimer));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(RPTimer));

    // CDC
    let strings = static_init!(
        [&str; 3],
        [
            "Raspberry Pi",                // Manufacturer
            "pico explorer base - TockOS", // Product
            "00000000000000000",           // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &peripherals.usb,
        //capsules::usb::cdc::MAX_CTRL_PACKET_SIZE_RP2040,
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

    // UART
    // Create a shared UART channel for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(cdc, 115200)
        .finalize(components::uart_mux_component_static!());

    // Uncomment this to use UART as an output
    // let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
    //     .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    cdc.enable();
    cdc.attach();

    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            RPGpioPin,
            // Used for serial communication. Comment them in if you don't use serial.
            // 0 => &peripherals.pins.get_pin(RPGpio::GPIO0),
            // 1 => &peripherals.pins.get_pin(RPGpio::GPIO1),
            // Used for Buzzer.
            // 2 => &peripherals.pins.get_pin(RPGpio::GPIO2),
            3 => peripherals.pins.get_pin(RPGpio::GPIO3),
            4 => peripherals.pins.get_pin(RPGpio::GPIO4),
            5 => peripherals.pins.get_pin(RPGpio::GPIO5),
            6 => peripherals.pins.get_pin(RPGpio::GPIO6),
            7 => peripherals.pins.get_pin(RPGpio::GPIO7),
            20 => peripherals.pins.get_pin(RPGpio::GPIO20),
            21 => peripherals.pins.get_pin(RPGpio::GPIO21),
            22 => peripherals.pins.get_pin(RPGpio::GPIO22),
        ),
    )
    .finalize(components::gpio_component_static!(RPGpioPin<'static>));

    let led = LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, RPGpioPin<'static>>,
        LedHigh::new(peripherals.pins.get_pin(RPGpio::GPIO25))
    ));

    peripherals.adc.init();

    // Set PWM function for Buzzer.
    peripherals
        .pins
        .get_pin(RPGpio::GPIO2)
        .set_function(GpioFunction::PWM);

    let adc_mux = components::adc::AdcMuxComponent::new(&peripherals.adc)
        .finalize(components::adc_mux_component_static!(Adc));

    let temp_sensor = components::temperature_rp2040::TemperatureRp2040Component::new(
        adc_mux,
        Channel::Channel4,
        1.721,
        0.706,
    )
    .finalize(components::temperature_rp2040_adc_component_static!(
        rp2040::adc::Adc
    ));

    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        temp_sensor,
    )
    .finalize(components::temperature_component_static!(
        TemperatureRp2040Sensor
    ));

    //set CLK, MOSI and CS pins in SPI mode
    let spi_clk = peripherals.pins.get_pin(RPGpio::GPIO18);
    let spi_csn = peripherals.pins.get_pin(RPGpio::GPIO17);
    let spi_mosi = peripherals.pins.get_pin(RPGpio::GPIO19);
    spi_clk.set_function(GpioFunction::SPI);
    spi_csn.set_function(GpioFunction::SPI);
    spi_mosi.set_function(GpioFunction::SPI);
    let mux_spi = components::spi::SpiMuxComponent::new(&peripherals.spi0)
        .finalize(components::spi_mux_component_static!(Spi));

    let bus = components::bus::SpiMasterBusComponent::new(
        mux_spi,
        hil::spi::cs::IntoChipSelect::<_, hil::spi::cs::ActiveLow>::into_cs(
            peripherals.pins.get_pin(RPGpio::GPIO17),
        ),
        20_000_000,
        kernel::hil::spi::ClockPhase::SampleLeading,
        kernel::hil::spi::ClockPolarity::IdleLow,
    )
    .finalize(components::spi_bus_component_static!(Spi));

    let tft = components::st77xx::ST77XXComponent::new(
        mux_alarm,
        bus,
        Some(peripherals.pins.get_pin(RPGpio::GPIO16)),
        None,
        &capsules_extra::st77xx::ST7789H2,
    )
    .finalize(components::st77xx_component_static!(
        // bus type
        capsules_extra::bus::SpiMasterBus<
            'static,
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, Spi>,
        >,
        // timer type
        RPTimer,
        // pin type
        RPGpioPin,
    ));

    let _ = tft.init();

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            RPGpioPin,
            (
                peripherals.pins.get_pin(RPGpio::GPIO12),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), // A
            (
                peripherals.pins.get_pin(RPGpio::GPIO13),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), // B
            (
                peripherals.pins.get_pin(RPGpio::GPIO14),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), // X
            (
                peripherals.pins.get_pin(RPGpio::GPIO15),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), // Y
        ),
    )
    .finalize(components::button_component_static!(RPGpioPin));

    let screen = components::screen::ScreenComponent::new(
        board_kernel,
        capsules_extra::screen::DRIVER_NUM,
        tft,
        Some(tft),
    )
    .finalize(components::screen_component_static!(57600));

    let adc_channel_0 = components::adc::AdcComponent::new(adc_mux, Channel::Channel0)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_1 = components::adc::AdcComponent::new(adc_mux, Channel::Channel1)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_2 = components::adc::AdcComponent::new(adc_mux, Channel::Channel2)
        .finalize(components::adc_component_static!(Adc));

    let adc_syscall =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                adc_channel_0,
                adc_channel_1,
                adc_channel_2,
            ));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);
    // PROCESS CONSOLE
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm0p::support::reset),
    )
    .finalize(components::process_console_component_static!(RPTimer));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    //--------------------------------------------------------------------------
    // BUZZER
    //--------------------------------------------------------------------------
    use kernel::hil::buzzer::Buzzer;
    use kernel::hil::time::Alarm;

    let mux_pwm = components::pwm::PwmMuxComponent::new(&peripherals.pwm)
        .finalize(components::pwm_mux_component_static!(rp2040::pwm::Pwm));

    let virtual_pwm_buzzer =
        components::pwm::PwmPinUserComponent::new(mux_pwm, rp2040::gpio::RPGpio::GPIO2)
            .finalize(components::pwm_pin_user_component_static!(rp2040::pwm::Pwm));

    let virtual_alarm_buzzer = static_init!(
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            rp2040::timer::RPTimer,
        >,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );

    virtual_alarm_buzzer.setup();

    let pwm_buzzer = static_init!(
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                rp2040::timer::RPTimer,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, rp2040::pwm::Pwm>,
        >,
        capsules_extra::buzzer_pwm::PwmBuzzer::new(
            virtual_pwm_buzzer,
            virtual_alarm_buzzer,
            capsules_extra::buzzer_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
        )
    );

    let buzzer_driver = static_init!(
        capsules_extra::buzzer_driver::Buzzer<
            'static,
            capsules_extra::buzzer_pwm::PwmBuzzer<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                    'static,
                    rp2040::timer::RPTimer,
                >,
                capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, rp2040::pwm::Pwm>,
            >,
        >,
        capsules_extra::buzzer_driver::Buzzer::new(
            pwm_buzzer,
            capsules_extra::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
            board_kernel.create_grant(
                capsules_extra::buzzer_driver::DRIVER_NUM,
                &memory_allocation_capability
            )
        )
    );

    pwm_buzzer.set_client(buzzer_driver);

    virtual_alarm_buzzer.set_alarm_client(pwm_buzzer);

    let pico_explorer_base = PicoExplorerBase {
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        alarm,
        gpio,
        led,
        console,
        adc: adc_syscall,
        temperature: temp,
        buzzer_driver,
        button,
        screen,
        scheduler,
        systick: cortexm0p::systick::SysTick::new_with_calibration(125_000_000),
    };

    let platform_type = match peripherals.sysinfo.get_platform() {
        sysinfo::Platform::Asic => "ASIC",
        sysinfo::Platform::Fpga => "FPGA",
    };

    debug!(
        "RP2040 Revision {} {}",
        peripherals.sysinfo.get_revision(),
        platform_type
    );
    debug!("Initialization complete. Enter main loop");

    // These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    //--------------------------------------------------------------------------
    // PIO
    //--------------------------------------------------------------------------

    let mut pio: Pio = Pio::new_pio0();

    let _pio_pwm = PioPwm::new(&mut pio, &peripherals.clocks);
    // This will start a PWM with PIO with the set frequency and duty cycle on the specified pin.
    // pio_pwm
    //     .start(
    //         &RPGpio::GPIO7,
    //         pio_pwm.get_maximum_frequency_hz() / 125000, /*1_000*/
    //         pio_pwm.get_maximum_duty_cycle() / 2,
    //     )
    //     .unwrap();

    (board_kernel, pico_explorer_base, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
