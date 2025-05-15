// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Reference Manual for the Imxrt-1052 development board
//!
//! - <https://www.nxp.com/webapp/Download?colCode=IMXRT1050RM>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::hil::gpio::Configure;
use kernel::hil::led::LedLow;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, static_init};

// use components::fxos8700::Fxos8700Component;
// use components::ninedof::NineDofComponent;
use imxrt1050::iomuxc::DriveStrength;
use imxrt1050::iomuxc::MuxMode;
use imxrt1050::iomuxc::OpenDrainEn;
use imxrt1050::iomuxc::PadId;
use imxrt1050::iomuxc::PullKeepEn;
use imxrt1050::iomuxc::PullUpDown;
use imxrt1050::iomuxc::Sion;
use imxrt1050::iomuxc::Speed;
use imxrt10xx as imxrt1050;

// Unit Tests for drivers.
// #[allow(dead_code)]
// mod virtual_uart_rx_test;

/// Support routines for debugging I/O.
pub mod io;

/// Defines a vector which contains the boot section
pub mod boot_header;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

type Chip = imxrt1050::chip::Imxrt10xx<imxrt1050::chip::Imxrt10xxDefaultPeripherals>;
static mut CHIP: Option<&'static Chip> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Manually setting the boot header section that contains the FCB header
#[used]
#[link_section = ".boot_hdr"]
static BOOT_HDR: [u8; 8192] = boot_header::BOOT_HDR;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

// const NUM_LEDS: usize = 1;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct Imxrt1050EVKB {
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, imxrt1050::gpt::Gpt1<'static>>,
    >,
    button: &'static capsules_core::button::Button<'static, imxrt1050::gpio::Pin<'static>>,
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, imxrt1050::gpio::Pin<'static>>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, imxrt1050::gpio::Pin<'static>>,
        1,
    >,
    ninedof: &'static capsules_extra::ninedof::NineDof<'static>,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm7::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for Imxrt1050EVKB {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_extra::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            _ => f(None),
        }
    }
}

impl KernelResources<imxrt1050::chip::Imxrt10xx<imxrt1050::chip::Imxrt10xxDefaultPeripherals>>
    for Imxrt1050EVKB
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm7::systick::SysTick;
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

/// Helper function called during bring-up that configures DMA.
/// DMA for imxrt1050-evkb is not implemented yet.
// unsafe fn setup_dma() {
// }

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions(
    peripherals: &'static imxrt1050::chip::Imxrt10xxDefaultPeripherals,
) {
    use imxrt1050::gpio::PinId;

    peripherals.ccm.enable_iomuxc_clock();
    peripherals.ccm.enable_iomuxc_snvs_clock();

    peripherals.ports.gpio1.enable_clock();

    // User_LED is connected to GPIO_AD_B0_09.
    // Values set accordingly to the evkbimxrt1050_iled_blinky SDK example

    // First we configure the pin in GPIO mode and disable the Software Input
    // on Field, so that the Input Path is determined by functionality.
    peripherals.iomuxc.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT5, // ALT5 for AdB0_09: GPIO1_IO09 of instance: gpio1
        Sion::Disabled,
        9,
    );

    // Configure the pin resistance value, pull up or pull down and other
    // physical aspects.
    peripherals.iomuxc.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        9,
        PullUpDown::Pus0_100kOhmPullDown,   // 100K Ohm Pull Down
        PullKeepEn::Pke1PullKeeperEnabled,  // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode0OpenDrainDisabled, // Output is CMOS, either 0 logic or 1 logic
        Speed::Medium2,                     // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    // Configuring the GPIO_AD_B0_09 as output
    let pin = peripherals.ports.pin(PinId::AdB0_09);
    pin.make_output();
    kernel::debug::assign_gpios(Some(pin), None, None);

    // User_Button is connected to IOMUXC_SNVS_WAKEUP.
    peripherals.ports.gpio5.enable_clock();

    // We configure the pin in GPIO mode and disable the Software Input
    // on Field, so that the Input Path is determined by functionality.
    peripherals.iomuxc_snvs.enable_sw_mux_ctl_pad_gpio(
        MuxMode::ALT5, // ALT5 for AdB0_09: GPIO5_IO00 of instance: gpio5
        Sion::Disabled,
        0,
    );

    // Configuring the IOMUXC_SNVS_WAKEUP pin as input
    peripherals.ports.pin(PinId::Wakeup).make_input();
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals(peripherals: &imxrt1050::chip::Imxrt10xxDefaultPeripherals) {
    // LPUART1 IRQn is 20
    cortexm7::nvic::Nvic::new(imxrt1050::nvic::LPUART1).enable();

    // TIM2 IRQn is 28
    peripherals.gpt1.enable_clock();
    peripherals.gpt1.start(
        peripherals.ccm.perclk_sel(),
        peripherals.ccm.perclk_divider(),
    );
    cortexm7::nvic::Nvic::new(imxrt1050::nvic::GPT1).enable();
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    Imxrt1050EVKB,
    &'static imxrt1050::chip::Imxrt10xx<imxrt1050::chip::Imxrt10xxDefaultPeripherals>,
) {
    imxrt1050::init();

    let ccm = static_init!(imxrt1050::ccm::Ccm, imxrt1050::ccm::Ccm::new());
    let peripherals = static_init!(
        imxrt1050::chip::Imxrt10xxDefaultPeripherals,
        imxrt1050::chip::Imxrt10xxDefaultPeripherals::new(ccm)
    );
    peripherals.ccm.set_low_power_mode();
    peripherals.lpuart1.disable_clock();
    peripherals.lpuart2.disable_clock();
    peripherals
        .ccm
        .set_uart_clock_sel(imxrt1050::ccm::UartClockSelection::PLL3);
    peripherals.ccm.set_uart_clock_podf(1);
    peripherals.lpuart1.set_baud();

    set_pin_primary_functions(peripherals);

    setup_peripherals(peripherals);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    let chip = static_init!(Chip, Chip::new(peripherals));
    CHIP = Some(chip);

    // LPUART1

    // Enable tx and rx from iomuxc
    // TX is on pad GPIO_AD_B0_12
    // RX is on pad GPIO_AD_B0_13
    // Values set accordingly to the evkbimxrt1050_hello_world SDK example

    // First we configure the pin in LPUART mode and disable the Software Input
    // on Field, so that the Input Path is determined by functionality.
    peripherals.iomuxc.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT2, // ALT2: LPUART1_TXD of instance: lpuart1
        Sion::Disabled,
        13,
    );
    peripherals.iomuxc.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT2, // ALT2: LPUART1_RXD of instance: lpuart1
        Sion::Disabled,
        14,
    );

    // Configure the pin resistance value, pull up or pull down and other
    // physical aspects.
    peripherals.iomuxc.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        13,
        PullUpDown::Pus0_100kOhmPullDown,   // 100K Ohm Pull Down
        PullKeepEn::Pke1PullKeeperEnabled,  // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode0OpenDrainDisabled, // Output is CMOS, either 0 logic or 1 logic
        Speed::Medium2,                     // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );
    peripherals.iomuxc.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        14,
        PullUpDown::Pus0_100kOhmPullDown,   // 100K Ohm Pull Down
        PullKeepEn::Pke1PullKeeperEnabled,  // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode0OpenDrainDisabled, // Output is CMOS, either 0 logic or 1 logic
        Speed::Medium2,                     // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    // Enable clock
    peripherals.lpuart1.enable_clock();

    let lpuart_mux = components::console::UartMuxComponent::new(&peripherals.lpuart1, 115200)
        .finalize(components::uart_mux_component_static!());
    (*addr_of_mut!(io::WRITER)).set_initialized();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        lpuart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(
        lpuart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // LEDs

    // Clock to Port A is enabled in `set_pin_primary_functions()
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, imxrt1050::gpio::Pin<'static>>,
        LedLow::new(peripherals.ports.pin(imxrt1050::gpio::PinId::AdB0_09)),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            imxrt1050::gpio::Pin,
            (
                peripherals.ports.pin(imxrt1050::gpio::PinId::Wakeup),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullDown
            )
        ),
    )
    .finalize(components::button_component_static!(imxrt1050::gpio::Pin));

    // ALARM
    let gpt1 = &peripherals.gpt1;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(gpt1).finalize(
        components::alarm_mux_component_static!(imxrt1050::gpt::Gpt1),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(imxrt1050::gpt::Gpt1));

    // GPIO
    // For now we expose only two pins
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            imxrt1050::gpio::Pin<'static>,
            // The User Led
            0 => peripherals.ports.pin(imxrt1050::gpio::PinId::AdB0_09)
        ),
    )
    .finalize(components::gpio_component_static!(
        imxrt1050::gpio::Pin<'static>
    ));

    // LPI2C
    // AD_B1_00 is LPI2C1_SCL
    // AD_B1_01 is LPI2C1_SDA
    // Values set accordingly to the evkbimxrt1050_bubble_peripheral SDK example

    // First we configure the pin in LPUART mode and enable the Software Input
    // on Field, so that we force input path of the pad.
    peripherals.iomuxc.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB1,
        MuxMode::ALT3, // ALT3:  LPI2C1_SCL of instance: lpi2c1
        Sion::Enabled,
        0,
    );
    // Selecting AD_B1_00 for LPI2C1_SCL in the Daisy Chain.
    peripherals.iomuxc.enable_lpi2c_scl_select_input();

    peripherals.iomuxc.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB1,
        MuxMode::ALT3, // ALT3:  LPI2C1_SDA of instance: lpi2c1
        Sion::Enabled,
        1,
    );
    // Selecting AD_B1_01 for LPI2C1_SDA in the Daisy Chain.
    peripherals.iomuxc.enable_lpi2c_sda_select_input();

    // Configure the pin resistance value, pull up or pull down and other
    // physical aspects.
    peripherals.iomuxc.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB1,
        0,
        PullUpDown::Pus3_22kOhmPullUp,     // 22K Ohm Pull Up
        PullKeepEn::Pke1PullKeeperEnabled, // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode1OpenDrainEnabled, // Open Drain Enabled (Output is Open Drain)
        Speed::Medium2,                    // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    peripherals.iomuxc.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB1,
        1,
        PullUpDown::Pus3_22kOhmPullUp,     // 22K Ohm Pull Up
        PullKeepEn::Pke1PullKeeperEnabled, // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode1OpenDrainEnabled, // Open Drain Enabled (Output is Open Drain)
        Speed::Medium2,                    // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    // Enabling the lpi2c1 clock and setting the speed.
    peripherals.lpi2c1.enable_clock();
    peripherals
        .lpi2c1
        .set_speed(imxrt1050::lpi2c::Lpi2cSpeed::Speed100k, 8);

    use imxrt1050::gpio::PinId;
    let mux_i2c = components::i2c::I2CMuxComponent::new(&peripherals.lpi2c1, None).finalize(
        components::i2c_mux_component_static!(imxrt1050::lpi2c::Lpi2c),
    );

    // Fxos8700 sensor
    let fxos8700 = components::fxos8700::Fxos8700Component::new(
        mux_i2c,
        0x1f,
        peripherals.ports.pin(PinId::AdB1_00),
    )
    .finalize(components::fxos8700_component_static!(
        imxrt1050::lpi2c::Lpi2c
    ));

    // Ninedof
    let ninedof = components::ninedof::NineDofComponent::new(
        board_kernel,
        capsules_extra::ninedof::DRIVER_NUM,
    )
    .finalize(components::ninedof_component_static!(fxos8700));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let imxrt1050 = Imxrt1050EVKB {
        console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        led,
        button,
        ninedof,
        alarm,
        gpio,

        scheduler,
        systick: cortexm7::systick::SysTick::new_with_calibration(792_000_000),
    };

    // Optional kernel tests
    //
    // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

    //--------------------------------------------------------------------------
    // Process Console
    //---------------------------------------------------------------------------
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        lpuart_mux,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(
        imxrt1050::gpt::Gpt1
    ));
    let _ = process_console.start();

    debug!("Tock OS initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
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

    (board_kernel, imxrt1050, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, board, chip) = start();
    board_kernel.kernel_loop(&board, chip, Some(&board.ipc), &main_loop_capability);
}
