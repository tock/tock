//! Radio/BLE driver for nrf51dk
//!
//! So far the capsule is as simple as it can be i.e. it receives
//! which channel from the userapp to advertise on
//!
//! TODO:
//!     - BLE/radio state
//!     - Guard to ensure mutex
//!     - Logic .... separate which layers to be handler where
//!     - more discuss with @alevy
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 09, 2017

use core::cell::Cell;
use kernel::{AppId, Driver, Callback, AppSlice, Shared, Container};
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::radio_nrf51dk::{RadioDriver, Client};
use kernel::process::Error;
use kernel::returncode::ReturnCode;
pub static mut BUF: [u8; 16] = [0; 16];

pub struct App {
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            tx_callback: None,
            rx_callback: None,
            app_read: None,
            app_write: None,
        }
    }
}

pub struct Radio<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> {
    radio: &'a R,
    busy: Cell<bool>,
    app: Container<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    alarm: &'a A,
    frequency: Cell<usize>,
}
// 'a = lifetime
// R - type Radio
impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Radio<'a, R, A> {
    pub fn new(radio: &'a R,
               container: Container<App>,
               buf: &'static mut [u8],
               alarm: &'a A)
               -> Radio<'a, R, A> {
        Radio {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: TakeCell::new(buf),
            alarm: alarm,
            frequency: Cell::new(37),
        }
    }

    pub fn capsule_init(&self) {
        self.radio.init()
    }
    pub fn toggle_led(&self) {

        if self.frequency.get() == 39 {
            self.frequency.set(37);
        }
        else{
            self.frequency.set(self.frequency.get() +1 );
        }
        self.radio.set_channel(self.frequency.get());

        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write.as_mut().map(|slice| {
                    self.kernel_tx.take().map(|buf| {
                        for (i, c) in slice.as_ref()[0..16]
                            .iter()
                            .enumerate() {
                            if buf.len() < i {
                                break;
                            }
                            buf[i] = *c;
                        }
                        self.radio.transmit(0, buf, 16);
                    });

                });
            });
        }


        let interval = (4100 as u32);
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}

impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> hil::time::Client for Radio<'a, R, A> {
    fn fired(&self) {
        self.toggle_led();
    }
}


impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Client for Radio<'a, R, A> {
    #[inline(never)]
    #[no_mangle]
    fn receive_done(&self, rx_data: &'static mut [u8], rx_len: u8) -> ReturnCode {
        // TODO add offset size etc....
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                if app.app_read.is_some() {
                    let dest = app.app_read.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    // 0 .. 16 <-> int i = 0; i < 16; i++
                    for (i, c) in rx_data[0..16].iter().enumerate() {
                        d[i] = *c;
                    }
                }
                app.rx_callback.map(|mut cb| { cb.schedule(12, 0, 0); });
            });
        }
        self.kernel_tx.replace(rx_data);
        ReturnCode::SUCCESS
    }

    fn transmit_done(&self, tx_data: &'static mut [u8], len: u8) -> ReturnCode {
        // only notify userland
        for cntr in self.app.iter() {
            cntr.enter(|app, _| { app.tx_callback.map(|mut cb| { cb.schedule(13, 0, 0); }); });
        }
        self.kernel_tx.replace(tx_data);
        ReturnCode::SUCCESS
    }
}

// Implementation of the Driver Trait/Interface
impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Driver for Radio<'a, R, A> {
    //  0 -  rx, must be called each time to get a an rx interrupt, TODO nicer approach
    //  2 -  tx, call for each message
    //  ...
    //  ...
    //  TODO channel configuration etc for bluetooth compatible packets
    //  TODO add guard for mutex etc
    #[inline(never)]
    #[no_mangle]
    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => {
                self.radio.receive();
                ReturnCode::SUCCESS
            }
            1 => {
                for cntr in self.app.iter() {
                    cntr.enter(|app, _| {
                        app.app_write.as_mut().map(|slice| {

                            self.kernel_tx.take().map(|buf| {
                                for (i, c) in slice.as_ref()[0..16]
                                    .iter()
                                    .enumerate() {
                                    if buf.len() < i {
                                        break;
                                    }
                                    buf[i] = *c;
                                }
                                self.radio.transmit(0, buf, 16);
                            });

                        });
                    });
                }
                ReturnCode::SUCCESS
            }
            // SET CHANNEL
            2 => {
                match data {
                    e @ 37...39 => {
                        self.radio.set_channel(e);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::FAIL,

                }
            }
            //Start ADV_BLE
            3 => {
                let interval = (4100 as u32);
                let tics = self.alarm.now().wrapping_add(interval);
                self.alarm.set_alarm(tics);
                ReturnCode::SUCCESS

            }
            _ => ReturnCode::EALREADY,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {

        match subscribe_num {
            0 => {
                self.app
                    .enter(callback.app_id(), |app_tmp, _| {
                        app_tmp.rx_callback = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        _ => ReturnCode::ENOSUPPORT,
                    })
            }
            // DONT KNOW IF WE NEED THIS REMOVE LATER IF NOT
            1 => {
                self.app
                    .enter(callback.app_id(), |app, _| {
                        app.tx_callback = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        _ => ReturnCode::ENOSUPPORT,
                    })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                self.app
                    .enter(appid, |app, _| {
                        app.app_read = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            1 => {
                self.app
                    .enter(appid, |app, _| {
                        app.app_write = Some(slice);
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
}
