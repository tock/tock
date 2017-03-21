//! The radio capsule provides userspace applications with the ability
//! to send and receive 802.15.4 packets

// System call interface for sending and receiving 802.15.4 packets.
//
// Author: Philip Levis
// Date: Jan 12 2017
//


use core::cell::Cell;
use kernel::{AppId, Driver, Callback, AppSlice, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil::radio;
use kernel::returncode::ReturnCode;

struct App {
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    cfg_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
}

pub struct RadioDriver<'a, R: radio::Radio + 'a> {
    radio: &'a R,
    busy: Cell<bool>,
    app: MapCell<App>,
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a, R: radio::Radio> RadioDriver<'a, R> {
    pub fn new(radio: &'a R) -> RadioDriver<'a, R> {
        RadioDriver {
            radio: radio,
            busy: Cell::new(false),
            app: MapCell::empty(),
            kernel_tx: TakeCell::empty(),
        }
    }

    pub fn config_buffer(&mut self, tx_buf: &'static mut [u8]) {
        self.kernel_tx.replace(tx_buf);
    }
}

impl<'a, R: radio::Radio> Driver for RadioDriver<'a, R> {
    fn allow(&self, _appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                let appc = match self.app.take() {
                    None => {
                        App {
                            tx_callback: None,
                            rx_callback: None,
                            cfg_callback: None,
                            app_read: Some(slice),
                            app_write: None,
                        }
                    }
                    Some(mut appc) => {
                        appc.app_read = Some(slice);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            }
            1 => {
                let appc = match self.app.take() {
                    None => {
                        App {
                            tx_callback: None,
                            rx_callback: None,
                            cfg_callback: None,
                            app_read: None,
                            app_write: Some(slice),
                        }
                    }
                    Some(mut appc) => {
                        appc.app_write = Some(slice);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 /* transmit done*/  => {
                let appc = match self.app.take() {
                    None => App {
                        tx_callback: Some(callback),
                        rx_callback: None,
                        cfg_callback: None,
                        app_read: None,
                        app_write: None,
                    },
                    Some(mut appc) => {
                        appc.tx_callback = Some(callback);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            },
            1 /* receive */ => {
                let appc = match self.app.take() {
                    None => App {
                        tx_callback: None,
                        rx_callback: Some(callback),
                        cfg_callback: None,
                        app_read: None,
                        app_write: None,
                    },
                    Some(mut appc) => {
                        appc.rx_callback = Some(callback);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            },
            2 /* config */ => {
                let appc = match self.app.take() {
                    None => App {
                        tx_callback: None,
                        rx_callback: None,
                        cfg_callback: Some(callback),
                        app_read: None,
                        app_write: None,
                    },
                    Some(mut appc) => {
                        appc.cfg_callback = Some(callback);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }

    // 0: check if present
    // 1: set 16-bit address
    // 2: set PAN id
    // 3: set channel
    // 4: set tx power
    // 5: transmit packet

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* set 16-bit address */ => {
                self.radio.config_set_address(arg1 as u16);
                ReturnCode::SUCCESS
            },
            2 /* set PAN id */ => {
                self.radio.config_set_pan(arg1 as u16);
                ReturnCode::SUCCESS
            },
            3 /* set channel */ => { // not yet supported
                self.radio.config_set_channel(arg1 as u8)
            },
            4 /* set tx power */ => { // not yet supported
                let mut val = arg1 as i32;
                val = val - 128; // Library adds 128 to make unsigned
                self.radio.config_set_tx_power(val as i8)
            },
            5 /* tx packet */ => {
                // Don't transmit if we're busy, the radio is off, or
                // we don't have a buffer yet.
                if self.busy.get() {
                    return ReturnCode::EBUSY;
                } else if !self.radio.is_on() {
                    return ReturnCode::EOFF;
                } else if self.kernel_tx.is_none() {
                    return ReturnCode::ENOMEM;
                } else if self.app.is_none() {
                    return ReturnCode::ERESERVE;
                }

                // The argument packs the 16-bit destination address
                // and length in the 32-bit argument. Bits 0-15 are
                // the address and bits 16-23 are the length.
                let mut rval = ReturnCode::SUCCESS;
                self.app.map(|app| {
                    let mut blen = 0;
                    // If write buffer too small, return
                    app.app_write.as_mut().map(|w| {
                        blen = w.len();
                    });
                    let len: usize = (arg1 >> 16) & 0xff;
                    let addr: u16 = (arg1 & 0xffff) as u16;
                    if blen < len {
                        rval = ReturnCode::ESIZE;
                        return;
                    }
                    let offset = self.radio.payload_offset(false, false) as usize;
                    // Copy the packet into the kernel buffer
                    self.kernel_tx.map(|kbuf| {
                        app.app_write.as_mut().map(|src| {
                            for (i, c) in src.as_ref()[0..len].iter().enumerate() {
                                kbuf[i + offset] = *c;
                            }
                        });
                    });
                    let transmit_len = len as u8 + self.radio.header_size(false, false);
                    let kbuf = self.kernel_tx.take().unwrap();
                    rval = self.radio.transmit(addr, kbuf, transmit_len, false);
                    if rval == ReturnCode::SUCCESS {
                        self.busy.set(true);
                    }
                });
                rval
            },
            6 /* check if on */ => {
                if self.radio.is_on() {
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EOFF
                }
            }
            7 /* commit config */ => {
                self.radio.config_commit()
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, R: radio::Radio> radio::TxClient for RadioDriver<'a, R> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.app.map(move |app| {
            self.kernel_tx.replace(buf);
            self.busy.set(false);
            app.tx_callback
                .take()
                .map(|mut cb| { cb.schedule(usize::from(result), acked as usize, 0); });
        });
    }
}

impl<'a, R: radio::Radio> radio::RxClient for RadioDriver<'a, R> {
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {
        if self.app.is_some() {
            self.app.map(move |app| {
                if app.app_read.is_some() {
                    let offset = self.radio.payload_offset(false, false) as usize;
                    let dest = app.app_read.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    for (i, c) in buf[offset..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                    app.rx_callback
                        .take()
                        .map(|mut cb| { cb.schedule(usize::from(result), 0, 0); });
                }
                self.radio.set_receive_buffer(buf);
            });
        } else {
            self.radio.set_receive_buffer(buf);
        }
    }
}

impl<'a, R: radio::Radio> radio::ConfigClient for RadioDriver<'a, R> {
    fn config_done(&self, result: ReturnCode) {
        self.app.map(move |app| {
            app.cfg_callback.take().map(|mut cb| { cb.schedule(usize::from(result), 0, 0); });
        });
    }
}
