use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{TakeCell, OptionalCell, MapCell};
use kernel::{AppId, AppSlice, Shared, Callback, Driver, ReturnCode, Grant};
use kernel::hil::radio_client;

// Syscall number
pub const DRIVER_NUM: usize = 0xCC_13_12;

#[derive(Debug, Clone, Copy)]
pub enum HeliumState {
    NotInitialized,
    Idle,
    PendingCommand,
    Pending(radio_client::RadioOperation),
    Done,
    Invalid,
}

// #[derive(Default)]
pub struct App {
    process_status: Option<HeliumState>,
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_cfg: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    app_read: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<(u16, Option<RfcOperationStatus>)>, // Change u32 to keyid and fec mode later on during implementation
}

impl Default for App {
    fn default() -> App {
        App {
            process_status: Some(HeliumState::NotInitialized),
            tx_callback: None,
            rx_callback: None,
            app_cfg: None,
            app_write: None,
            app_read: None,
            pending_tx: None,
        }
    }
}

pub struct VirtualRadioDriver<'a, R>  
where
    R: radio_client::Radio,
{
    radio: &'a R,
    app: Grant<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    current_app: OptionalCell<AppId>,
    tx_client: OptionalCell<&'static radio_client::TxClient>,
    rx_client: OptionalCell<&'static radio_client::RxClient>,
}

impl<R> VirtualRadioDriver<'a, R> 
where 
    R: radio_client::Radio,
{
    pub fn new(
        radio: &'a R,
        container: Grant<App>,
        tx_buf: &'static mut [u8],
    ) -> VirtualRadioDriver<'a, R> 
    {   
        VirtualRadioDriver {
            radio: radio,
            app: container,
            kernel_tx: TakeCell::new(tx_buf),
            current_app: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
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

}

impl<R> Driver for VirtualRadioDriver<'a, R> 
where
    R: radio_client::Radio,
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
        match subscribe_num {
            0 => self.do_with_app(app_id, |app| {
                app.rx_callback = callback;
                ReturnCode::SUCCESS
            }),
            1 => self.do_with_app(app_id, |app| {
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
    /// - `1`: Power on radio
    /// - `2`: Return radio status. SUCCESS/EOFF = on/off.

    fn command(&self, command_num: usize, _r2: usize, _r3: usize, _appid: AppId) -> ReturnCode {
        match command_num {
            // Handle callback for CMDSTA after write to CMDR
            0 => ReturnCode::SUCCESS,
            1 => {
                let status = self.radio.initialize();
                match status {
                    ReturnCode::SUCCESS => ReturnCode::SUCCESS,
                    _ => ReturnCode::FAIL,
                }
            }
            2 => {
                if self.radio.is_on() {
                    ReturnCode::SUCCESS
                }
                else {
                    ReturnCode::EOFF
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<R: radio_client::Radio> radio_client::TxClient for VirtualRadioDriver<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.tx_client.map(move |c| {
            c.transmit_event(buf, result);
        });
    }
}

impl<R: radio_client::Radio> radio_client::RxClient for VirtualRadioDriver<'a, R> {
    fn receive_event(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        result: ReturnCode,
    ) {
        // Filter packets by destination because radio is in promiscuous mode
        let addr_match = false;
        // CHECK IF THE RECEIVE PACKET DECAUT AND DECODE IS OK HERE 

        if addr_match {
            self.rx_client.map(move |c| {
                c.receive_event(buf, frame_len, crc_valid, result);
            });
        } else {
            self.radio.set_receive_buffer(buf);
        }
    }
}

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
