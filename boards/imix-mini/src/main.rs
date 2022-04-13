//! Minimal board file for Imix development platform.
//!
//! - <https://github.com/tock/tock/tree/master/boards/imix>
//! - <https://github.com/tock/imix>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

mod imix_components;
use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::hil::led::LedHigh;
use kernel::hil::Controller;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};
use sam4l::chip::Sam4lDefaultPeripherals;

use components;
use components::alarm::{AlarmDriverComponent, AlarmMuxComponent};
use components::led::LedsComponent;

/// Support routines for debugging I/O.
///
/// Note: Use of this module will trample any other USART3 configuration.
pub mod io;

// Helper functions for enabling/disabling power on Imix submodules
mod power;

// State for loading apps.

const NUM_PROCS: usize = 4;

// how should the kernel respond when a process faults
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static sam4l::chip::Sam4l<Sam4lDefaultPeripherals>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

struct Imix {
    console: &'static capsules::console::Console<'static>,
    alarm: &'static AlarmDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    led:
        &'static capsules::led::LedDriver<'static, LedHigh<'static, sam4l::gpio::GPIOPin<'static>>>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for Imix {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            _ => f(None),
        }
    }
}

impl KernelResources<sam4l::chip::Sam4l<Sam4lDefaultPeripherals>> for Imix {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        &self
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

unsafe fn set_pin_primary_functions(peripherals: &Sam4lDefaultPeripherals) {
    use sam4l::gpio::PeripheralFunction::{A, B, C, E};

    // Right column: Imix pin name
    // Left  column: SAM4L peripheral function
    peripherals.pa[04].configure(Some(A)); // AD0         --  ADCIFE AD0
    peripherals.pa[05].configure(Some(A)); // AD1         --  ADCIFE AD1
    peripherals.pa[06].configure(Some(C)); // EXTINT1     --  EIC EXTINT1
    peripherals.pa[07].configure(Some(A)); // AD1         --  ADCIFE AD2
    peripherals.pa[08].configure(None); //... RF233 IRQ   --  GPIO pin
    peripherals.pa[09].configure(None); //... RF233 RST   --  GPIO pin
    peripherals.pa[10].configure(None); //... RF233 SLP   --  GPIO pin
    peripherals.pa[13].configure(None); //... TRNG EN     --  GPIO pin
    peripherals.pa[14].configure(None); //... TRNG_OUT    --  GPIO pin
    peripherals.pa[17].configure(None); //... NRF INT     -- GPIO pin
    peripherals.pa[18].configure(Some(A)); // NRF CLK     -- USART2_CLK
    peripherals.pa[20].configure(None); //... D8          -- GPIO pin
    peripherals.pa[21].configure(Some(E)); // TWI2 SDA    -- TWIM2_SDA
    peripherals.pa[22].configure(Some(E)); // TWI2 SCL    --  TWIM2 TWCK
    peripherals.pa[25].configure(Some(A)); // USB_N       --  USB DM
    peripherals.pa[26].configure(Some(A)); // USB_P       --  USB DP
    peripherals.pb[00].configure(Some(A)); // TWI1_SDA    --  TWIMS1 TWD
    peripherals.pb[01].configure(Some(A)); // TWI1_SCL    --  TWIMS1 TWCK
    peripherals.pb[02].configure(Some(A)); // AD3         --  ADCIFE AD3
    peripherals.pb[03].configure(Some(A)); // AD4         --  ADCIFE AD4
    peripherals.pb[04].configure(Some(A)); // AD5         --  ADCIFE AD5
    peripherals.pb[05].configure(Some(A)); // VHIGHSAMPLE --  ADCIFE AD6
    peripherals.pb[06].configure(Some(A)); // RTS3        --  USART3 RTS
    peripherals.pb[07].configure(None); //... NRF RESET   --  GPIO
    peripherals.pb[09].configure(Some(A)); // RX3         --  USART3 RX
    peripherals.pb[10].configure(Some(A)); // TX3         --  USART3 TX
    peripherals.pb[11].configure(Some(A)); // CTS0        --  USART0 CTS
    peripherals.pb[12].configure(Some(A)); // RTS0        --  USART0 RTS
    peripherals.pb[13].configure(Some(A)); // CLK0        --  USART0 CLK
    peripherals.pb[14].configure(Some(A)); // RX0         --  USART0 RX
    peripherals.pb[15].configure(Some(A)); // TX0         --  USART0 TX
    peripherals.pc[00].configure(Some(A)); // CS2         --  SPI Nperipherals.pcS2
    peripherals.pc[01].configure(Some(A)); // CS3 (RF233) --  SPI Nperipherals.pcS3
    peripherals.pc[02].configure(Some(A)); // CS1         --  SPI Nperipherals.pcS1
    peripherals.pc[03].configure(Some(A)); // CS0         --  SPI Nperipherals.pcS0
    peripherals.pc[04].configure(Some(A)); // MISO        --  SPI MISO
    peripherals.pc[05].configure(Some(A)); // MOSI        --  SPI MOSI
    peripherals.pc[06].configure(Some(A)); // SCK         --  SPI CLK
    peripherals.pc[07].configure(Some(B)); // RTS2 (BLE)  -- USART2_RTS
    peripherals.pc[08].configure(Some(E)); // CTS2 (BLE)  -- USART2_CTS
                                           //peripherals.pc[09].configure(None); //... NRF GPIO    -- GPIO
                                           //peripherals.pc[10].configure(None); //... USER LED    -- GPIO
    peripherals.pc[09].configure(Some(E)); // ACAN1       -- ACIFC comparator
    peripherals.pc[10].configure(Some(E)); // ACAP1       -- ACIFC comparator
    peripherals.pc[11].configure(Some(B)); // RX2 (BLE)   -- USART2_RX
    peripherals.pc[12].configure(Some(B)); // TX2 (BLE)   -- USART2_TX
                                           //peripherals.pc[13].configure(None); //... ACC_INT1    -- GPIO
                                           //peripherals.pc[14].configure(None); //... ACC_INT2    -- GPIO
    peripherals.pc[13].configure(Some(E)); //... ACBN1    -- ACIFC comparator
    peripherals.pc[14].configure(Some(E)); //... ACBP1    -- ACIFC comparator
    peripherals.pc[16].configure(None); //... SENSE_PWR   --  GPIO pin
    peripherals.pc[17].configure(None); //... NRF_PWR     --  GPIO pin
    peripherals.pc[18].configure(None); //... RF233_PWR   --  GPIO pin
    peripherals.pc[19].configure(None); //... TRNG_PWR    -- GPIO Pin
    peripherals.pc[22].configure(None); //... KERNEL LED  -- GPIO Pin
    peripherals.pc[24].configure(None); //... USER_BTN    -- GPIO Pin
    peripherals.pc[25].configure(Some(B)); // LI_INT      --  EIC EXTINT2
    peripherals.pc[26].configure(None); //... D7          -- GPIO Pin
    peripherals.pc[27].configure(None); //... D6          -- GPIO Pin
    peripherals.pc[28].configure(None); //... D5          -- GPIO Pin
    peripherals.pc[29].configure(None); //... D4          -- GPIO Pin
    peripherals.pc[30].configure(None); //... D3          -- GPIO Pin
    peripherals.pc[31].configure(None); //... D2          -- GPIO Pin
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn get_peripherals(
    pm: &'static sam4l::pm::PowerManager,
) -> &'static Sam4lDefaultPeripherals {
    static_init!(Sam4lDefaultPeripherals, Sam4lDefaultPeripherals::new(pm))
}

/// Main function.
///
/// This is called after RAM initialization is complete.
#[no_mangle]
pub unsafe fn main() {
    sam4l::init();
    let pm = static_init!(sam4l::pm::PowerManager, sam4l::pm::PowerManager::new());
    let peripherals = get_peripherals(pm);

    pm.setup_system_clock(
        sam4l::pm::SystemClockSource::PllExternalOscillatorAt48MHz {
            frequency: sam4l::pm::OscillatorFrequency::Frequency16MHz,
            startup_mode: sam4l::pm::OscillatorStartup::FastStart,
        },
        &peripherals.flash_controller,
    );

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions(peripherals);

    peripherals.setup_dma();
    let chip = static_init!(
        sam4l::chip::Sam4l<Sam4lDefaultPeripherals>,
        sam4l::chip::Sam4l::new(pm, peripherals)
    );
    CHIP = Some(chip);

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let main_cap = create_capability!(capabilities::MainLoopCapability);

    power::configure_submodules(
        &peripherals.pa,
        &peripherals.pb,
        &peripherals.pc,
        power::SubmoduleConfig {
            rf233: true,
            nrf51422: true,
            sensors: true,
            trng: true,
        },
    );

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 5], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // # CONSOLE
    peripherals.usart3.set_mode(sam4l::usart::UsartMode::Uart);
    use kernel::hil::uart;
    use kernel::hil::uart::Configure;
    let _ = peripherals.usart3.configure(uart::Parameters {
        baud_rate: 115200,
        width: uart::Width::Eight,
        stop_bits: uart::StopBits::One,
        parity: uart::Parity::None,
        hw_flow_control: false,
    });
    components::debug_writer::DebugWriterNoMuxComponent::new(&peripherals.usart3).finalize(());

    //let uart_mux =
    //    UartMuxComponent::new(&peripherals.usart3, 115200, dynamic_deferred_caller).finalize(());
    //let console =
    //    ConsoleComponent::new(board_kernel, capsules::console::DRIVER_NUM, uart_mux).finalize(());
    //DebugWriterComponent::new(uart_mux).finalize(());

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let console = static_init!(
        capsules::console::Console<'static>,
        capsules::console::Console::new(
            &peripherals.usart3,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(capsules::console::DRIVER_NUM, &grant_cap)
        )
    );
    use kernel::hil;
    hil::uart::Transmit::set_transmit_client(&peripherals.usart3, console);
    // NOTE: no receive client set

    // # TIMER
    let mux_alarm = AlarmMuxComponent::new(&peripherals.ast)
        .finalize(components::alarm_mux_component_helper!(sam4l::ast::Ast));
    peripherals.ast.configure(mux_alarm);
    let alarm = AlarmDriverComponent::new(board_kernel, capsules::alarm::DRIVER_NUM, mux_alarm)
        .finalize(components::alarm_component_helper!(sam4l::ast::Ast));

    let led = LedsComponent::new(components::led_component_helper!(
        LedHigh<'static, sam4l::gpio::GPIOPin>,
        LedHigh::new(&peripherals.pc[10]),
    ))
    .finalize(components::led_component_buf!(
        LedHigh<'static, sam4l::gpio::GPIOPin>
    ));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    let imix = Imix {
        console,
        alarm,
        led,
        scheduler,
        systick: cortexm4::systick::SysTick::new(),
    };

    // Optional kernel test. Note that these might conflict
    // with normal operation (e.g., steal callbacks from drivers, etc.),
    // so do not run these and expect all services/applications to work.
    // Once everything is virtualized in the kernel this won't be a problem.
    // -pal, 11/20/18
    //
    //test::virtual_uart_rx_test::run_virtual_uart_receive(uart_mux);

    debug!("Initialization complete. Entering main loop");

    /// These symbols are defined in the linker script.
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
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop::<_, _, NUM_PROCS, 0>(&imix, chip, None, &main_cap);
}
