//! Provides userspace applications with the ability
//! to send and receive 802.15.4 packets.

// System call interface for sending and receiving 802.15.4 packets.
//
// Author: Philip Levis
// Date: Jan 12 2017
//

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, Driver, Callback, AppSlice, Shared};
use kernel::ReturnCode;
use kernel::common::take_cell::{MapCell, TakeCell};
use mac;
use net::ieee802154::{MacAddress, Header};

struct App {
    tx_callback: Option<Callback>,
    rx_callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            tx_callback: None,
            rx_callback: None,
            app_read: None,
            app_write: None,
        }
    }
}

pub struct RadioDriver<'a, M: mac::Mac + 'a> {
    mac: &'a M,
    busy: Cell<bool>,
    app: MapCell<App>,
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a, M: mac::Mac> RadioDriver<'a, M> {
    pub fn new(mac: &'a M) -> RadioDriver<'a, M> {
        RadioDriver {
            mac: mac,
            busy: Cell::new(false),
            app: MapCell::new(App::default()),
            kernel_tx: TakeCell::empty(),
        }
    }

    pub fn config_buffer(&mut self, tx_buf: &'static mut [u8]) {
        self.kernel_tx.replace(tx_buf);
    }
}

impl<'a, M: mac::Mac> Driver for RadioDriver<'a, M> {
    fn allow(&self, _appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                self.app.map(|app| app.app_read = Some(slice));
                ReturnCode::SUCCESS
            }
            1 => {
                self.app.map(|app| app.app_write = Some(slice));
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 /* transmit done*/  => {
                self.app.map(|app| app.tx_callback = Some(callback));
                ReturnCode::SUCCESS
            },
            1 /* receive */ => {
                self.app.map(|app| app.rx_callback = Some(callback));
                ReturnCode::SUCCESS
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    // 0: check if present
    // 1: set 16-bit address
    // 2: set PAN id
    // 3: set channel
    // 4: set tx power
    // 5: transmit packet

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* set 16-bit address */ => {
                self.mac.set_address(arg1 as u16);
                ReturnCode::SUCCESS
            },
            2 /* set PAN id */ => {
                self.mac.set_pan(arg1 as u16);
                ReturnCode::SUCCESS
            },
            3 /* set channel */ => {
                self.mac.set_channel(arg1 as u8)
            },
            4 /* set tx power */ => {
                let mut val = arg1 as i32;
                val = val - 128; // Library adds 128 to make unsigned
                self.mac.set_tx_power(val as i8)
            },
            5 /* tx packet */ => {
                // Don't transmit if we're busy, the radio is off, or
                // we don't have a buffer yet.
                if self.busy.get() {
                    return ReturnCode::EBUSY;
                } else if !self.mac.is_on() {
                    return ReturnCode::EOFF;
                } else if self.kernel_tx.is_none() {
                    return ReturnCode::ENOMEM;
                } else if self.app.is_none() {
                    return ReturnCode::ERESERVE;
                }

                // The argument packs the 16-bit destination address
                // and length in the 32-bit argument. Bits 0-15 are
                // the address and bits 16-23 are the length.
                let mut rval = ReturnCode::SUCCESS;
                self.app.map(|app| {
                    let mut blen = 0;
                    // If write buffer too small, return
                    app.app_write.as_mut().map(|w| {
                        blen = w.len();
                    });
                    let len: usize = (arg1 >> 16) & 0xff;
                    let addr: u16 = (arg1 & 0xffff) as u16;
                    if blen < len {
                        rval = ReturnCode::ESIZE;
                        return;
                    }

                    // Copy the packet into the kernel frame
                    let frame_info = self.kernel_tx.map_or(None, |tx_buf| {
                        // Prepare frame headers
                        let pan = self.mac.get_pan();
                        let src_addr = MacAddress::Short(self.mac.get_address());
                        let mut frame_info = match self.mac.prepare_data_frame(
                            tx_buf, pan, MacAddress::Short(addr), pan, src_addr,
                            None) {
                            Ok(info) => info,
                            Err(_) => {
                                rval = ReturnCode::FAIL;
                                return None;
                            }
                        };

                        // Copy the payload from userspace into kernelspace
                        app.app_write.as_mut().map(|src| {
                            rval = frame_info.append_payload(tx_buf,
                                                             &src.as_ref()[..len]);
                        });
                        Some(frame_info)
                    });
                    if rval != ReturnCode::SUCCESS {
                        return;
                    }

                    // Try to transmit the frame, otherwise at least get the
                    // frame back.
                    let res = self.mac.transmit(self.kernel_tx.take().unwrap(),
                                                frame_info.unwrap());
                    if let Some(tx_buf) = res.1 {
                        self.kernel_tx.replace(tx_buf);
                    }
                    rval = res.0;
                });
                if rval == ReturnCode::SUCCESS {
                    self.busy.set(true);
                }
                rval
            },
            6 /* check if on */ => {
                if self.mac.is_on() {
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::EOFF
                }
            }
            7 /* commit config */ => {
                self.mac.config_commit()
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, M: mac::Mac> mac::TxClient for RadioDriver<'a, M> {
    fn send_done(&self, tx_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.app.map(move |app| {
            self.kernel_tx.replace(tx_buf);
            self.busy.set(false);
            app.tx_callback
                .take()
                .map(|mut cb| { cb.schedule(usize::from(result), acked as usize, 0); });
        });
    }
}

impl<'a, M: mac::Mac> mac::RxClient for RadioDriver<'a, M> {
    fn receive<'b>(&self,
                   buf: &'b [u8],
                   /* We ignore the header because we pass the entire frame to
                    * userspace */
                   _: Header<'b>,
                   data_offset: usize,
                   data_len: usize,
                   result: ReturnCode) {
        if self.app.is_some() {
            self.app.map(move |app| if app.app_read.is_some() {
                let dest = app.app_read.as_mut().unwrap();
                let d = &mut dest.as_mut();
                let len = cmp::min(d.len(), data_offset + data_len);
                d[..len].copy_from_slice(&buf[..len]);
                app.rx_callback
                    .take()
                    .map(|mut cb| {
                        cb.schedule(usize::from(result), data_offset, data_offset + data_len);
                    });
            });
        }
    }
}
