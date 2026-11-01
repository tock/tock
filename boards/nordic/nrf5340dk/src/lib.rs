// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board file for the Tock kernel on the Nordic Semiconductor nRF5340
//! development kit, shared by the kernels built from it.
//!
//! The kernel runs on the application core (Cortex-M33), in secure state, or in
//! non-secure state behind the secure stage of the `nrf5340dk-ns`
//! configuration. In secure state, it releases the network core from reset at
//! boot; in non-secure state, the secure stage has done that.

#![no_std]
#![deny(missing_docs)]

use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil::led::LedLow;
use kernel::hil::time::Counter;
use kernel::hil::usb::Client;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::StaticRef;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{capabilities, create_capability, debug, static_init};
use nrf5340::ble_ipc::{BleMailboxRegisters, BleProxy};
use nrf5340::chip::Nrf5340DefaultPeripherals;
use nrf5340::cryptocell::SecureCryptoCell;
use nrf5340::gpio::Pin;
use nrf5340::pinmux::Pinmux;

// The nRF5340 DK LEDs and buttons, all active low.
const LED1_PIN: Pin = Pin::P0_28;
const LED2_PIN: Pin = Pin::P0_29;
const LED3_PIN: Pin = Pin::P0_30;
const LED4_PIN: Pin = Pin::P0_31;
const BUTTON1_PIN: Pin = Pin::P0_23;
const BUTTON2_PIN: Pin = Pin::P0_24;
const BUTTON3_PIN: Pin = Pin::P0_08;
const BUTTON4_PIN: Pin = Pin::P0_09;

/// UART transmit pin, connected to the J-Link virtual COM port.
pub const UART_TXD: Pin = Pin::P0_20;
/// UART receive pin, connected to the J-Link virtual COM port.
pub const UART_RXD: Pin = Pin::P0_22;

// The network core's console UART, on the other J-Link virtual COM port.
const NET_UART_TXD: Pin = Pin::P1_01;
const NET_UART_RXD: Pin = Pin::P1_00;

// The DK's MX25R6435F external flash, on the QSPI pins, driven by SPIM1.
const FLASH_MOSI: Pin = Pin::P0_13;
const FLASH_MISO: Pin = Pin::P0_14;
const FLASH_WRITE_PROTECT: Pin = Pin::P0_15;
const FLASH_HOLD: Pin = Pin::P0_16;
const FLASH_SCK: Pin = Pin::P0_17;
const FLASH_CS: Pin = Pin::P0_18;

/// The BLE mailbox: the last 8 KB of RAM, which `layout.ld` leaves out of the
/// kernel's RAM and this kernel shares with the network core. It must match
/// `../nrf5340dk-net`.
const BLE_MAILBOX_ADDRESS: usize = 0x2007E000;
// # Safety
//
// This memory belongs to neither kernel, and only the BLE IPC drivers access
// it, through volatile reads and writes.
const BLE_MAILBOX: StaticRef<BleMailboxRegisters> =
    unsafe { StaticRef::new(BLE_MAILBOX_ADDRESS as *const BleMailboxRegisters) };

/// UART Writer
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

/// The USB CDC-ACM console, a second instance of the console driver.
const CONSOLE2_DRIVER_NUM: usize = (1 << 24) | capsules_core::console::DRIVER_NUM;

/// The chip type of this board.
pub type ChipHw = nrf5340::chip::Nrf5340<'static, Nrf5340DefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x1000}

type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type AdcHw = nrf5340::adc::Adc<'static>;
type AlarmHw = nrf5340::rtc::Rtc<'static>;
type BleHw = BleProxy<'static>;
type CryptoHw = nrf5340::cryptocell::CryptoCell<'static>;
type GpioHw = nrf5340::gpio::GPIOPin<'static>;
type SpiHw = nrf5340::spi::SPIM<'static>;
type LedHw = LedLow<'static, GpioHw>;

type AdcDriver = components::adc::AdcDedicatedComponentType<AdcHw>;
type AlarmDriver = components::alarm::AlarmDriverComponentType<AlarmHw>;
type BleDriver = components::ble::BLEComponentType<BleHw, AlarmHw>;
type LedDriver = components::led::LedsComponentType<LedHw, 4>;
type ButtonDriver = components::button::ButtonComponentType<GpioHw>;
type ConsoleDriver = components::console::ConsoleComponentType;
type RngDriver = components::rng::RngComponentType<CryptoHw>;
type ShaDriver = components::sha::ShaDriverComponentType<CryptoHw, 32>;

// TicKV on the external flash.
type Mx25r6435f = components::mx25r6435f::Mx25r6435fComponentType<SpiHw, GpioHw, AlarmHw>;
const TICKV_PAGE_SIZE: usize =
    core::mem::size_of::<<Mx25r6435f as kernel::hil::flash::Flash>::Page>();
type Siphasher24 = components::siphash::Siphasher24ComponentType;
type TicKVDedicatedFlash =
    components::tickv::TicKVDedicatedFlashComponentType<Mx25r6435f, Siphasher24, TICKV_PAGE_SIZE>;
type TicKVKVStore = components::kv::TicKVKVStoreComponentType<
    TicKVDedicatedFlash,
    capsules_extra::tickv::TicKVKeyType,
>;
type KVStorePermissions = components::kv::KVStorePermissionsComponentType<TicKVKVStore>;
type VirtualKVPermissions = components::kv::VirtualKVPermissionsComponentType<KVStorePermissions>;
type KVDriver = components::kv::KVDriverComponentType<VirtualKVPermissions>;
kernel::define_capability_type!(ProcessConsoleCap:
    kernel::capabilities::ProcessManagementCapability,
    kernel::capabilities::ProcessStartCapability
);
type ProcessConsoleDriver =
    components::process_console::ProcessConsoleComponentType<AlarmHw, ProcessConsoleCap>;

/// Supported drivers by the platform
pub struct Platform {
    adc: &'static AdcDriver,
    alarm: &'static AlarmDriver,
    ble_radio: &'static BleDriver,
    button: &'static ButtonDriver,
    console: &'static ConsoleDriver,
    console2: &'static ConsoleDriver,
    kv_driver: &'static KVDriver,
    led: &'static LedDriver,
    pconsole: &'static ProcessConsoleDriver,
    rng: &'static RngDriver,
    scheduler: &'static SchedulerInUse,
    sha: &'static ShaDriver,
    systick: cortexm33::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            CONSOLE2_DRIVER_NUM => f(Some(self.console2)),
            capsules_extra::kv_driver::DRIVER_NUM => f(Some(self.kv_driver)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::sha256_driver::DRIVER_NUM => f(Some(self.sha)),
            _ => f(None),
        }
    }
}

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = cortexm33::systick::SysTick;
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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
///
/// In non-secure state, the CryptoCell runs through `gateway`.
#[inline(never)]
pub unsafe fn start(
    gateway: Option<&'static dyn SecureCryptoCell>,
) -> (&'static kernel::Kernel, Platform, &'static ChipHw) {
    ChipHw::init();

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Bind global variables to this thread.
    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    // SAFETY: This is the only copy and only user of the DMA peripherals.
    let peripherals = static_init!(Nrf5340DefaultPeripherals, unsafe {
        Nrf5340DefaultPeripherals::new()
    });
    peripherals.init();
    if let Some(gateway) = gateway {
        peripherals.cryptocell.set_gateway(gateway);
    }
    // In non-secure state, the secure stage has done this.
    if cortexm33::trustzone::is_secure() {
        // The DK's 32.768 kHz crystal, as Nordic configures it.
        peripherals
            .lfxo
            .connect(nrf5340::lfxo::LoadCapacitance::Internal7pF);
        peripherals
            .spu
            .make_ram_nonsecure(BLE_MAILBOX_ADDRESS..BLE_MAILBOX_ADDRESS + 0x2000);
        peripherals.network.start(&[NET_UART_TXD, NET_UART_RXD]);
    }
    peripherals.clock.start(nrf5340::clock::LfclkSource::Lfxo);

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let gpio_port = &peripherals.gpio_port;

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            GpioHw,
            (
                &gpio_port[BUTTON1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &gpio_port[BUTTON2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &gpio_port[BUTTON3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &gpio_port[BUTTON4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::button_component_static!(GpioHw));

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHw,
        LedLow::new(&gpio_port[LED1_PIN]),
        LedLow::new(&gpio_port[LED2_PIN]),
        LedLow::new(&gpio_port[LED3_PIN]),
        LedLow::new(&gpio_port[LED4_PIN]),
    ));

    let chip = static_init!(ChipHw, nrf5340::chip::Nrf5340::new(peripherals));
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let rtc = &peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(AlarmHw));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(AlarmHw));

    // The Arduino analog pins A0 to A5.
    let adc_channels = static_init!(
        [nrf5340::adc::AdcChannelSetup; 6],
        [
            nrf5340::adc::AdcChannelSetup::new(nrf5340::adc::AdcChannel::AnalogInput0),
            nrf5340::adc::AdcChannelSetup::new(nrf5340::adc::AdcChannel::AnalogInput1),
            nrf5340::adc::AdcChannelSetup::new(nrf5340::adc::AdcChannel::AnalogInput2),
            nrf5340::adc::AdcChannelSetup::new(nrf5340::adc::AdcChannel::AnalogInput3),
            nrf5340::adc::AdcChannelSetup::new(nrf5340::adc::AdcChannel::AnalogInput4),
            nrf5340::adc::AdcChannelSetup::new(nrf5340::adc::AdcChannel::AnalogInput5),
        ]
    );
    let adc = components::adc::AdcDedicatedComponent::new(
        &peripherals.adc,
        adc_channels,
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::adc_dedicated_component_static!(AdcHw));
    // Measure VDD, the reference voltage readings are relative to.
    peripherals.adc.calibrate();

    // The CryptoCell's TRNG and SHA-256.
    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &peripherals.cryptocell,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::rng_component_static!(CryptoHw));
    let sha = components::sha::ShaDriverComponent::new(
        board_kernel,
        capsules_extra::sha256_driver::DRIVER_NUM,
        &peripherals.cryptocell,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::sha_driver_component_static!(CryptoHw, 32));

    // The external flash, with a TicKV key-value store over its first 32
    // sectors.
    let mux_spi = components::spi::SpiMuxComponent::new(&peripherals.spim1)
        .finalize(components::spi_mux_component_static!(SpiHw));
    peripherals.spim1.configure(
        Pinmux::new(FLASH_MOSI),
        Pinmux::new(FLASH_MISO),
        Pinmux::new(FLASH_SCK),
    );
    let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
        Some(&gpio_port[FLASH_WRITE_PROTECT]),
        Some(&gpio_port[FLASH_HOLD]),
        &gpio_port[FLASH_CS],
        mux_alarm,
        mux_spi,
    )
    .finalize(components::mx25r6435f_component_static!(
        SpiHw, GpioHw, AlarmHw
    ));
    let page_buffer = static_init!(
        <Mx25r6435f as kernel::hil::flash::Flash>::Page,
        <Mx25r6435f as kernel::hil::flash::Flash>::Page::default()
    );
    let sip_hash = components::siphash::Siphasher24Component::new()
        .finalize(components::siphasher24_component_static!());
    let tickv = components::tickv::TicKVDedicatedFlashComponent::new(
        sip_hash,
        mx25r6435f,
        0,
        (capsules_extra::mx25r6435f::SECTOR_SIZE as usize) * 32,
        page_buffer,
    )
    .finalize(components::tickv_dedicated_flash_component_static!(
        Mx25r6435f,
        Siphasher24,
        TICKV_PAGE_SIZE,
    ));
    let tickv_kv_store = components::kv::TicKVKVStoreComponent::new(tickv).finalize(
        components::tickv_kv_store_component_static!(
            TicKVDedicatedFlash,
            capsules_extra::tickv::TicKVKeyType,
        ),
    );
    let kv_store_permissions = components::kv::KVStorePermissionsComponent::new(tickv_kv_store)
        .finalize(components::kv_store_permissions_component_static!(
            TicKVKVStore
        ));
    let mux_kv = components::kv::KVPermissionsMuxComponent::new(kv_store_permissions).finalize(
        components::kv_permissions_mux_component_static!(KVStorePermissions),
    );
    let virtual_kv_driver = components::kv::VirtualKVPermissionsComponent::new(mux_kv).finalize(
        components::virtual_kv_permissions_component_static!(KVStorePermissions),
    );
    let kv_driver = components::kv::KVDriverComponent::new(
        virtual_kv_driver,
        board_kernel,
        capsules_extra::kv_driver::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::kv_driver_component_static!(
        VirtualKVPermissions
    ));

    // BLE advertising, through the radio on the network core.
    let ble_proxy = static_init!(BleHw, BleProxy::new(BLE_MAILBOX, &peripherals.ipc));
    ble_proxy.init();
    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        capsules_extra::ble_advertising_driver::DRIVER_NUM,
        ble_proxy,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::ble_component_static!(AlarmHw, BleHw));

    peripherals
        .uarte0
        .initialize(Pinmux::new(UART_TXD), Pinmux::new(UART_RXD), None, None);

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uarte0, 115200)
        .finalize(components::uart_mux_component_static!());

    let process_console_cap = unsafe { kernel::mint_defined_capability!(ProcessConsoleCap) };
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm33::support::reset),
        process_console_cap,
    )
    .finalize(components::process_console_component_static!(
        AlarmHw,
        ProcessConsoleCap,
    ));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // USB CDC-ACM on the nRF5340 USB connector, as a second console.
    let strings = static_init!(
        [&str; 3],
        [
            "Tock",               // Manufacturer
            "nRF5340DK - TockOS", // Product
            "000000000000",       // Serial number
        ]
    );
    let cdc = components::cdc::CdcAcmComponent::new(
        &peripherals.usbd,
        capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x2341,
        0x005a,
        strings,
        mux_alarm,
        None,
    )
    .finalize(components::cdc_acm_component_static!(
        nrf5340::usbd::Usbd,
        nrf5340::rtc::Rtc
    ));
    let uart_mux2 = components::console::UartMuxComponent::new(cdc, 115200)
        .finalize(components::uart_mux_component_static!());
    let console2 = components::console::ConsoleComponent::new(
        board_kernel,
        CONSOLE2_DRIVER_NUM,
        uart_mux2,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());
    cdc.enable();
    cdc.attach();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        adc,
        alarm,
        ble_radio,
        button,
        console,
        console2,
        kv_driver,
        led,
        pconsole,
        rng,
        scheduler,
        sha,
        // The application core runs at 64 MHz after reset.
        systick: cortexm33::systick::SysTick::new_with_calibration(64_000_000),
    };

    let _ = platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");

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

    // Load processes without checking credentials, and give each one a fixed
    // ShortId from its name, so that it can keep its own data in the KV store.
    let checking_policy = components::appid::checker_null::AppCheckerNullComponent::new()
        .finalize(components::app_checker_null_component_static!());
    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());
    kernel::create_typed_capability!(app_store_cap, AppStoreCap: kernel::capabilities::ApplicationStorageCapability);
    let storage_permissions_policy =
        components::storage_permissions::individual::StoragePermissionsIndividualComponent::new(
            app_store_cap,
        )
        .finalize(
            components::storage_permissions_individual_component_static!(
                ChipHw,
                kernel::process::ProcessStandardDebugFull,
                AppStoreCap,
            ),
        );
    let _loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        process_management_capability,
    )
    .finalize(components::process_loader_sequential_component_static!(
        ChipHw,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS
    ));

    (board_kernel, platform, chip)
}
