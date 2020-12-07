//! TODO

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::KVStore as usize;

use core::cell::Cell;
use core::hash::SipHasher;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::flash::{self, Client, Flash};
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};
use tickfs;
use tickfs::AsyncTickFS;

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    Init,
    GetKey,
}

pub struct TickFSFlastCtrl<'a, F: Flash + 'static> {
    flash: &'a F,
    flash_read_buffer: TakeCell<'static, F::Page>,
    region_offset: usize,
}

impl<'a, F: Flash> TickFSFlastCtrl<'a, F> {
    pub fn new(
        flash: &'a F,
        flash_read_buffer: &'static mut F::Page,
        region_offset: usize,
    ) -> TickFSFlastCtrl<'a, F> {
        Self {
            flash,
            flash_read_buffer: TakeCell::new(flash_read_buffer),
            region_offset,
        }
    }
}

impl<'a, F: Flash> tickfs::flash_controller::FlashController<512> for TickFSFlastCtrl<'a, F> {
    fn read_region(
        &self,
        region_number: usize,
        _offset: usize,
        _buf: &mut [u8; 512],
    ) -> Result<(), tickfs::error_codes::ErrorCode> {
        if self
            .flash
            .read_page(
                self.region_offset + region_number,
                self.flash_read_buffer.take().unwrap(),
            )
            .is_err()
        {
            Err(tickfs::error_codes::ErrorCode::ReadFail)
        } else {
            Err(tickfs::error_codes::ErrorCode::ReadNotReady(region_number))
        }
    }

    fn write(&self, address: usize, buf: &[u8]) -> Result<(), tickfs::error_codes::ErrorCode> {
        let data_buf = self.flash_read_buffer.take().unwrap();

        for (i, d) in buf.iter().enumerate() {
            data_buf.as_mut()[i] = *d;
        }

        if self
            .flash
            .write_page((0x20040000 + address) / 512, data_buf)
            .is_err()
        {
            return Err(tickfs::error_codes::ErrorCode::WriteFail);
        }

        Err(tickfs::error_codes::ErrorCode::WriteNotReady(address))
    }

    fn erase_region(&self, region_number: usize) -> Result<(), tickfs::error_codes::ErrorCode> {
        self.flash.erase_page(self.region_offset + region_number);

        Err(tickfs::error_codes::ErrorCode::EraseNotReady(region_number))
    }
}

pub struct KVStoreDriver<'a, F: Flash + 'static> {
    tickfs: AsyncTickFS<'a, TickFSFlastCtrl<'a, F>, SipHasher, 512>,
    apps: Grant<App>,
    appid: OptionalCell<AppId>,
    operation: Cell<Operation>,
    static_key_buf: TakeCell<'static, [u8]>,
    static_value_buf: TakeCell<'static, [u8]>,
}

impl<'a, F: Flash> KVStoreDriver<'a, F> {
    pub fn new(
        flash: &'a F,
        grant: Grant<App>,
        tickfs_read_buf: &'static mut [u8; 512],
        flash_read_buffer: &'static mut F::Page,
        region_offset: usize,
        static_key_buf: &'static mut [u8; 64],
        static_value_buf: &'static mut [u8; 64],
    ) -> KVStoreDriver<'a, F> {
        let tickfs = AsyncTickFS::<TickFSFlastCtrl<F>, SipHasher, 512>::new(
            TickFSFlastCtrl::new(flash, flash_read_buffer, region_offset),
            tickfs_read_buf,
            tickfs_read_buf.len(),
        );

        Self {
            tickfs,
            apps: grant,
            appid: OptionalCell::empty(),
            operation: Cell::new(Operation::None),
            static_key_buf: TakeCell::new(static_key_buf),
            static_value_buf: TakeCell::new(static_value_buf),
        }
    }

    pub fn initalise(&self) {
        let _ret = self
            .tickfs
            .initalise((&mut SipHasher::new(), &mut SipHasher::new()));
        self.operation.set(Operation::Init);
    }
}

impl<'a, F: Flash> Client<F> for KVStoreDriver<'a, F> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: flash::Error) {
        self.tickfs.set_read_buffer(pagebuffer.as_mut());
        self.tickfs
            .tickfs
            .controller
            .flash_read_buffer
            .replace(pagebuffer);
        let (ret, _buf_buffer) = self
            .tickfs
            .continue_operation((&mut SipHasher::new(), &mut SipHasher::new()));

        match self.operation.get() {
            Operation::Init => {
                match ret {
                    Ok(tickfs::success_codes::SuccessCode::Complete)
                    | Ok(tickfs::success_codes::SuccessCode::Written) => {
                        self.operation.set(Operation::None)
                    }
                    _ => {}
                }
            }
            Operation::GetKey => {
                if ret.is_ok() {
                    self.appid.map(|id| {
                        self.apps
                            .enter(*id, |app, _| {
                                app.callback.map(|cb| {
                                    cb.schedule(0, 0, 0);
                                });
                            })
                            .unwrap();
                    });

                    self.operation.set(Operation::None);
                }
            }
            _ => unreachable!(),
        }
    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _error: flash::Error) {
        self.tickfs
            .tickfs
            .controller
            .flash_read_buffer
            .replace(pagebuffer);

        match self.operation.get() {
            Operation::Init => {
                self.operation.set(Operation::None);
            }
            _ => unreachable!(),
        }
    }

    fn erase_complete(&self, _error: flash::Error) {
        self.tickfs
            .continue_operation((&mut SipHasher::new(), &mut SipHasher::new()));
    }
}

impl<'a, F: Flash> Driver for KVStoreDriver<'a, F> {
    /// Specify memory regions to be used.
    ///
    /// ### `allow_num`
    ///
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Key buffer
            0 => self
                .apps
                .enter(appid, |app, _| {
                    app.key = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or(ReturnCode::FAIL),

            // Value buffer
            1 => self
                .apps
                .enter(appid, |app, _| {
                    app.value = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or(ReturnCode::FAIL),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Subscribe to events.
    ///
    /// ### `subscribe_num`
    ///
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        appid: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                // set callback
                self.apps
                    .enter(appid, |app, _| {
                        app.callback.insert(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or(ReturnCode::FAIL)
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Access the KV Store
    ///
    /// ### `command_num`
    ///
    fn command(
        &self,
        command_num: usize,
        key_len: usize,
        _data2: usize,
        appid: AppId,
    ) -> ReturnCode {
        match command_num {
            // Append key
            0 => ReturnCode::SUCCESS,

            // Get key
            1 => self
                .apps
                .enter(appid, |app, _| {
                    if let Some(key) = app.key.take() {
                        if let Some(value) = app.value.take() {
                            self.appid.set(appid);
                            if self.operation.get() != Operation::None {
                                panic!("Not ready");
                            }
                            self.operation.set(Operation::GetKey);
                            app.key_len.set(key_len);

                            let key_buf = self.static_key_buf.take().unwrap();
                            key_buf[0..key_len].copy_from_slice(&key.as_ref()[0..key_len]);

                            let value_buf = self.static_value_buf.take().unwrap();

                            let _ret = self.tickfs.get_key(
                                &mut SipHasher::new(),
                                &key_buf[0..key_len],
                                value_buf,
                            );

                            app.value.replace(value);
                            app.key.replace(key);

                            ReturnCode::SUCCESS
                        } else {
                            app.key.replace(key);
                            ReturnCode::EBUSY
                        }
                    } else {
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Invalidate ke
            2 => ReturnCode::SUCCESS,

            // Trigger garbage collection
            3 => ReturnCode::SUCCESS,

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

pub struct App {
    callback: OptionalCell<Callback>,
    _pending_run_app: Option<AppId>,
    key: Option<AppSlice<Shared, u8>>,
    key_len: OptionalCell<usize>,
    value: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: OptionalCell::empty(),
            _pending_run_app: None,
            key: None,
            key_len: OptionalCell::empty(),
            value: None,
        }
    }
}
