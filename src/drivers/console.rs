use hil::{Driver,Callback,AppSlice,Shared};
use hil::uart::{UART, Reader};

pub struct Console<U: UART + 'static> {
    uart: &'static mut U,
    read_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
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
    fn allow(&mut self, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_num {
            0 => {
                self.read_buffer = Some(slice);
                self.read_idx = 0;
                0
            },
            _ => -1
        }
    }

    fn subscribe(&mut self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read line */ => {
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
                self.read_buffer = self.read_buffer.take().map(|mut rb| {
                    use ::core::raw::Repr;
                    self.read_callback.as_mut().map(|cb| {
                        let buf = rb.as_mut();
                        cb.schedule(idx, (buf.repr().data as usize), 0);
                    });
                    rb
                });
                self.read_idx = 0;
            },
            _ => {
                let idx = self.read_idx;
                if self.read_buffer.is_some() &&
                    self.read_idx < self.read_buffer.as_ref().unwrap().len() {

                    self.read_buffer.as_mut().map(|buf| {
                        buf.as_mut()[idx] = c;
                    });
                    self.read_idx += 1;
                }
            }
        }
    }
}

