// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for Console and ConsoleOrdered. These are two
//! alternative implementations of the serial console system call
//! interface. Console allows prints of arbitrary length but does not
//! have ordering or atomicity guarantees. ConsoleOrdered, in
//! contrast, has limits on the maximum lengths of prints but provides
//! a temporal ordering and ensures a print is atomic at least up to
//! particular length (typically 200 bytes). Console is useful when
//! userspace is printing large messages. ConsoleOrdered is useful
//! when you are debugging and there are inter-related messages from
//! the kernel and userspace, whose ordering is important to maintain.
//!
//!
//! This provides three Components, `ConsoleComponent` and
//! `ConsoleOrderedComponent`, which implement a buffered read/write
//! console over a serial port, and `UartMuxComponent`, which provides
//! multiplexed access to hardware UART. As an example, the serial
//! port used for console on Imix is typically USART3 (the DEBUG USB
//! connector).
//!
//! Usage
//! -----
//! ```rust
//! let uart_mux = UartMuxComponent::new(&sam4l::usart::USART3,
//!                                      115200,
//!                                      deferred_caller).finalize(components::uart_mux_component_static!());
//! let console = ConsoleComponent::new(board_kernel, uart_mux)
//!    .finalize(console_component_static!());
//! ```
// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 1/08/2023

use capsules_core::console;
use capsules_core::console_ordered::ConsoleOrdered;

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_core::virtualizers::virtual_uart::{MuxUart, UartDevice};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::time::{self, Alarm};
use kernel::hil::uart;

use capsules_core::console::DEFAULT_BUF_SIZE;

#[macro_export]
macro_rules! uart_mux_component_static {
    // Common logic for both branches
    ($rx_buffer_len: expr) => {{
        use capsules_core::virtualizers::virtual_uart::MuxUart;
        use kernel::static_buf;
        let uart_mux = static_buf!(MuxUart<'static>);
        let rx_buf = static_buf!([u8; $rx_buffer_len]);
        (uart_mux, rx_buf)
    }};
    () => {
        $crate::uart_mux_component_static!(capsules_core::virtualizers::virtual_uart::RX_BUF_LEN);
    };
    ($rx_buffer_len: literal) => {
        $crate::uart_mux_component_static!($rx_buffer_len);
    };
}

pub struct UartMuxComponent<const RX_BUF_LEN: usize> {
    uart: &'static dyn uart::Uart<'static>,
    baud_rate: u32,
}

impl<const RX_BUF_LEN: usize> UartMuxComponent<RX_BUF_LEN> {
    pub fn new(
        uart: &'static dyn uart::Uart<'static>,
        baud_rate: u32,
    ) -> UartMuxComponent<RX_BUF_LEN> {
        UartMuxComponent { uart, baud_rate }
    }
}

impl<const RX_BUF_LEN: usize> Component for UartMuxComponent<RX_BUF_LEN> {
    type StaticInput = (
        &'static mut MaybeUninit<MuxUart<'static>>,
        &'static mut MaybeUninit<[u8; RX_BUF_LEN]>,
    );
    type Output = &'static MuxUart<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let rx_buf = s.1.write([0; RX_BUF_LEN]);
        let uart_mux = s.0.write(MuxUart::new(self.uart, rx_buf, self.baud_rate));
        kernel::deferred_call::DeferredCallClient::register(uart_mux);

        uart_mux.initialize();
        hil::uart::Transmit::set_transmit_client(self.uart, uart_mux);
        hil::uart::Receive::set_receive_client(self.uart, uart_mux);

        uart_mux
    }
}

#[macro_export]
macro_rules! console_component_static {
    // Common logic for both branches
    ($rx_buffer_len: expr, $tx_buffer_len: expr) => {{
        use capsules_core::console::{Console, DEFAULT_BUF_SIZE};
        use capsules_core::virtualizers::virtual_uart::UartDevice;
        use kernel::static_buf;
        let read_buf = static_buf!([u8; $rx_buffer_len]);
        let write_buf = static_buf!([u8; $tx_buffer_len]);
        // Create virtual device for console.
        let console_uart = static_buf!(UartDevice);
        let console = static_buf!(Console<'static>);
        (write_buf, read_buf, console_uart, console)
    }};
    () => {
        $crate::console_component_static!(DEFAULT_BUF_SIZE, DEFAULT_BUF_SIZE);
    };
    ($rx_buffer_len: literal, $tx_buffer_len: literal) => {
        $crate::console_component_static!($rx_buffer_len, $tx_buffer_len);
    };
}

pub struct ConsoleComponent<const RX_BUF_LEN: usize, const TX_BUF_LEN: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    uart_mux: &'static MuxUart<'static>,
}

impl<const RX_BUF_LEN: usize, const TX_BUF_LEN: usize> ConsoleComponent<RX_BUF_LEN, TX_BUF_LEN> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        uart_mux: &'static MuxUart,
    ) -> ConsoleComponent<RX_BUF_LEN, TX_BUF_LEN> {
        ConsoleComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            uart_mux: uart_mux,
        }
    }
}

impl<const RX_BUF_LEN: usize, const TX_BUF_LEN: usize> Component
    for ConsoleComponent<RX_BUF_LEN, TX_BUF_LEN>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; TX_BUF_LEN]>,
        &'static mut MaybeUninit<[u8; RX_BUF_LEN]>,
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<console::Console<'static>>,
    );
    type Output = &'static console::Console<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let write_buffer = s.0.write([0; TX_BUF_LEN]);

        let read_buffer = s.1.write([0; RX_BUF_LEN]);

        let console_uart = s.2.write(UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        let console = s.3.write(console::Console::new(
            console_uart,
            write_buffer,
            read_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);

        console
    }
}
#[macro_export]
macro_rules! console_ordered_component_static {
    ($A:ty $(,)?) => {{
        let mux_alarm = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let read_buf = static_buf!([u8; capsules_core::console::DEFAULT_BUF_SIZE]);
        let console_uart =
            kernel::static_buf!(capsules_core::virtualizers::virtual_uart::UartDevice);
        let console = kernel::static_buf!(ConsoleOrdered<'static, VirtualMuxAlarm<'static, $A>>);
        (mux_alarm, read_buf, console_uart, console)
    };};
}

pub struct ConsoleOrderedComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    uart_mux: &'static MuxUart<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
    atomic_size: usize,
    retry_timer: u32,
    write_timer: u32,
}

impl<A: 'static + time::Alarm<'static>> ConsoleOrderedComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        uart_mux: &'static MuxUart<'static>,
        alarm_mux: &'static MuxAlarm<'static, A>,
        atomic_size: usize,
        retry_timer: u32,
        write_timer: u32,
    ) -> ConsoleOrderedComponent<A> {
        ConsoleOrderedComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            uart_mux: uart_mux,
            alarm_mux: alarm_mux,
            atomic_size: atomic_size,
            retry_timer: retry_timer,
            write_timer: write_timer,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for ConsoleOrderedComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<[u8; DEFAULT_BUF_SIZE]>,
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<ConsoleOrdered<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static ConsoleOrdered<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_alarm1 = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        virtual_alarm1.setup();

        let read_buffer = static_buffer.1.write([0; DEFAULT_BUF_SIZE]);

        let console_uart = static_buffer.2.write(UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        let console = static_buffer.3.write(ConsoleOrdered::new(
            console_uart,
            virtual_alarm1,
            read_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.atomic_size,
            self.retry_timer,
            self.write_timer,
        ));

        virtual_alarm1.set_alarm_client(console);
        hil::uart::Receive::set_receive_client(console_uart, console);
        console
    }
}
