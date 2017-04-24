//! Radio/BLE Capsule
//!
//! The capsule is implemented on top of a virtual timer
//! in order to send periodic BLE advertisements without blocking
//! the entire kernel
//!
//! Currently advertisements with name and configured in
//! userland are supported.
//!
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
pub static mut BUF: [u8; 32] = [0; 32];

pub struct App {
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    // used for adv name
    app_write: Option<AppSlice<Shared, u8>>,
    // specific data in adv
    app_write_data: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            tx_callback: None,
            rx_callback: None,
            app_read: None,
            app_write: None,
            app_write_data: None,
        }
    }
}

pub struct Radio<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> {
    radio: &'a R,
    busy: Cell<bool>,
    app: Container<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    kernel_tx_data: TakeCell<'static, [u8]>,
    alarm: &'a A,
    frequency: Cell<usize>,
    advertise: Cell<bool>,
}
// 'a = lifetime
// R - type Radio
impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Radio<'a, R, A> {
    pub fn new(radio: &'a R,
               container: Container<App>,
               buf: &'static mut [u8],
               buf1: &'static mut [u8],
               alarm: &'a A)
               -> Radio<'a, R, A> {
        Radio {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: TakeCell::new(buf),
            kernel_tx_data: TakeCell::new(buf1),
            alarm: alarm,
            frequency: Cell::new(37),
            advertise: Cell::new(false),
        }
    }

    pub fn capsule_init(&self) {
        self.radio.init()
    }

    // change name of the function? not that clear
    // added a nested closure get other data from userland
    pub fn send_userland_buffer(&self) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write.as_ref().map(|slice| {
                    // advertisement name
                    self.kernel_tx.take().map(|buf| {
                        // suggestion how to loop through the buffer without knowing the length
                        let len = slice.len();
                        // debug!("len: {:?}\r\n", len);
                        for (out, inp) in buf.iter_mut().zip(slice.as_ref()[0..len].iter()) {
                            *out = *inp;
                        }

                        app.app_write_data.as_ref().map(|slice2| {
                            // advertisement data
                            self.kernel_tx_data.take().map(move |buf2| {
                                let len2 = slice2.len();
                                // debug!("len2: {:?}\r\n", len2);
                                for (out, inp) in buf2.iter_mut()
                                    .zip(slice2.as_ref()[0..len2].iter()) {
                                    *out = *inp;
                                }
                                if len + len2 < 17 {
                                    self.radio.transmit(buf, len, buf2, len2);
                                } else {
                                    // TODO: return error
                                }
                                // if len + len2 < 30 then send (or similar)
                                // else return error
                                //debug!("total len {:?}\r\n", len + len2);
                                // unsafe {
                                //     self.kernel_tx_data.replace(&mut BUF);
                                // }
                            });
                        });
                        // kernel_tx_data works only for 1 transmitt then it "consumed"
                        // need to replaced after take()
                        // when it's fixed move transmit into the inner closure
                    });

                });
            });
        }
    }

    pub fn configure_periodic_alarm(&self) {
        let mut interval = 3545 as u32;
        if self.frequency.get() == 39 {
            //interval = 41000 as u32;
            self.frequency.set(37);
        } else {
            self.frequency.set(self.frequency.get() + 1);
        }
        self.radio.set_channel(self.frequency.get());
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}

impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> hil::time::Client for Radio<'a, R, A> {
    // this method is called once the virtual timer has been expired
    // used to periodically send BLE advertisements without blocking the kernel
    fn fired(&self) {
        if self.advertise.get() == true {
            self.configure_periodic_alarm();
            self.send_userland_buffer();
        }
    }
}


impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Client for Radio<'a, R, A> {
    fn receive_done(&self,
                    rx_data: &'static mut [u8],
                    dmy: &'static mut [u8],
                    rx_len: u8)
                    -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                if app.app_read.is_some() {
                    let dest = app.app_read.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in rx_data[0..rx_len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                }
                app.rx_callback.map(|mut cb| { cb.schedule(12, 0, 0); });
            });
        }
        self.kernel_tx.replace(rx_data);
        self.kernel_tx_data.replace(dmy);
        ReturnCode::SUCCESS
    }

    fn transmit_done(&self,
                     tx_data: &'static mut [u8],
                     dmy: &'static mut [u8],
                     len: u8)
                     -> ReturnCode {
        // only notify userland
        for cntr in self.app.iter() {
            cntr.enter(|app, _| { app.tx_callback.map(|mut cb| { cb.schedule(13, 0, 0); }); });
        }
        self.kernel_tx.replace(tx_data);
        self.kernel_tx_data.replace(dmy);
        ReturnCode::SUCCESS
    }
}

// Implementation of the Driver Trait/Interface
impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Driver for Radio<'a, R, A> {
    //  0 -  rx, must be called each time to get a an rx interrupt, TODO nicer approach
    //  1 -  tx, call for each message
    //  3 -  send BLE advertisements periodically
    //  4 -  disable periodc BLE advertisementes
    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => {
                self.radio.receive();
                ReturnCode::SUCCESS
            }
            1 => {
                self.send_userland_buffer();
                ReturnCode::SUCCESS
            }
            //Start ADV_BLE
            3 => {
                if self.busy.get() == false {
                    self.busy.set(true);
                    self.advertise.set(true);
                    self.configure_periodic_alarm();
                    self.send_userland_buffer();
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::FAIL
                }
            }
            //Stop ADV_BLE
            4 => {
                self.advertise.set(false);
                self.busy.set(false);
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
            // used for data buf for advertisement
            2 => {
                self.app
                    .enter(appid, |app, _| {
                        app.app_write_data = Some(slice);
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
