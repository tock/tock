use common::take_cell::TakeCell;
use process::{AppId, AppSlice, Container, Callback, Shared};
use hil::Driver;
use hil::uart::{UART, Client};

pub struct App {
    read_callback: Option<Callback>,
    write_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    pending_write: bool,
    read_idx: usize
}

impl Default for App {
    fn default() -> App {
        App {
            read_callback: None,
            write_callback: None,
            read_buffer: None,
            write_buffer: None,
            write_len: 0,
            pending_write: false,
            read_idx: 0
        }
    }
}

pub static mut WRITE_BUF : [u8; 64] = [0; 64];

pub struct Console<'a, U: UART + 'a> {
    uart: &'a U,
    apps: Container<App>,
    in_progress: TakeCell<AppId>,
    buffer: TakeCell<&'static mut [u8]>
}

impl<'a, U: UART> Console<'a, U> {
    pub fn new(uart: &'a U, buffer: &'static mut [u8],
                     container: Container<App>) -> Console<'a, U> {
        Console {
            uart: uart,
            apps: container,
            in_progress: TakeCell::empty(),
            buffer: TakeCell::new(buffer)
        }
    }

    pub fn initialize(&self) {
        self.uart.enable_tx();
        self.uart.enable_rx();
    }
}

impl<'a, U: UART> Driver for Console<'a, U> {
    fn allow(&self, appid: AppId,
             allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_num {
            0 => {
                self.apps.enter(appid, |app, _| {
                    app.read_buffer = Some(slice);
                    app.read_idx = 0;
                    0
                }).unwrap_or(-1)
            },
            1 => {
                self.apps.enter(appid, |app, _| {
                    app.write_buffer = Some(slice);
                    0
                }).unwrap_or(-1)
            }
            _ => -1
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    app.read_callback = Some(callback);
                    0
                }).unwrap_or(-1)
            },
            1 /* putstr/write_done */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    match app.write_buffer.take() {
                        Some(slice) => {
                            app.write_callback = Some(callback);
                            app.write_len = slice.len();
                            if self.in_progress.is_none() {
                                self.in_progress.replace(callback.app_id());
                                self.buffer.take().map(|buffer| {
                                    for (i, c) in slice.as_ref().iter().enumerate() {
                                        if buffer.len() <= i {
                                            break;
                                        }
                                        buffer[i] = *c;
                                    }
                                    self.uart.send_bytes(buffer, app.write_len);
                                });
                            } else {
                                app.pending_write = true;
                                app.write_buffer = Some(slice);
                            }
                            0
                        },
                        None => -1
                    }
                }).unwrap_or(-1)
            },
            _ => -1
        }
    }

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> isize {
        match cmd_num {
            0 /* putc */ => { self.uart.send_byte(arg1 as u8); 1 },
            _ => -1
        }
    }
}

impl<'a, U: UART> Client for Console<'a, U> {
    fn write_done(&self, buffer: &'static mut [u8]) {
        // Write TX is done, notify appropriate app and start another
        // transaction if pending
        self.buffer.replace(buffer);
        self.in_progress.take().map(|appid| {
            self.apps.enter(appid, |app, _| {
                app.write_callback.map(|mut cb| {
                    cb.schedule(app.write_len, 0, 0);
                });
                app.write_len = 0;
            })
        });

        for cntr in self.apps.iter() {
            let started_tx = cntr.enter(|app, _| {
                if app.pending_write {
                    app.pending_write = false;
                    app.write_buffer.as_ref().map(|slice| {
                        self.buffer.take().map(|buffer| {
                            for (i, c) in slice.as_ref().iter().enumerate() {
                                if buffer.len() <= i {
                                    break;
                                }
                                buffer[i] = *c;
                            }
                            self.uart.send_bytes(buffer, app.write_len);
                        });
                        self.in_progress.replace(app.appid());
                        true
                    }).unwrap_or(false)
                } else {
                    false
                }
            });
            if started_tx {
                break;
            }
        }
    }

    fn read_done(&self, c: u8) {
        match c as char {
            '\r' => {},
            '\n' => {
                self.apps.each(|app| {
                    let idx = app.read_idx;
                    app.read_buffer = app.read_buffer.take().map(|mut rb| {
                        app.read_callback.as_mut().map(|cb| {
                            let buf = rb.as_mut();
                            cb.schedule(idx, (buf.as_ptr() as usize), 0);
                        });
                        rb
                    });
                    app.read_idx = 0;
                });
            },
            _ => {
                self.apps.each(|app| {
                    let idx = app.read_idx;
                    if app.read_buffer.is_some() &&
                        app.read_idx < app.read_buffer.as_ref().unwrap().len() {

                        app.read_buffer.as_mut().map(|buf| {
                            buf.as_mut()[idx] = c;
                        });
                        app.read_idx += 1;
                    }
                });
            }
        }
    }
}

