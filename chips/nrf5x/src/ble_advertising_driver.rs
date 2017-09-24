//! System Call implementation for the Bluetooth Low Energy driver
//!
//! The capsule is implemented on top of a virtual timer
//! in order to send periodic BLE advertisements without blocking the kernel.
//!
//! The advertisement interval is configured from the user application.
//! The allowed range is between 20 ms and 10240 ms, lower or higher values will
//! be set to these values. Advertisements are sent on channels 37, 38 and 39
//! which are currently controlled by the chip.
//!
//! The total size of the combined payload is 31 bytes, the capsule ignores payloads
//! which exceed this limit. To clear the payload, the `ble_adv_clear_data`
//! function can be used. This function clears the payload, including the name.
//!
//! Only start and send are asynchronous and need to use the busy flag.
//! However, the synchronous calls such as set tx power, advertisement interval
//! and set payload can only by performed once the radio is not active.
//! The reason why is that they can be interleaved by an interrupt
//!
//! ### Allow system call
//! Each advertisement type corresponds to an allow number from 0 to 0xFF which
//! is handled by a giant pattern matching in this module
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * SUCCESS: The buffer has successfully been filled
//! * ENOSUPPORT: Invalid allow_num
//! * ENOMEM: No sufficient memory available
//! * EINVAL: Invalid address of the buffer or other error
//! * EBUSY: The driver is currently busy with other tasks
//! * ENOSUPPORT: The operation is not supported
//!
//! ### Subscribe system call
//!  The 'subscribe' system call supports two arguments `subscribe_num' and 'callback'.
//! 'subscribe' is used to specify the specific operation, currently:
//!
//! * 0: provides a callback user-space when a device scanning for advertisements
//!          and the callback is used to invoke user-space processes.
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * ENOMEM:    Not sufficient amount memory
//! * EINVAL:    Invalid operation
//!
//! ### Command system call
//! The `command` system call supports two arguments `cmd` and 'sub_cmd'.
//! 'cmd' is used to specify the specific operation, currently
//! the following cmd's are supported:
//!
//! * 0: start advertisement
//! * 1: stop advertisement
//! * 2: configure tx power
//! * 3: configure advertise interval
//! * 4: clear the advertisement payload
//! * 5: start scanning
//!
//! The possible return codes from the 'command' system call indicate the following:
//!
//! * SUCCESS:      The command was successful
//! * EBUSY:        The driver is currently busy with other tasks
//! * ENOSUPPORT:   The operation is not supported
//!
//! ### Authors
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: June 22, 2017


use ble_advertising_hil;
use core::cell::Cell;
use kernel;
use kernel::hil::time::Frequency;
use kernel::returncode::ReturnCode;

/// Syscall Number
pub const DRIVER_NUM: usize = 0x03_00_00;

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

// Advertising Modes
// FIXME: Only BLE_GAP_CONN_MODE_NON supported
pub const BLE_GAP_CONN_MODE_NON: usize = 0x00;
pub const BLE_GAP_CONN_MODE_DIR: usize = 0x01;
pub const BLE_GAP_CONN_MODE_UND: usize = 0x02;
pub const BLE_GAP_SCAN_MODE_NON: usize = 0x03;
pub const BLE_GAP_SCAN_MODE_DIR: usize = 0x04;
pub const BLE_GAP_SCAN_MODE_UND: usize = 0x05;


#[derive(Default)]
pub struct App {
    app_write: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_read: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
}

pub struct BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
    radio: &'a B,
    busy: Cell<bool>,
    app: kernel::Grant<App>,
    kernel_tx: kernel::common::take_cell::TakeCell<'static, [u8]>,
    alarm: &'a A,
    advertisement_interval: Cell<u32>,
    is_advertising: Cell<bool>,
    offset: Cell<usize>,
}

impl<'a, B, A> BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
    pub fn new(radio: &'a B,
               container: kernel::Grant<App>,
               tx_buf: &'static mut [u8],
               alarm: &'a A)
               -> BLE<'a, B, A> {
        BLE {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: kernel::common::take_cell::TakeCell::new(tx_buf),
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
        let mut ret = ReturnCode::ESIZE;
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write.as_ref().map(|slice| {
                    let len = slice.len();
                    // Each AD TYP consists of TYPE (1 byte), LENGTH (1 byte) and
                    // PAYLOAD (0 - 31 bytes)
                    // This is why we add 2 to start the payload at the correct position.
                    let i = self.offset.get() + len + 2;
                    if i <= 31 {
                        self.kernel_tx.take().map(|data| {
                            for (out, inp) in data.iter_mut().zip(slice.as_ref()[0..len].iter()) {
                                *out = *inp;
                            }
                            let tmp = self.radio
                                .set_advertisement_data(ad_type, data, len, self.offset.get() + 8);
                            self.kernel_tx.replace(tmp);
                            self.offset.set(i);
                            ret = ReturnCode::SUCCESS;
                        });
                    }
                });
            });
        }
        ret
    }

    // FIXME: More verbose error indication
    fn set_adv_addr(&self) -> ReturnCode {
        let mut ret = ReturnCode::FAIL;
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.app_write.as_ref().map(|slice| if slice.len() == 6 {
                    self.kernel_tx.take().map(|data| {
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

impl<'a, B, A> kernel::hil::time::Client for BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
    // this method is called once the virtual timer has been expired
    // used to periodically send BLE advertisements without blocking the kernel
    fn fired(&self) {
        if self.busy.get() {
            if self.is_advertising.get() {
                self.radio.start_advertisement_tx(37);
            } else {
                self.radio.start_advertisement_rx(37);
            }
            self.configure_periodic_alarm();
        }
    }
}

impl<'a, B, A> ble_advertising_hil::RxClient for BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                if app.app_read.is_some() {
                    let dest = app.app_read.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in buf[0..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                }
                app.scan_callback
                    .map(|mut cb| { cb.schedule(usize::from(result), 0, 0); });
            });
        }
    }
}

// Implementation of SYSCALL interface
impl<'a, B, A> kernel::Driver for BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
    fn command(&self, command_num: usize, data: usize, _: usize, _: kernel::AppId) -> ReturnCode {
        match (command_num, self.busy.get()) {
            // START BLE
            (0, false) => {
                self.busy.set(true);
                self.is_advertising.set(true);
                self.configure_periodic_alarm();
                ReturnCode::SUCCESS
            }
            //Stop ADV_BLE
            (1, _) => {
                self.is_advertising.set(false);
                self.busy.set(false);
                ReturnCode::SUCCESS
            }
            (2, false) => self.radio.set_advertisement_txpower(data),
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
            // Clear payload
            (4, false) => {
                self.offset.set(0);
                self.radio.clear_adv_data();
                ReturnCode::SUCCESS
            }
            // Passive scanning mode
            (5, false) => {
                self.busy.set(true);
                self.configure_periodic_alarm();
                ReturnCode::SUCCESS
            }

            (_, true) => ReturnCode::EBUSY,
            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(&self,
             appid: kernel::AppId,
             allow_num: usize,
             slice: kernel::AppSlice<kernel::Shared, u8>)
             -> ReturnCode {

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
                    .unwrap_or_else(|err| err.into());
                if ret == ReturnCode::SUCCESS {
                    self.set_adv_data(allow_num)
                } else {
                    ret
                }
            }
            // Set advertisement address
            (0x30, false) => {
                let ret = self.app
                    .enter(appid, |app, _| {
                        app.app_write = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into());
                if ret == ReturnCode::SUCCESS {
                    self.set_adv_addr()
                } else {
                    ret
                }
            }
            // Passive scanning
            (0x31, false) => {
                self.app
                    .enter(appid, |app, _| {
                        app.app_read = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            (_, true) => ReturnCode::EBUSY,

            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }


    fn subscribe(&self, subscribe_num: usize, callback: kernel::Callback) -> ReturnCode {
        match subscribe_num {
            // Callback for scanning
            0 => {
                self.app
                    .enter(callback.app_id(), |app, _| {
                        app.scan_callback = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
