use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use kernel::debug;
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT:   u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Status-register bits
const STATUS_OUTPUT_FULL: u8 = 1 << 0; // data ready
const STATUS_INPUT_FULL:  u8 = 1 << 1; // input buffer full (controller busy)

/// Timeout limit for spin loops
const TIMEOUT_LIMIT: usize = 1_000_000;

/// Depth of the scan-code ring buffer
const BUFFER_SIZE: usize = 32;

/// Wait until the PS/2 input buffer is clear (safe to write), or timeout.
fn wait_input_ready() {
    let mut count = 0;
    while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_INPUT_FULL != 0 {
        count += 1;
        if count >= TIMEOUT_LIMIT {
            debug!("PS/2 wait_input_ready timed out");
            break;
        }
    }
}

/// Wait until data is available in the PS/2 output buffer, or timeout.
fn wait_output_ready() {
    let mut count = 0;
    while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_OUTPUT_FULL == 0 {
        count += 1;
        if count >= TIMEOUT_LIMIT {
            debug!("PS/2 wait_output_ready timed out");
            break;
        }
    }
}

/// Read one byte from the data port (0x60).
pub fn read_data() -> u8 {
    wait_output_ready();
    unsafe { io::inb(PS2_DATA_PORT) }
}

/// Send a command byte to the controller (port 0x64).
pub fn write_command(cmd: u8) {
    wait_input_ready();
    unsafe { io::outb(PS2_STATUS_PORT, cmd) };
}

/// Write a data byte to the data port (0x60).
pub fn write_data(data: u8) {
    wait_input_ready();
    unsafe { io::outb(PS2_DATA_PORT, data) };
}

/// PS/2 controller driver (the “8042” peripheral)
pub struct Ps2Controller<'a> {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head:   Cell<usize>,
    tail:   Cell<usize>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Ps2Controller<'a> {
    /// Create a new PS/2 controller instance.
    pub fn new() -> Self {
        Ps2Controller {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head:   Cell::new(0),
            tail:   Cell::new(0),
            _marker: PhantomData,
        }
    }

    /// Initialize the PS/2 controller
    /// Steps:
    /// 1) Disable both channels
    /// 2) Flush output buffer
    /// 3) Controller self-test
    /// 4) Configure IRQ1 in config byte
    /// 5) Test keyboard port
    /// 6) Enable keyboard scanning
    /// 7) Re-enable keyboard channel
    /// 8) Unmask IRQ1 on master PIC
    pub fn init(&self) {
        unsafe {
            // 1) Disable keyboard and auxiliary channels
            write_command(0xAD);
            write_command(0xA7);

            // 2) Flush any pending output
            while io::inb(PS2_STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
                let _ = read_data();
            }

            // 3) Controller self-test (0xAA -> should return 0x55)
            write_command(0xAA);
            wait_output_ready();
            let res = read_data();
            if res != 0x55 {
                debug!("PS/2 self-test failed: {:02x}", res);
            }

            // 4) Read-Modify-Write config byte (enable IRQ1)
            write_command(0x20);
            let mut cfg = read_data();
            cfg |= 1 << 0; // enable IRQ1
            write_command(0x60);
            write_data(cfg);

            // 5) Test keyboard port (0xAB -> should return 0x00)
            write_command(0xAB);
            wait_output_ready();
            let port_ok = read_data();
            if port_ok != 0x00 {
                debug!("PS/2 keyboard-port test failed: {:02x}", port_ok);
            }

            // 6) Enable scanning on keyboard device (0xF4 -> expect 0xFA)
            write_data(0xF4);
            wait_output_ready();
            let ack = read_data();
            if ack != 0xFA {
                debug!("PS/2 keyboard enable-scan ACK failed: {:02x}", ack);
            }

            // 7) Re-enable keyboard channel
            write_command(0xAE);

            // 8) Unmask IRQ1 on master PIC
            const PIC1_DATA: u16 = 0x21;
            let mask = io::inb(PIC1_DATA);
            io::outb(PIC1_DATA, mask & !(1 << 1));
        }
    }

    /// Handle a keyboard interrupt: read a scan-code and buffer it.
    pub fn handle_interrupt(&self) {
        let code = read_data();
        self.push_code(code);
        // PIC EOI is done by the interrupt stub
    }

    /// Pop the next scan-code, or None if buffer is empty.
    pub fn pop_scan_code(&self) -> Option<u8> {
        let head = self.head.get();
        let tail = self.tail.get();
        if head == tail {
            None
        } else {
            let byte = self.buffer.borrow()[tail];
            self.tail.set((tail + 1) % BUFFER_SIZE);
            Some(byte)
        }
    }

    /// Internal: push a scan-code into the ring buffer, dropping oldest if full.
    fn push_code(&self, byte: u8) {
        let head = self.head.get();
        let next = (head + 1) % BUFFER_SIZE;
        if next == self.tail.get() {
            // buffer full, advance tail
            self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
        }
        self.buffer.borrow_mut()[head] = byte;
        self.head.set(next);
    }
}
