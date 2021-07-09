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
//! # use kernel::{hil, static_init};
//! # use capsules::nrf51822_serialization;
//! # use capsules::nrf51822_serialization::Nrf51822Serialization;
//!
//! let nrf_serialization = static_init!(
//!     Nrf51822Serialization<usart::USART>,
//!     Nrf51822Serialization::new(&usart::USART3,
//!                                &mut nrf51822_serialization::WRITE_BUF,
//!                                &mut nrf51822_serialization::READ_BUF));
//! hil::uart::UART::set_client(&usart::USART3, nrf_serialization);
//! ```

use core::cmp;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::uart;
use kernel::{
    CommandReturn, Driver, ErrorCode, Grant, ProcessId, ReadOnlyProcessBuffer,
    ReadWriteProcessBuffer, ReadableProcessBuffer, WriteableProcessBuffer,
};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Nrf51822Serialization as usize;

#[derive(Default)]
pub struct App {
    tx_buffer: ReadOnlyProcessBuffer,
    rx_buffer: ReadWriteProcessBuffer,
}

// Local buffer for passing data between applications and the underlying
// transport hardware.
pub static mut WRITE_BUF: [u8; 600] = [0; 600];
pub static mut READ_BUF: [u8; 600] = [0; 600];

// We need two resources: a UART HW driver and driver state for each
// application.
pub struct Nrf51822Serialization<'a> {
    uart: &'a dyn uart::UartAdvanced<'a>,
    reset_pin: &'a dyn hil::gpio::Pin,
    apps: Grant<App, 1>,
    active_app: OptionalCell<ProcessId>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a> Nrf51822Serialization<'a> {
    pub fn new(
        uart: &'a dyn uart::UartAdvanced<'a>,
        grant: Grant<App, 1>,
        reset_pin: &'a dyn hil::gpio::Pin,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
    ) -> Nrf51822Serialization<'a> {
        Nrf51822Serialization {
            uart: uart,
            reset_pin: reset_pin,
            apps: grant,
            active_app: OptionalCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    pub fn initialize(&self) {
        let _ = self.uart.configure(uart::Parameters {
            baud_rate: 250000,
            width: uart::Width::Eight,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::Even,
            hw_flow_control: true,
        });
    }

    pub fn reset(&self) {
        self.reset_pin.make_output();
        self.reset_pin.clear();
        // minimum hold time is 200ns, ~20ns per instruction, so overshoot a bit
        for _ in 0..10 {
            self.reset_pin.clear();
        }
        self.reset_pin.set();
    }
}

impl Driver for Nrf51822Serialization<'_> {
    /// Pass application space memory to this driver.
    ///
    /// This also sets which app is currently using this driver. Only one app
    /// can control the nRF51 serialization driver.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Provide a RX buffer.
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_type: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        let res = match allow_type {
            // Provide an RX buffer.
            0 => {
                self.active_app.set(appid);
                self.apps
                    .enter(appid, |app, _| {
                        core::mem::swap(&mut app.rx_buffer, &mut slice);
                    })
                    .map_err(ErrorCode::from)
            }

            _ => Err(ErrorCode::NOSUPPORT),
        };

        if let Err(e) = res {
            Err((slice, e))
        } else {
            Ok(slice)
        }
    }

    /// Pass application space memory to this driver.
    ///
    /// This also sets which app is currently using this driver. Only one app
    /// can control the nRF51 serialization driver.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Provide a TX buffer.
    fn allow_readonly(
        &self,
        appid: ProcessId,
        allow_type: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        let res = match allow_type {
            // Provide a TX buffer.
            0 => {
                self.active_app.set(appid);
                self.apps
                    .enter(appid, |app, _| {
                        core::mem::swap(&mut app.tx_buffer, &mut slice)
                    })
                    .map_err(ErrorCode::from)
            }

            _ => Err(ErrorCode::NOSUPPORT),
        };

        if let Err(e) = res {
            Err((slice, e))
        } else {
            Ok(slice)
        }
    }

    // Register a callback to the Nrf51822Serialization driver.
    //
    // The callback will be called when a TX finishes and when
    // RX data is available.
    //
    // ### `subscribe_num`
    //
    // - `0`: Set callback.

    /// Issue a command to the Nrf51822Serialization driver.
    ///
    /// ### `command_type`
    ///
    /// - `0`: Driver check.
    /// - `1`: Send the allowed buffer to the nRF.
    /// - `2`: Received from the nRF into the allowed buffer.
    /// - `3`: Reset the nRF51822.
    fn command(
        &self,
        command_type: usize,
        arg1: usize,
        _: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_type {
            0 /* check if present */ => CommandReturn::success(),

            // Send a buffer to the nRF51822 over UART.
            1 => {
                self.apps.enter(appid, |app, _| {
                    app.tx_buffer.enter(|slice| {
                        let write_len = slice.len();
                        self.tx_buffer.take().map_or(CommandReturn::failure(ErrorCode::FAIL), |buffer| {
                            for (i, c) in slice.iter().enumerate() {
                                buffer[i] = c.get();
                            }
                            let _ = self.uart.transmit_buffer(buffer, write_len);
                            CommandReturn::success()
                        })
                    }).unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
                }).unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            // Receive from the nRF51822
            2 => {
                self.rx_buffer.take().map_or(CommandReturn::failure(ErrorCode::RESERVE), |buffer| {
                    let len = arg1;
                    if len > buffer.len() {
                        CommandReturn::failure(ErrorCode::SIZE)
                    } else {
                        let _ = self.uart.receive_automatic(buffer, len, 250);
                        CommandReturn::success_u32(len as u32)
                    }
                })
            }

            // Initialize the nRF51822 by resetting it.
            3 => {
                self.reset();
                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

// Callbacks from the underlying UART driver.
impl uart::TransmitClient for Nrf51822Serialization<'_> {
    // Called when the UART TX has finished.
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        _rcode: Result<(), ErrorCode>,
    ) {
        self.tx_buffer.replace(buffer);

        self.active_app.map(|appid| {
            let _ = self.apps.enter(*appid, |_app, upcalls| {
                // Call the callback after TX has finished
                upcalls.schedule_upcall(0, 1, 0, 0);
            });
        });
    }

    fn transmitted_word(&self, _rcode: Result<(), ErrorCode>) {}
}

impl uart::ReceiveClient for Nrf51822Serialization<'_> {
    // Called when a buffer is received on the UART.
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: Result<(), ErrorCode>,
        _error: uart::Error,
    ) {
        self.rx_buffer.replace(buffer);

        self.active_app.map(|appid| {
            let _ = self.apps.enter(*appid, |app, upcalls| {
                let len = app
                    .rx_buffer
                    .mut_enter(|rb| {
                        // Figure out length to copy.
                        let max_len = cmp::min(rx_len, rb.len());

                        // Copy over data to app buffer.
                        self.rx_buffer.map_or(0, |buffer| {
                            for idx in 0..max_len {
                                rb[idx].set(buffer[idx]);
                            }
                            max_len
                        })
                    })
                    .unwrap_or(0);

                // Notify the serialization library in userspace about the
                // received buffer.
                //
                // Note: This indicates how many bytes were received by
                // hardware, regardless of how much space (if any) was
                // available in the buffer provided by the app.
                upcalls.schedule_upcall(0, 4, rx_len, len);
            });
        });

        // Restart the UART receive.
        self.rx_buffer.take().map(|buffer| {
            let len = buffer.len();
            let _ = self.uart.receive_automatic(buffer, len, 250);
        });
    }

    fn received_word(&self, _word: u32, _rcode: Result<(), ErrorCode>, _err: uart::Error) {}
}
