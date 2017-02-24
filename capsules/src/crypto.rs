//! Crypto Capsule
//!
//!
//! Provides a simple driver for userspace applications to encrypt and decrypt messages


use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil::aes::{AESDriver, Client};
use kernel::process::Error;

pub static mut BUF: [u8; 64] = [0; 64];

pub struct App {
    callback: Option<Callback>,
    key_buf: Option<AppSlice<Shared, u8>>,
    pt_buf: Option<AppSlice<Shared, u8>>,
    ct_buf: Option<AppSlice<Shared, u8>>,
}


impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            key_buf: None,
            pt_buf: None,
            ct_buf: None,
        }
    }
}

pub struct Crypto<'a, E: AESDriver + 'a> {
    crypto: &'a E,
    apps: Container<App>,
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a, E: AESDriver + 'a> Crypto<'a, E> {
    pub fn new(crypto: &'a E, container: Container<App>, buf: &'static mut [u8]) -> Crypto<'a, E> {
        Crypto {
            crypto: crypto,
            apps: container,
            kernel_tx: TakeCell::new(buf),
        }
    }
}

impl<'a, E: AESDriver + 'a> Client for Crypto<'a, E> {
    fn encrypt_done(&self, ct: &'static mut [u8], len: u8) -> ReturnCode {
        // panic!("ct {:?}\r\n", ct);
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.ct_buf.is_some() {
                    let dest = app.ct_buf.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in ct[0..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                }
                app.callback.map(|mut cb| { cb.schedule(1, 0, 0); });
            });
        }
        self.kernel_tx.replace(ct);
        ReturnCode::SUCCESS
    }

    fn decrypt_done(&self, pt: &'static mut [u8], len: u8) -> ReturnCode {
        // panic!("decrypt done {:?}\r\n LEN: {:?}\r\n", pt, len);
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.pt_buf.is_some() {
                    let dest = app.pt_buf.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in pt[0..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                }
                app.callback.map(|mut cb| { cb.schedule(2, 0, 0); });
            });
        }
        self.kernel_tx.replace(pt);
        ReturnCode::SUCCESS
    }

    fn set_key_done(&self, key: &'static mut [u8], len: u8) -> ReturnCode {
        // panic!("KEY {:?}\n", key);
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| { app.callback.map(|mut cb| { cb.schedule(0, 0, 0); }); });
        }
        self.kernel_tx.replace(key);
        ReturnCode::SUCCESS
    }
}


impl<'a, E: AESDriver> Driver for Crypto<'a, E> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.key_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.ct_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            2 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.pt_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
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
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    // This code violates the DRY-principle but don't care about it at moment
    fn command(&self, command_num: usize, len: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => {
                for cntr in self.apps.iter() {
                    cntr.enter(|app, _| {
                        app.key_buf.as_mut().map(|slice| {
                            self.kernel_tx.take().map(|buf| {
                                for (i, c) in slice.as_ref()[0..len]
                                    .iter()
                                    .enumerate() {
                                    if buf.len() < i {
                                        break;
                                    }
                                    buf[i] = *c;
                                }
                                self.crypto.set_key(buf, len as u8);
                            });
                        });
                    });
                }
                ReturnCode::SUCCESS
            }
            1 => {
                for cntr in self.apps.iter() {
                    cntr.enter(|app, _| {
                        app.ct_buf.as_mut().map(|slice| {
                            self.kernel_tx.take().map(|buf| {
                                for (i, c) in slice.as_ref()[0..len]
                                    .iter()
                                    .enumerate() {
                                    if buf.len() < i {
                                        break;
                                    }
                                    buf[i] = *c;
                                }
                                self.crypto.encrypt(buf, len as u8);
                            });
                        });
                    });
                }
                ReturnCode::SUCCESS
            }
            2 => {
                for cntr in self.apps.iter() {
                    cntr.enter(|app, _| {
                        app.pt_buf.as_mut().map(|slice| {
                            self.kernel_tx.take().map(|buf| {
                                // panic!("DECRYPT\r\n");
                                for (i, c) in slice.as_ref()[0..len]
                                    .iter()
                                    .enumerate() {
                                    if buf.len() < i {
                                        break;
                                    }
                                    buf[i] = *c;
                                }
                                self.crypto.decrypt(buf, len as u8);
                            });
                        });
                    });
                }
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
