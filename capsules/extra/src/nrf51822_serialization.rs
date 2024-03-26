// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::hil::uart;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::packet_buffer::PacketBufferMut;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Nrf51822Serialization as usize;

/// IDs for subscribed upcalls.
mod upcall {
    /// Callback will be called when a TX finishes and when RX data is
    /// available.
    pub const TX_DONE_RX_READY: usize = 0;
    /// Number of upcalls.
    pub const COUNT: u8 = 1;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// TX buffer.
    ///
    /// This also sets which app is currently using this driver. Only one app
    /// can control the nRF51 serialization driver.
    pub const TX: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// RX buffer.
    ///
    /// This also sets which app is currently using this driver. Only one app
    /// can control the nRF51 serialization driver.
    pub const RX: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

#[derive(Default)]
pub struct App;

// Local buffer for passing data between applications and the underlying
// transport hardware.
pub const WRITE_BUF_LEN: usize = 600;
pub const READ_BUF_LEN: usize = 600;

// We need two resources: a UART HW driver and driver state for each
// application.
pub struct Nrf51822Serialization<'a> {
    uart: &'a dyn uart::UartAdvanced<'a>,
    reset_pin: &'a dyn hil::gpio::Pin,
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    active_app: OptionalCell<ProcessId>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a> Nrf51822Serialization<'a> {
    pub fn new(
        uart: &'a dyn uart::UartAdvanced<'a>,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
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

impl SyscallDriver for Nrf51822Serialization<'_> {
    /// Issue a command to the Nrf51822Serialization driver.
    ///
    /// ### `command_type`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Send the allowed buffer to the nRF.
    /// - `2`: Received from the nRF into the allowed buffer.
    /// - `3`: Reset the nRF51822.
    fn command(
        &self,
        command_type: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_type {
            0 => CommandReturn::success(),

            // Send a buffer to the nRF51822 over UART.
            1 => {
                self.apps
                    .enter(processid, |_, kernel_data| {
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::TX)
                            .and_then(|tx| {
                                tx.enter(|slice| {
                                    let write_len = slice.len();
                                    self.tx_buffer.take().map_or(
                                        CommandReturn::failure(ErrorCode::FAIL),
                                        |buffer| {
                                            for (i, c) in slice.iter().enumerate() {
                                                buffer[i] = c.get();
                                            }
                                            // Set this as the active app for the transmit callback
                                            self.active_app.set(processid);
                                            let _ = self.uart.transmit_buffer(buffer, write_len);
                                            CommandReturn::success()
                                        },
                                    )
                                })
                            })
                            .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            // Receive from the nRF51822
            2 => {
                let len = arg1;

                // We only allow one app to use the NRF serialization capsule
                // (old legacy code, and a difficult thing to virtualize).
                // However, we would like to support restarting/updating apps.
                // But we don't want to allow a simultaneous app to disrupt the
                // app that got to the BLE serialization first. So we have to
                // find a compromise.
                //
                // We handle this by checking if the current active app still
                // exists. If it does, we leave it alone. Otherwise, we replace
                // it.
                self.active_app.map_or_else(
                    || {
                        // The app is not set, handle this for the normal case.
                        self.rx_buffer.take().map_or(
                            CommandReturn::failure(ErrorCode::RESERVE),
                            |buffer| {
                                if len > buffer.len() {
                                    CommandReturn::failure(ErrorCode::SIZE)
                                } else {
                                    // Set this as the active app for the
                                    // receive callback.
                                    self.active_app.set(processid);
                                    let _ = self.uart.receive_automatic(buffer, len, 250);
                                    CommandReturn::success_u32(len as u32)
                                }
                            },
                        )
                    },
                    |processid| {
                        // The app is set, check if it still exists.
                        if let Err(kernel::process::Error::NoSuchApp) =
                            self.apps.enter(processid, |_, _| {})
                        {
                            // The app we had as active no longer exists.
                            self.active_app.clear();
                            self.rx_buffer.take().map_or_else(
                                || {
                                    // We do not have the RF buffer as it is
                                    // currently in use by the underlying UART.
                                    // We don't have to do anything else except
                                    // update the active app.
                                    self.active_app.set(processid);
                                    CommandReturn::success_u32(len as u32)
                                },
                                |buffer| {
                                    if len > buffer.len() {
                                        CommandReturn::failure(ErrorCode::SIZE)
                                    } else {
                                        self.active_app.set(processid);
                                        // Use the buffer to start the receive.
                                        let _ = self.uart.receive_automatic(buffer, len, 250);
                                        CommandReturn::success_u32(len as u32)
                                    }
                                },
                            )
                        } else {
                            // Active app exists. Return error as there can only
                            // be one app using this capsule.
                            CommandReturn::failure(ErrorCode::RESERVE)
                        }
                    },
                )
            }

            // Initialize the nRF51822 by resetting it.
            3 => {
                self.reset();
                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

// Callbacks from the underlying UART driver.
impl uart::TransmitClient for Nrf51822Serialization<'_> {
    // Called when the UART TX has finished.
    fn transmitted_buffer(
        &self,
        buffer: PacketBufferMut,
        _tx_len: usize,
        _rcode: Result<(), ErrorCode>,
    ) {
        self.tx_buffer.replace(buffer);

        self.active_app.map(|processid| {
            let _ = self.apps.enter(processid, |_app, kernel_data| {
                // Call the callback after TX has finished
                kernel_data
                    .schedule_upcall(upcall::TX_DONE_RX_READY, (1, 0, 0))
                    .ok();
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

        // By default we continuously receive on UART. However, if we receive
        // and the active app is no longer existent, then we stop receiving.
        let mut repeat_receive = true;

        self.active_app.map(|processid| {
            if let Err(_err) = self.apps.enter(processid, |_, kernel_data| {
                let len = kernel_data
                    .get_readwrite_processbuffer(rw_allow::RX)
                    .and_then(|rx| {
                        rx.mut_enter(|rb| {
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
                    })
                    .unwrap_or(0);

                // Notify the serialization library in userspace about the
                // received buffer.
                //
                // Note: This indicates how many bytes were received by
                // hardware, regardless of how much space (if any) was
                // available in the buffer provided by the app.
                kernel_data
                    .schedule_upcall(upcall::TX_DONE_RX_READY, (4, rx_len, len))
                    .ok();
            }) {
                // The app we had as active no longer exists. Clear that and
                // stop receiving. This puts us back in an idle state. A new app
                // can use the BLE serialization.
                self.active_app.clear();
                repeat_receive = false;
            }
        });

        if repeat_receive {
            // Restart the UART receive.
            self.rx_buffer.take().map(|buffer| {
                let len = buffer.len();
                let _ = self.uart.receive_automatic(buffer, len, 250);
            });
        }
    }

    fn received_word(&self, _word: u32, _rcode: Result<(), ErrorCode>, _err: uart::Error) {}
}
