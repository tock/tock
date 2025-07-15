use core::marker::PhantomData;
use kernel::debug;
use x86::registers::io;
use core::cell::{Cell, RefCell};

/// PS/2 controller ports
const PS2_DATA_PORT:   u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Status-register bits (read from 0x64)
const STATUS_OUTPUT_FULL: u8 = 1 << 0; // data ready
const STATUS_INPUT_FULL:  u8 = 1 << 1; // controller busy

const BUFFER_SIZE: usize = 32;
const TIMEOUT: usize = 1_000_000;

/// Block until the PS/2 input buffer is empty (safe to write).
fn wait_input_ready() {
    let mut cnt = 0;
    while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_INPUT_FULL != 0 {
        cnt += 1;
        if cnt >= TIMEOUT { return; }
    }
}

/// Block until there is data in the PS/2 output buffer.
fn wait_output_ready() {
    let mut cnt = 0;
    while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_OUTPUT_FULL == 0 {
        cnt += 1;
        if cnt >= TIMEOUT { return; }
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
    _marker: PhantomData<&'a ()>,
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head:   Cell<usize>,
    tail:   Cell<usize>,
}

impl<'a> Ps2Controller<'a> {
    /// Constructor — for now takes no args, adjust later when you wire up ports.
    pub fn new() -> Self {
        Ps2Controller {
            _marker: PhantomData,
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head:   Cell::new(0),
            tail:   Cell::new(0),
        }
    }

    /// Run the basic init sequence (disable, flush, config, self-test, enable).
    pub fn init(&self) {
        // 1) Disable keyboard port (0xAD)
        write_command(0xAD);
        // Disable second channel (0xA7)
        write_command(0xA7);

        // 2) Flush any pending bytes
        while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_OUTPUT_FULL != 0 {
            let _ = read_data();
        }

        // 2a) Controller self-test
        write_command(0xAA);
        let res = read_data();
        if res != 0x55 {
            debug!("PS/2 self-test failed: {:02x}", res);
            return;
        }

        // 3) Read config byte (0x20), set IRQ1-enable, write it back (0x60)
        write_command(0x20);
        let mut cfg = read_data();
        cfg |= 1 << 0; // bit 0 = IRQ1 enable
        write_data(0x60);
        write_data(cfg);

        // 4) Re-enable keyboard port (0xAE)
        write_command(0xAE);


    }

    /// Called from IRQ1 to read a scan-code byte and buffer it.
    pub fn handle_interrupt(&self) {
        let sc = read_data();
        self.push_code(sc);
        // (PIC EOI is already done by the interrupt stub)
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

    /// Internal: push a scan-code, dropping oldest on overflow.
    fn push_code(&self, byte: u8) {
        let head = self.head.get();
        let next = (head + 1) % BUFFER_SIZE;
        let tail = self.tail.get();
        if next != tail {
            self.buffer.borrow_mut()[head] = byte;
            self.head.set(next);
        } else {
            // buffer full, drop oldest
            self.tail.set((tail + 1) % BUFFER_SIZE);
            self.buffer.borrow_mut()[head] = byte;
            self.head.set(next);
        }
    }
}