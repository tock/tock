#![allow(unused_imports)]

use core::cell::Cell;
use core::cmp;
use fixedvec::FixedVec;
use kernel::common::cells::{TakeCell, OptionalCell, MapCell};
use kernel::{AppId, AppSlice, Shared, Callback, Driver, ReturnCode, Grant};
use kernel::hil::{radio_client, time::Alarm, time::Frequency, time::Client};
use net::stream::{decode_bytes, decode_u8, encode_bytes, encode_u8, SResult};
use helium::{device::TxClient, device::Device, framer, framer::FecType};

// static mut PAYLOAD: [u8; 256] = [0; 256];

// Syscall number
pub const DRIVER_NUM: usize = 0xCC_13_12;

#[derive(Debug, Clone, Copy)]
pub enum HeliumState {
    NotInitialized,
    Idle(PowerMode),
    PendingCommand,
    Pending(radio_client::RadioOperation),
    Done,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum PowerMode {
    Active,
    Sleep,
    DeepSleep,
}

// pub static mut RFC_STACK: [HeliumState; 6] = [HeliumState::NotInitialized; 6];

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

// #[derive(Default)]
pub struct App {
    process_status: Option<HeliumState>,
    alarm_data: AlarmData,
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_cfg: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    app_read: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<(u16, Option<FecType>)>, // Change u32 to keyid and fec mode later on during implementation
    tx_interval_ms: u32, // 400 ms is maximum per FCC
}

impl Default for App {
    fn default() -> App {
        App {
            process_status: Some(HeliumState::NotInitialized),
            alarm_data: AlarmData::new(),
            tx_callback: None,
            rx_callback: None,
            app_cfg: None,
            app_write: None,
            app_read: None,
            pending_tx: None,
            tx_interval_ms: 400,
        }
    }
}

pub struct Helium<'a, D> 
where
    D: Device<'a>,
{
    app: Grant<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    current_app: OptionalCell<AppId>,
    device: &'a D,
}

impl<D> Helium<'a, D> 
where 
    D: Device<'a>,
{
    pub fn new(
        container: Grant<App>,
        tx_buf: &'static mut [u8],
        device: &'a D,
    ) -> Helium<'a, D> 
    {   
        Helium {
            app: container,
            kernel_tx: TakeCell::new(tx_buf),
            current_app: OptionalCell::empty(),
            device: device,
        }
    }

    /// Utility function to perform an action on an app in a system call.
    #[inline]
    fn do_with_app<F>(&self, appid: AppId, closure: F) -> ReturnCode
    where
        F: FnOnce(&mut App) -> ReturnCode,
    {
        self.app
            .enter(appid, |app, _| closure(app))
            .unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform an action using an app's config buffer.
    #[inline]
    fn do_with_cfg<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&[u8]) -> ReturnCode,
    {
        self.app
            .enter(appid, |app, _| {
                app.app_cfg
                    .take()
                    .as_ref()
                    .map_or(ReturnCode::EINVAL, |cfg| {
                        if cfg.len() != len {
                            return ReturnCode::EINVAL;
                        }
                        closure(cfg.as_ref())
                    })
            }).unwrap_or_else(|err| err.into())
    }

    /// Utility function to perform a write to an app's config buffer.
    #[inline]
    fn do_with_cfg_mut<F>(&self, appid: AppId, len: usize, closure: F) -> ReturnCode
    where
        F: FnOnce(&mut [u8]) -> ReturnCode,
    {
        self.app
            .enter(appid, |app, _| {
                app.app_cfg
                    .take()
                    .as_mut()
                    .map_or(ReturnCode::EINVAL, |cfg| {
                        if cfg.len() != len {
                            return ReturnCode::EINVAL;
                        }
                        closure(cfg.as_mut())
                    })
            }).unwrap_or_else(|err| err.into())
    }

    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `AppId`.
    fn get_next_tx_if_idle(&self) -> Option<AppId> {
        if self.current_app.is_some() {
            return None;
        }
        let mut pending_app = None;
        for app in self.app.iter() {
            app.enter(|app, _| {
                if app.pending_tx.is_some() {
                    pending_app = Some(app.appid());
                }
            });
            if pending_app.is_some() {
                break;
            }
        }
        pending_app
    }

    /// Performs `appid`'s pending transmission asynchronously. If the
    /// transmission is not successful, the error is returned to the app via its
    /// `tx_callback`. Assumes that the driver is currently idle and the app has
    /// a pending transmission.
    #[inline]
    fn perform_tx_async(&self, appid: AppId) {
        let result = self.perform_tx_sync(appid);
        if result != ReturnCode::SUCCESS {
            let _ = self.app.enter(appid, |app, _| {
                app.tx_callback
                    .take()
                    .map(|mut cb| cb.schedule(result.into(), 0, 0));
            });
        }
    }

    /// Performs `appid`'s pending transmission synchronously. The result is
    /// returned immediately to the app. Assumes that the driver is currently
    /// idle and the app has a pending transmission.
    #[inline]
    fn perform_tx_sync(&self, appid: AppId) -> ReturnCode {
        self.do_with_app(appid, |app| {
            let _device_id = match app.pending_tx.take() {
                Some(pending_tx) => pending_tx,
                None => {
                    return ReturnCode::SUCCESS;
                }
            };
            
            let result = self.kernel_tx.take().map_or(ReturnCode::ENOMEM, |kbuf| {
                // Frame header implementation for Helium prep here. Currently unknown so removing
                // 802154 stuff
                let seq: u8 = 0;
                let fec_type = None; 
                let frame = match self.device.prepare_data_frame(
                    kbuf,
                    seq,
                    fec_type, 
                ) {
                    Ok(frame) => frame,
                    Err(kbuf) => {
                        self.kernel_tx.replace(kbuf);
                        return ReturnCode::FAIL;
                    }
                };
                // Finally, transmit the frame
                let (result, mbuf) = self.device.transmit(frame);
                if let Some(buf) = mbuf {
                    self.kernel_tx.replace(buf);
                }
                result
            });
            if result == ReturnCode::SUCCESS {
                self.current_app.set(appid);
            }
            result
        })
    }

    /// Schedule the next transmission if there is one pending. Performs the
    /// transmission asynchronously, returning any errors via callbacks.
    #[inline]
    fn do_next_tx_async(&self) {
        self.get_next_tx_if_idle()
            .map(|appid| self.perform_tx_async(appid));
    }

    /// Schedule the next transmission if there is one pending. If the next
    /// transmission happens to be the one that was just queued, then the
    /// transmission is synchronous. Hence, errors must be returned immediately.
    /// On the other hand, if it is some other app, then return any errors via
    /// callbacks.
    #[inline]
    fn do_next_tx_sync(&self, new_appid: AppId) -> ReturnCode {
        self.get_next_tx_if_idle()
            .map(|appid| {
                if appid == new_appid {
                    self.perform_tx_sync(appid)
                } else {
                    self.perform_tx_async(appid);
                    ReturnCode::SUCCESS
                }
            }).unwrap_or(ReturnCode::SUCCESS)
    }
}


impl<D> Driver for Helium<'a, D>
where
    D: Device<'a>,
{
    /// Setup buffers to read/write from.
    ///
    ///  `allow_num`
    ///
    /// - `0`: Read buffer. Will contain the received frame.
    /// - `1`: Write buffer. Contains the frame payload to be transmitted.
    /// - `2`: Config buffer. Used to contain miscellaneous data associated with
    ///        some commands because the system call parameters / return codes are
    ///        not enough to convey the desired information.
    fn allow(&self, appid: AppId, allow_num: usize, slice: Option<AppSlice<Shared, u8>>) -> ReturnCode {
        match allow_num {
            0 | 1 | 2 => self.do_with_app(appid, |app| {
                match allow_num {
                    0 => app.app_read = slice,
                    1 => app.app_write = slice,
                    2 => app.app_cfg = slice,
                    _ => {}
                }
                ReturnCode::SUCCESS
            }),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    ///  `subscribe_num`
    /// - `0`: Setup callback for when frame is received.
    /// - `1`: Setup callback for when frame is transmitted.
    fn subscribe(&self, subscribe_num: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        let sub: HeliumCallback = subscribe_num.into();
        match sub {
            HeliumCallback::RxCallback => self.do_with_app(app_id, |app| {
                app.rx_callback = callback;
                ReturnCode::SUCCESS
            }),
            HeliumCallback::TxCallback => self.do_with_app(app_id, |app| {
                app.tx_callback = callback;
                ReturnCode::SUCCESS
            }),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
    /// COMMANDS
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Return radio status. SUCCESS/EOFF = on/off.
    /// - `2`: Set transmission power.
    /// - `3`: Get the transmission power.

    fn command(&self, command_num: usize, r2: usize, _r3: usize, appid: AppId) -> ReturnCode {
        let command: HeliumCommand = command_num.into();
        match command {
            // Handle callback for CMDSTA after write to CMDR
            HeliumCommand::DriverCheck => ReturnCode::SUCCESS,
            HeliumCommand::GetRadioStatus => {
                if self.device.is_on() {
                    ReturnCode::SUCCESS
                }
                else {
                    ReturnCode::EOFF
                }
            }
            HeliumCommand::SetTxPower => ReturnCode::ENOSUPPORT, // Link to set tx power in radio
            HeliumCommand::GetTxPower => ReturnCode::ENOSUPPORT, // Link to get tx power in radio
            HeliumCommand::SetNextTx => {
                self.do_with_app(appid, |app| {
                    if app.pending_tx.is_some() {
                        return ReturnCode::EBUSY;
                    }
                    let addr = r2 as u16;

                    let next_tx = app.app_cfg.as_ref().and_then(|cfg| {
                        if cfg.len() != 11 {
                            return None;
                        }
                        let fec = match FecType::from_slice(cfg.as_ref()[0]) {// The first entry `[0]` should be the encoding type
                            Some(fec) => fec,
                            None => {return None;}
                        };
                        
                        if fec == FecType::None {
                            return Some((addr, None));
                        }
                        Some((addr, Some(fec)))
                    });
                    if next_tx.is_none() {
                        return ReturnCode::EINVAL;
                    }
                    app.pending_tx = next_tx;

                    self.do_next_tx_sync(appid)
                })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<D> TxClient for Helium<'a, D>
where
    D: Device<'a>,
{
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.kernel_tx.replace(buf);
        self.current_app.take().map(|appid| {
            let _ = self.app.enter(appid, |app, _| {
                app.tx_callback
                    .take()
                    .map(|mut cb| cb.schedule(result.into(), 0, 0));
            });
        });
        self.do_next_tx_async();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HeliumCallback {
    RxCallback = 0,
    TxCallback = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum HeliumCommand {
    DriverCheck = 0,
    GetRadioStatus = 1,
    SetTxPower = 2,
    GetTxPower = 3,
    SetNextTx = 4,
    Invalid = 5,
}
/*
#[derive(Debug, Clone, Copy)]
pub enum RfcOperationStatus {
    Idle,
    Pending,
    Active,
    Skipped,
    SendDone,
    CommandDone,
    LastCommandDone,
    RxOk,
    TxDone,
    Setup,
    Invalid,
}

impl From<usize> for RfcOperationStatus {
    fn from(val: usize) -> RfcOperationStatus {
        match val {
            0 => RfcOperationStatus::Idle,
            1 => RfcOperationStatus::Pending,
            2 => RfcOperationStatus::Active,
            3 => RfcOperationStatus::Skipped,
            4 => RfcOperationStatus::SendDone,
            5 => RfcOperationStatus::TxDone,
            6 => RfcOperationStatus::CommandDone,
            7 => RfcOperationStatus::LastCommandDone,
            8 => RfcOperationStatus::RxOk,
            9 => RfcOperationStatus::TxDone,
            val => {
                debug_assert!(false, "{} does not represent a valid command.", val);
                RfcOperationStatus::Invalid
            }
        }
    }
}
*/
impl From<&'a HeliumCallback> for usize {
    fn from(cmd: &HeliumCallback) -> usize {
        *cmd as usize
    }
}

impl From<usize> for HeliumCallback {
    fn from(val: usize) -> HeliumCallback {
        match val {
            0 => HeliumCallback::RxCallback,
            1 => HeliumCallback::TxCallback,
            _ => panic!("Not a valid callback num")
        }
    }
}

impl From<&'a HeliumCommand> for usize {
    fn from(cmd: &HeliumCommand) -> usize {
        *cmd as usize
    }
}

impl From<usize> for HeliumCommand {
    fn from(val: usize) -> HeliumCommand {
        match val {
            0 => HeliumCommand::DriverCheck,
            1 => HeliumCommand::GetRadioStatus,
            2 => HeliumCommand::SetTxPower,
            3 => HeliumCommand::GetTxPower,
            4 => HeliumCommand::SetNextTx,
            val => {
                debug_assert!(false, "{} does not represent a valid command.", val);
                HeliumCommand::Invalid
            }
        }
    }
}
