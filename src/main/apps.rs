#[allow(improper_ctypes)]
extern {
    fn __allow(driver_num: usize, allownum: usize, ptr: *mut (), len: usize) -> isize;
    fn __subscribe(driver_num: usize, subnum: usize, cb: usize) -> isize;
    fn __command(driver_num: usize, cmdnum: usize, arg1: usize) -> isize;
    fn __wait(a: usize, b: usize, c: usize) -> isize;
}

fn allow(driver_num: usize, allownum: usize, ptr: *mut (), len: usize) -> isize {
    unsafe {
        __allow(driver_num, allownum, ptr, len)
    }
}

fn command(driver_num: usize, cmdnum: usize, arg1: usize) -> isize {
    unsafe {
        __command(driver_num, cmdnum, arg1)
    }
}

fn subscribe(driver_num: usize, cmdnum: usize, callback: usize) -> isize {
    unsafe {
        __subscribe(driver_num, cmdnum, callback)
    }
}

fn wait() -> isize {
    unsafe {
        __wait(0, 0, 0)
    }
}

mod tmp006 {
    use super::{command, subscribe};

    pub fn enable_tmp006() {
        command(2, 0, 0);
    }

    pub fn subscribe_temperature(f: fn(i16)) {
        subscribe(2, 0, f as usize);
    }
}

mod console {
    use super::{allow, command, subscribe};

    pub fn putc(c: char) {
        command(0, 0, c as usize);
    }

    pub fn puts(string: &str) {
        for c in string.chars() {
            putc(c);
        }
    }

    pub fn subscribe_read_line(buf: *mut u8, len: usize,
                               f: fn(usize, *mut u8)) -> isize {
        let res =  allow(0, 0, buf as *mut (), len);
        if res < 0 {
            res
        } else {
            subscribe(0, 0, f as usize)
        }
    }

}

mod gpio {
    use super::command;

    pub fn enable_pin(pin: usize) {
        command(1, 0, pin);
    }

    pub fn set_pin(pin: usize) {
        command(1, 2, pin);
    }

    pub fn clear_pin(pin: usize) {
        command(1, 3, pin);
    }

    pub fn toggle_pin(pin: usize) {
        command(1, 4, pin);
    }
}

pub mod app1 {
    use super::wait;
    use super::console::*;
    use super::gpio::*;
    use super::tmp006::*;
    use core::str;

    const WELCOME_MESSAGE: &'static str =
      "Welcome to Tock! Type \"help\" for a list of commands\r\n";

    const HELP_MESSAGE: &'static str =
r##"You may issue the following commands
  help          Prints this help message
  enable [pin]  Enables the GPIO pin
  set [pin]     Sets the GPIO pin
  clear [pin]   Clears the GPIO pin
  toggle [pin]  Toggles the GPIO pin
  echo [str]    Echos it's arguments
"##;

    const PROMPT: &'static str = "tock%> ";

    static mut BUF : *mut u8 = 0 as *mut u8;

    pub fn _start(mem_start: *mut u8, mem_size: usize) {
        unsafe {
            BUF = mem_start;
        }
        init();
        loop {
            wait();
        }
    }

    fn init() {
        puts(WELCOME_MESSAGE);
        unsafe {
            if subscribe_read_line(BUF, 40, line_read) < 0 {
                puts("Failed to subscribe to read");
                return
            }
        }
        subscribe_temperature(tmp_available);
        enable_tmp006();
        puts(PROMPT);
    }

    fn tmp_available(mut tmp: i16) {
        tmp = tmp / 32;
        puts("temperature read: ");
        putc((('0' as i16) + (tmp / 10)) as u8 as char);
        putc((('0' as i16) + (tmp % 10)) as u8 as char);
        puts("\r\n");
    }

    fn line_read(len: usize, b: *mut u8) {
        let buffer = ::core::raw::Slice { data: b, len: len };
        let line = unsafe { str::from_utf8(::core::mem::transmute(buffer)) };
        match line {
            Ok(cmd) => {
                parse_command(cmd);
            },
            Err(_) => puts("Invalid UTF8 sequence")
        }
        puts(PROMPT);
    }

    fn parse_command(line: &str) {
        let mut words = line.split(|c| {
            c == '\n' || c == '\r' || c == ' ' || c == '\t'
        }).filter(|s| { !s.is_empty() });
        let cmd = words.next();
        match cmd {
            Some("help") => {
                puts(HELP_MESSAGE);
            },
            Some("echo") => {
                words.next().map(|word| {
                    puts(word);
                });
                for w in words {
                    putc(' ');
                    puts(w);
                }
                puts("\r\n");
            },
            Some("enable") => {
                match words.next().map(|w| w.parse()) {
                    Some(Ok(pin)) => enable_pin(pin),
                    _ => {
                        puts("Error: first argument must be the pin number\r\n");
                    }
                }
            },
            Some("set") => {
                match words.next().map(|w| w.parse()) {
                    Some(Ok(pin)) => set_pin(pin),
                    _ => {
                        puts("Error: first argument must be the pin number\r\n");
                    }
                }
            },
            Some("clear") => {
                match words.next().map(|w| w.parse()) {
                    Some(Ok(pin)) => clear_pin(pin),
                    _ => {
                        puts("Error: first argument must be the pin number\r\n");
                    }
                }
            },
            Some("toggle") => {
                match words.next().map(|w| w.parse()) {
                    Some(Ok(pin)) => toggle_pin(pin),
                    _ => {
                        puts("Error: first argument must be the pin number\r\n");
                    }
                }
            },
            Some(c) => {
                puts("Unknown command: ");
                puts(c);
                puts("\r\n");
            },
            _ => {}
        }
    }
}

