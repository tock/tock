//! Board file for a LiteX SoC running in a Verilated simulation

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::hil::led::LedHigh;
use kernel::hil::time::{Alarm, Timer};
use kernel::platform::chip::InterruptService;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::cooperative::CooperativeSched;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::StaticRef;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

mod io;
mod litex_generated_constants;

// This module contains the LiteX SoC configuration options, register
// positions, interrupt mappings and other implementation details of
// the generated bitstream.
//
// Its values are used throughout the file, hence import it under a
// short name.
use litex_generated_constants as socc;

/// Structure for dynamic interrupt mapping, depending on the SoC
/// configuration
///
/// This struct is deliberately kept in the board crate. Because of
/// the configurable nature of LiteX, it does not make sense to define
/// a default interrupt mapping, as the interrupt numbers are
/// generated sequentially for all softcores.
struct LiteXSimInterruptablePeripherals {
    gpio0: &'static litex_vexriscv::gpio::LiteXGPIOController<'static, socc::SoCRegisterFmt>,
    uart0: &'static litex_vexriscv::uart::LiteXUart<'static, socc::SoCRegisterFmt>,
    timer0: &'static litex_vexriscv::timer::LiteXTimer<
        'static,
        socc::SoCRegisterFmt,
        socc::ClockFrequency,
    >,
    ethmac0: &'static litex_vexriscv::liteeth::LiteEth<'static, socc::SoCRegisterFmt>,
}

impl InterruptService<()> for LiteXSimInterruptablePeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt as usize {
            socc::UART_INTERRUPT => {
                self.uart0.service_interrupt();
                true
            }
            socc::TIMER0_INTERRUPT => {
                self.timer0.service_interrupt();
                true
            }
            socc::ETHMAC_INTERRUPT => {
                self.ethmac0.service_interrupt();
                true
            }
            socc::GPIO_INTERRUPT => {
                self.gpio0.service_interrupt();
                true
            }
            _ => false,
        }
    }

    unsafe fn service_deferred_call(&self, _task: ()) -> bool {
        false
    }
}

const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an
// empty list at least.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip and UART hardware for panic dumps
struct LiteXSimPanicReferences {
    chip: Option<&'static litex_vexriscv::chip::LiteXVexRiscv<LiteXSimInterruptablePeripherals>>,
    uart: Option<&'static litex_vexriscv::uart::LiteXUart<'static, socc::SoCRegisterFmt>>,
    process_printer: Option<&'static kernel::process::ProcessPrinterText>,
}
static mut PANIC_REFERENCES: LiteXSimPanicReferences = LiteXSimPanicReferences {
    chip: None,
    uart: None,
    process_printer: None,
};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct LiteXSim {
    gpio_driver: &'static core_capsules::gpio::GPIO<
        'static,
        litex_vexriscv::gpio::LiteXGPIOPin<'static, 'static, socc::SoCRegisterFmt>,
    >,
    button_driver: &'static core_capsules::button::Button<
        'static,
        litex_vexriscv::gpio::LiteXGPIOPin<'static, 'static, socc::SoCRegisterFmt>,
    >,
    led_driver: &'static core_capsules::led::LedDriver<
        'static,
        LedHigh<
            'static,
            litex_vexriscv::gpio::LiteXGPIOPin<'static, 'static, socc::SoCRegisterFmt>,
        >,
        8,
    >,
    console: &'static core_capsules::console::Console<'static>,
    lldb: &'static core_capsules::low_level_debug::LowLevelDebug<
        'static,
        core_capsules::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static core_capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static CooperativeSched<'static>,
    scheduler_timer: &'static VirtualSchedulerTimer<
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for LiteXSim {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            core_capsules::button::DRIVER_NUM => f(Some(self.button_driver)),
            core_capsules::led::DRIVER_NUM => f(Some(self.led_driver)),
            core_capsules::gpio::DRIVER_NUM => f(Some(self.gpio_driver)),
            core_capsules::console::DRIVER_NUM => f(Some(self.console)),
            core_capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            core_capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<litex_vexriscv::chip::LiteXVexRiscv<LiteXSimInterruptablePeripherals>>
    for LiteXSim
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type CredentialsCheckingPolicy = ();
    type Scheduler = CooperativeSched<'static>;
    type SchedulerTimer = VirtualSchedulerTimer<
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >;
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
    fn credentials_checking_policy(&self) -> &'static Self::CredentialsCheckingPolicy {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// and RAM setup.
#[no_mangle]
pub unsafe fn main() {
    // ---------- BASIC INITIALIZATION ----------
    // Basic setup of the riscv platform.
    rv32i::configure_trap_handler(rv32i::PermissionMode::Machine);

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // --------- TIMER & UPTIME CORE; ALARM INITIALIZATION ----------

    // Initialize the hardware timer
    let timer0 = static_init!(
        litex_vexriscv::timer::LiteXTimer<'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        litex_vexriscv::timer::LiteXTimer::new(StaticRef::new(
            socc::CSR_TIMER0_BASE
                as *const litex_vexriscv::timer::LiteXTimerRegisters<socc::SoCRegisterFmt>
        ),)
    );

    // The SoC is expected to feature the 64-bit uptime extension to the timer hardware
    let timer0_uptime = static_init!(
        litex_vexriscv::timer::LiteXTimerUptime<
            'static,
            socc::SoCRegisterFmt,
            socc::ClockFrequency,
        >,
        litex_vexriscv::timer::LiteXTimerUptime::new(timer0)
    );

    // Create the LiteXAlarm based on the hardware LiteXTimer core and
    // the uptime peripheral
    let litex_alarm = static_init!(
        litex_vexriscv::timer::LiteXAlarm<
            'static,
            'static,
            socc::SoCRegisterFmt,
            socc::ClockFrequency,
        >,
        litex_vexriscv::timer::LiteXAlarm::new(timer0_uptime, timer0)
    );
    timer0.set_timer_client(litex_alarm);
    litex_alarm.initialize();

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        MuxAlarm::new(litex_alarm)
    );
    litex_alarm.set_alarm_client(mux_alarm);

    // Userspace alarm driver
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        core_capsules::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<
                'static,
                litex_vexriscv::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
        >,
        core_capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(core_capsules::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    virtual_alarm_user.set_alarm_client(alarm);

    // Systick virtual alarm for scheduling
    let systick_virtual_alarm = static_init!(
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        VirtualMuxAlarm::new(mux_alarm)
    );
    systick_virtual_alarm.setup();

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<
            VirtualMuxAlarm<
                'static,
                litex_vexriscv::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
        >,
        VirtualSchedulerTimer::new(systick_virtual_alarm)
    );

    // ---------- UART ----------

    // Initialize the HW UART
    let uart0 = static_init!(
        litex_vexriscv::uart::LiteXUart<socc::SoCRegisterFmt>,
        litex_vexriscv::uart::LiteXUart::new(
            StaticRef::new(
                socc::CSR_UART_BASE
                    as *const litex_vexriscv::uart::LiteXUartRegisters<socc::SoCRegisterFmt>,
            ),
            None, // LiteX simulator has no UART phy
            dynamic_deferred_caller,
        )
    );
    uart0.initialize(
        dynamic_deferred_caller.register(uart0).unwrap(), // Unwrap fail = dynamic deferred caller out of slots
    );

    PANIC_REFERENCES.uart = Some(uart0);

    // Create a shared UART channel for the console and for kernel debug.
    //
    // The baudrate is ingnored, as no UART phy is present in the
    // verilated simulation.
    let uart_mux =
        components::console::UartMuxComponent::new(uart0, 115200, dynamic_deferred_caller)
            .finalize(components::uart_mux_component_static!());

    // ---------- ETHERNET ----------

    // Packet receive buffer
    let ethmac0_rxbuf0 = static_init!([u8; 1522], [0; 1522]);

    // ETHMAC peripheral
    let ethmac0 = static_init!(
        litex_vexriscv::liteeth::LiteEth<socc::SoCRegisterFmt>,
        litex_vexriscv::liteeth::LiteEth::new(
            StaticRef::new(
                socc::CSR_ETHMAC_BASE
                    as *const litex_vexriscv::liteeth::LiteEthMacRegisters<socc::SoCRegisterFmt>,
            ),
            socc::MEM_ETHMAC_BASE,
            socc::MEM_ETHMAC_SIZE,
            socc::ETHMAC_SLOT_SIZE,
            socc::ETHMAC_RX_SLOTS,
            socc::ETHMAC_TX_SLOTS,
            ethmac0_rxbuf0,
        )
    );

    // Initialize the ETHMAC controller
    ethmac0.initialize();

    // --------- GPIO CONTROLLER ----------
    type GPIOPin = litex_vexriscv::gpio::LiteXGPIOPin<'static, 'static, socc::SoCRegisterFmt>;

    // GPIO hardware controller
    let gpio0 = static_init!(
        litex_vexriscv::gpio::LiteXGPIOController<'static, socc::SoCRegisterFmt>,
        litex_vexriscv::gpio::LiteXGPIOController::new(
            StaticRef::new(
                socc::CSR_GPIO_BASE
                    as *const litex_vexriscv::gpio::LiteXGPIORegisters<socc::SoCRegisterFmt>
            ),
            32, // 32 GPIOs in the simulation
        ),
    );
    gpio0.initialize();

    // --------- GPIO DRIVER ----------

    let gpio_driver = components::gpio::GpioComponent::new(
        board_kernel,
        core_capsules::gpio::DRIVER_NUM,
        components::gpio_component_helper_owned!(
            GPIOPin,
            16 => gpio0.get_gpio_pin(16).unwrap(),
            17 => gpio0.get_gpio_pin(17).unwrap(),
            18 => gpio0.get_gpio_pin(18).unwrap(),
            19 => gpio0.get_gpio_pin(19).unwrap(),
            20 => gpio0.get_gpio_pin(20).unwrap(),
            21 => gpio0.get_gpio_pin(21).unwrap(),
            22 => gpio0.get_gpio_pin(22).unwrap(),
            23 => gpio0.get_gpio_pin(23).unwrap(),
            24 => gpio0.get_gpio_pin(24).unwrap(),
            25 => gpio0.get_gpio_pin(25).unwrap(),
            26 => gpio0.get_gpio_pin(26).unwrap(),
            27 => gpio0.get_gpio_pin(27).unwrap(),
            28 => gpio0.get_gpio_pin(28).unwrap(),
            29 => gpio0.get_gpio_pin(29).unwrap(),
            30 => gpio0.get_gpio_pin(30).unwrap(),
            31 => gpio0.get_gpio_pin(31).unwrap(),
        ),
    )
    .finalize(components::gpio_component_static!(GPIOPin));

    // ---------- LED DRIVER ----------

    let led_gpios = static_init!(
        [GPIOPin; 8],
        [
            gpio0.get_gpio_pin(0).unwrap(),
            gpio0.get_gpio_pin(1).unwrap(),
            gpio0.get_gpio_pin(2).unwrap(),
            gpio0.get_gpio_pin(3).unwrap(),
            gpio0.get_gpio_pin(4).unwrap(),
            gpio0.get_gpio_pin(5).unwrap(),
            gpio0.get_gpio_pin(6).unwrap(),
            gpio0.get_gpio_pin(7).unwrap(),
        ]
    );

    let led_driver =
        components::led::LedsComponent::new().finalize(components::led_component_static!(
            kernel::hil::led::LedHigh<GPIOPin>,
            LedHigh::new(&led_gpios[0]),
            LedHigh::new(&led_gpios[1]),
            LedHigh::new(&led_gpios[2]),
            LedHigh::new(&led_gpios[3]),
            LedHigh::new(&led_gpios[4]),
            LedHigh::new(&led_gpios[5]),
            LedHigh::new(&led_gpios[6]),
            LedHigh::new(&led_gpios[7]),
        ));

    // ---------- BUTTON ----------

    let button_driver = components::button::ButtonComponent::new(
        board_kernel,
        core_capsules::button::DRIVER_NUM,
        components::button_component_helper_owned!(
            GPIOPin,
            (
                gpio0.get_gpio_pin(8).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(9).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(10).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(11).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(12).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(13).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(14).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            (
                gpio0.get_gpio_pin(15).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
        ),
    )
    .finalize(components::button_component_static!(GPIOPin));

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ----------

    let interrupt_service = static_init!(
        LiteXSimInterruptablePeripherals,
        LiteXSimInterruptablePeripherals {
            gpio0,
            timer0,
            uart0,
            ethmac0,
        }
    );

    let chip = static_init!(
        litex_vexriscv::chip::LiteXVexRiscv<
            LiteXSimInterruptablePeripherals,
        >,
        litex_vexriscv::chip::LiteXVexRiscv::new(
            "Verilated LiteX on VexRiscv",
            interrupt_service
        )
    );

    PANIC_REFERENCES.chip = Some(chip);

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());

    PANIC_REFERENCES.process_printer = Some(process_printer);

    // Enable RISC-V interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::mext::SET + csr::mie::mie::msoft::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Unmask all interrupt sources in the interrupt controller
    chip.unmask_interrupts();

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        core_capsules::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux)
        .finalize(components::debug_writer_component_static!());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        core_capsules::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::low_level_debug_component_static!());

    debug!("Verilated LiteX+VexRiscv: initialization complete, entering main loop.");

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

    let scheduler = components::sched::cooperative::CooperativeComponent::new(&PROCESSES)
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let litex_sim = LiteXSim {
        gpio_driver: gpio_driver,
        button_driver: button_driver,
        led_driver: led_driver,
        console: console,
        alarm: alarm,
        lldb: lldb,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
        scheduler,
        scheduler_timer,
    };

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

    board_kernel.kernel_loop(&litex_sim, chip, Some(&litex_sim.ipc), &main_loop_cap);
}
