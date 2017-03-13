//! Radio/BLE Capsule
//!
//! The capsule is implemented on top of a virtual timer
//! in order to send periodic BLE advertisements without blocking
//! the kernel
//!
//! Currently advertisements with name and data configured in
//! userland are supported.
//! The name and data are only configured once i.e., by invoking
//! start_ble_advertisement() from userland and then periodic advertisement
//! are handled by the capsule on-top of virtual timers.
//!
//! The advertisment intervall is configured to every 150ms by sending on the
//! channels 37, 38 and 39 very shortly after each other.
//! The intervall is mainly picked to compare with other OS's.
//!
//! The radio chip module configures a default name which overwritten
//! if a name is entered in user space.
//!
//! Suggested improvements:
//! TODO: re-name the capsule to BLE and remove basic radio send and remove?!
//! TODO: Fix system call to set advertisement intervall
//! TODO: Fix system call to set advertisement type
//!
//!
//! ---ALLOW SYSTEM CALL ------------------------------------------------------------
//! The 'allow' system call is used to provide two different buffers and
//! the following allow_num's are supported:
//!
//!     * 5: A buffer with data to configure local name (0x09)
//!     * 6: A buffer to configure arbitary data (manufactor data 0xff)
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!     * SUCCESS: The buffer has successfully been filled
//!     * ENOSUPPORT: Invalid allow_num
//!     * ENOMEM: No sufficient memory available
//!     * EINVAL => Invalid address of the buffer or other error
//! ----------------------------------------------------------------------------------
//!
//! ---SUBSCRIBE SYSTEM CALL----------------------------------------------------------
//!  NOT NEEDED NOT FOR PURE ADVERTISEMENTS
//!
//! ------------------------------------------------------------------------------
//!
//! ---COMMAND SYSTEM CALL------------------------------------------------------------
//! The `command` system call supports two arguments `cmd` and 'sub_cmd'.
//! 'cmd' is used to specify the specific operation, currently
//! the following cmd's are supported:
//!     * 3: start advertisment
//!     * 4: stop advertisment
//! -----------------------------------------------------------------------------------
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: May 26, 2017


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
}
// 'a = lifetime
// R - type Radio
impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm +'a > Radio<'a, R, A> {
    pub fn new(radio: &'a R, container: Container<App>, buf: &'static mut [u8], alarm: &'a A ) -> Radio<'a, R, A> {
        Radio {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: TakeCell::new(buf),
            kernel_tx_data: TakeCell::new(buf1),
            alarm: alarm,
            // frequency: Cell::new(37),
            advertise: Cell::new(false),
            // 6 bytes for 'TockOS'
            // third byte is zero
            remaining: Cell::new(30 - 6),
        }
    }

    pub fn set_adv_name(&self) -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write
                    .as_ref()
                    .map(|slice| {
                        // advertisement name
                        let len = slice.len();
                        if len <= 28 {
                            self.remaining.set(30 - (len + 2));
                            self.kernel_tx
                                .take()
                                .map(|name| {
                                         for (out, inp) in name.iter_mut()
                                                 .zip(slice.as_ref()[0..len].iter()) {
                                             *out = *inp;
                                         }
                                         self.radio.set_adv_name(name, len);
                                     });
                        }
                    });
            });
        }
        ReturnCode::SUCCESS
    }


    pub fn set_adv_data(&self) -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write_data
                    .as_ref()
                    .map(|slice| {
                        let len = slice.len();
                        let i = (self.remaining.get() - (len + 2)) as isize;
                        if i >= 0 {
                            self.remaining.set(i as usize);
                            self.kernel_tx_data
                                .take()
                                .map(|data| {
                                    for (out, inp) in
                                        data.iter_mut().zip(slice.as_ref()[0..len].iter()) {
                                        *out = *inp;
                                    }
                                    if i >= 0 {
                                        self.radio.set_adv_data(data, len);
                                    }
                                });
                        }
                    });
            });
        }
        ReturnCode::SUCCESS
    }



    pub fn configure_periodic_alarm(&self) {
        self.radio.set_channel(37);
        let tics = self.alarm.now().wrapping_add(5017 as u32);
        self.alarm.set_alarm(tics);
    }

}

impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> hil::time::Client for Radio<'a, R, A> {
    // this method is called once the virtual timer has been expired
    // used to periodically send BLE advertisements without blocking the kernel
    fn fired(&self) {
        if self.advertise.get() == true {
            self.radio.start_adv();
        } else {
            self.radio.continue_adv();
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

    fn continue_adv(&self) {
        self.advertise.set(false);
        let tics = self.alarm.now().wrapping_add(2 as u32);
        self.alarm.set_alarm(tics);
    }

    fn done_adv(&self) -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| { app.tx_callback.map(|mut cb| { cb.schedule(13, 0, 0); }); });
        }
        self.advertise.set(true);
        self.configure_periodic_alarm();
        ReturnCode::SUCCESS
    }
}

// Implementation of the Driver Trait/Interface
impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Driver for Radio<'a, R, A> {
    //  0 -  rx, must be called each time to get a an rx interrupt, TODO nicer approach
    //  1 -  tx, call for each message
    //  3 -  send BLE advertisements periodically
    //  4 -  disable periodc BLE advertisementes
    fn command(&self, command_num: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => {
                self.radio.receive();
                ReturnCode::SUCCESS
            }
            1 => ReturnCode::SUCCESS,
            //Start ADV_BLE
            3 => {
                if self.busy.get() == false {
                    self.busy.set(true);
                    self.advertise.set(true);
                    self.configure_periodic_alarm();
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


    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match (allow_num, self.busy.get()) {
            (5, false) => {
                let ret = self.app
                    .enter(appid, |app, _| {
                        app.app_write = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                                        Error::OutOfMemory => ReturnCode::ENOMEM,
                                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                                        Error::NoSuchApp => ReturnCode::EINVAL,
                                    });
                if ret == ReturnCode::SUCCESS {
                    self.set_adv_name()
                } else {
                    ret
                }
            }

            // used for data buf for advertisement
            (6, false) => {
                let ret = self.app
                    .enter(appid, |app, _| {
                        app.app_write_data = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                                        Error::OutOfMemory => ReturnCode::ENOMEM,
                                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                                        Error::NoSuchApp => ReturnCode::EINVAL,
                                    });
                if ret == ReturnCode::SUCCESS {
                    self.set_adv_data()
                } else {
                    ret
                }
            }
            (_, true) => ReturnCode::EBUSY,

            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }
}
