//! Symmetric Block Cipher Capsule
//!
//! Provides a simple driver for userspace applications to encrypt and decrypt messages
//!
//! The key is assumed to be 16 bytes and configured once
//! FIXME: other key lengths e.g. AES256 should be supported
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 16, 2017

use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{SymmetricEncryptionDriver, Client};
use kernel::process::Error;

pub static mut BUF: [u8; 128] = [0; 128];
pub static mut IV: [u8; 16] = [0; 16];


// This enum shall keep track of the state of the AESDriver
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CryptoState {
    IDLE,
    ENCRYPT,
    DECRYPT,
    SETKEY,
}

pub struct App {
    callback: Option<Callback>,
    key_buf: Option<AppSlice<Shared, u8>>,
    data_buf: Option<AppSlice<Shared, u8>>,
    ctr_buf: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            key_buf: None,
            data_buf: None,
            ctr_buf: None,
        }
    }
}

pub struct Crypto<'a, E: SymmetricEncryptionDriver + 'a> {
    crypto: &'a E,
    apps: Container<App>,
    kernel_key: TakeCell<'static, [u8]>,
    kernel_data: TakeCell<'static, [u8]>,
    kernel_ctr: TakeCell<'static, [u8]>,
    key_configured: Cell<bool>,
    busy: Cell<bool>,
    state: Cell<CryptoState>,
}

impl<'a, E: SymmetricEncryptionDriver + 'a> Crypto<'a, E> {
    pub fn new(crypto: &'a E,
               container: Container<App>,
               buf1: &'static mut [u8],
               buf2: &'static mut [u8],
               ctr: &'static mut [u8])
               -> Crypto<'a, E> {
        Crypto {
            crypto: crypto,
            apps: container,
            kernel_key: TakeCell::new(buf1),
            kernel_data: TakeCell::new(buf2),
            kernel_ctr: TakeCell::new(ctr),
            key_configured: Cell::new(false),
            busy: Cell::new(false),
            state: Cell::new(CryptoState::IDLE),
        }
    }
}

impl<'a, E: SymmetricEncryptionDriver + 'a> Client for Crypto<'a, E> {
    fn crypt_done(&self, data: &'static mut [u8], len: u8) -> ReturnCode {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.data_buf.is_some() {
                    let dest = app.data_buf.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in data[0..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                }
                if self.state.get() == CryptoState::ENCRYPT {
                    app.callback.map(|mut cb| { cb.schedule(1, 0, 0); });
                } else if self.state.get() == CryptoState::DECRYPT {
                    app.callback.map(|mut cb| { cb.schedule(2, 0, 0); });
                }
            });
        }
        // tmp
        unsafe {
            self.kernel_ctr.replace(&mut IV);
        }
        // indicate that the encryption driver not busy
        self.busy.set(false);
        self.state.set(CryptoState::IDLE);
        self.kernel_data.replace(data);
        ReturnCode::SUCCESS
    }

    fn set_key_done(&self, key: &'static mut [u8], _: u8) -> ReturnCode {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| { app.callback.map(|mut cb| { cb.schedule(0, 0, 0); }); });
        }
        self.kernel_key.replace(key);
        // indicate that the key is configured
        self.key_configured.set(true);
        // indicate that the encryption driver not busy
        self.busy.set(false);

        self.state.set(CryptoState::IDLE);
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
            1...2 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.data_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            3 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.ctr_buf = Some(slice);
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

    fn command(&self, command_num: usize, len: usize, _: AppId) -> ReturnCode {
        match command_num {
            // set key, it is assumed that it is always 16 bytes
            // can only be performed once at the moment
            0 => {
                if len == 16 && !self.key_configured.get() && !self.busy.get() &&
                   self.state.get() == CryptoState::IDLE {
                    for cntr in self.apps.iter() {

                        // indicate busy state
                        self.busy.set(true);
                        self.state.set(CryptoState::SETKEY);

                        cntr.enter(|app, _| {
                            app.key_buf.as_ref().map(|slice| {
                                self.kernel_key.take().map(|buf| {
                                    for (out, inp) in buf.iter_mut()
                                        .zip(slice.as_ref()[0..16].iter()) {
                                        *out = *inp;
                                    }
                                    self.crypto.set_key(buf);
                                });
                            });
                        });
                    }
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::FAIL
                }
            }
            // encryption/decryption driver
            // FIXME: better error handling
            e @ 1...2 => {
                if self.key_configured.get() && !self.busy.get() &&
                   self.state.get() == CryptoState::IDLE {
                    for cntr in self.apps.iter() {
                        self.busy.set(true);

                        // set state
                        match e {
                            1 => self.state.set(CryptoState::ENCRYPT),
                            2 => self.state.set(CryptoState::DECRYPT),
                            _ => (),
                        }

                        cntr.enter(|app, _| {
                            app.data_buf.as_ref().map(|slice| {
                                self.kernel_data.take().map(|buf| {
                                    for (out, inp) in buf.iter_mut()
                                        .zip(slice.as_ref()[0..len].iter()) {
                                        *out = *inp;
                                    }
                                    app.ctr_buf.as_ref().map(|slice2| {
                                        self.kernel_ctr.take().map(move |ctr| {
                                            for (out, inp) in ctr.iter_mut()
                                                .zip(slice2.as_ref()[0..16].iter()) {
                                                *out = *inp;
                                            }
                                            self.crypto.aes128_crypt_ctr(buf, ctr, len as u8);
                                        });
                                    });
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
