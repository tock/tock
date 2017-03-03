//! Symmetric Block Cipher Capsule
//!
//! Provides a simple driver for userspace applications to encrypt and decrypt messages using block
//!
//! The key is assumed to be 16 bytes and configured once
//!
//! We have been thinking about the logic specifically for chip on nrf51dk and how to deal with
//! messages that are longer than 27 bytes.
//! Because the limit for the un-encrypted payload is 27 bytes and encrypted payload is 31 bytes
//! (note this will probably differ between different chips)
//!
//! However, we are considering two different approaches:
//!     1) pad each to be equal to 27 bytes i.e. if a message is 5 bytes add 22 bytes of zeros or
//!        similar. Or if the message is longer than 27 bytes, slice the first 27 bytes and then
//!        continue until all bytes have been encrypted/decrypted. The logic becomes
//!        straightforward
//!     2) use dynamic size i.e can be 4,5,6 etc and the logic need to handle that. May be a little
//!        bit trickier to find the MAC/MIC and so on.
//!
//! A good idea is perhaps to implement the capsule similar to the rng capsule, with a variable
//! that keeps track of the number of remaining bytes to be encrypted/decrypted.
//!
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 03, 2017

use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{SymmetricEncryptionDriver, Client};
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

pub struct Crypto<'a, E: SymmetricEncryptionDriver + 'a> {
    crypto: &'a E,
    apps: Container<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    key_configured: Cell<bool>,
    busy: Cell<bool>,
    remaining: Cell<usize>,
}

impl<'a, E: SymmetricEncryptionDriver + 'a> Crypto<'a, E> {
    pub fn new(crypto: &'a E, container: Container<App>, buf: &'static mut [u8]) -> Crypto<'a, E> {
        Crypto {
            crypto: crypto,
            apps: container,
            kernel_tx: TakeCell::new(buf),
            key_configured: Cell::new(false),
            busy: Cell::new(false),
            remaining: Cell::new(0),
        }
    }
}

impl<'a, E: SymmetricEncryptionDriver + 'a> Client for Crypto<'a, E> {
    fn encrypt_done(&self, ct: &'static mut [u8], len: u8) -> ReturnCode {
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
        // indicate that the encryption driver not busy
        self.busy.set(false);
        self.kernel_tx.replace(ct);
        ReturnCode::SUCCESS
    }

    fn decrypt_done(&self, pt: &'static mut [u8], len: u8) -> ReturnCode {
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
        // indicate that the encryption driver not busy
        self.busy.set(false);
        self.kernel_tx.replace(pt);
        ReturnCode::SUCCESS
    }

    fn set_key_done(&self, key: &'static mut [u8], _: u8) -> ReturnCode {
        // this callback may be un-necessary
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| { app.callback.map(|mut cb| { cb.schedule(0, 0, 0); }); });
        }
        self.kernel_tx.replace(key);
        // indicate that the key is configured
        self.key_configured.set(true);
        // indicate that the encryption driver not busy
        self.busy.set(false);
        ReturnCode::SUCCESS
    }
}


impl<'a, E: SymmetricEncryptionDriver> Driver for Crypto<'a, E> {
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
    fn command(&self, command_num: usize, len: usize, _: AppId) -> ReturnCode {
        match command_num {
            // set key, it is assumed that it is always 16 bytes
            // can only be performed once at the moment
            0 => {
                if len == 16 && !self.key_configured.get() && !self.busy.get() {
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
                } else {
                    ReturnCode::FAIL
                }
            }
            // encryption driver
            // FIXME: better error handling
            1 => {
                if self.key_configured.get() && !self.busy.get() {
                    for cntr in self.apps.iter() {
                        self.busy.set(true);
                        cntr.enter(|app, _| {
                            app.ct_buf.as_mut().map(|slice| {
                                self.kernel_tx.take().map(|buf| {
                                    for (i, c) in slice.as_ref()[0..len]
                                        .iter()
                                        .enumerate() {
                                        // buf len is not the same as len abort
                                        if buf.len() < i {
                                            self.busy.set(false);
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
                } else {
                    ReturnCode::EBUSY
                }
            }
            2 => {
                if self.key_configured.get() && !self.busy.get() {
                    for cntr in self.apps.iter() {
                        self.busy.set(true);
                        cntr.enter(|app, _| {
                            app.pt_buf.as_mut().map(|slice| {
                                self.kernel_tx.take().map(|buf| {
                                    for (i, c) in slice.as_ref()[0..len]
                                        .iter()
                                        .enumerate() {
                                        if buf.len() < i {
                                            self.busy.set(false);
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
                } else {
                    ReturnCode::EBUSY
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
