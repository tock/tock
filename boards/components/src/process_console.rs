//! Component for ProcessConsole, the command console.
//!
//! This provides one Component, ProcessConsoleComponent, which implements a
//! command console for controlling processes over a UART bus. On imix this is
//! typically USART3 (the DEBUG USB connector).
//!
//! Usage
//! -----
//! ```rust
//! let pconsole = ProcessConsoleComponent::new(board_kernel, uart_mux, alarm_mux, process_printer)
//!     .finalize(process_console_component_static!());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use capsules::process_console::{self, ProcessConsole};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_uart::{MuxUart, UartDevice};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::process::ProcessPrinter;

#[macro_export]
macro_rules! process_console_component_static {
    ($A: ty $(,)?) => {{
        let alarm = kernel::static_buf!(capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let uart = kernel::static_buf!(capsules::virtual_uart::UartDevice);
        let pconsole = kernel::static_buf!(
            capsules::process_console::ProcessConsole<
                capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                components::process_console::Capability,
            >
        );

        let write_buffer = kernel::static_buf!([u8; capsules::process_console::WRITE_BUF_LEN]);
        let read_buffer = kernel::static_buf!([u8; capsules::process_console::READ_BUF_LEN]);
        let queue_buffer = kernel::static_buf!([u8; capsules::process_console::QUEUE_BUF_LEN]);
        let command_buffer = kernel::static_buf!([u8; capsules::process_console::COMMAND_BUF_LEN]);

        (
            alarm,
            uart,
            write_buffer,
            read_buffer,
            queue_buffer,
            command_buffer,
            pconsole,
        )
    };};
}

pub struct ProcessConsoleComponent<A: 'static + Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
    process_printer: &'static dyn ProcessPrinter,
}

impl<A: 'static + Alarm<'static>> ProcessConsoleComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
        alarm_mux: &'static MuxAlarm<'static, A>,
        process_printer: &'static dyn ProcessPrinter,
    ) -> ProcessConsoleComponent<A> {
        ProcessConsoleComponent {
            board_kernel: board_kernel,
            uart_mux: uart_mux,
            alarm_mux: alarm_mux,
            process_printer,
        }
    }
}

// These constants are defined in the linker script for where the
// kernel is placed in memory on chip.
extern "C" {
    static _estack: u8;
    static _sstack: u8;
    static _stext: u8;
    static _srodata: u8;
    static _etext: u8;
    static _srelocate: u8;
    static _erelocate: u8;
    static _szero: u8;
    static _ezero: u8;
}

pub struct Capability;
unsafe impl capabilities::ProcessManagementCapability for Capability {}

impl<A: 'static + Alarm<'static>> Component for ProcessConsoleComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<[u8; capsules::process_console::WRITE_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules::process_console::READ_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules::process_console::QUEUE_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules::process_console::COMMAND_BUF_LEN]>,
        &'static mut MaybeUninit<ProcessConsole<'static, VirtualMuxAlarm<'static, A>, Capability>>,
    );
    type Output =
        &'static process_console::ProcessConsole<'static, VirtualMuxAlarm<'static, A>, Capability>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        // Create virtual device for console.
        let console_uart = static_buffer.1.write(UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        // Get addresses of where the kernel is placed to enable additional
        // debugging in process console.
        let kernel_addresses = process_console::KernelAddresses {
            stack_start: &_sstack as *const u8,
            stack_end: &_estack as *const u8,
            text_start: &_stext as *const u8,
            text_end: &_etext as *const u8,
            read_only_data_start: &_srodata as *const u8,
            relocations_start: &_srelocate as *const u8,
            relocations_end: &_erelocate as *const u8,
            bss_start: &_szero as *const u8,
            bss_end: &_ezero as *const u8,
        };

        let console_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        console_alarm.setup();

        let write_buffer = static_buffer
            .2
            .write([0; capsules::process_console::WRITE_BUF_LEN]);
        let read_buffer = static_buffer
            .3
            .write([0; capsules::process_console::READ_BUF_LEN]);
        let queue_buffer = static_buffer
            .4
            .write([0; capsules::process_console::QUEUE_BUF_LEN]);
        let command_buffer = static_buffer
            .5
            .write([0; capsules::process_console::COMMAND_BUF_LEN]);

        let console = static_buffer.6.write(ProcessConsole::new(
            console_uart,
            console_alarm,
            self.process_printer,
            write_buffer,
            read_buffer,
            queue_buffer,
            command_buffer,
            self.board_kernel,
            kernel_addresses,
            Capability,
        ));
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);
        console_alarm.set_alarm_client(console);

        console
    }
}
