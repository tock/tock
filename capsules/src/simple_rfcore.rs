#![allow(unused)]
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::rfcore;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

// Syscall number
pub const DRIVER_NUM: usize = 0xCC_13_12;

#[derive(Debug, Clone, Copy)]
pub enum HeliumState {
    NotInitialized,
    Idle,
    Tx(TxState),
    Rx(RxState),
    PendingCommand,
    PendingTx(rfcore::RadioOperation),
    Done,
    Invalid,
}

#[derive(Debug, Copy, Clone)]
pub enum TxState {
    TxReady,
    TxPending,
}

#[derive(Debug, Copy, Clone)]
pub enum RxState {
    RxReady,
    RxPending,
}

pub struct App {
    process_status: Option<HeliumState>,
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_send: Option<AppSlice<Shared, u8>>,
    app_receive: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<(u16, Option<u32>)>, // Change u32 to keyid and fec mode later on during implementation
}

impl App {
    fn attempt_transmit<'a, R>(&self, virtual_radio: &VirtualRadioDriver<'a, R>) -> ReturnCode
    where
        R: rfcore::Radio,
    {
        self.app_send
            .as_ref()
            .map(|app_data| {
                virtual_radio
                    .kernel_tx
                    .take()
                    .map(|ktx| {
                        let app_data_len = app_data.len();
                        let app_data_ref = &app_data.as_ref()[..app_data_len];
                        ktx[..app_data_len].copy_from_slice(app_data_ref);
                        let (result, rbuf) = virtual_radio.radio.transmit(ktx, app_data_len);
                        rbuf.map(|r| virtual_radio.kernel_tx.replace(r));
                        result
                    }).unwrap_or(ReturnCode::FAIL)
            }).unwrap_or(ReturnCode::FAIL)
    }
}

impl Default for App {
    fn default() -> App {
        App {
            process_status: Some(HeliumState::NotInitialized),
            tx_callback: None,
            rx_callback: None,
            app_send: None,
            app_receive: None,
            pending_tx: None,
        }
    }
}

pub struct VirtualRadioDriver<'a, R>
where
    R: rfcore::Radio,
{
    radio: &'a R,
    app: Grant<App>,
    kernel_tx: TakeCell<'static, [u8]>,
    current_app: OptionalCell<AppId>,
    frequency: Cell<u16>,
}

impl<R> VirtualRadioDriver<'a, R>
where
    R: rfcore::Radio,
{
    pub fn new(
        radio: &'a R,
        container: Grant<App>,
        tx_buf: &'static mut [u8],
    ) -> VirtualRadioDriver<'a, R> {
        VirtualRadioDriver {
            radio: radio,
            app: container,
            kernel_tx: TakeCell::new(tx_buf),
            current_app: OptionalCell::empty(),
            frequency: Cell::new(0x0393),
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

    fn parse_incoming_rx(&self, buf: &'static mut [u8], len: usize) -> HeliumState {
        // Do decoding header after decauterize here and other things if needed, for now assuming
        // the whole word is a single packet and if crc and decaut valid, its ok
        HeliumState::Idle
    }
}

impl<R> Driver for VirtualRadioDriver<'a, R>
where
    R: rfcore::Radio,
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
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            0 | 1 => self.do_with_app(appid, |app| {
                match allow_num {
                    0 => app.app_send = slice,
                    1 => app.app_receive = slice,
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
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
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
    /// - `2`: Power down radio
    /// - `3`: Return radio status. SUCCESS/EOFF = on/off.
    /// - `4`: Set radio TX power (post radio setup)
    /// - `5`: "Gracefull" stop radio operation command
    /// - `6`: Get radio command status
    /// - `7`: Force stop radio operation (no powerdown)
    /// - `8`: Set next TX transaction
    fn command(&self, command_num: usize, data: usize, _r3: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            1 => {
                self.radio.initialize();
                ReturnCode::SUCCESS
            }
            2 => {
                let status = self.radio.stop();
                match status {
                    ReturnCode::SUCCESS => ReturnCode::SUCCESS,
                    _ => ReturnCode::FAIL,
                }
            }
            3 => {
                if self.radio.is_on() {
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EOFF
                }
            }
            4 => {
                let status = self.radio.set_tx_power(data as u16);
                match status {
                    ReturnCode::SUCCESS => ReturnCode::SUCCESS,
                    _ => ReturnCode::FAIL,
                }
            }
            5 => {
                let status = self.radio.send_stop_command();
                match status {
                    ReturnCode::SUCCESS => ReturnCode::SUCCESS,
                    _ => ReturnCode::FAIL,
                }
            }
            6 => {
                // TODO Parsing with the returned Option<retval> which is some u32 hex code the
                // radio responds with during radio operation command processing
                let (status, _retval) = self.radio.get_command_status();
                status
            }
            7 => self.radio.send_kill_command(),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<R: rfcore::Radio> rfcore::TxClient for VirtualRadioDriver<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.current_app.map(|appid| {
            let _ = self.app.enter(*appid, |app, _| {
                match app.process_status {
                    // Need to arbitrate between Tx mode and Rx mode here
                    Some(HeliumState::Idle) => {
                        app.process_status = Some(HeliumState::Tx(TxState::TxPending));
                        self.current_app.set(app.appid());
                        self.radio.initialize();
                        self.radio.set_frequency(self.frequency.get());
                        app.attempt_transmit(&self);
                    }

                    Some(HeliumState::Tx(TxState::TxReady)) => {
                        app.process_status = Some(HeliumState::Tx(TxState::TxPending));
                        self.current_app.set(app.appid());
                        self.radio.set_frequency(self.frequency.get());
                        app.attempt_transmit(&self);
                    }

                    Some(HeliumState::Tx(TxState::TxPending)) => {
                        app.attempt_transmit(&self);
                    }
                    _ => (),
                }
            });
        });
    }
}

impl<R: rfcore::Radio> rfcore::RxClient for VirtualRadioDriver<'a, R> {
    fn receive_event(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        result: ReturnCode,
    ) {
        self.current_app.map(|appid| {
            let _ = self.app.enter(*appid, move |app, _| {
                // No CRC, drop
                if !crc_valid {
                    self.radio.set_receive_buffer(buf);
                    return;
                }

                let decaut_valid = true;
                // CHECK IF THE RECEIVE PACKET DECAUT AND DECODE IS OK HERE
                if !decaut_valid {
                    self.radio.set_receive_buffer(buf);
                    return;
                }

                match app.process_status {
                    // Need to arbitrate between config in Tx mode or Rx mode here
                    Some(HeliumState::Idle) => {
                        app.process_status = Some(HeliumState::Rx(RxState::RxReady));
                        self.current_app.set(app.appid());
                        self.radio.set_frequency(self.frequency.get());
                    }
                    Some(HeliumState::Rx(RxState::RxReady)) => {
                        let next_status = self.parse_incoming_rx(buf, frame_len);
                        app.process_status = Some(next_status);
                    }
                    _ => (),
                }
            });
        });
    }
}
