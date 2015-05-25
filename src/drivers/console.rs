use core::prelude::*;
use hil::uart::{UART, Reader};

pub struct Console<U: UART + 'static> {
    uart: &'static mut U,
    buf: [u8; 40],
    i: usize
}

impl<U: UART> Console<U> {
    pub fn new(uart: &'static mut U) -> Console<U> {
        Console {
            uart: uart,
            buf: [0; 40],
            i: 0
        }
    }

    pub fn initialize(&mut self) {
        self.uart.enable_tx();
        self.uart.enable_rx();

        self.putstr("> ");
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

impl<U: UART> Reader for Console<U> {
    fn read_done(&mut self, c: u8) {
        use core::str;

        match c as char {
            '\n' => {
                let mut buf = [0; 40];
                let mut i = 0;
                while i < self.i {
                    buf[i] = self.buf[i];
                    i += 1;
                }
                if str::from_utf8(&buf[0..self.i]) == Ok("help") {
                    self.putstr("Welcome to Tock. You can issue the following commands\r\n");
                    self.putstr("\thelp\t\tPrints this help message\r\n");
                    self.putstr("\techo [str]\tEchos it's arguments\r\n");
                } else if str::from_utf8(&buf[0..5]) == Ok("echo ") {
                    self.putbytes(&buf[5..i]);
                    self.putstr("\r\n");
                } else {
                    self.putstr("Unexpected command: ");
                    self.putbytes(&buf[0..i]);
                    self.putstr("\r\n");
                }
                self.i = 0;
                self.putstr("> ");
            },
            _ => {
                self.buf[self.i] = c;
                self.i += 1;
            }
        }
    }
}

