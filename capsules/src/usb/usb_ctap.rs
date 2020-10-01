use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall number
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::UsbCtap as usize;

pub const CTAP_CMD_CHECK: usize = 0;
pub const CTAP_CMD_CONNECT: usize = 1;
pub const CTAP_CMD_TRANSMIT: usize = 2;
pub const CTAP_CMD_RECEIVE: usize = 3;
pub const CTAP_CMD_TRANSMIT_OR_RECEIVE: usize = 4;
pub const CTAP_CMD_CANCEL: usize = 5;

pub const CTAP_ALLOW_TRANSMIT: usize = 1;
pub const CTAP_ALLOW_RECEIVE: usize = 2;
pub const CTAP_ALLOW_TRANSMIT_OR_RECEIVE: usize = 3;

pub const CTAP_SUBSCRIBE_TRANSMIT: usize = 1;
pub const CTAP_SUBSCRIBE_RECEIVE: usize = 2;
pub const CTAP_SUBSCRIBE_TRANSMIT_OR_RECEIVE: usize = 3;

pub const CTAP_CALLBACK_TRANSMITED: usize = 1;
pub const CTAP_CALLBACK_RECEIVED: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Transmit,
    Receive,
    TransmitOrReceive,
}

impl Side {
    fn can_transmit(&self) -> bool {
        match self {
            Side::Transmit | Side::TransmitOrReceive => true,
            Side::Receive => false,
        }
    }

    fn can_receive(&self) -> bool {
        match self {
            Side::Receive | Side::TransmitOrReceive => true,
            Side::Transmit => false,
        }
    }
}

#[derive(Default)]
pub struct App {
    // Only one app can be connected to this driver, to avoid needing to route packets among apps.
    // This field tracks this status.
    connected: bool,
    // Currently enabled transaction side. Subscribing to a callback or allowing a buffer
    // automatically sets the corresponding side. Clearing both the callback and the buffer resets
    // the side to None.
    side: Option<Side>,
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,
    // Whether the app is waiting for the kernel signaling a packet transfer.
    waiting: bool,
}

impl App {
    fn check_side(&mut self) {
        if self.callback.is_none() && self.buffer.is_none() && !self.waiting {
            self.side = None;
        }
    }

    fn set_side(&mut self, side: Side) -> bool {
        match self.side {
            None => {
                self.side = Some(side);
                true
            }
            Some(app_side) => side == app_side,
        }
    }

    fn is_ready_for_command(&self, side: Side) -> bool {
        self.buffer.is_some() && self.callback.is_some() && self.side == Some(side)
    }
}

pub struct CtapUsbSyscallDriver<'a> {
    usb_hid: &'a dyn hil::usb_hid::UsbHid<'a>,
    apps: Grant<App>,
    // Buffers to pass to the USB-HID HIL
    send_buffer: TakeCell<'static, [u8; 64]>,
    recv_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a> CtapUsbSyscallDriver<'a> {
    pub fn new(
        usb_hid: &'a dyn hil::usb_hid::UsbHid<'a>,
        apps: Grant<App>,
        send_buffer: &'static mut [u8; 64],
        recv_buffer: &'static mut [u8; 64],
    ) -> Self {
        CtapUsbSyscallDriver {
            usb_hid,
            apps,
            send_buffer: TakeCell::new(send_buffer),
            recv_buffer: TakeCell::new(recv_buffer),
        }
    }
}

impl<'a> hil::usb_hid::Client<'a> for CtapUsbSyscallDriver<'a> {
    fn can_receive(&'a self) -> bool {
        let mut result = false;
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if app.connected {
                    result = app.waiting
                        && app.side.map_or(false, |side| side.can_receive())
                        && app.buffer.is_some();
                }
            });
        }
        result
    }

    fn packet_received(
        &self,
        result: ReturnCode,
        buffer: &'static mut [u8; 64],
        len: usize,
        endpoint: usize,
    ) {
        assert_eq!(result, ReturnCode::SUCCESS);
        assert_eq!(len, 64);
        assert_eq!(endpoint, 1);
        let packet: &[u8; 64] = &*buffer;
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if app.connected && app.waiting && app.side.map_or(false, |side| side.can_receive())
                {
                    if let Some(buf) = &mut app.buffer {
                        // Copy the packet to the app's allowed buffer.
                        buf.as_mut().copy_from_slice(packet);
                        app.waiting = false;
                        // Signal to the app that a packet is ready.
                        app.callback
                            .map(|mut cb| cb.schedule(CTAP_CALLBACK_RECEIVED, 0, 0));
                    }
                }
            });
        }
        assert!(self.recv_buffer.is_none());
        self.recv_buffer.replace(buffer);
    }

    fn packet_transmitted(
        &self,
        result: ReturnCode,
        buffer: &'static mut [u8; 64],
        len: usize,
        endpoint: usize,
    ) {
        assert_eq!(result, ReturnCode::SUCCESS);
        assert_eq!(len, 64);
        assert_eq!(endpoint, 1);
        let previous = self.send_buffer.replace(buffer);
        assert!(previous.is_none());
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if app.connected
                    && app.waiting
                    && app.side.map_or(false, |side| side.can_transmit())
                {
                    app.waiting = false;
                    // Signal to the app that the packet was sent.
                    app.callback
                        .map(|mut cb| cb.schedule(CTAP_CALLBACK_TRANSMITED, 0, 0));
                }
            });
        }
    }
}

impl<'a> Driver for CtapUsbSyscallDriver<'a> {
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        let side = match allow_num {
            CTAP_ALLOW_TRANSMIT => Side::Transmit,
            CTAP_ALLOW_RECEIVE => Side::Receive,
            CTAP_ALLOW_TRANSMIT_OR_RECEIVE => Side::TransmitOrReceive,
            _ => return ReturnCode::ENOSUPPORT,
        };
        self.apps
            .enter(appid, |app, _| {
                if !app.connected {
                    ReturnCode::ERESERVE
                } else {
                    if let Some(buf) = &slice {
                        if buf.len() != 64 {
                            return ReturnCode::EINVAL;
                        }
                    }
                    if !app.set_side(side) {
                        return ReturnCode::EALREADY;
                    }
                    app.buffer = slice;
                    app.check_side();
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        appid: AppId,
    ) -> ReturnCode {
        let side = match subscribe_num {
            CTAP_SUBSCRIBE_TRANSMIT => Side::Transmit,
            CTAP_SUBSCRIBE_RECEIVE => Side::Receive,
            CTAP_SUBSCRIBE_TRANSMIT_OR_RECEIVE => Side::TransmitOrReceive,
            _ => return ReturnCode::ENOSUPPORT,
        };
        self.apps
            .enter(appid, |app, _| {
                if !app.connected {
                    ReturnCode::ERESERVE
                } else {
                    if !app.set_side(side) {
                        return ReturnCode::EALREADY;
                    }
                    app.callback = callback;
                    app.check_side();
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn command(&self, cmd_num: usize, _arg1: usize, _arg2: usize, appid: AppId) -> ReturnCode {
        match cmd_num {
            CTAP_CMD_CHECK => ReturnCode::SUCCESS,
            CTAP_CMD_CONNECT => {
                // First, check if any app is already connected to this driver.
                let mut busy = false;
                for app in self.apps.iter() {
                    app.enter(|app, _| {
                        busy |= app.connected;
                    });
                }

                self.apps
                    .enter(appid, |app, _| {
                        if app.connected {
                            ReturnCode::EALREADY
                        } else if busy {
                            ReturnCode::EBUSY
                        } else {
                            app.connected = true;
                            ReturnCode::SUCCESS
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }
            CTAP_CMD_TRANSMIT => self
                .apps
                .enter(appid, |app, _| {
                    if !app.connected {
                        ReturnCode::ERESERVE
                    } else {
                        if app.is_ready_for_command(Side::Transmit) {
                            if app.waiting {
                                ReturnCode::EALREADY
                            } else {
                                assert!(self.send_buffer.is_some());
                                let send_buffer: &'static mut [u8; 64] =
                                    self.send_buffer.take().unwrap();
                                send_buffer.copy_from_slice(app.buffer.as_ref().unwrap().as_ref());
                                match self.usb_hid.send_buffer(send_buffer) {
                                    Ok(n) => {
                                        assert_eq!(n, 64);
                                        app.waiting = true;
                                        ReturnCode::SUCCESS
                                    }
                                    Err((_code, send_buffer)) => {
                                        self.send_buffer.replace(send_buffer);
                                        ReturnCode::EBUSY
                                    }
                                }
                            }
                        } else {
                            ReturnCode::EINVAL
                        }
                    }
                })
                .unwrap_or_else(|err| err.into()),
            CTAP_CMD_RECEIVE => self
                .apps
                .enter(appid, |app, _| {
                    if !app.connected {
                        ReturnCode::ERESERVE
                    } else {
                        if app.is_ready_for_command(Side::Receive) {
                            if app.waiting {
                                ReturnCode::EALREADY
                            } else {
                                app.waiting = true;
                                assert!(self.recv_buffer.is_some());
                                let recv_buffer: &'static mut [u8; 64] =
                                    self.recv_buffer.take().unwrap();
                                let result = self.usb_hid.receive_buffer(recv_buffer);
                                assert!(result.is_ok());
                                ReturnCode::SUCCESS
                            }
                        } else {
                            ReturnCode::EINVAL
                        }
                    }
                })
                .unwrap_or_else(|err| err.into()),
            CTAP_CMD_TRANSMIT_OR_RECEIVE => self
                .apps
                .enter(appid, |app, _| {
                    if !app.connected {
                        ReturnCode::ERESERVE
                    } else {
                        if app.is_ready_for_command(Side::TransmitOrReceive) {
                            if app.waiting {
                                ReturnCode::EALREADY
                            } else {
                                // Indicates to the driver that we can receive any pending packet.
                                app.waiting = true;
                                assert!(self.recv_buffer.is_some());
                                let recv_buffer: &'static mut [u8; 64] =
                                    self.recv_buffer.take().unwrap();
                                let result = self.usb_hid.receive_buffer(recv_buffer);
                                assert!(result.is_ok());

                                if !app.waiting {
                                    // The call to receive_packet() collected a pending packet.
                                    ReturnCode::SUCCESS
                                } else {
                                    // Indicates to the driver that we have a packet to send.
                                    assert!(self.send_buffer.is_some());
                                    let send_buffer: &'static mut [u8; 64] =
                                        self.send_buffer.take().unwrap();
                                    send_buffer
                                        .copy_from_slice(app.buffer.as_ref().unwrap().as_ref());
                                    match self.usb_hid.send_buffer(send_buffer) {
                                        Ok(n) => {
                                            assert_eq!(n, 64);
                                            ReturnCode::SUCCESS
                                        }
                                        Err((_code, send_buffer)) => {
                                            self.send_buffer.replace(send_buffer);
                                            ReturnCode::EBUSY
                                        }
                                    }
                                }
                            }
                        } else {
                            ReturnCode::EINVAL
                        }
                    }
                })
                .unwrap_or_else(|err| err.into()),
            CTAP_CMD_CANCEL => self
                .apps
                .enter(appid, |app, _| {
                    if !app.connected {
                        ReturnCode::ERESERVE
                    } else {
                        if app.waiting {
                            // FIXME: if cancellation failed, the app should still wait. But that
                            // doesn't work yet.
                            app.waiting = false;
                            assert!(self.recv_buffer.is_none() || self.send_buffer.is_none());

                            // Cancel receiving.
                            let recv_result = if self.recv_buffer.is_none() {
                                match self.usb_hid.receive_cancel() {
                                    Ok(recv_buffer) => {
                                        self.recv_buffer.replace(recv_buffer);
                                        ReturnCode::SUCCESS
                                    }
                                    Err(code) => {
                                        assert_eq!(code, ReturnCode::EBUSY);
                                        ReturnCode::EBUSY
                                    }
                                }
                            } else {
                                ReturnCode::SUCCESS
                            };

                            // Cancel sending.
                            let send_result = if self.send_buffer.is_none() {
                                match self.usb_hid.send_cancel() {
                                    Ok(send_buffer) => {
                                        self.send_buffer.replace(send_buffer);
                                        ReturnCode::SUCCESS
                                    }
                                    Err(code) => {
                                        assert_eq!(code, ReturnCode::EBUSY);
                                        ReturnCode::EBUSY
                                    }
                                }
                            } else {
                                ReturnCode::SUCCESS
                            };

                            match (recv_result, send_result) {
                                (ReturnCode::SUCCESS, ReturnCode::SUCCESS) => ReturnCode::SUCCESS,
                                // Cannot cancel now because the transaction is already in process.
                                // The app should wait for the callback instead.
                                (ReturnCode::SUCCESS, ReturnCode::EBUSY)
                                | (ReturnCode::EBUSY, ReturnCode::SUCCESS)
                                | (ReturnCode::EBUSY, ReturnCode::EBUSY) => ReturnCode::EBUSY,
                                _ => unreachable!(),
                            }
                        } else {
                            ReturnCode::EALREADY
                        }
                    }
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
