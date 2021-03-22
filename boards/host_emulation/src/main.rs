#![feature(const_fn)]
use std::io;

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::AppId;
use kernel::Platform;
use kernel::{create_capability, static_init};

mod async_data_stream;
mod chip;
mod emulation_config;
mod i2cp;
mod ipc_syscalls;
mod log;
mod process;
mod syscall;
mod syscall_transport;
mod systick;
mod uart;

use crate::process::{EmulatedProcess, UnixProcess};
use crate::syscall::HostStoredState;

pub type Result<T> = std::result::Result<T, EmulationError>;

#[derive(Debug)]
pub enum EmulationError {
    IoError(io::Error),
    ChannelError,
    PartialMessage(usize, usize),
    Custom(String),
}

impl From<io::Error> for EmulationError {
    fn from(error: io::Error) -> Self {
        EmulationError::IoError(error)
    }
}

impl std::fmt::Display for EmulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulationError::IoError(e) => write!(f, "{}", e),
            EmulationError::ChannelError => write!(f, "Channel Error"),
            EmulationError::PartialMessage(e, a) => {
                write!(f, "Unexpected message length. Expected {}, got {}.", e, a)
            }
            EmulationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

static mut UNINITIALIZED_PROCESSES: [Option<&'static UnixProcess>; 4] = [None, None, None, None];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; 4] =
    [None, None, None, None];

static mut CHIP: Option<&'static chip::HostChip> = None;

static mut EXTERNAL_PROCESS_CAP: &dyn capabilities::ExternalProcessCapability =
    &create_capability!(capabilities::ExternalProcessCapability);

pub static mut UART0: uart::UartIO = uart::UartIO::create("0");

pub static mut I2CP: [i2cp::I2CPeripheral; 3] = [
    i2cp::I2CPeripheral::new("1"),
    i2cp::I2CPeripheral::new("2"),
    i2cp::I2CPeripheral::new("3"),
];
/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct HostBoard {
    console: &'static capsules::console::Console<'static>,
    lldb: &'static capsules::low_level_debug::LowLevelDebug<
        'static,
        capsules::virtual_uart::UartDevice<'static>,
    >,
}
/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for HostBoard {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let chip = CHIP.unwrap();

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());

    let ddc = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(ddc);

    let uart = &mut UART0;
    uart.initialize();

    let uart_mux = components::console::UartMuxComponent::new(uart, 0, ddc).finalize(());
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());

    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(board_kernel, uart_mux).finalize(());

    let host = HostBoard {
        console: console,
        lldb: lldb,
    };

    // Process setup. This takes the place of TBF headers
    for i in 0..UNINITIALIZED_PROCESSES.len() {
        let uninitialized_process = match UNINITIALIZED_PROCESSES[i] {
            Some(p) => p,
            None => break,
        };
        let state = static_init!(HostStoredState, HostStoredState::new(uninitialized_process));
        match EmulatedProcess::<chip::HostChip>::create(
            AppId::new_external(board_kernel, i, i, EXTERNAL_PROCESS_CAP),
            "Sample Process",
            chip,
            board_kernel,
            state,
            EXTERNAL_PROCESS_CAP,
        ) {
            Ok(p) => PROCESSES[i] = Some(static_init!(process::EmulatedProcess<chip::HostChip>, p)),
            Err(_) => panic!("Failed to start process #{}: ", i),
        }
    }

    board_kernel.kernel_loop(&host, chip, None, &main_loop_cap);
}

pub fn main() {
    unsafe {
        chip::HostChip::basic_setup();
        let chip = static_init!(chip::HostChip, chip::HostChip::new());
        let app_path = chip::HostChip::get_app_path();

        UNINITIALIZED_PROCESSES[0] = Some(static_init!(UnixProcess, UnixProcess::new(app_path, 0)));
        CHIP = Some(chip);
        reset_handler();
    }
}
