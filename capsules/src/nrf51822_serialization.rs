//! Nrf51822Serialization is the kernel-level driver that provides
//! the UART API that the nRF51822 serialization library requires.
//!
//! This capsule handles interfacing with the UART driver, and includes
//! some nuances that keep the Nordic BLE serialization library happy.

use kernel::{AppId, Callback, AppSlice, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil::uart::{self, UARTAdvanced, Client};

struct App {
    callback: Option<Callback>,
    tx_buffer: Option<AppSlice<Shared, u8>>,
    rx_buffer: Option<AppSlice<Shared, u8>>,
    rx_recv_so_far: usize, // How many RX bytes we have currently received.
    rx_recv_total: usize, // The total number of bytes we expect to receive.
}

// Local buffer for storing data between when the application passes it to
// use
pub static mut WRITE_BUF: [u8; 256] = [0; 256];
pub static mut READ_BUF: [u8; 600] = [0; 600];

// We need two resources: a UART HW driver and driver state for each
// application.
pub struct Nrf51822Serialization<'a, U: UARTAdvanced + 'a> {
    uart: &'a U,
    app: MapCell<App>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a, U: UARTAdvanced> Nrf51822Serialization<'a, U> {
    pub fn new(uart: &'a U,
               tx_buffer: &'static mut [u8],
               rx_buffer: &'static mut [u8])
               -> Nrf51822Serialization<'a, U> {
        Nrf51822Serialization {
            uart: uart,
            app: MapCell::empty(),
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

impl<'a, U: UARTAdvanced> Driver for Nrf51822Serialization<'a, U> {
    /// Pass application space memory to this driver.
    fn allow(&self, _appid: AppId, allow_type: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_type {
            // Provide an RX buffer.
            0 => {
                let resapp = match self.app.take() {
                    Some(mut app) => {
                        app.rx_buffer = Some(slice);
                        app.rx_recv_so_far = 0;
                        app.rx_recv_total = 0;
                        app
                    }
                    None => {
                        App {
                            callback: None,
                            tx_buffer: None,
                            rx_buffer: Some(slice),
                            rx_recv_so_far: 0,
                            rx_recv_total: 0,
                        }
                    }
                };
                self.app.replace(resapp);
                ReturnCode::SUCCESS
            }

            // Provide a TX buffer.
            1 => {
                let resapp = match self.app.take() {
                    Some(mut app) => {
                        app.tx_buffer = Some(slice);
                        app
                    }
                    None => {
                        App {
                            callback: None,
                            tx_buffer: Some(slice),
                            rx_buffer: None,
                            rx_recv_so_far: 0,
                            rx_recv_total: 0,
                        }
                    }
                };
                self.app.replace(resapp);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Register a callback to the Nrf51822Serialization driver.
    ///
    /// The callback will be called when a TX finishes and when
    /// RX data is available.
    #[inline(never)]
    fn subscribe(&self, subscribe_type: usize, callback: Callback) -> ReturnCode {
        match subscribe_type {
            // Add a callback
            0 => {
                let resapp = match self.app.take() {
                    Some(mut app) => {
                        app.callback = Some(callback);
                        app
                    }
                    None => {
                        // can't start receiving until DMA has been set up
                        //  we'll start here when subscribe is first called
                        self.rx_buffer
                            .take()
                            .map(|buffer| { self.uart.receive_automatic(buffer, 250); });

                        App {
                            callback: Some(callback),
                            tx_buffer: None,
                            rx_buffer: None,
                            rx_recv_so_far: 0,
                            rx_recv_total: 0,
                        }
                    }
                };
                self.app.replace(resapp);

                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Issue a command to the Nrf51822Serialization driver.
    fn command(&self, command_type: usize, _: usize, _: AppId) -> ReturnCode {

        match command_type {
            0 /* check if present */ => ReturnCode::SUCCESS,

            // Send a buffer to the nRF51822 over UART.
            1 => {
                // On a TX, send the first byte of the TX buffer.
                // TODO(bradjc): Need to match this to the correct app!
                //               Can't just use 0!
                //
                // When could app be NULL? What return code should be here?
                // XXX ReturnCode -----------,,,,,,,,,,,,,,,,
                self.app.map_or(ReturnCode::FAIL, |appst| {

                    match appst.tx_buffer.take() {
                        Some(slice) => {
                            let write_len = slice.len();
                            self.tx_buffer.take().map(|buffer| {
                                for (i, c) in slice.as_ref().iter().enumerate() {
                                    buffer[i] = *c;
                                }
                                self.uart.transmit(buffer, write_len);
                            });
                            ReturnCode::SUCCESS
                        }
                        None => ReturnCode::FAIL, /* XXX: ReturnCode */
                        // ^When could this happen - when there wasn't an allow
                        // first? Maybe ERESERVE?
                    }
                })
            }

            // Ask the kernel to callback the application. This is used to
            // keep the state machine in the Nordic BLE serialization library
            // happy so that events look like they are occuring as the library
            // expects, since it doesn't expect there to be an underlying
            // kernel.
            9001 => {
                self.app.map(|appst| {
                    appst.callback.as_mut().map(|mut cb| {
                        // schedule an event just to wake up from yield
                        cb.schedule(17, 0, 0);
                    });
                });

                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

// Callbacks from the underlying UART driver.
impl<'a, U: UARTAdvanced> Client for Nrf51822Serialization<'a, U> {
    // Called when the UART TX has finished
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        self.tx_buffer.replace(buffer);
        // TODO(bradjc): Need to match this to the correct app!
        //               Can't just use 0!
        self.app.map(|appst| {
            // Call the callback after TX has finished
            appst.callback.as_mut().map(|mut cb| { cb.schedule(1, 0, 0); });
        });
    }

    // Called when a buffer is received on the UART
    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, _error: uart::Error) {

        self.rx_buffer.replace(buffer);

        self.app.map(|appst| {
            appst.rx_buffer = appst.rx_buffer.take().map(|mut rb| {

                // figure out length to copy
                let mut max_len = rx_len;
                if rb.len() < rx_len {
                    max_len = rb.len();
                }

                // copy over data to app buffer
                self.rx_buffer.map(|buffer| for idx in 0..max_len {
                    rb.as_mut()[idx] = buffer[idx];
                });

                appst.callback.as_mut().map(|cb| {
                    // send the whole darn buffer to the serialization layer
                    cb.schedule(4, rx_len, 0);
                });

                rb
            });
        });

        // restart the uart receive
        self.rx_buffer.take().map(|buffer| { self.uart.receive_automatic(buffer, 250); });
    }
}
