use core::cell::RefCell;
use hil::{AppId,Driver,Callback,AppSlice,Shared,NUM_PROCS};
use hil::uart::{UART, Client};

struct App {
    read_callback: Option<Callback>,
    write_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    read_idx: usize
}

pub struct Console<'a, U: UART + 'a> {
    uart: &'a U,
    apps: [RefCell<Option<App>>; NUM_PROCS],
}

impl<'a, U: UART> Console<'a, U> {
    pub const fn new(uart: &'a U) -> Console<U> {
        Console {
            uart: uart,
            apps: [RefCell::new(None)]
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
                let mut appc = self.apps[app].borrow_mut();
                if appc.is_none() {
                    *appc = Some(App {
                        read_callback: None,
                        read_buffer: Some(slice),
                        read_idx: 0,
                        write_buffer: None,
                        write_len: 0,
                        write_callback: None
                    })
                } else {
                    appc.as_mut().map(|app| {
                        app.read_buffer = Some(slice);
                        app.read_idx = 0;
                    });
                }
                0
            },
            1 => {
                let mut appc = self.apps[app].borrow_mut();
                if appc.is_none() {
                    *appc = Some(App {
                        read_callback: None,
                        read_buffer: None,
                        read_idx: 0,
                        write_buffer: Some(slice),
                        write_len: 0,
                        write_callback: None
                    })
                } else {
                    appc.as_mut().map(|app| app. write_buffer = Some(slice) );
                }
                0
            }
            _ => -1
        }
    }

    #[inline(never)]
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ => {
                let mut app = self.apps[0].borrow_mut();
                if app.is_none() {
                    *app = Some(App {
                        read_callback: Some(callback),
                        read_buffer: None,
                        read_idx: 0,
                        write_buffer: None,
                        write_len: 0,
                        write_callback: None
                    });
                } else {
                    app.as_mut().map(|a| a. read_callback = Some(callback) );
                }
                0
            },
            1 /* putstr/write_done */ => {
                match self.apps[0].borrow_mut().as_mut() {
                    None => {
                        -1
                    },
                    Some(app) => {
                        match app.write_buffer.take() {
                            Some(slice) => {
                                app.write_callback = Some(callback);
                                app.write_len = slice.len();
                                self.uart.send_bytes(slice);
                                0
                            },
                            None => -1
                        }
                    }
                }
            },
            _ => -1
        }
    }

    fn command(&self, cmd_num: usize, arg1: usize) -> isize {
        match cmd_num {
            0 /* putc */ => { self.uart.send_byte(arg1 as u8); 1 },
            _ => -1
        }
    }
}

fn each_some<'a, T, I, F>(lst: I, f: F)
        where T: 'a, I: Iterator<Item=&'a RefCell<Option<T>>>, F: Fn(&mut T) {
    for item in lst {
        item.borrow_mut().as_mut().map(|i| f(i));
    }
}

impl<'a, U: UART> Client for Console<'a, U> {
    fn write_done(&self) {
        self.apps[0].borrow_mut().as_mut().map(|app| {
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
                each_some(self.apps.iter(), |app| {
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
                each_some(self.apps.iter(), |app| {
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

