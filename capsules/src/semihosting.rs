//! ARM Semihosting console implementation.
//!
//! Usage
//! -----
//!
//! ```
//! pub struct Platform {
//!     // Other fields omitted for clarity
//!     console: &'static capsules::console::Console<'static>,
//! }
//! ```
//!
//! In `reset_handler()`:
//!
//! ```
//! let virt_alarm = static_init!(
//!     capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
//!     capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
//! );
//!
//! let semihosting = static_init!(
//!     capsules::semihosting::Semihosting<VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>>,
//!     capsules::semihosting::Semihosting::new(virt_alarm)
//! );
//! hil::time::Alarm::set_client(virt_alarm, semihosting);
//!
//! let uart_mux = components::console::UartMuxComponent::new(semihosting, 0, dynamic_deferred_caller)
//!     .finalize(());
//!
//! let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
//!
//! // Create the debugger object that handles calls to `debug!()`.
//! components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());
//! ```

use core::cell::Cell;
use kernel::hil::{uart, time};
use kernel::hil::time::Frequency;
use kernel::ReturnCode;
use kernel::common::cells::{OptionalCell, TakeCell};
use cortex_m_semihosting::hio;

pub struct Semihosting<'a, A: time::Alarm<'a>> {
    alarm: &'a A, // Dummy alarm so we can get a callback.
    stdout: hio::HStdout,
    client: OptionalCell<&'a dyn uart::TransmitClient>,
    client_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
}

impl<'a, A: time::Alarm<'a>> Semihosting<'a, A> {
    pub fn new(alarm: &'a A) -> Semihosting<'a, A> {
        Semihosting {
            alarm,
            stdout: hio::hstdout().unwrap(),
            client: OptionalCell::empty(),
            client_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
        }
    }
}

impl<'a, A: time::Alarm<'a>> uart::Uart<'a> for Semihosting<'a, A> {}
impl<'a, A: time::Alarm<'a>> uart::UartData<'a> for Semihosting<'a, A> {}

impl<'a, A: time::Alarm<'a>> uart::Transmit<'a> for Semihosting<'a, A> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.stdout.clone().write_all(&mut tx_data[..tx_len]).is_ok() {
            // Save the client buffer so we can pass it back with the callback.
            self.client_buffer.replace(&mut tx_data[..tx_len]);
            self.tx_len.set(tx_len);

            // Start a short timer so that we get a callback and
            // can issue the callback to the client.
            let interval = (100 as u32) * <A::Frequency>::frequency() / 1000000;
            let tics = self.alarm.now().wrapping_add(interval);
            self.alarm.set_alarm(tics);
            (ReturnCode::SUCCESS, None)
        } else {
            (ReturnCode::EBUSY, Some(tx_data))
        }
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}

impl<A: time::Alarm<'a>> time::AlarmClient for Semihosting<'a, A> {
    fn fired(&self) {
        self.client.map(|client| {
            self.client_buffer.take().map(|buffer| {
                client.transmitted_buffer(buffer, self.tx_len.get(), ReturnCode::SUCCESS);
            });
        });
    }
}

// Dummy implementation so this can act as the underlying UART for a
// virtualized UART MUX.
impl<'a, A: time::Alarm<'a>> uart::Configure for Semihosting<'a, A> {
    fn configure(&self, _parameters: uart::Parameters) -> ReturnCode {
        ReturnCode::FAIL
    }
}

// Dummy implementation so this can act as the underlying UART for a
// virtualized UART MUX.
impl<'a, A: time::Alarm<'a>> uart::Receive<'a> for Semihosting<'a, A> {
    fn set_receive_client(&self, _client: &'a dyn uart::ReceiveClient) {}

    fn receive_buffer(
        &self,
        buffer: &'static mut [u8],
        _len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::FAIL, Some(buffer))
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}
