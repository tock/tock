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
//!
//! The allow systems calls are used for buffers from allocated by userland
//!
//! There are two different buffers:
//! * 0: Advertising data
//! * 1: Passive scanning buffer
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * SUCCESS: The buffer has successfully been filled
//! * ENOMEM: No sufficient memory available
//! * EINVAL: Invalid address of the buffer or other error
//! * EBUSY: The driver is currently busy with other tasks
//! * ENOSUPPORT: The operation is not supported
//! * ERROR: Operation `map` on Option failed
//!
//! ### Subscribe system call
//!
//!  The `subscribe` system call supports two arguments `subscribe number' and `callback`.
//!  The `subscribe` is used to specify the specific operation, currently:
//!
//! * 0: provides a callback user-space when a device scanning for advertisements
//!      and the callback is used to invoke user-space processes.
//!
//! The possible return codes from the `allow` system call indicate the following:
//!
//! * ENOMEM:    Not sufficient amount memory
//! * EINVAL:    Invalid operation
//!
//! ### Command system call
//!
//! The `command` system call supports two arguments `command number` and `subcommand number`.
//! `command number` is used to specify the specific operation, currently
//! the following commands are supported:
//!
//! * 0: start advertisement
//! * 1: stop advertisement or scanning
//! * 5: start scanning
//!
//! The possible return codes from the `command` system call indicate the following:
//!
//! * SUCCESS:      The command was successful
//! * EBUSY:        The driver is currently busy with other tasks
//! * ENOSUPPORT:   The operation is not supported
//!
//! Usage
//! -----
//!
//! You need a device that provides the `kernel::BleAdvertisementDriver` trait along with a virtual
//! timer to perform events and not block the entire kernel
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

// # Implementation
//
// Advertising virtualization works by implementing a virtual periodic timer for each process. The
// timer is configured to fire at each advertising interval, as specified by the process. When a
// timer fires, we serialize the advertising packet for that process (using the provided AdvData
// payload, generated address and PDU type) and perform one advertising event (on each of three
// channels).
//
// This means that advertising events can collide. In this case, we just defer one of the
// advertisements. Because we add a pseudo random pad to the timer interval each time (as required
// by the Bluetooth specification) multiple collisions of the same processes are highly unlikely.

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;
use kernel::hil::time::{Frequency, Ticks};
use kernel::ReturnCode;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::BleAdvertising as usize;

/// Advertisement Buffer
pub static mut BUF: [u8; PACKET_LENGTH] = [0; PACKET_LENGTH];

const PACKET_ADDR_LEN: usize = 6;
const PACKET_LENGTH: usize = 39;
const ADV_HEADER_TXADD_OFFSET: usize = 6;

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
    Enabled(u32, u32),
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

type AdvPduType = u8;

// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.3
const ADV_IND: AdvPduType = 0b0000;
#[allow(dead_code)]
const ADV_DIRECTED_IND: AdvPduType = 0b0001;
const ADV_NONCONN_IND: AdvPduType = 0b0010;
#[allow(dead_code)]
const SCAN_REQ: AdvPduType = 0b0011;
#[allow(dead_code)]
const SCAN_RESP: AdvPduType = 0b0100;
#[allow(dead_code)]
const CONNECT_IND: AdvPduType = 0b0101;
const ADV_SCAN_IND: AdvPduType = 0b0110;

/// Process specific memory
pub struct App {
    process_status: Option<BLEState>,
    alarm_data: AlarmData,

    // Advertising meta-data
    adv_data: Option<kernel::AppSlice<kernel::Shared, u8>>,
    address: [u8; PACKET_ADDR_LEN],
    pdu_type: AdvPduType,
    advertisement_interval_ms: u32,
    tx_power: u8,
    /// The state of an app-specific pseudo random number.
    ///
    /// For example, it can be used for the pseudo-random `advDelay` parameter.
    /// It should be read using the `random_number` method, which updates it as
    /// well.
    random_nonce: u32,

    // Scanning meta-data
    scan_buffer: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
}

impl Default for App {
    fn default() -> App {
        App {
            alarm_data: AlarmData::new(),
            adv_data: None,
            scan_buffer: None,
            address: [0; PACKET_ADDR_LEN],
            pdu_type: ADV_NONCONN_IND,
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
        self.address = [
            0xf0,
            (appid.id() & 0xff) as u8,
            ((appid.id() << 8) & 0xff) as u8,
            ((appid.id() << 16) & 0xff) as u8,
            ((appid.id() << 24) & 0xff) as u8,
            0xf0,
        ];
        ReturnCode::SUCCESS
    }

    fn send_advertisement<'a, B, A>(&self, ble: &BLE<'a, B, A>, channel: RadioChannel) -> ReturnCode
    where
        B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
        A: kernel::hil::time::Alarm<'a>,
    {
        self.adv_data.as_ref().map_or(ReturnCode::FAIL, |adv_data| {
            ble.kernel_tx.take().map_or(ReturnCode::FAIL, |kernel_tx| {
                let adv_data_len = cmp::min(kernel_tx.len() - PACKET_ADDR_LEN - 2, adv_data.len());
                let adv_data_corrected = &adv_data.as_ref()[..adv_data_len];
                let payload_len = adv_data_corrected.len() + PACKET_ADDR_LEN;
                {
                    let (header, payload) = kernel_tx.split_at_mut(2);
                    header[0] = self.pdu_type;
                    match self.pdu_type {
                        ADV_IND | ADV_NONCONN_IND | ADV_SCAN_IND => {
                            // Set TxAdd because AdvA field is going to be a "random"
                            // address
                            header[0] |= 1 << ADV_HEADER_TXADD_OFFSET;
                        }
                        _ => {}
                    }
                    // The LENGTH field is 6-bits wide, so make sure to truncate it
                    header[1] = (payload_len & 0x3f) as u8;

                    let (adva, data) = payload.split_at_mut(6);
                    adva.copy_from_slice(&self.address);
                    data[..adv_data_len].copy_from_slice(adv_data_corrected);
                }
                let total_len = cmp::min(PACKET_LENGTH, payload_len + 2);
                let result = ble
                    .radio
                    .transmit_advertisement(kernel_tx, total_len, channel);
                ble.kernel_tx.replace(result);
                ReturnCode::SUCCESS
            })
        })
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
        self.alarm_data.expiration = Expiration::Enabled(now, period_ms);
    }
}

pub struct BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
    A: kernel::hil::time::Alarm<'a>,
{
    radio: &'a B,
    busy: Cell<bool>,
    app: kernel::Grant<App>,
    kernel_tx: kernel::common::cells::TakeCell<'static, [u8]>,
    alarm: &'a A,
    sending_app: OptionalCell<kernel::AppId>,
    receiving_app: OptionalCell<kernel::AppId>,
}

impl<B, A> BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
    A: kernel::hil::time::Alarm<'a>,
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
            kernel_tx: kernel::common::cells::TakeCell::new(tx_buf),
            alarm: alarm,
            sending_app: OptionalCell::empty(),
            receiving_app: OptionalCell::empty(),
        }
    }

    // Determines which app timer will expire next and sets the underlying alarm
    // to it.
    //
    // This method iterates through all grants so it should be used somewhat
    // sparingly. Moreover, it should _not_ be called from within a grant,
    // since any open grant will not be iterated over and the wrong timer will
    // likely be chosen.
    fn reset_active_alarm(&self) {
        let now = self.alarm.now();
        let mut next_ref = u32::max_value();
        let mut next_dt = u32::max_value();
        let mut next_dist = u32::max_value();
        for app in self.app.iter() {
            app.enter(|app, _| match app.alarm_data.expiration {
                Expiration::Enabled(reference, dt) => {
                    let exp = reference.wrapping_add(dt);
                    let t_dist = exp.wrapping_sub(now.into_u32());
                    if next_dist > t_dist {
                        next_ref = reference;
                        next_dt = dt;
                        next_dist = t_dist;
                    }
                }
                Expiration::Disabled => {}
            });
        }
        if next_ref != u32::max_value() {
            self.alarm.set_alarm(A::Ticks::from(next_ref), A::Ticks::from(next_dt));
        }
    }
}

// Timer alarm
impl<B, A> kernel::hil::time::AlarmClient for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
    A: kernel::hil::time::Alarm<'a>,
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
    fn alarm(&self) {
        let now = self.alarm.now();

        self.app.each(|app| {
            if let Expiration::Enabled(reference, dt) = app.alarm_data.expiration {
                let exp = A::Ticks::from(reference.wrapping_add(dt));
                let t0 = A::Ticks::from(app.alarm_data.t0);
                let expired = t0.within_range(exp, now);
                if expired {
                    if self.busy.get() {
                        // The radio is currently busy, so we won't be able to start the
                        // operation at the appropriate time. Instead, reschedule the
                        // operation for later. This is _kind_ of simulating actual
                        // on-air interference. 3 seems like a small number of ticks.
                        debug!("BLE: operation delayed for app {:?}", app.appid());
                        app.set_next_alarm::<A::Frequency>(self.alarm.now().into_u32());
                        return;
                    }

                    app.alarm_data.expiration = Expiration::Disabled;

                    match app.process_status {
                        Some(BLEState::AdvertisingIdle) => {
                            self.busy.set(true);
                            app.process_status =
                                Some(BLEState::Advertising(RadioChannel::AdvertisingChannel37));
                            self.sending_app.set(app.appid());
                            self.radio.set_tx_power(app.tx_power);
                            app.send_advertisement(&self, RadioChannel::AdvertisingChannel37);
                        }
                        Some(BLEState::ScanningIdle) => {
                            self.busy.set(true);
                            app.process_status =
                                Some(BLEState::Scanning(RadioChannel::AdvertisingChannel37));
                            self.receiving_app.set(app.appid());
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
impl<B, A> ble_advertising::RxClient for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
    A: kernel::hil::time::Alarm<'a>,
{
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {
        self.receiving_app.map(|appid| {
            let _ = self.app.enter(*appid, |app, _| {
                // Validate the received data, because ordinary BLE packets can be bigger than 39
                // bytes. Thus, we need to check for that!
                // Moreover, we use the packet header to find size but the radio reads maximum
                // 39 bytes.
                // Therefore, we ignore payloads with a header size bigger than 39 because the
                // channels 37, 38 and 39 should only be used for advertisements!
                // Packets that are bigger than 39 bytes are likely `Channel PDUs` which should
                // only be sent on the other 37 RadioChannel channels.

                if len <= PACKET_LENGTH as u8 && result == ReturnCode::SUCCESS {
                    // write to buffer in userland
                    let success = app
                        .scan_buffer
                        .as_mut()
                        .map(|userland| {
                            for (dst, src) in userland.iter_mut().zip(buf[0..len as usize].iter()) {
                                *dst = *src;
                            }
                        })
                        .is_some();

                    if success {
                        app.scan_callback.map(|mut cb| {
                            cb.schedule(usize::from(result), len as usize, 0);
                        });
                    }
                }

                match app.process_status {
                    Some(BLEState::Scanning(RadioChannel::AdvertisingChannel37)) => {
                        app.process_status =
                            Some(BLEState::Scanning(RadioChannel::AdvertisingChannel38));
                        self.receiving_app.set(app.appid());
                        self.radio.set_tx_power(app.tx_power);
                        self.radio
                            .receive_advertisement(RadioChannel::AdvertisingChannel38);
                    }
                    Some(BLEState::Scanning(RadioChannel::AdvertisingChannel38)) => {
                        app.process_status =
                            Some(BLEState::Scanning(RadioChannel::AdvertisingChannel39));
                        self.receiving_app.set(app.appid());
                        self.radio
                            .receive_advertisement(RadioChannel::AdvertisingChannel39);
                    }
                    Some(BLEState::Scanning(RadioChannel::AdvertisingChannel39)) => {
                        self.busy.set(false);
                        app.process_status = Some(BLEState::ScanningIdle);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now().into_u32());
                    }
                    // Invalid state => don't care
                    _ => (),
                }
            });
            self.reset_active_alarm();
        });
    }
}

// Callback from the radio once a TX event occur
impl<B, A> ble_advertising::TxClient for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
    A: kernel::hil::time::Alarm<'a>,
{
    // The ReturnCode indicates valid CRC or not, not used yet but could be used for
    // re-transmissions for invalid CRCs
    fn transmit_event(&self, _crc_ok: ReturnCode) {
        self.sending_app.map(|appid| {
            let _ = self.app.enter(*appid, |app, _| {
                match app.process_status {
                    Some(BLEState::Advertising(RadioChannel::AdvertisingChannel37)) => {
                        app.process_status =
                            Some(BLEState::Advertising(RadioChannel::AdvertisingChannel38));
                        self.sending_app.set(app.appid());
                        self.radio.set_tx_power(app.tx_power);
                        app.send_advertisement(&self, RadioChannel::AdvertisingChannel38);
                    }

                    Some(BLEState::Advertising(RadioChannel::AdvertisingChannel38)) => {
                        app.process_status =
                            Some(BLEState::Advertising(RadioChannel::AdvertisingChannel39));
                        self.sending_app.set(app.appid());
                        app.send_advertisement(&self, RadioChannel::AdvertisingChannel39);
                    }

                    Some(BLEState::Advertising(RadioChannel::AdvertisingChannel39)) => {
                        self.busy.set(false);
                        app.process_status = Some(BLEState::AdvertisingIdle);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now().into_u32());
                    }
                    // Invalid state => don't care
                    _ => (),
                }
            });
            self.reset_active_alarm();
        });
    }
}

// System Call implementation
impl<B, A> kernel::Driver for BLE<'a, B, A>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
    A: kernel::hil::time::Alarm<'a>,
{
    fn command(
        &self,
        command_num: usize,
        data: usize,
        interval: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {
            // Start periodic advertisements
            0 => self
                .app
                .enter(appid, |app, _| {
                    if let Some(BLEState::Initialized) = app.process_status {
                        let pdu_type = data as AdvPduType;
                        match pdu_type {
                            ADV_IND | ADV_NONCONN_IND | ADV_SCAN_IND => {
                                app.pdu_type = pdu_type;
                                app.process_status = Some(BLEState::AdvertisingIdle);
                                app.random_nonce = self.alarm.now().into_u32();
                                app.advertisement_interval_ms = cmp::max(20, interval as u32);
                                app.set_next_alarm::<A::Frequency>(self.alarm.now().into_u32());
                                self.reset_active_alarm();
                                ReturnCode::SUCCESS
                            }
                            _ => ReturnCode::EINVAL,
                        }
                    } else {
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Stop periodic advertisements or passive scanning
            1 => self
                .app
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
                                tx_power @ 0..=10 | tx_power @ 0xec..=0xff => {
                                    // query the underlying chip if the power level is supported
                                    let status = self.radio.set_tx_power(tx_power);
                                    if let ReturnCode::SUCCESS = status {
                                        app.tx_power = tx_power;
                                    }
                                    status
                                }
                                _ => ReturnCode::EINVAL,
                            }
                        } else {
                            ReturnCode::EBUSY
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Passive scanning mode
            5 => self
                .app
                .enter(appid, |app, _| {
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::ScanningIdle);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now().into_u32());
                        self.reset_active_alarm();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EBUSY
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
            // Advertisement buffer
            0 => self
                .app
                .enter(appid, |app, _| {
                    app.adv_data = slice;
                    if let ReturnCode::SUCCESS = app.generate_random_address(appid) {
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::FAIL
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Passive scanning buffer
            1 => self
                .app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::NotInitialized) | Some(BLEState::Initialized) => {
                        app.scan_buffer = slice;
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EINVAL,
                })
                .unwrap_or_else(|err| err.into()),

            // Operation not supported
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
            0 => self
                .app
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
