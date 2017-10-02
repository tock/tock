//! Provides userspace applications with the ability to encrypt and decrypt
//! messages.
//!
//! Userspace Interface
//! -------------------
//!
//! The system calls allow, subscribe and command are used to initiate the
//! driver. The methods `set_key_done()` and `crypt_done()` are invoked by chip
//! to send back the result of the operation and then passed the user
//! application via the callback from the subscribe call.
//!
//! ### `allow` System Call
//!
//! The `allow` system call is used to provide three different buffers and the
//! following allow_num's are supported:
//!
//! * 0: A buffer with the key to be used for encryption and decryption.
//!   Currently it can only configured once.
//! * 1: A buffer with data that will be encrypted and/or decrypted
//! * 4: A buffer to configure to initial counter when counter mode of block
//!   cipher is used.
//!
//! The possible return codes from the 'allow' system call indicate the
//! following:
//!
//! * `SUCCESS`: The buffer has successfully been filled.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `ENOMEM`: No sufficient memory available.
//! * `EINVAL`: Invalid address of the buffer or other error.
//!
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will receive the result of
//! configuring the key, encryption or decryption. The possible return from the
//! `subscribe` system call indicates the following:
//!
//! * `SUCCESS`: the callback been successfully been configured.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `ENOMEM`: No sufficient memory available.
//! * `EINVAL`: Invalid address of the buffer or other error.
//!
//!
//! ### `command` System Call
//!
//! The `command` system call supports two arguments `cmd` and `sub_cmd`. `cmd`
//! is used to specify the specific operation, currently the following cmd's are
//! supported:
//!
//! * `0`: configure the key
//! * `2`: encryption
//! * `3`: decryption
//!
//! `sub_cmd` is used to specify the specific algorithm to be used and currently
//!  the following sub_cmd's are supported:
//!
//! * `0`: aes128 counter-mode
//!
//! The possible return from the 'command' system call indicates the following:
//!
//! * `SUCCESS`:    The operation has been successful.
//! * `EBUSY`:      The driver is busy.
//! * `ESIZE`:      Invalid key size currently is must be 16, 24 or 32 bytes.
//! * `ENOSUPPORT`: Invalid `cmd` or `sub_cmd`.
//! * `EFAIL`:      The key is configured or other error.

// Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
// Author: Fredrik Nilsson <frednils@student.chalmers.se>
// Date: March 31, 2017

use core::cell::Cell;
use kernel::{AppId, AppSlice, Grant, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{SymmetricEncryption, Client};

/// Syscall number
pub const DRIVER_NUM: usize = 0x40000;

pub static mut BUF: [u8; 128] = [0; 128];
pub static mut KEY: [u8; 16] = [0; 16];
pub static mut IV: [u8; 16] = [0; 16];


/// This enum shall keep track of the state of the AESDriver
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

pub struct Crypto<'a, E: SymmetricEncryption + 'a> {
    crypto: &'a E,
    apps: Grant<App>,
    kernel_key: TakeCell<'static, [u8]>,
    kernel_data: TakeCell<'static, [u8]>,
    kernel_ctr: TakeCell<'static, [u8]>,
    key_configured: Cell<bool>,
    busy: Cell<bool>,
    state: Cell<CryptoState>,
}

impl<'a, E: SymmetricEncryption + 'a> Crypto<'a, E> {
    pub fn new(crypto: &'a E,
               grant: Grant<App>,
               key: &'static mut [u8],
               data: &'static mut [u8],
               ctr: &'static mut [u8])
               -> Crypto<'a, E> {
        Crypto {
            crypto: crypto,
            apps: grant,
            kernel_key: TakeCell::new(key),
            kernel_data: TakeCell::new(data),
            kernel_ctr: TakeCell::new(ctr),
            key_configured: Cell::new(false),
            busy: Cell::new(false),
            state: Cell::new(CryptoState::IDLE),
        }
    }

    fn aes128_crypt_ctr(&self, state: CryptoState) -> ReturnCode {
        if self.key_configured.get() && !self.busy.get() && self.state.get() == CryptoState::IDLE {
            for cntr in self.apps.iter() {
                cntr.enter(|app, _| {
                    self.busy.set(true);
                    self.state.set(state);
                    app.data_buf.as_ref().map(|slice| {
                        let len1 = slice.len();
                        // Copy data from app slice to kernel buffer.
                        self.kernel_data.take().map(|buf| {
                            for (out, inp) in buf.iter_mut().zip(slice.as_ref()[0..len1].iter()) {
                                *out = *inp;
                            }
                            app.ctr_buf.as_ref().map(|slice2| {
                                let len2 = slice2.len();
                                // Copy counter value to kernel buffer.
                                self.kernel_ctr.take().map(move |ctr| {
                                    for (out, inp) in ctr.iter_mut()
                                        .zip(slice2.as_ref()[0..len2].iter()) {
                                        *out = *inp;
                                    }
                                    self.crypto.aes128_crypt_ctr(buf, ctr, len1);
                                });
                            });
                        });
                    });
                });
            }
            ReturnCode::SUCCESS
        } else if self.busy.get() == true {
            ReturnCode::EBUSY
        } else {
            ReturnCode::FAIL
        }
    }
}

impl<'a, E: SymmetricEncryption + 'a> Client for Crypto<'a, E> {
    fn crypt_done(&self,
                  data: &'static mut [u8],
                  dmy: &'static mut [u8],
                  len: usize)
                  -> ReturnCode {
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
                app.callback
                    .map(|mut cb| { cb.schedule(self.state.get() as usize, 0, 0); });
            });
        }
        self.busy.set(false);
        self.state.set(CryptoState::IDLE);
        self.kernel_data.replace(data);
        self.kernel_ctr.replace(dmy);
        ReturnCode::SUCCESS
    }
}

impl<'a, E: SymmetricEncryption> Driver for Crypto<'a, E> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.key_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.data_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            4 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.ctr_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
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
                    .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, cmd: usize, sub_cmd: usize, _: usize, _: AppId) -> ReturnCode {
        match cmd {
            // set key, it is assumed to 16, 24 or 32 bytes
            // e.g. aes-128, aes-128 and aes-256
            0 => {
                if !self.key_configured.get() && !self.busy.get() &&
                   self.state.get() == CryptoState::IDLE {
                    let mut ret = ReturnCode::SUCCESS;
                    for cntr in self.apps.iter() {
                        // indicate busy state
                        self.busy.set(true);
                        self.state.set(CryptoState::SETKEY);

                        cntr.enter(|app, _| {
                            app.key_buf
                                .as_ref()
                                .map(|slice| {
                                    let len = slice.len();
                                    if len == 16 || len == 24 || len == 32 {
                                        self.kernel_key
                                            .take()
                                            .map(|buf| {
                                                for (out, inp) in buf.iter_mut()
                                                    .zip(slice.as_ref()[0..len].iter()) {
                                                    *out = *inp;
                                                }
                                                let tmp = self.crypto.set_key(buf, len);
                                                self.kernel_key.replace(tmp);
                                                // indicate that the key is configured
                                                self.key_configured.set(true);
                                                // indicate that the encryption driver not busy
                                                self.busy.set(false);

                                                self.state.set(CryptoState::IDLE);
                                            });
                                    } else {
                                        self.busy.set(false);
                                        self.state.set(CryptoState::IDLE);
                                        ret = ReturnCode::ESIZE;
                                    }
                                });
                        });
                    }
                    ret
                } else if self.busy.get() {
                    ReturnCode::EBUSY
                } else {
                    ReturnCode::FAIL
                }
            }
            // encryption driver
            // the sub-command is supposed to be used for selection
            // encryption algorithm and block cipher mode
            2 => {
                match sub_cmd {
                    // aes-ctr-128
                    0 => self.aes128_crypt_ctr(CryptoState::ENCRYPT),
                    _ => ReturnCode::ENOSUPPORT,
                }
            }
            // decryption driver
            // command sets decryption mode
            // sub_command sets algorithm currently only aes-ctr
            3 => {
                match sub_cmd {
                    // aes-128-ctr
                    0 => self.aes128_crypt_ctr(CryptoState::DECRYPT),
                    _ => ReturnCode::ENOSUPPORT,
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
