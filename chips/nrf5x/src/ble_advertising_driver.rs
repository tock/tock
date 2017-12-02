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


#[derive(Debug, Copy, Clone)]
struct Ticks {
    app: kernel::AppId,
    state: TicksState,
}

impl Ticks {
    pub fn new(a: kernel::AppId, s: TicksState) -> Self {
        Ticks { app: a, state: s}
    }
}


#[derive(Debug, Copy, Clone)]
enum TicksState {
    Expired(u32),
    NotExpired(u32),
    Disabled,
}


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


// PDU TYPES
// 0x00 - ADV_IND
// 0x01 - ADV_DIRECT_IND
// 0x02 - ADV_NONCONN_IND
// 0x03 - SCAN_REQ
// 0x04 - SCAN_RSP
// 0x05 - CONNECT_REQ
// 0x06 - ADV_SCAN_IND
//
//  Advertising Type   Connectable  Scannable   Directed    GAP Name
//  ADV_IND            Yes           Yes         No          Connectable Undirected Advertising
//  ADV_DIRECT_IND     Yes           No          Yes         Connectable Directed Advertising
//  ADV_NONCONN_IND    Yes           No          No          Non-connectible Undirected Advertising
//  ADV_SCAN_IND       Yes           Yes         No          Scannable Undirected Advertising
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
    ScanningIdle,
    Scanning,
    AdvertisingIdle,
    Advertising,
}


#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
pub struct AlarmData {
    t0: u32,
    expiration: Expiration,
}

impl AlarmData {
    fn new() -> AlarmData {
        AlarmData {
            t0: 0,
            expiration: Expiration::Disabled,
        }
    }
}


pub struct App {
    advertisement_buf: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_write: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_read: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
    offset: Cell<usize>,
    process_status: Option<BLEState>,
    advertisement_interval_ms: Cell<u32>,
    alarm_data: AlarmData,
    // not used yet....
    tx_power: Cell<u8>,
}


impl Default for App {
    fn default() -> App {
        App {
            advertisement_buf: None,
            alarm_data: AlarmData::new(),
            app_write: None,
            app_read: None,
            scan_callback: None,
            offset: Cell::new(PACKET_PAYLOAD_START),
            process_status: Some(BLEState::NotInitialized),
            tx_power: Cell::new(0),
            advertisement_interval_ms: Cell::new(200),
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
    dummy: Cell<Option<Ticks>>,
    enqueued: Cell<Option<kernel::AppId>>,
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
            dummy: Cell::new(None),
            enqueued: Cell::new(None),
        }
    }

    fn initialize_advertisement_buffer(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|buf| {
                        for i in buf.as_mut()[PACKET_START..PACKET_LENGTH].iter_mut() {
                            *i = 0x00;
                        }
                        ReturnCode::SUCCESS
                    })
                .unwrap_or_else(|| ReturnCode::EINVAL)
            })
        .unwrap_or_else(|err| err.into())
    }

    // Vol 6, Part B 1.3.2.1 Static Device Address
    // A static address is a 48-bit randomly generated address and shall meet the following
    // requirements:
    // • The two most significant bits of the address shall be equal to 1
    // • At least one bit of the random part of the address shall be 0
    // • At least one bit of the random part of the address shall be 1
    //
    // Note that endianness is a potential problem here as this is suppose to be platform
    // independent therefore use 0xf0 as both byte 1 and byte 6 i.e., the two most significant bits
    // are equal to one regardless of endianness
    //
    // Byte 1            0xf0
    // Byte 2-5          random
    // Byte 6            0xf0
    // FIXME: For now use AppId as "randomness"
    fn generate_random_address(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_mut()
                    .map(|data| {
                        data.as_mut()[PACKET_HDR_LEN] = 6;
                        data.as_mut()[PACKET_ADDR_START] = 0xf0;
                        data.as_mut()[PACKET_ADDR_START + 1] = (appid.idx() & 0xff) as u8;
                        data.as_mut()[PACKET_ADDR_START + 2] = ((appid.idx() << 8) & 0xff) as u8;
                        data.as_mut()[PACKET_ADDR_START + 3] = ((appid.idx() << 16) & 0xff) as u8;
                        data.as_mut()[PACKET_ADDR_START + 4] = ((appid.idx() << 24) & 0xff) as u8;
                        data.as_mut()[PACKET_ADDR_END] = 0xf0;
                        ReturnCode::SUCCESS
                    })
                .unwrap_or_else(|| ReturnCode::ESIZE)
            })
        .unwrap_or_else(|err| err.into())
    }

    // Hard-coded
    // Advertising Type  Connectable Scannable  Directed   GAP Name
    // ADV_NONCONN_IND   Yes         No         No         Non-connectible Undirected Advertising
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
    // The hardware module then sets the actual payload.

    // But because we borrow the struct mutabily we can't borrow it immutably at the same time
    // First we copy the data from the allow call to a the TakeCell buffer
    // And then copy that TakeCell buffer to AppSlice advertisement buffer

    fn set_advertisement_data(&self, gap_type: BLEGapType, appid: kernel::AppId) -> ReturnCode {
        // these variables are workaround because we can't access other data members
        // when we have a mutable borrow!!!

        // keep track of the end data -> update after as the new index afterward
        let mut end = 0;
        // index + Buffer length + 2 (1 byte for Length, 1 byte for AD Type)
        let mut buf_len = 0;
        // Current index in the buffer
        let mut index = 0;

        self.app
            .enter(appid, |app, _| {
                let status =
                    app.app_write
                    .as_ref()
                    .map(|slice| {

                        // get current index
                        index = app.offset.get();
                        // get end
                        end = index + slice.len() + 2;
                        buf_len = slice.len() + 2;

                        // Copy data from the "WRITE" AppSlice to the TakeCell if there is
                        // space left
                        if end <= PACKET_LENGTH {
                            self.kernel_tx
                                .map(|data| {
                                    data.as_mut()[0] = slice.len() as u8 + 1;
                                    data.as_mut()[1] = gap_type as u8;
                                    for (out, inp) in
                                        data.as_mut()[2..2 + slice.len()].iter_mut().zip(
                                            slice.as_ref()[0..slice.len()].iter(),
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

                // All data copied the TakeCell then copy it to the Advertisement Buffer AppSlice
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
                        // Update offset according to the addes bytes
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

    fn replace_advertisement_buffer(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| {
                app.advertisement_buf
                    .as_ref()
                    .map(|slice| {
                        self.kernel_tx
                            .take()
                            .map(|data| {
                                for (out, inp) in
                                    data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                                        .iter_mut()
                                        .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                                        {
                                            *out = *inp;
                                        }
                                let result = self.radio.set_advertisement_data(data, PACKET_LENGTH);
                                self.kernel_tx.replace(result);
                                ReturnCode::SUCCESS
                            })
                        .unwrap_or_else(|| ReturnCode::EINVAL)
                    })
                .unwrap_or_else(|| ReturnCode::EINVAL)
            })
        .unwrap_or_else(|err| err.into())
    }

    fn set_single_alarm(&self, appid: kernel::AppId) -> ReturnCode {
        self.app
            .enter(appid, |app, _| if let Expiration::Disabled =
                   app.alarm_data.expiration
                   {
                       // configure alarm perhaps move this to a separate function
                       app.alarm_data.t0 = self.alarm.now();
                       let period_ms = app.advertisement_interval_ms.get() * <A::Frequency>::frequency() / 1000;
                       let alarm_time = app.alarm_data.t0.wrapping_add(period_ms);
                       app.alarm_data.expiration = Expiration::Abs(period_ms);
                       self.alarm.set_alarm(alarm_time);
                       ReturnCode::SUCCESS
                   } else {
                       ReturnCode::EBUSY
                   })
        .unwrap_or_else(|err| err.into())
    }

    #[inline(never)]
    #[no_mangle]
    fn get_current_process(&self) -> Option<kernel::AppId> {
        self.dummy.set(None);
        let now = self.alarm.now();

        self.app.each(|app| if let Expiration::Abs(period) =
                      app.alarm_data.expiration
                      {

                          // as alarm value is 32 bits and it will wrapp after 2^32
                          // if `t0` has a bigger ticks value than `now`
                          // then we assume that the ticks value has wrapped and
                          // now happend before t0

                          let fired_ticks = match now.checked_sub(app.alarm_data.t0) {
                              None => {
                                  let d = now.checked_add(app.alarm_data.t0).unwrap_or(
                                      <u32>::max_value(),
                                      );
                                  <u32>::max_value() - d
                              }
                              Some(v) => v,
                          };


                          let candidate = match period.checked_sub(fired_ticks) {
                              None => {
                                  Ticks {
                                      app: app.appid(),
                                      state: TicksState::Expired(fired_ticks - period)
                                  }
                              }
                              Some(v) => {
                                  Ticks {
                                      app: app.appid(),
                                      state: TicksState::NotExpired(v)
                                  }
                              }
                          };

                          // debug!(
                          //     "app: {:?} \t now: {:?} \t period {:?} \t t0 {:?}  \t candidate {:?} \t fired_ticks {:?}",
                          //     app.appid(),
                          //     now,
                          //     period,
                          //     app.alarm_data.t0,
                          //     candidate,
                          //     fired_ticks
                          //     );

                          // debug!("cand: {:?} \t current: {:?}", candidate, self.dummy.get());


                          if let Some(current) = self.dummy.get() {
                              //compare here
                              let (curr, old) = match (candidate.state, current.state) {
                                  (TicksState::Disabled, _) => (current, candidate),
                                  (TicksState::Expired(cand), TicksState::Expired(curr)) if cand > curr => (candidate, current),
                                  (TicksState::Expired(_), TicksState::Expired(_)) => (current, candidate),
                                  (TicksState::Expired(_), _) => (candidate, current),
                                  (TicksState::NotExpired(_), TicksState::Expired(_)) => (current, candidate),
                                  (TicksState::NotExpired(cand), TicksState::NotExpired(curr)) if cand >= curr => (current, candidate),
                                  (TicksState::NotExpired(_), _) => (candidate, current),
                              };

                              self.dummy.set(Some(curr));

                              if let TicksState::Expired(t) = old.state {
                                  self.enqueued.set(Some(old.app));
                              }

                          }


                          else {
                              self.dummy.set(Some(candidate));
                          }


                      });
        Some(self.dummy.get().unwrap().app)
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
        let now = self.alarm.now();
        let appid = self.get_current_process();
        debug!("app: {:?} fired {:?}", appid, now);

        // assumption AppId: 0xff is not used
        let _ = self.app.enter(appid.unwrap_or(kernel::AppId::new(0xff)),
        |app, _| match app.process_status {
            Some(BLEState::AdvertisingIdle) if !self.busy.get() => {
                self.busy.set(true);
                app.process_status = Some(BLEState::Advertising);
                self.replace_advertisement_buffer(app.appid());
                self.radio.start_advertisement_tx(app.appid());
                self.set_single_alarm(app.appid());
            }
            Some(BLEState::ScanningIdle) if !self.busy.get() => {
                self.busy.set(true);
                app.process_status = Some(BLEState::Scanning);
                self.replace_advertisement_buffer(app.appid());
                self.radio.start_advertisement_rx(app.appid());
                self.set_single_alarm(app.appid());

            }
            Some(BLEState::ScanningIdle) |
                Some(BLEState::AdvertisingIdle) => {
                    debug!("app {:?}\t  alarm {:?}", appid, self.alarm.now());
                    self.set_single_alarm(app.appid());
                }
            _ => (),
        });
    }
}

impl<'a, B, A> ble_advertising_hil::RxClient for BLE<'a, B, A>
where B: ble_advertising_hil::BleAdvertisementDriver + 'a,
      A: kernel::hil::time::Alarm + 'a
{
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode, appid: kernel::AppId) {
        let _ = self.app.enter(appid, |app, _| {
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

    fn advertisement_fired(&self, appid: kernel::AppId) {
        let _ = self.app.enter(appid, |app, _| {
            let _ = match app.process_status {
                Some(BLEState::Advertising) => {
                    app.process_status = Some(BLEState::AdvertisingIdle);
                    app.alarm_data.expiration = Expiration::Disabled;
                    self.busy.set(false)
                }
                Some(BLEState::Scanning) => {
                    app.process_status = Some(BLEState::ScanningIdle);
                    app.alarm_data.expiration = Expiration::Disabled;
                    self.busy.set(false)
                }
                _ => panic!("invalid state {:?}\r\n", app.process_status),
            };

            // enable new alarm for period advertisements
            self.set_single_alarm(appid);

        });


        // // TODO: here we should check for enqueued apps
        if let Some(other_appid) = self.enqueued.get() {
            // self.busy.set(true);
            debug!("enqueued: {:?} but NOT IMPLEMENTED YET!!!! ALSO WE NEED A QUEUE HERE", other_appid);
            // let _ = self.app.enter(other_appid, |app, _| {
            //     app.process_status = Some(BLEState::Advertising);
            //     self.replace_advertisement_buffer(app.appid());
            //     self.radio.start_advertisement_tx(app.appid());
            // });
        }
    }
}

// Implementation of SYSCALL interface
impl<'a, B, A> kernel::Driver for BLE<'a, B, A>
where
B: ble_advertising_hil::BleAdvertisementDriver + 'a,
A: kernel::hil::time::Alarm + 'a,
{
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
                let result = self.app
                    .enter(appid, |app, _| if app.process_status ==
                           Some(BLEState::Initialized)
                           {
                               app.process_status = Some(BLEState::AdvertisingIdle);
                               self.set_single_alarm(appid);
                               ReturnCode::SUCCESS
                           } else {
                               ReturnCode::EBUSY
                           })
                .unwrap_or_else(|err| err.into());

                if result == ReturnCode::SUCCESS {
                    if let None = self.dummy.get() {
                        self.dummy.set(Some(Ticks::new(appid, TicksState::Disabled)));
                    }
                }
                result
            }

            // Stop perodic advertisements or scanning
            1 => {
                self.app
                    .enter(appid, |app, _| match app.process_status {
                        Some(BLEState::AdvertisingIdle) |
                            Some(BLEState::ScanningIdle) => {
                                app.process_status = Some(BLEState::Initialized);
                                ReturnCode::SUCCESS
                            }
                        _ => ReturnCode::EBUSY,
                    })
                .unwrap_or_else(|err| err.into())
            }

            // Configure transmitted power
            //
            // This is not supported by the user-space interface anymore, REMOVE?!
            // Perhaps better to let the chip decide this?!
            2 => {
                self.app
                    .enter(
                        appid,
                        |app, _| if app.process_status != Some(BLEState::ScanningIdle) &&
                        app.process_status != Some(BLEState::AdvertisingIdle)
                        {
                            match data as u8 {
                                // this what nRF5X support at moment
                                // two complement
                                // 0x04 = 4 dBm, 0x00 = 0 dBm, 0xFC = -4 dBm, 0xF8 = -8 dBm
                                // 0xF4 = -12 dBm, 0xF0 = -16 dBm, 0xEC = -20 dBm, 0xD8 = -40 dBm
                                0x04 | 0x00 | 0xFC | 0xF8 | 0xF4 | 0xF0 | 0xEC | 0xD8 => {
                                    app.tx_power.set(data as u8);
                                    ReturnCode::SUCCESS
                                }
                                _ => ReturnCode::EINVAL,
                            }
                        } else {
                            ReturnCode::EBUSY
                        },
                        )
                    .unwrap_or_else(|err| err.into())
            }

            // Configure advertisement interval
            // Vol 6, Part B 4.4.2.2
            // The advertisment interval shall an integer multiple of 0.625ms in the range of
            // 20ms to 10240 ms!
            //
            // data - advertisement interval in ms
            // FIXME: add check that data is a multiple of 0.625
            3 => {
                self.app
                    .enter(appid, |app, _| match app.process_status {
                        Some(BLEState::Scanning) |
                            Some(BLEState::Advertising) => ReturnCode::EBUSY,
                        _ => {
                            if data < 20 {
                                app.advertisement_interval_ms.set(20);
                            } else if data > 10240 {
                                app.advertisement_interval_ms.set(10420);
                            } else {
                                app.advertisement_interval_ms.set(data as u32);
                            }
                            ReturnCode::SUCCESS
                        }
                    })
                .unwrap_or_else(|err| err.into())
            }

            // Reset payload when the kernel is not actively advertising
            // FIXME: Add guard
            4 => {
                let result = self.reset_payload(appid);
                match result {
                    ReturnCode::SUCCESS => {
                        self.app
                            .enter(appid, |app, _| {
                                app.offset.set(PACKET_PAYLOAD_START);
                                ReturnCode::SUCCESS
                            })
                        .unwrap_or_else(|err| err.into())
                    }
                    e @ _ => e,
                }
            }
            // Passive scanning mode
            5 => {
                self.app
                    .enter(appid, |app, _| if app.process_status ==
                           Some(BLEState::Initialized)
                           {
                               app.process_status = Some(BLEState::ScanningIdle);
                               self.set_single_alarm(appid)
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
                    .enter(appid, |app, _| if let Some(BLEState::Initialized) =
                           app.process_status
                           {

                               let status = self.generate_random_address(appid);
                               if status == ReturnCode::SUCCESS {
                                   self.configure_advertisement_pdu(appid)
                               } else {
                                   status
                               }
                           } else {
                               ReturnCode::EINVAL
                           })
                .unwrap_or_else(|err| err.into())
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        appid: kernel::AppId,
        allow_num: usize,
        slice: kernel::AppSlice<kernel::Shared, u8>,
        ) -> ReturnCode {

        match from_usize(allow_num) {

            Some(AllowType::BLEGap(gap_type)) => {
                self.app
                    .enter(appid, |app, _| if app.process_status !=
                           Some(BLEState::NotInitialized)
                           {
                               app.app_write = Some(slice);
                               self.set_advertisement_data(gap_type, appid);
                               ReturnCode::SUCCESS
                           } else {
                               ReturnCode::EINVAL
                           })
                .unwrap_or_else(|err| err.into())
            }

            Some(AllowType::PassiveScanning) => {
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
                    .enter(appid, |app, _| if let Some(BLEState::NotInitialized) =
                           app.process_status
                           {
                               app.advertisement_buf = Some(slice);
                               app.process_status = Some(BLEState::Initialized);
                               self.initialize_advertisement_buffer(appid);
                               ReturnCode::SUCCESS
                           } else {
                               ReturnCode::EINVAL
                           })
                .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
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
