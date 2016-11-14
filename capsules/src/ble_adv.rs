use kernel::{AppId, AppSlice, Container, Callback, Shared, Driver};
use kernel::common::take_cell::TakeCell;
use kernel::hil::gpio::{Pin};
use kernel::hil::uart::{self, UART, Client};

pub struct App {
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            buffer: None,
        }
    }
}

pub static mut BUF: [u8; 64] = [0; 64];

pub struct BleAdv<'a, U: UART + 'a> {
    uart: &'a U,
    apps: Container<App>,
    buffer: TakeCell<&'static mut [u8]>,
    ready_pin: &'a Pin,
}

impl<'a, U: UART> BleAdv<'a, U> {
    pub fn new(uart: &'a U,
               ready_pin: &'a Pin,
               buffer: &'static mut [u8],
               container: Container<App>)
               -> BleAdv<'a, U> {
        BleAdv {
            uart: uart,
            ready_pin: ready_pin,
            apps: container,
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn initialize(&self) {
        self.uart.init(uart::UARTParams {
            baud_rate: 115200,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }
}

impl<'a, U: UART> Driver for BleAdv<'a, U> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.buffer = Some(slice);
                        0
                    })
                    .unwrap_or(-1)
            }
            _ => -1,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.apps.enter(callback.app_id(), |app, _| {
                    app.callback = Some(callback);
                    0
                }).unwrap_or(-1)
            },
            _ => -1
        }
    }

    fn command(&self, cmd_num: usize, _: usize, _: AppId) -> isize {
        match cmd_num {
            0 => {
                self.buffer.take().map(|buffer| {
                    self.uart.receive(buffer, 64);
                    self.ready_pin.make_output();
                    self.ready_pin.set();
                    0
                }).unwrap_or(-2)
            },
            _ => -1
        }
    }
}

impl<'a, U: UART> Client for BleAdv<'a, U> {
    fn transmit_complete(&self, _buffer: &'static mut [u8], _error: uart::Error) {
    }

    fn receive_complete(&self,
                        rx_buffer: &'static mut [u8],
                        rx_len: usize,
                        _error: uart::Error) {
        self.apps.each(|app| {
            app.buffer.as_mut().map(|buffer| {
                for (d,s) in buffer.as_mut().iter_mut().zip(rx_buffer.iter()) {
                    *d = *s;
                }
            });
            app.callback.as_mut().map(|cb| cb.schedule(0, 0, 0));
        });
        self.buffer.replace(rx_buffer);
    }
}

