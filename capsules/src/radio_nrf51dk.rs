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
//! Only start and send are asyncronous and need to use the busy flag.
//! However, the syncronous calls such as set tx power, advertisement interval
//! and set payload can only by performed once the radio is not active
//!
//! Suggested improvements:
//! TODO: re-name the capsule to BLE and remove basic radio send and remove?!
//!
//!
//! ---ALLOW SYSTEM CALL ------------------------------------------------------------
//! The 'allow' system call is used to provide two different buffers and
//! the following allow_num's are supported:
//!
//!     * 0: A buffer with data to configure local name (0x09)
//!     * 1: A buffer to configure arbitary data (manufactor data 0xff)
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
//!     * 0: start advertisment
//!     * 1: stop advertisment
//!     * 2: configure tx power
//!     * 3: configure advertise interval
//! -----------------------------------------------------------------------------------
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: May 31, 2017


use core::cell::Cell;
use kernel::{AppId, Driver, Callback, AppSlice, Shared, Container};
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::radio_nrf51dk::{RadioDriver, Client};
use kernel::process::Error;
use kernel::returncode::ReturnCode;
pub static mut BUF: [u8; 32] = [0; 32];


// AD TYPES
pub const BLE_HS_ADV_TYPE_FLAGS: usize = 0x01;
pub const BLE_HS_ADV_TYPE_INCOMP_UUIDS16: usize = 0x02;
pub const BLE_HS_ADV_TYPE_COMP_UUIDS16: usize = 0x03;
pub const BLE_HS_ADV_TYPE_INCOMP_UUIDS32: usize = 0x04;
pub const BLE_HS_ADV_TYPE_COMP_UUIDS32: usize = 0x05;
pub const BLE_HS_ADV_TYPE_INCOMP_UUIDS128: usize = 0x06;
pub const BLE_HS_ADV_TYPE_COMP_UUIDS128: usize = 0x07;
pub const BLE_HS_ADV_TYPE_INCOMP_NAME: usize = 0x08;
pub const BLE_HS_ADV_TYPE_COMP_NAME: usize = 0x09;
pub const BLE_HS_ADV_TYPE_TX_PWR_LVL: usize = 0x0a;
pub const BLE_HS_ADV_TYPE_SLAVE_ITVL_RANGE: usize = 0x12;
pub const BLE_HS_ADV_TYPE_SOL_UUIDS16: usize = 0x14;
pub const BLE_HS_ADV_TYPE_SOL_UUIDS128: usize = 0x15;
pub const BLE_HS_ADV_TYPE_SVC_DATA_UUID16: usize = 0x16;
pub const BLE_HS_ADV_TYPE_PUBLIC_TGT_ADDR: usize = 0x17;
pub const BLE_HS_ADV_TYPE_RANDOM_TGT_ADDR: usize = 0x18;
pub const BLE_HS_ADV_TYPE_APPEARANCE: usize = 0x19;
pub const BLE_HS_ADV_TYPE_ADV_ITVL: usize = 0x1a;
pub const BLE_HS_ADV_TYPE_SVC_DATA_UUID32: usize = 0x20;
pub const BLE_HS_ADV_TYPE_SVC_DATA_UUID128: usize = 0x21;
pub const BLE_HS_ADV_TYPE_URI: usize = 0x24;
pub const BLE_HS_ADV_TYPE_MFG_DATA: usize = 0xff;



pub struct App {
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    // used for adv data
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
    // we should probably add a BLE state-machine here
    interval: Cell<u32>,
    advertise: Cell<bool>,
    offset: Cell<usize>,
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
            // 5017 : every 150 ms!?
            // how do is that comptued?
            // 1 clock cycle, 1/(16*10^6) = 6.25e-8
            // 5007 * 6.25e-8 ~= 0.31ms
            // TODO: check this if other CPUs shall be supported
            interval: Cell::new(5017),
            advertise: Cell::new(false),
            // 6 bytes for 'TockOS'
            // third byte is zero
            offset: Cell::new(0),
        }
    }

    pub fn set_adv_data(&self, ad_type: usize) -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write
                    .as_ref()
                    .map(|slice| {
                        let len = slice.len();
                        let i = self.offset.get() + len + 2;
                        debug!("set_adv_data {:?}\r\n", i);
                        if i < 31 {
                            self.kernel_tx
                                .take()
                                .map(|data| {
                                    for (out, inp) in
                                        data.iter_mut().zip(slice.as_ref()[0..len].iter()) {
                                        *out = *inp;
                                    }
                                    let tmp = self.radio
                                        .set_adv_data(ad_type, data, len, self.offset.get() + 9);
                                    self.kernel_tx.replace(tmp);
                                    self.offset.set(i);
                                });
                        }
                    });
            });
        }
        ReturnCode::SUCCESS
    }

    pub fn configure_periodic_alarm(&self) {
        self.radio.set_channel(37);
        let tics = self.alarm.now().wrapping_add(self.interval.get());
        self.alarm.set_alarm(tics);
    }


    pub fn set_adv_data(&self) -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write_data
                    .as_ref()
                    .map(|slice| {
                        let len = slice.len();
                        self.kernel_tx_data
                            .take()
                            .map(|data| {
                                     for (out, inp) in
                                    data.iter_mut().zip(slice.as_ref()[0..len].iter()) {
                                         *out = *inp;
                                     }
                                     self.radio.set_adv_data(data, len);
                                 });
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
        else {
            self.radio.send();
        }
    }
}

impl<'a, R: RadioDriver + 'a, A: hil::time::Alarm + 'a> Client for Radio<'a, R, A> {
    fn receive_done(&self,
                    rx_data: &'static mut [u8],
                    _: &'static mut [u8],
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
    //  0 -  send BLE advertisements periodically
    //  1 -  disable periodc BLE advertisementes
    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        match (command_num, self.busy.get()) {
            // START BLE
            (0, false) => {
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
            (1, true) => {
                self.advertise.set(false);
                self.busy.set(false);
                ReturnCode::SUCCESS
            }
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
            // See this as a giant case switch or if else statements
            (e @ BLE_HS_ADV_TYPE_FLAGS, false) |
            (e @ BLE_HS_ADV_TYPE_INCOMP_UUIDS16, false) |
            (e @ BLE_HS_ADV_TYPE_COMP_UUIDS16, false) |
            (e @ BLE_HS_ADV_TYPE_INCOMP_UUIDS32, false) |
            (e @ BLE_HS_ADV_TYPE_COMP_UUIDS32, false) |
            (e @ BLE_HS_ADV_TYPE_INCOMP_UUIDS128, false) |
            (e @ BLE_HS_ADV_TYPE_COMP_UUIDS128, false) |
            (e @ BLE_HS_ADV_TYPE_INCOMP_NAME, false) |
            (e @ BLE_HS_ADV_TYPE_COMP_NAME, false) |
            (e @ BLE_HS_ADV_TYPE_TX_PWR_LVL, false) |
            (e @ BLE_HS_ADV_TYPE_SLAVE_ITVL_RANGE, false) |
            (e @ BLE_HS_ADV_TYPE_SOL_UUIDS16, false) |
            (e @ BLE_HS_ADV_TYPE_SOL_UUIDS128, false) |
            (e @ BLE_HS_ADV_TYPE_SVC_DATA_UUID16, false) |
            (e @ BLE_HS_ADV_TYPE_PUBLIC_TGT_ADDR, false) |
            (e @ BLE_HS_ADV_TYPE_RANDOM_TGT_ADDR, false) |
            (e @ BLE_HS_ADV_TYPE_APPEARANCE, false) |
            (e @ BLE_HS_ADV_TYPE_ADV_ITVL, false) |
            (e @ BLE_HS_ADV_TYPE_SVC_DATA_UUID32, false) |
            (e @ BLE_HS_ADV_TYPE_SVC_DATA_UUID128, false) |
            (e @ BLE_HS_ADV_TYPE_URI, false) |
            (e @ BLE_HS_ADV_TYPE_MFG_DATA, false) => {
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
                    self.set_adv_data(e)
                } else {
                    ret
                }
            }
            (_, true) => ReturnCode::EBUSY,

            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }
}
