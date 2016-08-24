use common::take_cell::TakeCell;
use hil::uart::{UART, Client};
use main::{AppId, Callback, AppSlice, Driver, Shared};

///
/// Nrf51822Serialization is the kernel-level driver that provides
/// the UART API that the nRF51822 serialization library requires.
///

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

// We need two resources: a UART HW driver and driver state for each
// application.
pub struct Nrf51822Serialization<'a, U: UART + 'a> {
    uart: &'a U,
    app: TakeCell<App>,
    buffer: TakeCell<&'static mut [u8]>,
}

impl<'a, U: UART> Nrf51822Serialization<'a, U> {
    pub fn new(uart: &'a U, buffer: &'static mut [u8]) -> Nrf51822Serialization<'a, U> {
        Nrf51822Serialization {
            uart: uart,
            app: TakeCell::empty(),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn initialize(&self) {
        self.uart.enable_tx();
        self.uart.enable_rx();
    }
}

impl<'a, U: UART> Driver for Nrf51822Serialization<'a, U> {
    /// Pass application space memory to this driver.
    ///
    /// allow_type: 0 - Provide an RX buffer
    /// allow_type: 1 - Provide an TX buffer
    ///
    fn allow(&self, _appid: AppId, allow_type: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_type {
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
                0
            }
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
                0
            }
            _ => -1,
        }
    }

    /// Register a callback to the Nrf51822Serialization driver.
    ///
    /// The callback will be called when a TX finishes and when
    /// RX data is available.
    ///
    /// subscribe_type: 0 - add the callback
    ///
    #[inline(never)]
    fn subscribe(&self, subscribe_type: usize, callback: Callback) -> isize {
        match subscribe_type {
            0 => {
                let resapp = match self.app.take() {
                    Some(mut app) => {
                        app.callback = Some(callback);
                        app
                    }
                    None => {
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
                0
            }
            _ => -1,
        }
    }

    /// Issue a command to the Nrf51822Serialization driver.
    ///
    /// command_type: 0 - Write a byte to the UART.
    ///
    fn command(&self, command_type: usize, _: usize, _: AppId) -> isize {

        match command_type {
            0 => {
                // On a TX, send the first byte of the TX buffer.
                // TODO(bradjc): Need to match this to the correct app!
                //               Can't just use 0!
                let result = self.app.map(|appst| {

                    match appst.tx_buffer.take() {
                        Some(slice) => {
                            let write_len = slice.len();
                            self.buffer.take().map(|buffer| {
                                for (i, c) in slice.as_ref().iter().enumerate() {
                                    buffer[i] = *c;
                                }
                                self.uart.send_bytes(buffer, write_len);
                            });
                            0
                        }
                        None => -2,
                    }
                });
                result.unwrap_or(-1)
            }
            _ => -1,
        }
    }
}

// Callbacks from the underlying UART driver.
impl<'a, U: UART> Client for Nrf51822Serialization<'a, U> {
    // Called when the UART TX has finished
    fn write_done(&self, buffer: &'static mut [u8]) {
        self.buffer.replace(buffer);
        // TODO(bradjc): Need to match this to the correct app!
        //               Can't just use 0!
        self.app.map(|appst| {
            // Call the callback after TX has finished
            appst.callback.as_mut().map(|mut cb| {
                cb.schedule(1, 0, 0);
            });
        });
    }

    // Called when a byte is received on the UART
    fn read_done(&self, c: u8) {
        self.app.map(|appst| {
            // The PHY layer of the serialization protocol calls for a 16 byte
            // length field to start the packet. After we receive the first two
            // bytes we then know how long to wait for to get the rest of
            // the packet.

            // Save a local copy of this so we can use it after we have a borrow
            let rx_count = appst.rx_recv_so_far;

            if appst.rx_buffer.is_some() && rx_count < appst.rx_buffer.as_ref().unwrap().len() {

                // This is just some rust magic that only gets a mutable
                // reference to the RX buffer and adds the byte if the buffer
                // actually exists.
                // Yes, we did already check that the buffer exists above,
                // but I don't know what to do about that....
                appst.rx_buffer.as_mut().map(|buf| {
                    // Record the received byte
                    buf.as_mut()[rx_count] = c;

                });

                // Increment our counter since we got another byte.
                appst.rx_recv_so_far += 1;

                // Check if this was the second byte. If so, we can now
                // compute how many total bytes we expect to receive.
                if appst.rx_recv_so_far == 2 {
                    appst.rx_recv_total =
                        appst.rx_buffer.as_ref().unwrap().as_ref()[0] as usize |
                        ((appst.rx_buffer.as_ref().unwrap().as_ref()[1] as usize) << 8);

                    // After first byte let app know that a packet is inbound!
                    let rx_recv_total = appst.rx_recv_total;
                    appst.callback.as_mut().map(|mut cb| {
                        cb.schedule(2, rx_recv_total, 0);
                    });

                } else if appst.rx_recv_so_far > 2 {
                    // Check to see if we have gotten all of the data
                    // we want.
                    if appst.rx_recv_so_far == appst.rx_recv_total + 2 {
                        // we did!

                        // Callback the app with an RX done signal
                        let rx_recv_so_far = appst.rx_recv_so_far;
                        appst.callback.as_mut().map(|mut cb| {
                            cb.schedule(3, rx_recv_so_far, 0);
                        });

                        // Reset this for the next RX
                        appst.rx_recv_so_far = 0;
                    }
                }
            }
        });
    }
}
