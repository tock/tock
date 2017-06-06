//! BLE Capsule
//!
//! The capsule is implemented on top of a virtual timer
//! in order to send periodic BLE advertisements without blocking
//! the kernel
//!
//! The advertisment interval is configured from the user application. The allowed range
//! is between 20 ms and 10240 ms, lower or higher values will be set to these values.
//! Advertisements are sent on channels 37, 38 and 39 with a very shortly time between each
//! transmission.
//!
//! The radio chip module configures a default name which is replaced
//! if a name is entered in user space.
//!
//! The total size of the combined payload is 30 bytes, the capsule ignores payloads which
//! exceed this limit. To clear the payload, the ble_adv_clear_data can be used. This function
//! clears the payload, including the name.
//!
//! Only start and send are asynchronous and need to use the busy flag.
//! However, the synchronous calls such as set tx power, advertisement interval
//! and set payload can only by performed once the radio is not active.
//! The reason why is that they can be interleaved by an interrupt
//!
//! ---ALLOW SYSTEM CALL ------------------------------------------------------------
//! Each AD TYP corresponds to an allow number from 0 to 0xFF which is matched
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!     * SUCCESS: The buffer has successfully been filled
//!     * ENOSUPPORT: Invalid allow_num
//!     * ENOMEM: No sufficient memory available
//!     * EINVAL: Invalid address of the buffer or other error
//!     * EBUSY: The driver is currently busy with other tasks
//!     * ENOSUPPORT: The operation is not supported
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
//!     * 4: clear the advertisement payload
//!
//! The possible return codes from the 'command' system call indicate the following:
//!     * SUCCESS:      The command was successful
//!     * EBUSY:        The driver is currently busy with other tasks
//!     * ENOSUPPORT:   The operation is not supported
//! -----------------------------------------------------------------------------------
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: June 2, 2017


use core::cell::Cell;
use kernel::{AppId, Driver, AppSlice, Shared, Container};
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::time::Frequency;
use kernel::process::Error;
use kernel::returncode::ReturnCode;
use radio;
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


#[derive(Default)]
pub struct App {
    app_write: Option<AppSlice<Shared, u8>>,
}

pub struct BLE<'a, A: hil::time::Alarm + 'a> {
    radio: &'a radio::Radio,
    busy: Cell<bool>,
    app: Container<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    alarm: &'a A,
    advertisement_interval: Cell<u32>,
    is_advertising: Cell<bool>,
    offset: Cell<usize>,
}

impl<'a, A: hil::time::Alarm + 'a> BLE<'a, A> {
    pub fn new(radio: &'a radio::Radio,
               container: Container<App>,
               buf: &'static mut [u8],
               alarm: &'a A)
               -> BLE<'a, A> {
        BLE {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: TakeCell::new(buf),
            alarm: alarm,
            advertisement_interval: Cell::new(150 * <A::Frequency>::frequency() / 1000),
            is_advertising: Cell::new(false),
            // This keeps track of the position in the payload to enable multiple AD TYPES
            offset: Cell::new(0),
        }
    }

    // This function constructs an AD TYPE with type, data, length and offset.
    // It uses the offset to keep track of where to place the next AD TYPE in the buffer in
    // case multiple AD TYPES are provided.
    // The chip module then sets the actual payload.
    fn set_adv_data(&self, ad_type: usize) -> ReturnCode {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write
                    .as_ref()
                    .map(|slice| {
                        let len = slice.len();
                        // Each AD TYP consists of TYPE (1 byte), LENGTH (1 byte) and
                        // PAYLOAD (0 - 30 bytes)
                        // This is why we add 2 to start the payload at the correct position.
                        let i = self.offset.get() + len + 2;
                        if i < 31 {
                            self.kernel_tx
                                .take()
                                .map(|data| {
                                    for (out, inp) in data.iter_mut()
                                        .zip(slice.as_ref()[0..len].iter()) {
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

    // FIXME: More verbose error indication
    fn set_adv_addr(&self) -> ReturnCode {
        let mut ret = ReturnCode::FAIL;
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write
                    .as_ref()
                    .map(|slice| if slice.len() == 6 {
                        self.kernel_tx
                            .take()
                            .map(|data| {
                                for (out, inp) in data.iter_mut()
                                    .zip(slice.as_ref()[0..slice.len()].iter()) {
                                    *out = *inp;
                                }
                                let tmp = self.radio.set_advertisement_address(data);
                                self.kernel_tx.replace(tmp);
                                ret = ReturnCode::SUCCESS;
                            });
                    });
            });
        }
        ret
    }

    fn configure_periodic_alarm(&self) {
        let interval_in_tics = self.alarm.now().wrapping_add(self.advertisement_interval.get());
        self.alarm.set_alarm(interval_in_tics);
    }
}

impl<'a, A: hil::time::Alarm + 'a> hil::time::Client for BLE<'a, A> {
    // this method is called once the virtual timer has been expired
    // used to periodically send BLE advertisements without blocking the kernel
    fn fired(&self) {
        self.radio.start_adv();
        self.configure_periodic_alarm();
    }
}

// Implementation of SYSCALL interface
impl<'a, A: hil::time::Alarm + 'a> Driver for BLE<'a, A> {
    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        match (command_num, self.busy.get()) {
            // START BLE
            (0, false) => {
                if self.busy.get() == false {
                    self.busy.set(true);
                    self.is_advertising.set(true);
                    self.configure_periodic_alarm();
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::FAIL
                }
            }
            //Stop ADV_BLE
            (1, _) => {
                self.is_advertising.set(false);
                self.busy.set(false);
                ReturnCode::SUCCESS
            }
            (2, false) => self.radio.set_adv_txpower(data),
            (3, false) => {
                if data < 20 {
                    self.advertisement_interval.set(20 * <A::Frequency>::frequency() / 1000);
                } else if data > 10240 {
                    self.advertisement_interval.set(10240 * <A::Frequency>::frequency() / 1000);
                } else {
                    self.advertisement_interval
                        .set((data as u32) * <A::Frequency>::frequency() / 1000);
                }
                ReturnCode::SUCCESS
            }
            (4, false) => {
                self.offset.set(0);
                self.radio.clear_adv_data();
                ReturnCode::SUCCESS
            }
            (_, true) => ReturnCode::EBUSY,
            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match (allow_num, self.busy.get()) {
            // See this as a giant case switch or if else statements
            (BLE_HS_ADV_TYPE_FLAGS, false) |
            (BLE_HS_ADV_TYPE_INCOMP_UUIDS16, false) |
            (BLE_HS_ADV_TYPE_COMP_UUIDS16, false) |
            (BLE_HS_ADV_TYPE_INCOMP_UUIDS32, false) |
            (BLE_HS_ADV_TYPE_COMP_UUIDS32, false) |
            (BLE_HS_ADV_TYPE_INCOMP_UUIDS128, false) |
            (BLE_HS_ADV_TYPE_COMP_UUIDS128, false) |
            (BLE_HS_ADV_TYPE_INCOMP_NAME, false) |
            (BLE_HS_ADV_TYPE_COMP_NAME, false) |
            (BLE_HS_ADV_TYPE_TX_PWR_LVL, false) |
            (BLE_HS_ADV_TYPE_SLAVE_ITVL_RANGE, false) |
            (BLE_HS_ADV_TYPE_SOL_UUIDS16, false) |
            (BLE_HS_ADV_TYPE_SOL_UUIDS128, false) |
            (BLE_HS_ADV_TYPE_SVC_DATA_UUID16, false) |
            (BLE_HS_ADV_TYPE_PUBLIC_TGT_ADDR, false) |
            (BLE_HS_ADV_TYPE_RANDOM_TGT_ADDR, false) |
            (BLE_HS_ADV_TYPE_APPEARANCE, false) |
            (BLE_HS_ADV_TYPE_ADV_ITVL, false) |
            (BLE_HS_ADV_TYPE_SVC_DATA_UUID32, false) |
            (BLE_HS_ADV_TYPE_SVC_DATA_UUID128, false) |
            (BLE_HS_ADV_TYPE_URI, false) |
            (BLE_HS_ADV_TYPE_MFG_DATA, false) => {
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
                    self.set_adv_data(allow_num)
                } else {
                    ret
                }
            }
            (0x30, false) => {
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
                    self.set_adv_addr()
                } else {
                    ret
                }
            }
            (_, true) => ReturnCode::EBUSY,

            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }
}
