use kernel::{AppId, AppSlice, Container, Callback, Shared, Driver};
use kernel::common::take_cell::TakeCell;
use kernel::hil::uart::{self, UART, Client};

pub struct App {
    write_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    pending_write: bool,
    read_idx: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            write_callback: None,
            read_buffer: None,
            write_buffer: None,
            write_len: 0,
            pending_write: false,
            read_idx: 0,
        }
    }
}

pub static mut WRITE_BUF: [u8; 64] = [0; 64];

pub struct Console<'a, U: UART + 'a> {
    uart: &'a U,
    apps: Container<App>,
    in_progress: TakeCell<AppId>,
    tx_buffer: TakeCell<&'static mut [u8]>,
    baud_rate: u32,
}

impl<'a, U: UART> Console<'a, U> {
    pub fn new(uart: &'a U,
               baud_rate: u32,
               tx_buffer: &'static mut [u8],
               container: Container<App>)
               -> Console<'a, U> {
        Console {
            uart: uart,
            apps: container,
            in_progress: TakeCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            baud_rate: baud_rate,
        }
    }

    pub fn initialize(&self) {
        self.uart.init(uart::UARTParams {
            baud_rate: self.baud_rate,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }
}

impl<'a, U: UART> Driver for Console<'a, U> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.read_buffer = Some(slice);
                        app.read_idx = 0;
                        0
                    })
                    .unwrap_or(-1)
            }
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.write_buffer = Some(slice);
                        0
                    })
                    .unwrap_or(-1)
            }
            _ => -1,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ => {
                // read line is not implemented for console at this time
                -1
            },
            1 /* putstr/write_done */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    match app.write_buffer.take() {
                        Some(slice) => {
                            app.write_callback = Some(callback);
                            app.write_len = slice.len();
                            if self.in_progress.is_none() {
                                self.in_progress.replace(callback.app_id());
                                self.tx_buffer.take().map(|buffer| {
                                    for (i, c) in slice.as_ref().iter().enumerate() {
                                        if buffer.len() <= i {
                                            break;
                                        }
                                        buffer[i] = *c;
                                    }
                                    self.uart.transmit(buffer, app.write_len);
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
            0 /* putc */ => {
                self.tx_buffer.take().map(|buffer| {
                    buffer[0] = arg1 as u8;
                    self.uart.transmit(buffer, 1);
                });
                1
            },
            _ => -1
        }
    }
}

impl<'a, U: UART> Client for Console<'a, U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        // Write TX is done, notify appropriate app and start another
        // transaction if pending
        self.tx_buffer.replace(buffer);
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
                    app.write_buffer
                        .as_ref()
                        .map(|slice| {
                            self.tx_buffer.take().map(|buffer| {
                                for (i, c) in slice.as_ref().iter().enumerate() {
                                    if buffer.len() <= i {
                                        break;
                                    }
                                    buffer[i] = *c;
                                }
                                self.uart.transmit(buffer, app.write_len);
                            });
                            self.in_progress.replace(app.appid());
                            true
                        })
                        .unwrap_or(false)
                } else {
                    false
                }
            });
            if started_tx {
                break;
            }
        }
    }

    fn receive_complete(&self,
                        _rx_buffer: &'static mut [u8],
                        _rx_len: usize,
                        _error: uart::Error) {
        // this is currently unimplemented for console
    }
}
