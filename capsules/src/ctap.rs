//! Provides userspace with access CTAP devices over any transport
//! layer (USB HID, BLE, NFC). Currently only USB HID is supported.
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::usb::UsbController` and
//! `hil::usb_hid::UsbHid` trait.
//!
//! ```rust
//!     let ctap_send_buffer = static_init!([u8; 64], [0; 64]);
//!     let ctap_recv_buffer = static_init!([u8; 64], [0; 64]);
//!
//!     let (ctap, ctap_driver) = components::ctap::CtapComponent::new(
//!         &earlgrey::usbdev::USB,
//!         0x1337, // My important company
//!         0x0DEC, // My device name
//!         strings,
//!         board_kernel,
//!         ctap_send_buffer,
//!         ctap_recv_buffer,
//!     )
//!     .finalize(components::usb_ctap_component_helper!(lowrisc::usbdev::Usb));
//!
//!     ctap.enable();
//!     ctap.attach();
//! ```
//!

use core::cell::Cell;
use core::marker::PhantomData;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::usb_hid;
use kernel::{AppSlice, Callback, Driver, Grant, ProcessId, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::CtapHid as usize;

pub struct App {
    callback: OptionalCell<Callback>,
    recv_buf: Option<AppSlice<Shared, u8>>,
    send_buf: Option<AppSlice<Shared, u8>>,
    can_receive: Cell<bool>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: OptionalCell::empty(),
            recv_buf: None,
            send_buf: None,
            can_receive: Cell::new(false),
        }
    }
}

pub struct CtapDriver<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> {
    usb: Option<&'a U>,

    app: Grant<App>,
    appid: OptionalCell<ProcessId>,
    phantom: PhantomData<&'a U>,

    send_buffer: TakeCell<'static, [u8; 64]>,
    recv_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> CtapDriver<'a, U> {
    pub fn new(
        usb: Option<&'a U>,
        send_buffer: &'static mut [u8; 64],
        recv_buffer: &'static mut [u8; 64],
        grant: Grant<App>,
    ) -> CtapDriver<'a, U> {
        CtapDriver {
            usb: usb,
            app: grant,
            appid: OptionalCell::empty(),
            phantom: PhantomData,
            send_buffer: TakeCell::new(send_buffer),
            recv_buffer: TakeCell::new(recv_buffer),
        }
    }
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> usb_hid::UsbHid<'a, [u8; 64]> for CtapDriver<'a, U> {
    fn send_buffer(
        &'a self,
        send: &'static mut [u8; 64],
    ) -> Result<usize, (ReturnCode, &'static mut [u8; 64])> {
        if let Some(usb) = self.usb {
            usb.send_buffer(send)
        } else {
            Err((ReturnCode::ENOSUPPORT, send))
        }
    }

    fn send_cancel(&'a self) -> Result<&'static mut [u8; 64], ReturnCode> {
        if let Some(usb) = self.usb {
            usb.send_cancel()
        } else {
            Err(ReturnCode::ENOSUPPORT)
        }
    }

    fn receive_buffer(
        &'a self,
        recv: &'static mut [u8; 64],
    ) -> Result<(), (ReturnCode, &'static mut [u8; 64])> {
        if let Some(usb) = self.usb {
            usb.receive_buffer(recv)
        } else {
            Err((ReturnCode::ENODEVICE, recv))
        }
    }

    fn receive_cancel(&'a self) -> Result<&'static mut [u8; 64], ReturnCode> {
        if let Some(usb) = self.usb {
            usb.receive_cancel()
        } else {
            Err(ReturnCode::ENOSUPPORT)
        }
    }
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> usb_hid::Client<'a, [u8; 64]> for CtapDriver<'a, U> {
    fn packet_received(
        &'a self,
        _result: ReturnCode,
        buffer: &'static mut [u8; 64],
        _endpoint: usize,
    ) {
        self.appid.map(|id| {
            self.app
                .enter(*id, |app, _| {
                    match app.recv_buf.as_mut() {
                        Some(dest) => {
                            dest.as_mut().copy_from_slice(buffer.as_ref());
                        }
                        None => {}
                    };

                    app.callback.map(|cb| {
                        cb.schedule(0, 0, 0);
                        // Set that we can't receive until the app says we can
                        app.can_receive.set(false);
                    });
                })
                .map_err(|err| {
                    if err == kernel::procs::Error::NoSuchApp
                        || err == kernel::procs::Error::InactiveApp
                    {}
                })
        });

        self.recv_buffer.replace(buffer);
    }

    fn packet_transmitted(
        &'a self,
        _result: ReturnCode,
        buffer: &'static mut [u8; 64],
        _endpoint: usize,
    ) {
        self.appid.map(|id| {
            self.app
                .enter(*id, |app, _| {
                    app.callback.map(|cb| {
                        cb.schedule(1, 0, 0);
                    });
                })
                .map_err(|err| {
                    if err == kernel::procs::Error::NoSuchApp
                        || err == kernel::procs::Error::InactiveApp
                    {}
                })
        });

        // Save our send buffer so we can use it later
        self.send_buffer.replace(buffer);
    }

    fn can_receive(&'a self) -> bool {
        self.appid
            .map(|id| {
                self.app
                    .enter(*id, |app, _| app.can_receive.get())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> Driver for CtapDriver<'a, U> {
    fn allow(
        &self,
        appid: ProcessId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Pass buffer for the recvieved data to be stored in
            0 => self
                .app
                .enter(appid, |app, _| {
                    app.recv_buf = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or(ReturnCode::FAIL),

            // Pass buffer for the sent data to be stored in
            1 => self
                .app
                .enter(appid, |app, _| {
                    app.send_buf = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or(ReturnCode::FAIL),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Subscribe to CtapDriver events.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Subscribe to interrupts from Ctap events.
    ///        The callback signature is `fn(direction: u32)`
    ///        `fn(0)` indicates a packet was recieved
    ///        `fn(1)` indicates a packet was transmitted
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        appid: ProcessId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                // set callback
                self.app
                    .enter(appid, |app, _| {
                        app.callback.insert(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or(ReturnCode::FAIL)
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        appid: ProcessId,
    ) -> ReturnCode {
        let can_access = self.appid.map_or(true, |owning_app| {
            if owning_app == &appid {
                // We own the Ctap device
                true
            } else {
                false
            }
        });

        if !can_access {
            return ReturnCode::EBUSY;
        }

        match command_num {
            // Send data
            0 => self
                .app
                .enter(appid, |app, _| {
                    self.appid.set(appid);
                    if let Some(usb) = self.usb {
                        match app.send_buf.as_ref() {
                            Some(d) => {
                                self.send_buffer.take().map(|buf| {
                                    let data = d.as_ref();

                                    // Copy the data into the static buffer
                                    buf.copy_from_slice(&data[0..]);

                                    let _ = usb.send_buffer(buf);
                                });
                            }
                            None => {
                                return ReturnCode::ERESERVE;
                            }
                        };
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                })
                .unwrap_or_else(|err| err.into()),
            // Allow receive
            1 => self
                .app
                .enter(appid, |app, _| {
                    self.appid.set(appid);
                    if let Some(usb) = self.usb {
                        app.can_receive.set(true);
                        if let Some(buf) = self.recv_buffer.take() {
                            match usb.receive_buffer(buf) {
                                Ok(_) => ReturnCode::SUCCESS,
                                Err((err, buffer)) => {
                                    self.recv_buffer.replace(buffer);
                                    err
                                }
                            }
                        } else {
                            ReturnCode::EBUSY
                        }
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                })
                .unwrap_or_else(|err| err.into()),
            // Cancel send
            2 => self
                .app
                .enter(appid, |_app, _| {
                    self.appid.set(appid);
                    if let Some(usb) = self.usb {
                        match usb.receive_cancel() {
                            Ok(buf) => {
                                self.recv_buffer.replace(buf);
                                ReturnCode::ENOSUPPORT
                            }
                            Err(err) => err,
                        }
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                })
                .unwrap_or_else(|err| err.into()),
            // Cancel receive
            3 => self
                .app
                .enter(appid, |_app, _| {
                    self.appid.set(appid);
                    if let Some(usb) = self.usb {
                        match usb.receive_cancel() {
                            Ok(buf) => {
                                self.recv_buffer.replace(buf);
                                ReturnCode::ENOSUPPORT
                            }
                            Err(err) => err,
                        }
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                })
                .unwrap_or_else(|err| err.into()),
            // Send or receive
            // This command has two parts.
            //    Part 1: Receive
            //            This will allow receives, the same as the Allow
            //            receive command above. If data is ready to receive
            //            the `packet_received()` callback will be called.
            //            When this happens the client callback will be
            //            scheduled and no send event will occur.
            //    Part 2: Send
            //            If no receive occurs we will be left in a start where
            //            future recieves will be allowed. This is the same
            //            outcome as calling the Allow receive command.
            //            As well as that we will then send the data in the
            //            send buffer.
            4 => self
                .app
                .enter(appid, |app, _| {
                    if let Some(usb) = self.usb {
                        if app.can_receive.get() {
                            // We are already receiving
                            ReturnCode::EBUSY
                        } else {
                            app.can_receive.set(true);
                            if let Some(buf) = self.recv_buffer.take() {
                                match usb.receive_buffer(buf) {
                                    Ok(_) => ReturnCode::SUCCESS,
                                    Err((err, buffer)) => {
                                        self.recv_buffer.replace(buffer);
                                        return err;
                                    }
                                }
                            } else {
                                return ReturnCode::EBUSY;
                            };

                            if !app.can_receive.get() {
                                // The call to receive_buffer() collected a pending packet.
                            } else {
                                match app.send_buf.as_ref() {
                                    Some(d) => {
                                        self.send_buffer.take().map(|buf| {
                                            let data = d.as_ref();

                                            // Copy the data into the static buffer
                                            buf.copy_from_slice(&data[0..]);

                                            let _ = usb.send_buffer(buf);
                                        });
                                    }
                                    None => {
                                        return ReturnCode::ERESERVE;
                                    }
                                };
                            }
                            ReturnCode::SUCCESS
                        }
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
