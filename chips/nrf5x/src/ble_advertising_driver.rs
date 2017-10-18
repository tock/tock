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

pub static mut BUF: [u8; 39] = [0; PACKET_LENGTH];


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
pub const BLE_ADV_IND: u8 = 0x00;
pub const BLE_ADV_DIRECT_IND: u8 = 0x01;
pub const BLE_ADV_NONCONNECT_IND: u8 = 0x02;
pub const BLE_SCAN_REQ: u8 = 0x03;
pub const BLE_SCAN_RSP: u8 = 0x04;
pub const BLE_CONNECT_REQ: u8 = 0x05;
pub const BLE_SCAN_IND: u8 = 0x06;


pub const PACKET_HDR_PDU: usize = 0;
pub const PACKET_HDR_LEN: usize = 1;
pub const PACKET_ADDR_START: usize = 2;
pub const PACKET_ADDR_END: usize = 7;
pub const PACKET_PAYLOAD_START: usize = 8;
pub const PACKET_LENGTH: usize = 39;



pub struct App {
    advertisement_buf: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_write: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_read: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
    offset: Cell<usize>,
    is_advertising: Cell<bool>,
    advertisement_interval: Cell<u32>, 
    // FIXME: move alarm also then we are "done"
    // however should the kernel keep track of interval "intersections" also?
}


impl Default for App {
    fn default() -> App {
        App {
            advertisement_buf: None,
            app_write: None,
            app_read: None,
            scan_callback: None,
            offset: Cell::new(PACKET_PAYLOAD_START),
            is_advertising: Cell::new(false),
            // FIXME: figure out to use associated type in kernel::hil::time:Time
            // Frequence::frequency();
            advertisement_interval: Cell::new(5000),
        }
    }
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
        }
    }

    #[inline(never)]
    #[no_mangle]
    fn initialize_advertisement_buffer(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|slice| {
                        slice.as_mut()[PACKET_HDR_PDU] = BLE_ADV_NONCONNECT_IND;
                        // here we should implement functionality to generate 6 random bytes
                        // to be used for advertisement addresson
                        // use address_size as packet size initially
                        slice.as_mut()[PACKET_HDR_LEN] = 6;
                        for i in slice.as_mut()[PACKET_ADDR_START..PACKET_ADDR_END + 1].iter_mut() {
                            *i = 0xff;
                        }
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|| ReturnCode::ESIZE)
            })
            .unwrap_or_else(|err| err.into())
    }

    // This function constructs an AD TYPE with type, data, length and offset.
    // It uses the offset to keep track of where to place the next AD TYPE in the buffer in
    // case multiple AD TYPES are provided.
    // The chip module then sets the actual payload.

    // But because we can't borrow app twice (different mutability)!!!
    // First we copy the data from the allow call to a the TakeCell buffer
    // And then copy that TakeCell buffer to AppSlice advertisement buffer

    #[inline(never)]
    #[no_mangle]
    fn set_advertisement_data(&self, ad_type: usize, appid: kernel::AppId) -> ReturnCode {
        debug!("set_advertisement_data\r\n");

        // these variables are workaround because we can't access other data members in the Grant
        // when we have a mutability borrow!!!
        // this code is messy
        // buf_len - ad_type + len + slice
        // slice - buffer received from allow call
        // in the first closure - append ad_type + len + slice
        // in the second closure - the first 7 bytes are AD_TYPE + address
        // the rest are advertisement data that's why we use index..end dest
        // and 0 .. buf_len in src
        let mut end = 0;
        let mut buf_len = 0;
        let mut index = 0;

        self.app
            .enter(appid, |app, _| {
                let status = app.app_write
                    .as_ref()
                    .map(|slice| {

                        index = app.offset.get();
                        end = app.offset.get() + slice.len() + 2;
                        buf_len = slice.len() + 2;

                        if end <= PACKET_LENGTH {
                            self.kernel_tx
                                .map(|data| {
                                    data.as_mut()[0] = slice.len() as u8 + 1;
                                    data.as_mut()[1] = ad_type as u8;
                                    debug!("ad_type {}   buf_size {}\r\n", ad_type, buf_len);
                                    for (out, inp) in data.as_mut()[2..2 + slice.len()]
                                        .iter_mut()
                                        .zip(slice.as_ref()[0..slice.len()].iter()) {
                                        *out = *inp;
                                    }
                                    ReturnCode::SUCCESS
                                })
                                .unwrap_or_else(|| ReturnCode::EINVAL)
                        } else {
                            ReturnCode::EINVAL
                        }
                    })
                    .unwrap_or_else(|| ReturnCode::EINVAL);

                if status == ReturnCode::SUCCESS {
                    let result = app.advertisement_buf
                        .as_mut()
                        .map(|data| {
                            data.as_mut()[PACKET_HDR_LEN] = (end - 2) as u8;
                            self.kernel_tx
                                .map(|slice| {
                                    for (out, inp) in data.as_mut()[index..end]
                                        .iter_mut()
                                        .zip(slice.as_ref()[0..buf_len].iter()) {
                                        *out = *inp;
                                    }
                                    ReturnCode::SUCCESS
                                })
                                .unwrap_or_else(|| ReturnCode::EINVAL)
                        })
                        .unwrap_or_else(|| ReturnCode::EINVAL);
                    if result == ReturnCode::SUCCESS {
                        app.offset.set(end);
                    }
                    result
                } else {
                    status
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn reset_payload(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|data| {
                        for byte in data.as_mut()[PACKET_PAYLOAD_START..PACKET_LENGTH].iter_mut() {
                            *byte = 0x00;
                        }
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|| ReturnCode::EINVAL)
            })
            .unwrap_or_else(|err| err.into())
    }

    fn replace_advertisement_buffer(&self) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.advertisement_buf.as_ref().map(|slice| {
                    self.kernel_tx.take().map(|data| {
                        for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                            .iter_mut()
                            .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter()) {
                            *out = *inp;
                        }
                        let result = self.radio.set_advertisement_data(data, PACKET_LENGTH);
                        self.kernel_tx.replace(result);
                    });
                });
            });
        }
    }

    // TODO: use AppId as parameter!?
    fn configure_periodic_alarm(&self) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                let interval_in_tics =
                    self.alarm.now().wrapping_add(app.advertisement_interval.get());
                self.alarm.set_alarm(interval_in_tics);
            });
        }
    }
}
impl<'a, B, A> kernel::hil::time::Client for BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
    // this method is called once the virtual timer has been expired
    // used to periodically send BLE advertisements without blocking the kernel
    fn fired(&self) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                if app.is_advertising.get() {
                    self.replace_advertisement_buffer();
                    self.radio.start_advertisement_tx(37);
                } else {
                    self.radio.start_advertisement_rx(37);
                }
                self.configure_periodic_alarm();
            });
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
    #[inline(never)]
    #[no_mangle]
    fn command(&self,
               command_num: usize,
               data: usize,
               _: usize,
               appid: kernel::AppId)
               -> ReturnCode {
        match command_num {

            // Start periodic advertisments
            0 => {
                self.app
                    .enter(appid, |app, _| if !self.busy.get() {
                        self.busy.set(true);
                        app.is_advertising.set(true);
                        self.configure_periodic_alarm();
                        ReturnCode::SUCCESS
                    }
                    else {
                        ReturnCode::EBUSY
                    }
                )
                // not checked to semantics but I assume for if this fails
                // the enter block is not executed!?
                // self.busy.set(false);
                //app.is_advertising.set(false);
                .unwrap_or_else(|err| err.into())
            }

            // Stop perodic advertisements
            1 => {
                self.app
                    .enter(appid, |app, _| {
                        app.is_advertising.set(false);
                        self.busy.set(false);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Configure transmitted power
            // FIXME: add guard that this is only allowed between advertisements for each process
            // however it's safe for different processes to change tx power between advertisements
            // or another process is advertising
            // but to enable this twpower must be moved into the grant (currently in radio)
            2 => self.radio.set_advertisement_txpower(data),

            // Configure advertisement intervall
            // FIXME: add guard that this is only allowed between advertisements
            // for each process however it's safe for different processes to change advertisement
            // intervall once another process is advertising
            // but to this twpower must be moved into the grant
            3 => {
                self.app
                    .enter(appid, |app, _| {
                        if data < 20 {
                            app.advertisement_interval
                                .set(20 * <A::Frequency>::frequency() / 1000);
                        } else if data > 10240 {
                            app.advertisement_interval
                                .set(10240 * <A::Frequency>::frequency() / 1000);
                        } else {
                            app.advertisement_interval
                                .set((data as u32) * <A::Frequency>::frequency() / 1000);
                        }
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Reset advertisement payload (not advertisement type and address)
            // FIXME: add guard that this is only allowed between advertisements
            // for each process however it's safe for different processes reset its payload
            // when another process is advertising
            4 => self.reset_payload(appid),

            // Passive scanning mode
            5 => {
                if !self.busy.get() {
                    self.busy.set(true);
                    self.configure_periodic_alarm();
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EBUSY
                }
            }

            // Initilize BLE Driver
            // Allow call to allocate the advertisement buffer must be
            // invoked before this!!!!!
            // Request advertisement address
            6 => self.initialize_advertisement_buffer(appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    #[inline(never)]
    #[no_mangle]
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
                self.app
                    .enter(appid, |app, _| {
                        app.app_write = Some(slice);
                        self.set_advertisement_data(allow_num, appid);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
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
            // Allocate memory for an advertisement buffer this unique for each
            // user-space process
            (0x32, false) => {
                debug!("allocate advertisement_buf\r\n slice len: {:?}",
                       slice.len());
                self.app
                    .enter(appid, |app, _| {
                        app.advertisement_buf = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            (_, true) => ReturnCode::EBUSY,

            (_, _) => ReturnCode::ENOSUPPORT,
        }
    }


    #[inline(never)]
    #[no_mangle]
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
