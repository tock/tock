//! Userspace driver for AT commands.
//!
//! This capsule exposes a userspace driver interface designed to support
//! sending AT commands to radios. To control a radio, userspace can `allow` a
//! TX buffer, and then use a command to send a desired number of bytes from
//! that buffer to the radio. Also, there is an optional TX done callback that
//! can be subscribed to. On the receive path, userspace applications should
//! `allow` an RX buffer and then use a command to enable receiving into that
//! buffer. This should likely be done _before_ a TX to make sure that no bytes
//! are missed.
//!
//! This driver uses grant space to accommodate multiple applications, but given
//! the nature of AT command based radios it isn't designed to be used by
//! multiple applications. The first application to request a TX or RX will win
//! with this driver, and other requests will return errors until the first
//! app's transaction finishes.
//!
//! This driver was implemented and tested using an ESP32 Wi-Fi chip
//! programmed to support AT commands. It is largely generic, and should support
//! any generic AT command-based radio, but it has not been tested extensively.
//!
//! Usage
//! -----
//!
//! ```
//! let at_driver = static_init!(
//!     at_commands_driver::AtCommands<'static, sam4l::usart::USART>,
//!     at_commands_driver::AtCommands::new(
//!         &sam4l::usart::USART2,
//!         &mut at_commands_driver::TX_BUF,
//!         &mut at_commands_driver::RX_BUF,
//!         board_kernel.create_grant(&memory_allocation_capability)
//!     )
//! );
//! hil::uart::UART::set_client(&sam4l::usart::USART2, at_driver);
//! ```

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::uart::UARTReceiveAdvanced;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0xffff0001;

#[derive(Default)]
pub struct App {
    tx_callback: Option<Callback>,
    tx_buffer: Option<AppSlice<Shared, u8>>,
    rx_callback: Option<Callback>,
    rx_buffer: Option<AppSlice<Shared, u8>>,
}

pub static mut TX_BUF: [u8; 3000] = [0; 3000];
pub static mut RX_BUF: [u8; 3000] = [0; 3000];

pub struct AtCommands<'a, U: UARTReceiveAdvanced> {
    uart: &'a U,
    apps: Grant<App>,
    tx_in_progress: OptionalCell<AppId>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: OptionalCell<AppId>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<U: UARTReceiveAdvanced> AtCommands<'a, U> {
    pub fn new(
        uart: &'a U,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        grant: Grant<App>,
    ) -> AtCommands<'a, U> {
        AtCommands {
            uart: uart,
            apps: grant,
            tx_in_progress: OptionalCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    pub fn initialize(&self, params: hil::uart::UARTParameters) {
        self.uart.configure(params);
    }
}

impl<U: UARTReceiveAdvanced> Driver for AtCommands<'a, U> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Share a TX buffer for sending AT commands to the radio.
    /// - `2`: Share a RX buffer for receiving data from the radio.
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    app.tx_buffer = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            2 => self
                .apps
                .enter(appid, |app, _| {
                    app.rx_buffer = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Finished sending AT command.
    /// - `2`: Received data from the radio.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            1 => self
                .apps
                .enter(app_id, |app, _| {
                    app.tx_callback = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            2 => self
                .apps
                .enter(app_id, |app, _| {
                    app.rx_callback = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Initiate AT commands and other interactions with the radio.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`.
    /// - `2`: Receives into a buffer passed via `allow`.
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 => {
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    app.tx_buffer.as_ref().map_or(ReturnCode::ENOMEM, |slice| {
                        // Check that we aren't trying to send more than the
                        // shared AppSlice.
                        if len > slice.len() {
                            ReturnCode::EINVAL
                        } else {
                            // Copy into our buffer and then TX.
                            self.tx_buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
                                for (i, c) in slice.as_ref()[0..len]
                                    .iter()
                                    .enumerate()
                                {
                                    if buffer.len() <= i {
                                        break;
                                    }
                                    buffer[i] = *c;
                                }

                                self.tx_in_progress.set(appid);
                                self.uart.transmit(buffer, len);
                                ReturnCode::SUCCESS
                            })
                        }
                    })
                }).unwrap_or_else(|err| err.into())
            }
            2 => {
                self.rx_buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
                    self.rx_in_progress.set(appid);
                    self.uart.receive_automatic(buffer, 100);
                    ReturnCode::SUCCESS
                })
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl<U: UARTReceiveAdvanced> hil::uart::Client for AtCommands<'a, U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: hil::uart::Error) {
        // Replace the buffer and then issue a callback to the app (if it
        // subscribed to the callback) notifying that the transmission has
        // finished.
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.take().map(|appid| {
            self.apps.enter(appid, |app, _| {
                app.tx_callback.map(|mut cb| {
                    cb.schedule(From::from(ReturnCode::SUCCESS), 0, 0);
                });
            })
        });
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: hil::uart::Error) {
        self.rx_in_progress
            .take()
            .map(|appid| {
                self.apps
                    .enter(appid, |app, _| {
                        app.rx_callback.map(|mut cb| {
                            // An iterator over the returned buffer yielding only the
                            // first `rx_len` bytes.
                            let rx_buffer = buffer.iter().take(rx_len);
                            match error {
                                hil::uart::Error::CommandComplete | hil::uart::Error::Aborted => {
                                    // Receive some bytes, signal error type and return bytes to process buffer
                                    if let Some(mut app_buffer) = app.rx_buffer.take() {
                                        for (a, b) in app_buffer.iter_mut().zip(rx_buffer) {
                                            *a = *b;
                                        }
                                        cb.schedule(From::from(ReturnCode::SUCCESS), rx_len, 0);
                                    } else {
                                        // Oops, no app buffer
                                        cb.schedule(From::from(ReturnCode::EINVAL), 0, 0);
                                    }
                                }
                                _ => {
                                    // Some UART error occurred
                                    cb.schedule(From::from(ReturnCode::FAIL), 0, 0);
                                }
                            }
                        });
                    })
                    .unwrap_or_default();
            })
            .unwrap_or_default();

        // Whatever happens, we want to make sure to replace the rx_buffer for future transactions
        self.rx_buffer.replace(buffer);
    }
}
