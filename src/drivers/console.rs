use core::prelude::*;
use hil::{Driver,Callback,AppPtr};
use hil::uart::{UART, Reader};

pub struct Console<U: UART + 'static> {
    uart: &'static mut U,
    read_callback: Option<Callback>,
    read_buffer: Option<AppPtr<[u8; 40]>>,
    read_idx: usize
}

impl<U: UART> Console<U> {
    pub fn new(uart: &'static mut U) -> Console<U> {
        Console {
            uart: uart,
            read_callback: None,
            read_buffer: None,
            read_idx: 0
        }
    }

    pub fn initialize(&mut self) {
        self.uart.enable_tx();
        self.uart.enable_rx();
    }

    pub fn putstr(&mut self, string: &str) {
        for c in string.bytes() {
            self.uart.send_byte(c);
        }
    }

    pub fn putbytes(&mut self, string: &[u8]) {
        for c in string {
            self.uart.send_byte(*c);
        }
    }
}

impl<U: UART> Driver for Console<U> {
    fn subscribe(&mut self, subscribe_num: usize, mut callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ => {
                self.read_buffer = callback.allocate([0; 40]);
                self.read_callback = Some(callback);
                0
            },
            _ => -1
        }
    }

    fn command(&mut self, cmd_num: usize, arg1: usize) -> isize {
        match cmd_num {
            0 /* putc */ => { self.uart.send_byte(arg1 as u8); 1 }
            _ => -1
        }
    }
}

impl<U: UART> Reader for Console<U> {
    fn read_done(&mut self, c: u8) {
        match c as char {
            '\r' => {},
            '\n' => {
                let idx = self.read_idx;
                self.read_buffer = self.read_buffer.take().map(|buf| {
                    use ::core::raw::Repr;
                    self.read_callback.as_mut().map(|cb| {
                        cb.schedule(idx, (buf.repr().data as usize), 0);
                    });
                    buf
                });
                self.read_idx = 0;
            },
            _ => {
                if self.read_idx < 40 {
                    self.read_buffer = self.read_buffer.take().map(|mut buf| {
                        buf[self.read_idx] = c;
                        buf
                    });
                    self.read_idx += 1;
                }
            }
        }
    }
}

