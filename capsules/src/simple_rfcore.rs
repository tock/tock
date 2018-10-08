use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::radio_client;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

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

pub trait Framer {
    fn prepare_data_frame(&self, buf: &'static mut [u8], _seq: u8) -> Result<Frame, &'static mut [u8]>;
}

impl<R> Framer for VirtualRadioDriver<'a, R>
where
    R: radio_client::Radio,
{
    fn prepare_data_frame(&self, buf: &'static mut [u8], seq: u8) -> Result<Frame, &'static mut [u8]> {
        let _header = Header {
            frame_type: FrameType::Data,
            id: None,
            seq: Some(seq),
        };

        // encode header here and return some result
        let frame = Frame {
            buf: buf,
            info: FrameInfo {
                frame_type: FrameType::Data,
                data_len: 0,
            },
            max_frame_size: 128,
        };
        Ok(frame)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Frame {
    buf: &'static mut [u8],
    info: FrameInfo,
    max_frame_size: usize,
}

impl Frame {
    pub fn into_buf(self) -> &'static mut [u8] {
        self.buf
    }

    pub fn append_payload(&mut self, payload: &[u8]) -> ReturnCode {
        if payload.len() > self.max_frame_size {
            return ReturnCode::ENOMEM;
        }
        self.buf.copy_from_slice(payload);
        self.info.data_len += payload.len();
        
        ReturnCode::SUCCESS
    }
}

pub struct Header {
    frame_type: FrameType,
    id: Option<u32>,
    seq: Option<u8>,
}

pub const FRAME_TYPE_MASK: u8 = 0b11;
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FrameType {
    Ping = 0b00,
    Data = 0b01,
    Fragment = 0b10,
    Acknowledgement = 0b11,
}

#[derive(Eq, PartialEq, Debug)]
pub struct FrameInfo {
    frame_type: FrameType,
    data_len: usize,
}

impl FrameType {
    pub fn from_slice(ft: u8) -> Option<FrameType> {
        match ft & FRAME_TYPE_MASK {
            0b00 => Some(FrameType::Ping),
            0b01 => Some(FrameType::Data),
            0b10 => Some(FrameType::Fragment),
            0b11 => Some(FrameType::Acknowledgement),
            _ => None,
        }
    }
}

// #[derive(Default)]
pub struct App {
    process_status: Option<HeliumState>,
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_cfg: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    app_read: Option<AppSlice<Shared, u8>>,
    pending_tx: Option<(u16, Option<FrameType>)>, // Change u32 to keyid and fec mode later on during implementation
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
    ) -> VirtualRadioDriver<'a, R> {
        VirtualRadioDriver {
            radio: radio,
            app: container,
            kernel_tx: TakeCell::new(tx_buf),
            current_app: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
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
                let seq: u8 = 0; // TEMP SEQ # ALWAYS 0 
                let frame = match self.prepare_data_frame(
                    kbuf,
                    seq
                    ) {
                    Ok(frame) => frame,
                    Err(kbuf) => {
                        self.kernel_tx.replace(kbuf);
                        return ReturnCode::FAIL;
                    }
                };
                // Transmit the framei
                let len = frame.info.data_len;
                let (result, mbuf) = self.radio.transmit(frame.into_buf(), len);
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
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
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

    fn command(&self, command_num: usize, r2: usize, _r3: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            1 => {
                let status = self.radio.initialize();
                match status {
                    ReturnCode::SUCCESS => ReturnCode::SUCCESS,
                    _ => ReturnCode::FAIL,
                }
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
                let status = self.radio.set_tx_power(r2 as u16);
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
            7 => {
                self.radio.send_kill_command()
            }
            8 => {
                self.do_with_app(appid, |app| {
                    if app.pending_tx.is_some() {
                        return ReturnCode::EBUSY;
                    }
                    let addr = r2 as u16;

                    let next_tx = app.app_cfg.as_ref().and_then(|cfg| {
                        if cfg.len() != 11 {
                            return None;
                        }

                        let frame_type = match FrameType::from_slice(cfg.as_ref()[0]) {
                            Some(frame_type) => frame_type,
                            None => {return None;} 
                        };
                        Some((addr, Some(frame_type))) 
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

impl<R: radio_client::Radio> radio_client::TxClient for VirtualRadioDriver<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        /*
        self.kernel_tx.replace(buf);
        self.current_app.take().map(|app_id| {
            let _ = self.app.enter(app_id, |app, _| {
                app.tx_callback
                    .take()
                    .map(|mut cb| cb.schedule(result.into(), 0, 0));
            });
        });
        */
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
        let decaut_valid = true;
        // CHECK IF THE RECEIVE PACKET DECAUT AND DECODE IS OK HERE
        if decaut_valid { 
            self.rx_client.map(move |c| {
                c.receive_event(buf, frame_len, crc_valid, result);
            });
        } else {
            self.radio.set_receive_buffer(buf);
        }
    }
}
