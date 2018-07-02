//! Provides userspace with the UART API that the nRF51822 serialization library
//! requires.
//!
//! This capsule handles interfacing with the UART driver, and includes some
//! nuances that keep the Nordic BLE serialization library happy.
//!
//! Usage
//! -----
//!
//! ```rust
//! let nrf_serialization = static_init!(
//!     Nrf51822Serialization<usart::USART>,
//!     Nrf51822Serialization::new(&usart::USART3,
//!                                &mut nrf51822_serialization::WRITE_BUF,
//!                                &mut nrf51822_serialization::READ_BUF));
//! hil::uart::UART::set_client(&usart::USART3, nrf_serialization);
//! ```

use core::cmp;
use kernel::common::cells::{MapCell, TakeCell};
use kernel::hil::uart::{self, Client, UARTReceiveAdvanced};
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};

/// Syscall number
pub const DRIVER_NUM: usize = 0x80004;

struct App {
    callback: Option<Callback>,
    tx_buffer: Option<AppSlice<Shared, u8>>,
    rx_buffer: Option<AppSlice<Shared, u8>>,
    rx_recv_so_far: usize, // How many RX bytes we have currently received.
    rx_recv_total: usize,  // The total number of bytes we expect to receive.
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            tx_buffer: None,
            rx_buffer: None,
            rx_recv_so_far: 0,
            rx_recv_total: 0,
        }
    }
}

// Local buffer for passing data between applications and the underlying
// transport hardware.
pub static mut WRITE_BUF: [u8; 600] = [0; 600];
pub static mut READ_BUF: [u8; 600] = [0; 600];

// We need two resources: a UART HW driver and driver state for each
// application.
pub struct Nrf51822Serialization<'a, U: UARTReceiveAdvanced> {
    uart: &'a U,
    app: MapCell<App>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<U: UARTReceiveAdvanced> Nrf51822Serialization<'a, U> {
    pub fn new(
        uart: &'a U,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
    ) -> Nrf51822Serialization<'a, U> {
        Nrf51822Serialization {
            uart: uart,
            app: MapCell::new(App::default()),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    pub fn initialize(&self) {
        self.uart.init(uart::UARTParams {
            baud_rate: 250000,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::Even,
            hw_flow_control: true,
        });
    }
}

impl<U: UARTReceiveAdvanced> Driver for Nrf51822Serialization<'a, U> {
    /// Pass application space memory to this driver.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Provide a RX buffer.
    /// - `1`: Provide a TX buffer.
    fn allow(
        &self,
        _appid: AppId,
        allow_type: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_type {
            // Provide an RX buffer.
            0 => self.app.map_or(ReturnCode::FAIL, |app| {
                app.rx_buffer = slice;
                app.rx_recv_so_far = 0;
                app.rx_recv_total = 0;
                ReturnCode::SUCCESS
            }),

            // Provide a TX buffer.
            1 => self.app.map_or(ReturnCode::FAIL, |app| {
                app.tx_buffer = slice;
                ReturnCode::SUCCESS
            }),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Register a callback to the Nrf51822Serialization driver.
    ///
    /// The callback will be called when a TX finishes and when
    /// RX data is available.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Set callback.
    fn subscribe(
        &self,
        subscribe_type: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_type {
            // Add a callback
            0 => {
                // work-around because `MapCell` don't provide `map_or_else`
                if self.app.map(|app| app.callback = callback).is_none() == true {
                    return ReturnCode::FAIL;
                }

                // Start the receive now that we have a callback.
                self.rx_buffer.take().map_or(ReturnCode::FAIL, |buffer| {
                    self.uart.receive_automatic(buffer, 250);
                    ReturnCode::SUCCESS
                })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Issue a command to the Nrf51822Serialization driver.
    ///
    /// ### `command_type`
    ///
    /// - `0`: Driver check.
    /// - `1`: Send the allowed buffer to the nRF.
    fn command(&self, command_type: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        match command_type {
            0 /* check if present */ => ReturnCode::SUCCESS,

            // Send a buffer to the nRF51822 over UART.
            1 => {
                // TODO(bradjc): Need to match this to the correct app!
                //               Can't just use 0!
                self.app.map_or(ReturnCode::FAIL, |app| {
                    app.tx_buffer.take().map_or(ReturnCode::FAIL, |slice| {
                        let write_len = slice.len();
                        self.tx_buffer.take().map_or(ReturnCode::FAIL, |buffer| {
                            for (i, c) in slice.as_ref().iter().enumerate() {
                                buffer[i] = *c;
                            }
                            self.uart.transmit(buffer, write_len);
                            ReturnCode::SUCCESS
                        })
                    })
                })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

// Callbacks from the underlying UART driver.
impl<U: UARTReceiveAdvanced> Client for Nrf51822Serialization<'a, U> {
    // Called when the UART TX has finished.
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        self.tx_buffer.replace(buffer);
        // TODO(bradjc): Need to match this to the correct app!
        //               Can't just use 0!
        self.app.map(|appst| {
            // Call the callback after TX has finished
            appst.callback.as_mut().map(|cb| {
                cb.schedule(1, 0, 0);
            });
        });
    }

    // Called when a buffer is received on the UART.
    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, _error: uart::Error) {
        self.rx_buffer.replace(buffer);

        self.app.map(|appst| {
            appst.rx_buffer = appst.rx_buffer.take().map(|mut rb| {
                // Figure out length to copy.
                let max_len = cmp::min(rx_len, rb.len());

                // Copy over data to app buffer.
                self.rx_buffer.map(|buffer| {
                    for idx in 0..max_len {
                        rb.as_mut()[idx] = buffer[idx];
                    }
                });
                appst.callback.as_mut().map(|cb| {
                    // Notify the serialization library in userspace about the
                    // received buffer.
                    cb.schedule(4, rx_len, 0);
                });

                rb
            });
        });

        // Restart the UART receive.
        self.rx_buffer
            .take()
            .map(|buffer| self.uart.receive_automatic(buffer, 250));
    }
}
