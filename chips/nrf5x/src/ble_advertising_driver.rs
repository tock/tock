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

pub static mut BUF: [u8; PACKET_LENGTH] = [0; PACKET_LENGTH];


#[allow(unused)]
struct BLEGap(BLEGapType);

enum AllowType {
    BLEGap(BLEGapType),
    PassiveScanning,
    InitAdvertisementBuffer,
}

// Gap Types only the ones that are defined in libtock are defined here
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(usize)]
enum BLEGapType {
    Flags = 0x01,
    IncompleteList16BitServiceIDs = 0x02,
    CompleteList16BitServiceIDs = 0x03,
    IncompleteList32BitServiceIDs = 0x04,
    CompleteList32BitServiceIDs = 0x05,
    IncompleteList128BitServiceIDs = 0x06,
    CompleteList128BitServiceIDs = 0x07,
    ShortedLocalName = 0x08,
    CompleteLocalName = 0x09,
    TxPowerLevel = 0x0A,
    DeviceId = 0x10,
    SlaveConnectionIntervalRange = 0x12,
    List16BitSolicitationIDs = 0x14,
    List128BitSolicitationIDs = 0x15,
    ServiceData = 0x16,
    Appearance = 0x19,
    AdvertisingInterval = 0x1A,
    ManufacturerSpecificData = 0xFF,
}

// dummy thing to convert usize to enum, FromPrimitive trait don't work
// because they have dependices to std
// if this is good idea, better to create a generic trait for this
fn from_usize(n: usize) -> Option<AllowType> {
    match n {
        0x01 => Some(AllowType::BLEGap(BLEGapType::Flags)),
        0x02 => Some(AllowType::BLEGap(BLEGapType::IncompleteList16BitServiceIDs)),
        0x03 => Some(AllowType::BLEGap(BLEGapType::CompleteList16BitServiceIDs)),
        0x04 => Some(AllowType::BLEGap(BLEGapType::IncompleteList32BitServiceIDs)),
        0x05 => Some(AllowType::BLEGap(BLEGapType::CompleteList32BitServiceIDs)),
        0x06 => Some(AllowType::BLEGap(
            BLEGapType::IncompleteList128BitServiceIDs,
        )),
        0x07 => Some(AllowType::BLEGap(BLEGapType::CompleteList128BitServiceIDs)),
        0x08 => Some(AllowType::BLEGap(BLEGapType::ShortedLocalName)),
        0x09 => Some(AllowType::BLEGap(BLEGapType::CompleteLocalName)),
        0x0A => Some(AllowType::BLEGap(BLEGapType::TxPowerLevel)),
        0x10 => Some(AllowType::BLEGap(BLEGapType::DeviceId)),
        0x12 => Some(AllowType::BLEGap(BLEGapType::SlaveConnectionIntervalRange)),
        0x14 => Some(AllowType::BLEGap(BLEGapType::List16BitSolicitationIDs)),
        0x15 => Some(AllowType::BLEGap(BLEGapType::List128BitSolicitationIDs)),
        0x16 => Some(AllowType::BLEGap(BLEGapType::ServiceData)),
        0x19 => Some(AllowType::BLEGap(BLEGapType::Appearance)),
        0x1A => Some(AllowType::BLEGap(BLEGapType::AdvertisingInterval)),
        0x31 => Some(AllowType::PassiveScanning),
        0x32 => Some(AllowType::InitAdvertisementBuffer),
        0xFF => Some(AllowType::BLEGap(BLEGapType::ManufacturerSpecificData)),
        _ => None,
    }
}

#[allow(unused)]
#[repr(u8)]
enum BLEAdvertisementType {
    ConnectUndirected = 0x00,
    ConnectDirected = 0x01,
    NonConnectUndirected = 0x02,
    ScanRequest = 0x03,
    ScanResponse = 0x04,
    ConnectRequest = 0x05,
    ScanUndirected = 0x06,
}

const PACKET_START: usize = 0;
const PACKET_HDR_PDU: usize = 0;
const PACKET_HDR_LEN: usize = 1;
const PACKET_ADDR_START: usize = 2;
const PACKET_ADDR_END: usize = 7;
const PACKET_PAYLOAD_START: usize = 8;
const PACKET_LENGTH: usize = 39;

#[derive(PartialEq, Debug)]
enum BLEState {
    NotInitialized,
    Initialized,
    Scanning,
    Advertising,
}


pub struct App {
    advertisement_buf: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_write: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_read: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
    offset: Cell<usize>,
    process_status: Option<BLEState>,
    advertisement_interval: Cell<u32>,
    // not used yet....
    tx_power: Cell<u8>,
}


impl Default for App {
    fn default() -> App {
        App {
            advertisement_buf: None,
            app_write: None,
            app_read: None,
            scan_callback: None,
            offset: Cell::new(PACKET_PAYLOAD_START),
            process_status: Some(BLEState::NotInitialized),
            tx_power: Cell::new(0),
            // FIXME: figure out to use associated type in kernel::hil::time:Time
            // Frequence::frequency();
            advertisement_interval: Cell::new(5000),
        }
    }
}


pub struct BLE<'a, B, A>
where
    B: ble_advertising_hil::BleAdvertisementDriver + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    radio: &'a B,
    busy: Cell<bool>,
    app: kernel::Grant<App>,
    kernel_tx: kernel::common::take_cell::TakeCell<'static, [u8]>,
    alarm: &'a A,
}

impl<'a, B, A> BLE<'a, B, A>
where
    B: ble_advertising_hil::BleAdvertisementDriver + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    pub fn new(
        radio: &'a B,
        container: kernel::Grant<App>,
        tx_buf: &'static mut [u8],
        alarm: &'a A,
    ) -> BLE<'a, B, A> {
        BLE {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: kernel::common::take_cell::TakeCell::new(tx_buf),
            alarm: alarm,
        }
    }

    fn initialize_advertisement_buffer(&self) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|buf| {
                        for i in buf.as_mut()[PACKET_START..PACKET_LENGTH].iter_mut() {
                            *i = 0x00;
                        }
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|| ReturnCode::EINVAL)
            });
        }
    }


    // TODO: Add "real" RNG
    fn generate_random_address(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|slice| {
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

    // Hard-coded to NonConnectUndirected
    fn configure_advertisement_pdu(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|slice| {
                        slice.as_mut()[PACKET_HDR_PDU] =
                            BLEAdvertisementType::NonConnectUndirected as u8;
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
    fn set_advertisement_data(&self, gap_type: BLEGapType, appid: kernel::AppId) -> ReturnCode {
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
                                    data.as_mut()[1] = gap_type as u8;
                                    debug!("gap_type {:?}   buf_size {}\r\n", gap_type, buf_len);
                                    for (out, inp) in
                                        data.as_mut()[2..2 + slice.len()].iter_mut().zip(
                                            slice.as_ref()
                                                [0..slice.len()]
                                                .iter(),
                                        )
                                    {
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
                                    for (out, inp) in data.as_mut()[index..end].iter_mut().zip(
                                        slice.as_ref()[0..buf_len].iter(),
                                    )
                                    {
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
                        for (out, inp) in
                            data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                                .iter_mut()
                                .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
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
                let interval_in_tics = self.alarm.now().wrapping_add(
                    app.advertisement_interval.get(),
                );
                self.alarm.set_alarm(interval_in_tics);
            });
        }
    }
}
impl<'a, B, A> kernel::hil::time::Client for BLE<'a, B, A>
    where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
          A: kernel::hil::time::Alarm + 'a
{
// This method is invoked once a virtual timer has expired
// And because we can have several processes running at the concurrently
// with overlapping intervals we use the busy flag to ensure mutual exclusion
// this may not be fair if the processes have similar interval one process
// may be starved.......
    fn fired(&self) {
        for cntr in self.app.iter() {
            cntr.enter(|app, _| {
                match app.process_status {
                    Some(BLEState::Advertising) if !self.busy.get() => {
                        self.busy.set(true);
                        self.replace_advertisement_buffer();
                        self.radio.start_advertisement_tx(37);
                        self.busy.set(false);
                    }
                    Some(BLEState::Scanning) if !self.busy.get() => {
                        self.busy.set(true);
// we want a empty buffer to read data into
                        self.initialize_advertisement_buffer();
                        self.replace_advertisement_buffer();
                        self.radio.start_advertisement_rx(37);
                        self.busy.set(false);
                    }
                    _ => (),
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
                    .map(|mut cb| { cb.schedule(usize::from(result), len as usize, 0); });
            });
        }
    }
}

// Implementation of SYSCALL interface
impl<'a, B, A> kernel::Driver for BLE<'a, B, A>
where
    B: ble_advertising_hil::BleAdvertisementDriver + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    #[inline(never)]
    #[no_mangle]
    fn command(
        &self,
        command_num: usize,
        data: usize,
        _: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {

            // Start periodic advertisments
            0 => {
                self.app
                    .enter(appid, |app, _| if app.process_status == Some(BLEState::Initialized) {
                        app.process_status = Some(BLEState::Advertising);
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
                    .enter(appid, |app, _| if app.process_status ==
                        Some(BLEState::Advertising)
                    {
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Configure transmitted power
            // FIXME: add guard that this is only allowed between advertisements for each process
            // however it's safe for different processes to change tx power between advertisements
            // or another process is advertising
            // but to enable this twpower must be moved into the grant (currently in radio)
            //
            // This is not supported by the user-space interface anymore, REMOVE?!
            // Perhaps better to let the chip decide this?!
            2 => {
                self.app
                    .enter(appid, |app, _| {
                        match data as u8 {
                            // this what nRF5X support at moment
                            // two complement
                            // 0x04 = 4 dBm, 0x00 = 0 dBm, 0xFC = -4 dBm, 0xF8 = -8 dBm
                            // 0xF4 = -12 dBm, 0x
                            0x04 | 0x00 | 0xFC | 0xF8 | 0xF4 | 0xF0 | 0xEC | 0xD8 => {
                                app.tx_power.set(data as u8);
                                ReturnCode::SUCCESS
                            }
                            _ => ReturnCode::EINVAL,
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Configure advertisement intervall
            // FIXME: add guard that this is only allowed between advertisements
            // for each process however it's safe for different processes to change advertisement
            // intervall once another process is advertising
            // but to this twpower must be moved into the grant
            3 => {
                self.app
                    .enter(appid, |app, _| {
                        if data < 20 {
                            app.advertisement_interval.set(
                                20 * <A::Frequency>::frequency() /
                                    1000,
                            );
                        } else if data > 10240 {
                            app.advertisement_interval.set(
                                10240 * <A::Frequency>::frequency() /
                                    1000,
                            );
                        } else {
                            app.advertisement_interval.set(
                                (data as u32) * <A::Frequency>::frequency() /
                                    1000,
                            );
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
                self.app
                    .enter(appid, |app, _| if app.process_status ==
                        Some(BLEState::Initialized)
                    {
                        app.process_status = Some(BLEState::Scanning);
                        self.configure_periodic_alarm();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EBUSY
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Initilize BLE Driver
            // Allow call to allocate the advertisement buffer must be
            // invoked before this!!!!!
            // Request advertisement address
            6 => {
                self.app
                    .enter(appid, |app, _| if app.process_status ==
                        Some(BLEState::Initialized)
                    {
                        if self.generate_random_address(appid) != ReturnCode::SUCCESS {
                            return ReturnCode::EINVAL;
                        }
                        self.configure_advertisement_pdu(appid)
                    } else {
                        ReturnCode::EINVAL
                    })
                    .unwrap_or_else(|err| err.into())
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    #[inline(never)]
    #[no_mangle]
    fn allow(
        &self,
        appid: kernel::AppId,
        allow_num: usize,
        slice: kernel::AppSlice<kernel::Shared, u8>,
    ) -> ReturnCode {

        match from_usize(allow_num) {

            Some(AllowType::BLEGap(gap_type)) => {
                self.app
                    .enter(appid, |app, _| {
                        app.app_write = Some(slice);
                        self.set_advertisement_data(gap_type, appid);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }

            Some(AllowType::PassiveScanning) => {
                debug!("allow passive_scanning\r\n");
                self.app
                    .enter(appid, |app, _| if app.process_status ==
                        Some(BLEState::Initialized)
                    {
                        app.app_read = Some(slice);
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    })
                    .unwrap_or_else(|err| err.into())
            }

            Some(AllowType::InitAdvertisementBuffer) => {
                self.app
                    .enter(appid, |app, _| {
                        app.advertisement_buf = Some(slice);
                        // assume for now this can't fail!!!
                        app.process_status = Some(BLEState::Initialized);
                        self.initialize_advertisement_buffer();
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }


    #[inline(never)]
    #[no_mangle]
    fn subscribe(&self, subscribe_num: usize, callback: kernel::Callback) -> ReturnCode {
        match subscribe_num {
            // Callback for scanning
            0 => {
                debug!("subscribe passive scanning\r\n");
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
