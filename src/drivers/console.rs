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
            read_idx: 0
        }
    }
}

pub static mut WRITE_BUF : [u8; 64] = [0; 64];

pub struct Console<'a, U: UART + 'a> {
    uart: &'a U,
    apps: Container<App>,
    buffer: TakeCell<&'static mut [u8]>
}

impl<'a, U: UART> Console<'a, U> {
    pub const fn new(uart: &'a U, buffer: &'static mut [u8],
                     container: Container<App>) -> Console<'a, U> {
        Console {
            uart: uart,
            apps: container,
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
        let app = appid.idx();
        match allow_num {
            0 => {
                self.apps.enter(appid, |app, _| {
                    app.read_buffer = Some(slice);
                    app.read_idx = 0;
                });
                0
            },
            1 => {
                self.apps.enter(appid, |app, _| {
                    app.write_buffer = Some(slice);
                });
                0
            }
            _ => -1
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    app.read_callback = Some(callback)
                });
                0
            },
            1 /* putstr/write_done */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    match app.write_buffer.take() {
                        Some(slice) => {
                            app.write_callback = Some(callback);
                            app.write_len = slice.len();
                            self.buffer.take().map(|buffer| {
                                for (i, c) in slice.as_ref().iter().enumerate() {
                                    buffer[i] = *c;
                                }
                                self.uart.send_bytes(buffer, app.write_len);
                            });
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

fn each_some<'a, T, I, F>(lst: I, mut f: F)
        where T: 'a, I: Iterator<Item=&'a TakeCell<T>>, F: FnMut(&mut T) {
    for item in lst {
        item.map(|i| f(i));
    }
}

impl<'a, U: UART> Client for Console<'a, U> {
    fn write_done(&self, buffer: &'static mut [u8]) {
        self.buffer.replace(buffer);
        self.apps.each(|app| {
            app.write_callback.take().map(|mut cb| {
                cb.schedule(app.write_len, 0, 0);
            });
            app.write_len = 0;
        });
    }

    fn read_done(&self, c: u8) {
        match c as char {
            '\r' => {},
            '\n' => {
                self.apps.each(|app| {
                    let idx = app.read_idx;
                    app.read_buffer = app.read_buffer.take().map(|mut rb| {
                        use core::raw::Repr;
                        app.read_callback.as_mut().map(|cb| {
                            let buf = rb.as_mut();
                            cb.schedule(idx, (buf.repr().data as usize), 0);
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

