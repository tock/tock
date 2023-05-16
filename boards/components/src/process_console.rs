// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for ProcessConsole, the command console.
//!
//! This provides one Component, ProcessConsoleComponent, which implements a
//! command console for controlling processes over a UART bus. On imix this is
//! typically USART3 (the DEBUG USB connector).
//!
//! Usage
//! -----
//! ```rust
//! let pconsole = ProcessConsoleComponent::new(board_kernel, uart_mux, alarm_mux, process_printer, Some(reset_function))
//!     .finalize(process_console_component_static!());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use capsules_core::process_console::{self, ProcessConsole};
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_core::virtualizers::virtual_uart::{MuxUart, UartDevice};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::process::ProcessPrinter;

#[macro_export]
macro_rules! process_console_component_static {
    ($A: ty, $COMMAND_HISTORY_LEN: expr $(,)?) => {{
        let alarm = kernel::static_buf!(capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let uart = kernel::static_buf!(capsules_core::virtualizers::virtual_uart::UartDevice);
        let pconsole = kernel::static_buf!(
            capsules_core::process_console::ProcessConsole<
                $COMMAND_HISTORY_LEN,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                components::process_console::Capability,
            >
        );

        let write_buffer = kernel::static_buf!([u8; capsules_core::process_console::WRITE_BUF_LEN]);
        let read_buffer = kernel::static_buf!([u8; capsules_core::process_console::READ_BUF_LEN]);
        let queue_buffer = kernel::static_buf!([u8; capsules_core::process_console::QUEUE_BUF_LEN]);
        let command_buffer = kernel::static_buf!([u8; capsules_core::process_console::COMMAND_BUF_LEN]);
        let command_history_buffer = kernel::static_buf!(
            [capsules_core::process_console::Command; $COMMAND_HISTORY_LEN]
        );

        (
            alarm,
            uart,
            write_buffer,
            read_buffer,
            queue_buffer,
            command_buffer,
            command_history_buffer,
            pconsole,
        )
    };};
    ($A: ty $(,)?) => {{
        $crate::process_console_component_static!($A, { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN })
    };};
}

pub struct ProcessConsoleComponent<const COMMAND_HISTORY_LEN: usize, A: 'static + Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
    process_printer: &'static dyn ProcessPrinter,
    reset_function: Option<fn() -> !>,
}

impl<const COMMAND_HISTORY_LEN: usize, A: 'static + Alarm<'static>>
    ProcessConsoleComponent<COMMAND_HISTORY_LEN, A>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
        alarm_mux: &'static MuxAlarm<'static, A>,
        process_printer: &'static dyn ProcessPrinter,
        reset_function: Option<fn() -> !>,
    ) -> ProcessConsoleComponent<COMMAND_HISTORY_LEN, A> {
        ProcessConsoleComponent {
            board_kernel,
            uart_mux,
            alarm_mux,
            process_printer,
            reset_function,
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

impl<const COMMAND_HISTORY_LEN: usize, A: 'static + Alarm<'static>> Component
    for ProcessConsoleComponent<COMMAND_HISTORY_LEN, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<[u8; capsules_core::process_console::WRITE_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules_core::process_console::READ_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules_core::process_console::QUEUE_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; capsules_core::process_console::COMMAND_BUF_LEN]>,
        &'static mut MaybeUninit<[capsules_core::process_console::Command; COMMAND_HISTORY_LEN]>,
        &'static mut MaybeUninit<
            ProcessConsole<'static, COMMAND_HISTORY_LEN, VirtualMuxAlarm<'static, A>, Capability>,
        >,
    );
    type Output = &'static process_console::ProcessConsole<
        'static,
        COMMAND_HISTORY_LEN,
        VirtualMuxAlarm<'static, A>,
        Capability,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        // Create virtual device for console.
        let console_uart = static_buffer.1.write(UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        // Get addresses of where the kernel is placed to enable additional
        // debugging in process console.
        // SAFETY: These statics are defined by the linker script, and we are merely creating
        // pointers to them.
        let kernel_addresses = unsafe {
            process_console::KernelAddresses {
                stack_start: &_sstack as *const u8,
                stack_end: &_estack as *const u8,
                text_start: &_stext as *const u8,
                text_end: &_etext as *const u8,
                read_only_data_start: &_srodata as *const u8,
                relocations_start: &_srelocate as *const u8,
                relocations_end: &_erelocate as *const u8,
                bss_start: &_szero as *const u8,
                bss_end: &_ezero as *const u8,
            }
        };

        let console_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        console_alarm.setup();

        let write_buffer = static_buffer
            .2
            .write([0; capsules_core::process_console::WRITE_BUF_LEN]);
        let read_buffer = static_buffer
            .3
            .write([0; capsules_core::process_console::READ_BUF_LEN]);
        let queue_buffer = static_buffer
            .4
            .write([0; capsules_core::process_console::QUEUE_BUF_LEN]);
        let command_buffer = static_buffer
            .5
            .write([0; capsules_core::process_console::COMMAND_BUF_LEN]);
        let command_history_buffer = static_buffer
            .6
            .write([capsules_core::process_console::Command::default(); COMMAND_HISTORY_LEN]);

        let console = static_buffer.7.write(ProcessConsole::new(
            console_uart,
            console_alarm,
            self.process_printer,
            write_buffer,
            read_buffer,
            queue_buffer,
            command_buffer,
            command_history_buffer,
            self.board_kernel,
            kernel_addresses,
            self.reset_function,
            Capability,
        ));
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);
        console_alarm.set_alarm_client(console);

        console
    }
}
