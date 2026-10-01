// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board file for the ESP32-C3-DevKit-RUST-2 RISC-V development platform.
//!

#![no_std]
#![no_main]

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use esp32_c3::chip::Esp32C3DefaultPeripherals;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, debug, hil, static_init};
use rv32i::csr;

pub mod io;

const NUM_PROCS: usize = 4;

type ChipHw = esp32_c3::chip::Esp32C3<'static, Esp32C3DefaultPeripherals<'static>>;
type AlarmHw = esp32_c3::timg::TimG<'static>;
type SchedulerTimerHw =
    components::virtual_scheduler_timer::VirtualSchedulerTimerNoMuxComponentType<AlarmHw>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

kernel::stack_size! {0x900}

type GpioPinHw = esp32::gpio::GpioPin<'static>;
type GpioDriver = components::gpio::GpioComponentType<GpioPinHw>;
type ConsoleDriver = components::console::ConsoleComponentType;
type AlarmDriver = components::alarm::AlarmDriverComponentType<AlarmHw>;
type RngDriver = components::rng::RngComponentType<esp32_c3::rng::Rng<'static>>;
type GpioHw = esp32::gpio::GpioPin<'static>;
type LedHw = components::sk68xx::Sk68xxLedComponentType<GpioHw, 3>;
type LedDriver = components::led::LedsComponentType<LedHw, 3>;
type ButtonDriver = components::button::ButtonComponentType<GpioHw>;

type I2cDeviceHw =
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, esp32_c3::i2c::I2c<'static>>;
type Shtc3Hw = components::shtc3::Shtc3ComponentType<
    VirtualMuxAlarm<'static, esp32_c3::timg::TimG<'static>>,
    I2cDeviceHw,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<Shtc3Hw>;
type HumidityDriver = components::humidity::HumidityComponentType<Shtc3Hw>;
type NineDofDriver = components::ninedof::NineDofComponentType;

struct ProcessManagementCapabilityObj {}
unsafe impl capabilities::ProcessManagementCapability for ProcessManagementCapabilityObj {}

type SchedulerInUse =
    components::sched::priority::PriorityComponentType<ProcessManagementCapabilityObj>;

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct Esp32C3Board {
    gpio: &'static GpioDriver,
    console: &'static ConsoleDriver,
    alarm: &'static AlarmDriver,
    scheduler: &'static SchedulerInUse,
    scheduler_timer: &'static SchedulerTimerHw,
    rng: &'static RngDriver,
    led: &'static LedDriver,
    button: &'static ButtonDriver,
    temperature: &'static TemperatureDriver,
    humidity: &'static HumidityDriver,
    ninedof: &'static NineDofDriver,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for Esp32C3Board {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules_extra::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            _ => f(None),
        }
    }
}

impl KernelResources<esp32_c3::chip::Esp32C3<'static, Esp32C3DefaultPeripherals<'static>>>
    for Esp32C3Board
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type ContextSwitchCallback = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = SchedulerTimerHw;
    type WatchDog = ();

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
        self.scheduler_timer
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static Esp32C3Board,
    &'static esp32_c3::chip::Esp32C3<'static, Esp32C3DefaultPeripherals<'static>>,
    &'static Esp32C3DefaultPeripherals<'static>,
) {
    use esp32_c3::sysreg::{CpuFrequency, PllFrequency};

    // only machine mode
    unsafe {
        esp32_c3::chip::configure_trap_handler();
    }

    // Initialize deferred calls very early.
    //
    // SAFETY: We must guarantee we are not calling this from any other
    // thread, which we are not.
    unsafe {
        kernel::deferred_call::initialize_deferred_call_state_unsafe::<
            <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
        >();
    }

    // Bind global variables to this thread.
    //
    // SAFETY: We must guarantee we are not calling this from any other
    // thread, which we are not.
    unsafe {
        let _ = PANIC_RESOURCES
            .bind_to_thread_unsafe::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
                PanicResources::new(),
            );
    }

    //
    // PERIPHERALS
    //
    let peripherals = static_init!(Esp32C3DefaultPeripherals, Esp32C3DefaultPeripherals::new());

    peripherals.timg0.disable_wdt();
    peripherals.rtc_cntl.disable_wdt();
    peripherals.rtc_cntl.disable_super_wdt();
    peripherals.rtc_cntl.enable_fosc();
    peripherals.sysreg.disable_timg0();
    peripherals.sysreg.enable_timg0();

    peripherals
        .sysreg
        .use_pll_clock_source(PllFrequency::MHz320, CpuFrequency::MHz160);

    // initialise capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    //
    // BOARD SETUP AND PROCESSES
    //

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    //
    // CONSOLE
    //

    // The DevKit-RUST-2 has no external USB-to-UART bridge chip: its
    // USB Type-C port is wired directly to the chip's native USB pins, so
    // UART0 (GPIO20/GPIO21) is only reachable via the pin headers. Use the
    // USB_SERIAL_JTAG peripheral's CDC-ACM serial port for the console and
    // kernel debug output instead, since that is what is reachable over
    // the USB cable.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.usb_serial_jtag, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
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

    // Create process printer for panic.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    //
    // GPIO
    //

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            esp32::gpio::GpioPin,
            // GPIO2 (WS2812 RGB LED), GPIO7/GPIO8 (onboard I2C bus to the
            // IMU and humidity sensor), and GPIO9 (BOOT button) are
            // dedicated to onboard peripherals on the DevKit-RUST-2 and are
            // not exposed here.
            0 => &peripherals.gpio[0],
            1 => &peripherals.gpio[1],
            3 => &peripherals.gpio[3],
            4 => &peripherals.gpio[4],
            5 => &peripherals.gpio[5],
            6 => &peripherals.gpio[6]
        ),
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::gpio_component_static!(esp32::gpio::GpioPin));

    //
    // I2C
    //

    // The onboard IMU (ICM-42670-P) and humidity/temperature sensor
    // (SHTC3) share the I2C_EXT0 bus, wired to SDA: GPIO7, SCL: GPIO8.
    peripherals.sysreg.enable_i2c0();
    peripherals
        .i2c
        .set_pins(&peripherals.gpio[7], &peripherals.gpio[8]);
    peripherals.i2c.set_frequency(100_000);

    let mux_i2c = components::i2c::I2CMuxComponent::new(&peripherals.i2c, None)
        .finalize(components::i2c_mux_component_static!(esp32_c3::i2c::I2c));

    //
    // ALARM
    //

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, esp32_c3::timg::TimG>,
        MuxAlarm::new(&peripherals.timg0)
    );
    hil::time::Alarm::set_alarm_client(&peripherals.timg0, mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, esp32_c3::timg::TimG>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, esp32_c3::timg::TimG>>,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    //
    // SENSORS
    //

    let shtc3 = components::shtc3::Shtc3Component::new(
        mux_i2c,
        capsules_extra::shtc3::BASE_ADDR,
        mux_alarm,
    )
    .finalize(components::shtc3_component_static!(
        esp32_c3::timg::TimG,
        esp32_c3::i2c::I2c
    ));

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        shtc3,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::temperature_component_static!(Shtc3Hw));

    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules_extra::humidity::DRIVER_NUM,
        shtc3,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::humidity_component_static!(Shtc3Hw));

    let icm42670p = components::icm42670p::Icm42670pComponent::new(
        mux_i2c,
        capsules_extra::icm42670p::BASE_ADDR,
        mux_alarm,
    )
    .finalize(components::icm42670p_component_static!(
        esp32_c3::timg::TimG,
        esp32_c3::i2c::I2c
    ));

    let ninedof = components::ninedof::NineDofComponent::new(
        board_kernel,
        capsules_extra::ninedof::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::ninedof_component_static!(icm42670p));

    //
    // LED
    //

    let led_gpio = &peripherals.gpio[2];
    let sk68xx = components::sk68xx::Sk68xxComponent::new(led_gpio, rv32i::support::nop)
        .finalize(components::sk68xx_component_static_esp32c3_160mhz!(GpioHw,));

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHw,
        capsules_extra::sk68xx::Sk68xxLed::new(sk68xx, 0), // red
        capsules_extra::sk68xx::Sk68xxLed::new(sk68xx, 1), // green
        capsules_extra::sk68xx::Sk68xxLed::new(sk68xx, 2), // blue
    ));

    //
    // BUTTONS
    //

    let button_boot_gpio = &peripherals.gpio[9];
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            GpioHw,
            (
                button_boot_gpio,
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::button_component_static!(GpioHw));

    //
    // SCHEDULER
    //

    let scheduler_timer_alarm = &peripherals.timg1;
    let scheduler_timer =
        components::virtual_scheduler_timer::VirtualSchedulerTimerNoMuxComponent::new(
            scheduler_timer_alarm,
        )
        .finalize(components::virtual_scheduler_timer_no_mux_component_static!(AlarmHw));

    let scheduler = components::sched::priority::PriorityComponent::new(
        board_kernel,
        ProcessManagementCapabilityObj {},
    )
    .finalize(components::priority_component_static!(
        ProcessManagementCapabilityObj
    ));

    //
    // PROCESS CONSOLE
    //

    kernel::create_typed_capability!(process_console_cap, ProcessConsoleCap:
        kernel::capabilities::ProcessManagementCapability,
        kernel::capabilities::ProcessStartCapability
    );
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        None,
        process_console_cap,
    )
    .finalize(components::process_console_component_static!(
        esp32_c3::timg::TimG,
        ProcessConsoleCap
    ));
    let _ = process_console.start();

    //
    // RNG
    //

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &peripherals.rng,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::rng_component_static!(esp32_c3::rng::Rng));

    //
    // CHIP AND INTERRUPTS
    //

    let c = unsafe { esp32_c3::chip::Esp32C3::new(peripherals) };
    let chip = static_init!(
        esp32_c3::chip::Esp32C3<
            Esp32C3DefaultPeripherals,
        >,
        c
    );
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Need to enable all interrupts for Tock Kernel
    chip.map_pic_interrupts();
    unsafe {
        chip.enable_pic_interrupts();
    }

    // enable interrupts globally
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    debug!("ESP32-C3 initialisation complete.");
    debug!("Entering main loop.");

    //
    // LOAD PROCESSES
    //

    // These symbols are defined in the linker script.
    unsafe extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let esp32_c3_board = static_init!(
        Esp32C3Board,
        Esp32C3Board {
            gpio,
            console,
            alarm,
            scheduler,
            scheduler_timer,
            rng,
            led,
            button,
            temperature,
            humidity,
            ninedof
        }
    );

    let app_flash = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        )
    };

    let app_ram = unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        )
    };

    kernel::process::load_processes(
        board_kernel,
        chip,
        app_flash,
        app_ram,
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    peripherals.init();

    (board_kernel, esp32_c3_board, chip, peripherals)
}

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[unsafe(no_mangle)]
pub unsafe fn main() {
    let (board_kernel, esp32_c3_board, chip, _peripherals) = unsafe { setup() };

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    board_kernel.kernel_loop(
        esp32_c3_board,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_cap,
    );
}
