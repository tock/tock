//! Provides userspace applications with the ability to communicate over the
//! LoRa network.

use core::cell::Cell;
use core::mem;
use kernel::{
    common::cells::{OptionalCell, TakeCell},
    debug,
    hil::lmic::LMIC,
};
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId, Upcall};
use kernel::{Read, ReadOnlyAppSlice, ReadWriteAppSlice};

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Lora as usize;

pub const MAX_LORA_PACKET_SIZE: usize = 256;

// LoRa transmissions are handled by copying desired tx data into a kernel buffer
// LoRa receptions are handled by copying out of a kernel buffer

// TODO: Handle if application buffer is larger than kernel buffer?
#[derive(Default)]
pub struct App {
    upcall: Upcall,
    app_read: ReadWriteAppSlice,
    app_write: ReadOnlyAppSlice,
    len: u8,
}

pub struct Lora<'a, L: LMIC> {
    lora_device: &'a L,
    busy: Cell<bool>,
    // kernel_len: Cell<usize>, TODO: do we need this?
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    grants: Grant<App>,
    current_process: OptionalCell<ProcessId>,
}

impl<'a, L: LMIC> Lora<'a, L> {
    pub fn new(lora_device: &'a L, grants: Grant<App>) -> Lora<'a, L> {
        Lora {
            lora_device,
            busy: Cell::new(false),
            kernel_read: TakeCell::empty(),
            kernel_write: TakeCell::empty(),
            grants,
            current_process: OptionalCell::empty(),
        }
    }

    pub fn config_buffers(&mut self, read: &'static mut [u8], write: &'static mut [u8]) {
        self.kernel_read.replace(read);
        self.kernel_write.replace(write);
    }

    // Assumes checks for busy already done.
    fn do_set_tx_data(&self, app: &mut App) {
        // TODO: Also having dummy return values is kind of wack. Is there a better
        // way to do this?
        self.kernel_write.map_or(0, |kernel_write_buf| {
            app.app_write.map_or(0, |src| {
                let end = app.len as usize; // NOTE: will silently overflow since app.len is u8

                for (i, c) in src.as_ref()[..end].iter().enumerate() {
                    kernel_write_buf[i] = *c;
                }
                0 // Dummy return
            });
            0 // Dummy return
        });
        debug!("do_set_tx_data");
        let _ = self
            .lora_device
            .set_tx_data(self.kernel_write.take().unwrap(), app.len);
        self.busy.set(false);
        // TODO: At some point, need to kernel_write.replace(some_buf); otherwise, kernel_write
        // will remain None
    }
}

impl<'a, L: LMIC> Driver for Lora<'a, L> {
    fn allow_readonly(
        &self,
        app: ProcessId,
        which: usize,
        mut slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        match which {
            0 => {
                let _ = self.grants.enter(app, |grant| {
                    mem::swap(&mut grant.app_write, &mut slice);
                });
                Ok(slice)
            }
            _ => Err((slice, ErrorCode::NOSUPPORT)),
        }
    }

    fn allow_readwrite(
        &self,
        app: ProcessId,
        which: usize,
        mut slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        match which {
            0 => {
                let _ = self.grants.enter(app, |grant| {
                    mem::swap(&mut grant.app_read, &mut slice);
                });
                Ok(slice)
            }
            _ => Err((slice, ErrorCode::NOSUPPORT)),
        }
    }

    fn subscribe(
        &self,
        subscribe_identifier: usize,
        mut upcall: Upcall,
        app_id: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        match subscribe_identifier {
            0 => {
                let _ = self.grants.enter(app_id, |grant| {
                    mem::swap(&mut grant.upcall, &mut upcall);
                });
                Ok(upcall)
            }
            _ => Err((upcall, ErrorCode::NOSUPPORT)),
        }
    }

    // 1: set tx data to transmit over LoRa network
    //   - requires app write buffer registered with allow
    fn command(&self, which: usize, r2: usize, _r3: usize, caller_id: ProcessId) -> CommandReturn {
        if which == 0 {
            // Handle this first as it should be returned unconditionally.
            return CommandReturn::success();
        }

        // Check if this driver is free, or alread dedicated to this process
        let empty_or_nonexistent_process = self.current_process.map_or(true, |current_process| {
            self.grants
                .enter(*current_process, |_| current_process == &caller_id)
                .unwrap_or(true)
        });
        if empty_or_nonexistent_process {
            self.current_process.set(caller_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match which {
            1 /* set_tx_data */ => {
                if self.busy.get() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.grants.enter(caller_id, |app| {
                    let app_write_len = app.app_write.map_or(0, |w| w.len());

                    if app_write_len >= r2 && r2 > 0 {
                        app.len = r2 as u8; // NOTE: will silently overflow
                        self.busy.set(true);
                        self.do_set_tx_data(app);
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }

                }).unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }
}
