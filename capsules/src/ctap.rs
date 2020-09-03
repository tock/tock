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
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Ctap as usize;

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

pub static mut WRITE_BUF: [u8; 64] = [0; 64];
pub static mut READ_BUF: [u8; 64] = [0; 64];

pub struct CtapDriver<'a, U: usb_hid::UsbHid<'a>> {
    usb: Option<&'a U>,

    app: Grant<App>,
    recv_appid: OptionalCell<AppId>,
    send_appid: OptionalCell<AppId>,
    phantom: PhantomData<&'a U>,

    send_buffer: TakeCell<'static, [u8; 64]>,
    recv_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a, U: usb_hid::UsbHid<'a>> CtapDriver<'a, U> {
    pub fn new(
        usb: Option<&'a U>,
        send_buffer: &'static mut [u8; 64],
        recv_buffer: &'static mut [u8; 64],
        grant: Grant<App>,
    ) -> CtapDriver<'a, U> {
        CtapDriver {
            usb: usb,
            app: grant,
            recv_appid: OptionalCell::empty(),
            send_appid: OptionalCell::empty(),
            phantom: PhantomData,
            send_buffer: TakeCell::new(send_buffer),
            recv_buffer: TakeCell::new(recv_buffer),
        }
    }

    pub fn allow_receive(&'a self) {
        if let Some(usb) = self.usb {
            usb.set_recv_buffer(self.recv_buffer.take().unwrap());
        }
    }
}

impl<'a, U: usb_hid::UsbHid<'a>> usb_hid::UsbHid<'a> for CtapDriver<'a, U> {
    fn set_recv_buffer(&'a self, recv: &'static mut [u8; 64]) {
        if let Some(usb) = self.usb {
            usb.set_recv_buffer(recv);
        }
    }

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

    fn allow_receive(&'a self) {
        unreachable!()
    }
}

impl<'a, U: usb_hid::UsbHid<'a>> usb_hid::Client<'a> for CtapDriver<'a, U> {
    fn packet_received(&'a self, buffer: &'static mut [u8; 64], _len: usize, _endpoint: usize) {
        self.recv_appid.map(|id| {
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

        // Give the USB back it's receive buffer
        if let Some(usb) = self.usb {
            usb.set_recv_buffer(buffer);
        }
    }

    fn packet_transmitted(
        &'a self,
        _result: Result<(), ReturnCode>,
        buffer: &'static mut [u8; 64],
        _len: usize,
        _endpoint: usize,
    ) {
        self.send_appid.map(|id| {
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
        self.recv_appid
            .map(|id| {
                self.app
                    .enter(*id, |app, _| app.can_receive.get())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

impl<'a, U: usb_hid::UsbHid<'a>> Driver for CtapDriver<'a, U> {
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Pass buffer for the recvieved data to be stored in
            0 => self
                .app
                .enter(appid, |app, _| {
                    app.recv_buf = slice;
                    self.recv_appid.set(appid);
                    ReturnCode::SUCCESS
                })
                .unwrap_or(ReturnCode::FAIL),

            // Pass buffer for the sent data to be stored in
            1 => self
                .app
                .enter(appid, |app, _| {
                    app.send_buf = slice;
                    self.send_appid.set(appid);
                    ReturnCode::SUCCESS
                })
                .unwrap_or(ReturnCode::FAIL),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Subscribe to HmacDriver events.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Subscribe to interrupts from HMAC events.
    ///        The callback signature is `fn(result: u32)`
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        appid: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                // set recv callback
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
        appid: AppId,
    ) -> ReturnCode {
        match command_num {
            // Send data
            0 => self
                .app
                .enter(appid, |app, _| {
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
                    if let Some(usb) = self.usb {
                        app.can_receive.set(true);
                        usb.allow_receive();
                        ReturnCode::SUCCESS
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
