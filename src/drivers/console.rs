use hil::{Driver,Callback};
use hil::uart::{UART, Reader};

pub struct Console<U: UART + 'static> {
    uart: &'static mut U,
    read_callback: Option<Callback>,
}

impl<U: UART> Console<U> {
    pub fn new(uart: &'static mut U) -> Console<U> {
        Console {
            uart: uart,
            read_callback: None,
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
    fn subscribe(&mut self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ =>
                { self.read_callback = Some(callback); 0 },
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
        self.read_callback.as_mut().map(|cb| {
              cb.schedule(c as usize, 0, 0);
        });
    }
}

