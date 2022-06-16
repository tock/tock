//! Component for ProcessConsole, the command console.
//!
//! This provides one Component, ProcessConsoleComponent, which implements a
//! command console for controlling processes over a UART bus. On imix this is
//! typically USART3 (the DEBUG USB connector).
//!
//! Usage
//! -----
//! ```rust
//! let pconsole = ProcessConsoleComponent::new(board_kernel, uart_mux).finalize(());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use core::marker::PhantomData;
use core::mem::MaybeUninit;

use capsules::process_console::{self, ProcessConsole};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::process::ProcessPrinter;
use kernel::{static_init, static_init_half};

#[macro_export]
macro_rules! process_console_component_helper {
    ($A: ty) => {{
        use capsules::process_console::ProcessConsole;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use components::process_console::Capability;
        use core::mem::MaybeUninit;

        static mut BUFFER: MaybeUninit<ProcessConsole<VirtualMuxAlarm<'static, $A>, Capability>> =
            MaybeUninit::uninit();

        static mut ALARM: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();

        (&mut BUFFER, &mut ALARM)
    }};
}

pub struct ProcessConsoleComponent<A: 'static + Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
    _alarm: PhantomData<A>,
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
            _alarm: PhantomData,
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
        &'static mut MaybeUninit<ProcessConsole<'static, VirtualMuxAlarm<'static, A>, Capability>>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
    );
    type Output =
        &'static process_console::ProcessConsole<'static, VirtualMuxAlarm<'static, A>, Capability>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        // Create virtual device for console.
        let console_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, true));
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

        let console_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        console_alarm.setup();

        let console = static_init_half!(
            static_buffer.0,
            ProcessConsole<'static, VirtualMuxAlarm<'static, A>, Capability>,
            ProcessConsole::new(
                console_uart,
                console_alarm,
                self.process_printer,
                &mut process_console::WRITE_BUF,
                &mut process_console::READ_BUF,
                &mut process_console::QUEUE_BUF,
                &mut process_console::COMMAND_BUF,
                self.board_kernel,
                kernel_addresses,
                Capability,
            )
        );
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);
        console_alarm.set_alarm_client(console);

        console
    }
}
