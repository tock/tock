//! Bluetooth Low Energy Advertising Driver
//!
//! A system call driver that exposes the Bluetooth Low Energy advertising
//! channel. The driver generates a unique static address for each process,
//! allowing each process to act as its own device and send or scan for
//! advertisements. Timing of advertising or scanning events is handled by the
//! driver but processes can request an advertising or scanning interval.
//! Processes can also control the TX power used for their advertisements.
//!
//! Data payloads are limited to 31 bytes since the maximum advertising channel
//! protocol data unit (PDU) is 37 bytes and includes a 6-byte header.
//!
//! ### Allow system call
//! The allow systems calls are used for buffers from allocated by userland
//!
//! There are three different buffers:
//!
//! * Bluetooth Low Energy Gap Types
//! * Passive Scanner
//! * Advertisement
//!
//! * 0: GAP Data
//! * 1: Passive Scanning
//! * 2: Advertising
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * SUCCESS: The buffer has successfully been filled
//! * ENOSUPPORT: Invalid allow_num
//! * ENOMEM: No sufficient memory available
//! * EINVAL: Invalid address of the buffer or other error
//! * EBUSY: The driver is currently busy with other tasks
//! * ENOSUPPORT: The operation is not supported
//! * ERROR: Operation `map` on Option failed
//!
//! ### Subscribe system call
//!  The 'subscribe' system call supports two arguments `subscribe_num' and 'callback'.
//! 'subscribe' is used to specify the specific operation, currently:
//!
//! * 0: provides a callback user-space when a device scanning for advertisements
//!      and the callback is used to invoke user-space processes.
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
//! * 3: configure advertisement interval
//! * 4: clear the advertisement payload
//! * 5: start scanning
//! * 6: initialize driver
//!
//! The possible return codes from the 'command' system call indicate the following:
//!
//! * SUCCESS:      The command was successful
//! * EBUSY:        The driver is currently busy with other tasks
//! * ENOSUPPORT:   The operation is not supported
//!
//! Usage
//! -----
//! ```
//! Advertisement:
//!
//!           +-------------------------------+
//!           | Initilize Advertisement Buffer|
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Request BLE Address           |
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Configure  ADV_TYPE           |
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Start Advertising             |
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Configure Alarm               |------------|
//!           +-------------------------------+            |
//!                          |                             |
//!           +-------------------------------+            |
//!           | Send Packet                   |------------|
//!           +-------------------------------+
//!
//! Client
//!           +-------------------------------+
//!           | Packet Sent or Error          |------------|
//!           +-------------------------------+            |
//!                         |                              |
//!           +-------------------------------+            |
//!           | Notify BLE Driver             |------------|
//!           +-------------------------------+
//!
//! ```
//!
//! ```
//! Passive Scanning:
//!
//!           +-----------------------+
//!           | Configure Callback    |
//!           +-----------------------+
//!                      |
//!           +-----------------------+
//!           | Initilize Scan Buffer |
//!           +-----------------------+
//!                      |
//!           +-----------------------+
//!           | Start Passive Scanning|
//!           +-----------------------+
//!                      |
//!           +-----------------------+
//!           | Configure Alarm       |--------------|
//!           +-----------------------+              |
//!                      |                           |
//!           +-----------------------+              |
//!           | Receive Packet        |--------------|
//!           +-----------------------+
//!
//! Client
//!           +-------------------------------+
//!           | Packet Received or Error      |------------|
//!           +-------------------------------+            |
//!                         |                              |
//!           +-------------------------------+            |
//!           | Notify BLE Driver             |------------|
//!           +-------------------------------+
//! ```
//!
//! You need a device that provides the `nrf5x::ble_advertising_hil::BleAdvertisementDriver` trait
//! along with a virtual timer to perform events and not block the entire kernel
//!
//! ```rust
//!     let ble_radio = static_init!(
//!     nrf5x::ble_advertising_driver::BLE
//!     <'static, nrf52::radio::Radio, VirtualMuxAlarm<'static, Rtc>>,
//!     nrf5x::ble_advertising_driver::BLE::new(
//!         &mut nrf52::radio::RADIO,
//!     kernel::Grant::create(),
//!         &mut nrf5x::ble_advertising_driver::BUF,
//!         ble_radio_virtual_alarm));
//!    nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_rx_client(&nrf52::radio::RADIO,
//!                                                                      ble_radio);
//!    nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_tx_client(&nrf52::radio::RADIO,
//!                                                                      ble_radio);
//!    ble_radio_virtual_alarm.set_client(ble_radio);
//! ```
//!
//! ### Authors
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: June 22, 2017

use core::cell::Cell;
use core::cmp;
use kernel;
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;
use kernel::hil::time::Frequency;
use kernel::returncode::ReturnCode;

/// Syscall Number
pub const DRIVER_NUM: usize = 0x03_00_00;

pub static mut BUF: [u8; PACKET_LENGTH] = [0; PACKET_LENGTH];

// ConnectUndirected (ADV_IND): connectable undirected advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.1
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | AdvData      |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
// ConnectDirected (ADV_DIRECT_IND): connectable directed advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.2
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | InitA        |
//           | (6 bytes) |      | (6 bytes)    |
//           +-----------+      +--------------+
//
// NonConnectDirected (ADV_NONCONN_IND): non-connectable undirected advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.3
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | AdvData      |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
//
// ScanUndirected (ADV_SCAN_IND): scannable undirected advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.4
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | AdvData      |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
// ScanRequest (SCAN_REQ)
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.2.1
//
//   PDU     +-----------+      +--------------+
//           | ScanA     |  -   | AdvA        |
//           | (6 bytes) |      | (6 bytes) |
//           +-----------+      +--------------+
//
// ScanResponse (SCAN_RSP)
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.2.2
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | ScanRspData  |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
// ConnectRequest (CON_REQ)
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.3.1
//
//   PDU     +-----------+      +--------------+     +--------------+
//           | InitA     |  -   | AdvA         |  -  | LLData       |
//           | (6 bytes) |      | 6 bytes      |     | 22 bytes     |
//           +-----------+      +--------------+     +--------------+
//
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
const EMPTY_PACKET_LEN: u8 = 6;

#[derive(PartialEq, Debug)]
enum BLEState {
    NotInitialized,
    Initialized,
    ScanningIdle,
    Scanning(RadioChannel),
    AdvertisingIdle,
    Advertising(RadioChannel),
}

#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
struct AlarmData {
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
    process_status: Option<BLEState>,
    advertisement_interval_ms: u32,
    alarm_data: AlarmData,
    tx_power: u8,
    /// The state of an app-specific pseudo random number.
    ///
    /// For example, it can be used for the pseudo-random `advDelay` parameter.
    /// It should be read using the `random_number` method, which updates it as
    /// well.
    random_nonce: u32,
}

impl Default for App {
    fn default() -> App {
        App {
            advertisement_buf: None,
            alarm_data: AlarmData::new(),
            app_write: None,
            app_read: None,
            scan_callback: None,
            process_status: Some(BLEState::NotInitialized),
            tx_power: 0,
            advertisement_interval_ms: 200,
            // Just use any non-zero starting value by default
            random_nonce: 0xdeadbeef,
        }
    }
}

impl App {
    fn initialize_advertisement_buffer(&mut self) -> ReturnCode {
        self.advertisement_buf
            .as_mut()
            .map(|buf| {
                for i in buf.as_mut()[PACKET_START..PACKET_LENGTH].iter_mut() {
                    *i = 0x00;
                }
                ReturnCode::SUCCESS
            })
            .unwrap_or(ReturnCode::FAIL)
    }

    // Bluetooth Core Specification:Vol. 6, Part B, section 1.3.2.1 Static Device Address
    //
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
    fn generate_random_address(&mut self, appid: kernel::AppId) -> ReturnCode {
        self.advertisement_buf
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
            .unwrap_or(ReturnCode::FAIL)
    }

    fn reset_payload(&mut self) -> ReturnCode {
        match self.process_status {
            Some(BLEState::Advertising(_)) | Some(BLEState::Scanning(_)) => ReturnCode::EBUSY,
            _ => {
                self.advertisement_buf
                    .as_mut()
                    .map(|data| {
                        for byte in data.as_mut()[PACKET_PAYLOAD_START..PACKET_LENGTH].iter_mut() {
                            *byte = 0x00;
                        }

                        // Reset header length
                        data.as_mut()[PACKET_HDR_LEN] = EMPTY_PACKET_LEN;

                        ReturnCode::SUCCESS
                    })
                    .unwrap_or(ReturnCode::EINVAL)
            }
        }
    }

    // Hard-coded to ADV_NONCONN_IND
    fn configure_advertisement_pdu(&mut self) -> ReturnCode {
        self.advertisement_buf
            .as_mut()
            .map(|slice| {
                slice.as_mut()[PACKET_HDR_PDU] = BLEAdvertisementType::NonConnectUndirected as u8;
                ReturnCode::SUCCESS
            })
            .unwrap_or(ReturnCode::FAIL)
    }

    fn set_gap_data(&mut self) -> ReturnCode {
        self.app_write
            .take()
            .as_ref()
            .map(|slice| {
                if slice.len() <= PACKET_LENGTH {
                    self.advertisement_buf
                        .as_mut()
                        .map(|data| {
                            for (dst, src) in data.as_mut()[PACKET_PAYLOAD_START..]
                                .iter_mut()
                                .zip(slice.as_ref()[0..slice.len()].iter())
                            {
                                *dst = *src;
                            }

                            // FIXME: move to its own function/method?!
                            // Update header length
                            data.as_mut()[PACKET_HDR_LEN] += slice.len() as u8;

                            ReturnCode::SUCCESS
                        })
                        // FIXME: better returncode to indicate that option is None
                        .unwrap_or(ReturnCode::FAIL)
                } else {
                    ReturnCode::ESIZE
                }
            })
            // FIXME: better returncode to indicate that option is None
            .unwrap_or(ReturnCode::FAIL)
    }

    fn send_advertisement<'a, B, A>(&self, ble: &BLE<'a, B, A>, channel: RadioChannel) -> ReturnCode
    where
        B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
    {
        self.advertisement_buf
            .as_ref()
            .map(|slice| {
                ble.kernel_tx
                    .take()
                    .map(|data| {
                        for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                            .iter_mut()
                            .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                        let result = ble.radio
                            .transmit_advertisement(data, PACKET_LENGTH, channel);
                        ble.kernel_tx.replace(result);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or(ReturnCode::FAIL)
            })
            .unwrap_or(ReturnCode::FAIL)
    }

    // Returns a new pseudo-random number and updates the randomness state.
    //
    // Uses the [Xorshift](https://en.wikipedia.org/wiki/Xorshift) algorithm to
    // produce pseudo-random numbers. Uses the `random_nonce` field to keep
    // state.
    fn random_nonce(&mut self) -> u32 {
        let mut next_nonce = ::core::num::Wrapping(self.random_nonce);
        next_nonce ^= next_nonce << 13;
        next_nonce ^= next_nonce >> 17;
        next_nonce ^= next_nonce << 5;
        self.random_nonce = next_nonce.0;
        self.random_nonce
    }

    // Set the next alarm for this app using the period and provided start time.
    fn set_next_alarm<F: Frequency>(&mut self, now: u32) {
        self.alarm_data.t0 = now;
        let nonce = self.random_nonce() % 10;

        let period_ms = (self.advertisement_interval_ms + nonce) * F::frequency() / 1000;
        self.alarm_data.expiration = Expiration::Abs(now.wrapping_add(period_ms));
    }
}

pub struct BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    radio: &'a B,
    busy: Cell<bool>,
    app: kernel::Grant<App>,
    kernel_tx: kernel::common::take_cell::TakeCell<'static, [u8]>,
    alarm: &'a A,
    sending_app: Cell<Option<kernel::AppId>>,
    receiving_app: Cell<Option<kernel::AppId>>,
}

impl<'a, B, A> BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
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
            sending_app: Cell::new(None),
            receiving_app: Cell::new(None),
        }
    }

    // Determines which app timer will expire next and sets the underlying alarm
    // to it.
    //
    // This method iterates through all grants so it should be used somewhat
    // sparringly. Moreover, it should _not_ be called from within a grant,
    // since any open grant will not be iterated over and the wrong timer will
    // likely be chosen.
    fn reset_active_alarm(&self) {
        let now = self.alarm.now();
        let mut next_alarm = u32::max_value();
        let mut next_dist = u32::max_value();
        for app in self.app.iter() {
            app.enter(|app, _| match app.alarm_data.expiration {
                Expiration::Abs(exp) => {
                    let t_dist = exp.wrapping_sub(now);
                    if next_dist > t_dist {
                        next_alarm = exp;
                        next_dist = t_dist;
                    }
                }
                Expiration::Disabled => {}
            });
        }
        if next_alarm != u32::max_value() {
            self.alarm.set_alarm(next_alarm);
        }
    }
}

// Timer alarm
impl<'a, B, A> kernel::hil::time::Client for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    // When an alarm is fired, we find which apps have expired timers. Expired
    // timers indicate a desire to perform some operation (e.g. start an
    // advertising or scanning event). We know which operation based on the
    // current app's state.
    //
    // In case of collision---if there is already an event happening---we'll
    // just delay the operation for next time and hope for the best. Since some
    // randomness is added for each period in an app's timer, collisions should
    // be rare in practice.
    //
    // TODO: perhaps break ties more fairly by prioritizing apps that have least
    // recently performed an operation.
    fn fired(&self) {
        let now = self.alarm.now();

        self.app.each(|app| {
            if let Expiration::Abs(exp) = app.alarm_data.expiration {
                let expired =
                    now.wrapping_sub(app.alarm_data.t0) >= exp.wrapping_sub(app.alarm_data.t0);
                if expired {
                    if self.busy.get() {
                        // The radio is currently busy, so we won't be able to start the
                        // operation at the appropriate time. Instead, reschedule the
                        // operation for later. This is _kind_ of simulating actual
                        // on-air interference
                        debug!("BLE: operationg delayed for app {:?}", app.appid());
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        return;
                    }

                    match app.process_status {
                        Some(BLEState::AdvertisingIdle) => {
                            self.busy.set(true);
                            app.process_status =
                                Some(BLEState::Advertising(RadioChannel::AdvertisingChannel37));
                            self.sending_app.set(Some(app.appid()));
                            self.radio.set_tx_power(app.tx_power);
                            app.send_advertisement(&self, RadioChannel::AdvertisingChannel37);
                        }
                        Some(BLEState::ScanningIdle) => {
                            self.busy.set(true);
                            app.process_status =
                                Some(BLEState::Scanning(RadioChannel::AdvertisingChannel37));
                            self.receiving_app.set(Some(app.appid()));
                            self.radio.set_tx_power(app.tx_power);
                            self.radio
                                .receive_advertisement(RadioChannel::AdvertisingChannel37);
                        }
                        _ => debug!(
                            "app: {:?} \t invalid state {:?}",
                            app.appid(),
                            app.process_status
                        ),
                    }
                }
            }
        });
        self.reset_active_alarm();
    }
}

// Callback from the radio once a RX event occur
impl<'a, B, A> ble_advertising::RxClient for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {
        if let Some(appid) = self.receiving_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                // Validate the received data, because ordinary BLE packets can be bigger than 39
                // bytes we need check for that!
                // Moreover, we use the packet header to find size but the radio reads maximum
                // 39 bytes.
                // Therefore, we ignore payloads with a header size bigger than 39 because the
                // channels 37, 38 and 39 should only be used for advertisements!
                // Packets that are bigger than 39 bytes are likely "Channel PDUs" which should
                // only be sent on the other 37 RadioChannel channels.

                let notify_userland = if len <= PACKET_LENGTH as u8 && app.app_read.is_some()
                    && result == ReturnCode::SUCCESS
                {
                    let dest = app.app_read.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in buf[0..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                    true
                } else {
                    false
                };

                if notify_userland {
                    app.scan_callback.map(|mut cb| {
                        cb.schedule(usize::from(result), len as usize, 0);
                    });
                }

                match app.process_status {
                    Some(BLEState::Scanning(RadioChannel::AdvertisingChannel37)) => {
                        app.process_status =
                            Some(BLEState::Scanning(RadioChannel::AdvertisingChannel38));
                        app.alarm_data.expiration = Expiration::Disabled;
                        self.receiving_app.set(Some(app.appid()));
                        self.radio.set_tx_power(app.tx_power);
                        self.radio
                            .receive_advertisement(RadioChannel::AdvertisingChannel38);
                    }
                    Some(BLEState::Scanning(RadioChannel::AdvertisingChannel38)) => {
                        app.process_status =
                            Some(BLEState::Scanning(RadioChannel::AdvertisingChannel39));
                        self.receiving_app.set(Some(app.appid()));
                        self.radio
                            .receive_advertisement(RadioChannel::AdvertisingChannel39);
                    }
                    Some(BLEState::Scanning(RadioChannel::AdvertisingChannel39)) => {
                        self.busy.set(false);
                        app.process_status = Some(BLEState::ScanningIdle);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                    }
                    // Invalid state => don't care
                    _ => (),
                }
            });
            self.reset_active_alarm();
        }
    }
}

// Callback from the radio once a TX event occur
impl<'a, B, A> ble_advertising::TxClient for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    // The ReturnCode indicates valid CRC or not, not used yet but could be used for
    // re-tranmissions for invalid CRCs
    fn transmit_event(&self, _crc_ok: ReturnCode) {
        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                match app.process_status {
                    Some(BLEState::Advertising(RadioChannel::AdvertisingChannel37)) => {
                        app.process_status =
                            Some(BLEState::Advertising(RadioChannel::AdvertisingChannel38));
                        app.alarm_data.expiration = Expiration::Disabled;
                        self.sending_app.set(Some(app.appid()));
                        self.radio.set_tx_power(app.tx_power);
                        app.send_advertisement(&self, RadioChannel::AdvertisingChannel38);
                    }

                    Some(BLEState::Advertising(RadioChannel::AdvertisingChannel38)) => {
                        app.process_status =
                            Some(BLEState::Advertising(RadioChannel::AdvertisingChannel39));
                        self.sending_app.set(Some(app.appid()));
                        app.send_advertisement(&self, RadioChannel::AdvertisingChannel39);
                    }

                    Some(BLEState::Advertising(RadioChannel::AdvertisingChannel39)) => {
                        self.busy.set(false);
                        app.process_status = Some(BLEState::AdvertisingIdle);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                    }
                    // Invalid state => don't care
                    _ => (),
                }
            });
            self.reset_active_alarm();
        }
    }
}

// System Call implementation
impl<'a, B, A> kernel::Driver for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig + 'a,
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
            0 => self.app
                .enter(appid, |app, _| {
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::AdvertisingIdle);
                        app.random_nonce = self.alarm.now();
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        self.reset_active_alarm();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Stop periodic advertisements or passive scanning
            1 => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::AdvertisingIdle) | Some(BLEState::ScanningIdle) => {
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EBUSY,
                })
                .unwrap_or_else(|err| err.into()),

            // Configure transmitted power
            // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part A], section 3
            //
            // Minimum Output Power:    0.01 mW (-20 dBm)
            // Maximum Output Power:    10 mW (+10 dBm)
            //
            // data - Transmitting power in dBm
            2 => {
                self.app
                    .enter(appid, |app, _| {
                        if app.process_status != Some(BLEState::ScanningIdle)
                            && app.process_status != Some(BLEState::AdvertisingIdle)
                        {
                            match data as u8 {
                                e @ 0...10 | e @ 0xec...0xff => {
                                    app.tx_power = e;
                                    // ask chip if the power level is supported
                                    self.radio.set_tx_power(e)
                                }
                                _ => ReturnCode::EINVAL,
                            }
                        } else {
                            ReturnCode::EBUSY
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Configure advertisement interval
            // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 4.4.2.2
            //
            // The advertisment interval shall an integer multiple of 0.625ms in the range of
            // 20ms to 10240 ms!
            //
            // data - advertisement interval in ms
            // FIXME: add check that data is a multiple of 0.625
            3 => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::Scanning(_)) | Some(BLEState::Advertising(_)) => {
                        ReturnCode::EBUSY
                    }
                    _ => {
                        app.advertisement_interval_ms = cmp::max(20, cmp::min(10240, data as u32));
                        ReturnCode::SUCCESS
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Reset payload when the kernel is not actively advertising
            // reset_payload checks whether the current app is correct state or not
            // i.e. if it's ok to reset the payload or not
            4 => self.app
                .enter(appid, |app, _| app.reset_payload())
                .unwrap_or_else(|err| err.into()),

            // Passive scanning mode
            5 => self.app
                .enter(appid, |app, _| {
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::ScanningIdle);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        self.reset_active_alarm();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Initilize BLE Driver
            // Allow call to allocate the advertisement buffer must be
            // invoked before this
            // Request advertisement address
            6 => self.app
                .enter(appid, |app, _| {
                    if let Some(BLEState::Initialized) = app.process_status {
                        let status = app.generate_random_address(appid);
                        if status == ReturnCode::SUCCESS {
                            app.configure_advertisement_pdu()
                        } else {
                            status
                        }
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        appid: kernel::AppId,
        allow_num: usize,
        slice: Option<kernel::AppSlice<kernel::Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Configure GAP Data
            0 => self.app
                .enter(appid, |app, _| {
                    if app.process_status != Some(BLEState::NotInitialized) {
                        app.app_write = slice;
                        app.set_gap_data()
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Passive Scanning
            1 => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::NotInitialized) | Some(BLEState::Initialized) => {
                        app.app_read = slice;
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EINVAL,
                })
                .unwrap_or_else(|err| err.into()),

            // Initialize Advertisement Buffer
            2 => self.app
                .enter(appid, |app, _| {
                    if let Some(BLEState::NotInitialized) = app.process_status {
                        app.advertisement_buf = slice;
                        app.process_status = Some(BLEState::Initialized);
                        app.initialize_advertisement_buffer();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<kernel::Callback>,
        app_id: kernel::AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Callback for scanning
            0 => self.app
                .enter(app_id, |app, _| match app.process_status {
                    Some(BLEState::NotInitialized) | Some(BLEState::Initialized) => {
                        app.scan_callback = callback;
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EINVAL,
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
