//! CRC driver

use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::hil;
use kernel::hil::crc::CrcAlg;
use kernel::process::Error;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,

    // if Some, the application is awaiting the result of a CRC
    //   using the given algorithm
    waiting: Option<hil::crc::CrcAlg>,
}

pub struct Crc<'a, C: hil::crc::CRC + 'a> {
    crc_unit: &'a C,
    apps: Container<App>,
    serving_app: Cell<Option<AppId>>,
}

impl<'a, C: hil::crc::CRC> Crc<'a, C> {
    pub fn new(crc_unit: &'a C, apps: Container<App>) -> Crc<'a, C> {
        Crc { crc_unit: crc_unit,
              apps: apps,
              serving_app: Cell::new(None),
            }
    }

    fn serve_waiting_apps(&self) {
        if self.serving_app.get().is_some() {
            // A computation is in progress
            return;
        }

        // Find a waiting app and start its requested computation
        let mut found = false;
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if let Some(poly) = app.waiting {
                    if let Some(buffer) = app.buffer.take() {
                        let r = self.crc_unit.compute(buffer.as_ref(), poly);
                        if r == ReturnCode::SUCCESS {
                            // The unit is now computing a CRC for this app
                            self.serving_app.set(Some(app.appid()));
                            found = true;
                        }
                        else {
                            // The app's request failed
                            if let Some(mut callback) = app.callback {
                                callback.schedule(From::from(r), 0, 0);
                            }
                            app.waiting = None;
                        }

                        // Put back taken buffer
                        app.buffer = Some(buffer);
                    }
                }
            });
            if found { break }
        }

        if !found {
            // Power down the CRC unit until next needed
            self.crc_unit.disable();
        }
    }
}

impl<'a, C: hil::crc::CRC> Driver for Crc<'a, C>  {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            // Provide user buffer to compute CRC over
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.buffer = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Set callback for CRC result
            0 => {
                self.apps
                    .enter(callback.app_id(), |app, _| {
                        app.callback = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // This driver is present
            0 => ReturnCode::SUCCESS,

            // Get version of CRC unit
            1 => ReturnCode::SuccessWithValue {
                                value: self.crc_unit.get_version() as usize
                             },

            // Request a CRC computation
            2 => {
                let result =
                    if let Some(alg) = alg_from_user_int(data) {
                        self.apps
                            .enter(appid, |app, _| {
                                if app.waiting.is_some() {
                                    // Each app may make only one request at a time
                                    ReturnCode::EBUSY
                                }
                                else {
                                    if app.callback.is_some() && app.buffer.is_some() {
                                        app.waiting = Some(alg);
                                        ReturnCode::SUCCESS
                                    }
                                    else { ReturnCode::EINVAL }
                                }
                            })
                            .unwrap_or_else(|err| {
                                match err {
                                    Error::OutOfMemory => ReturnCode::ENOMEM,
                                    Error::AddressOutOfBounds => ReturnCode::EINVAL,
                                    Error::NoSuchApp => ReturnCode::EINVAL,
                                }
                            })
                    }
                    else {
                        ReturnCode::EINVAL
                    };

                if result == ReturnCode::SUCCESS {
                    self.serve_waiting_apps();
                }
                result
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, C: hil::crc::CRC> hil::crc::Client for Crc<'a, C> {
    fn receive_result(&self, result: u32) {
        if let Some(appid) = self.serving_app.get() {
            self.apps
                .enter(appid, |app, _| {
                    if let Some(mut callback) = app.callback {
                        callback.schedule(From::from(ReturnCode::SUCCESS), result as usize, 0);
                    }
                    app.waiting = None;
                })
                .unwrap_or_else(|err| {
                    match err {
                        Error::OutOfMemory => {},
                        Error::AddressOutOfBounds => {},
                        Error::NoSuchApp => {},
                    }
                });

            self.serving_app.set(None);
            self.serve_waiting_apps();
        }
        else {
            // Ignore orphaned computation
        }
    }
}

fn alg_from_user_int(i: usize) -> Option<hil::crc::CrcAlg> {
    match i {
        0 => Some(CrcAlg::Crc32),
        1 => Some(CrcAlg::Crc32C),
        2 => Some(CrcAlg::Sam4L16),
        3 => Some(CrcAlg::Sam4L32),
        4 => Some(CrcAlg::Sam4L32C),
        _ => None
    }
}
