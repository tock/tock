use core::marker::PhantomData;
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT:   u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Status-register bits (read from 0x64)
const STATUS_OUTPUT_FULL: u8 = 1 << 0; // data ready
const STATUS_INPUT_FULL:  u8 = 1 << 1; // controller busy

/// Block until the PS/2 input buffer is empty (safe to write).
fn wait_input_ready() {
    // Spin until bit 1 of status is 0
    while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_INPUT_FULL != 0 {
        // spin
    }
}

/// Block until there is data in the PS/2 output buffer.
fn wait_output_ready() {
    // Spin until bit 0 of status is 1
    while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_OUTPUT_FULL == 0 {
        // spin
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
}

impl<'a> Ps2Controller<'a> {
    /// Constructor — for now takes no args, adjust later when you wire up ports.
    pub fn new() -> Self {
        Ps2Controller { _marker: PhantomData }
    }

    /// Run the basic init sequence (disable, flush, config, self-test, enable).
    pub fn init(&self) {
        /// Run a minimal init: disable port, flush stale data, enable IRQ1
        // 1) Disable keyboard port (0xAD)
        write_command(0xAD);

        // 3) Flush any pending bytes
        while unsafe {
            io::inb(PS2_STATUS_PORT)
        } & STATUS_OUTPUT_FULL != 0 {
            let _ = read_data();
        }
        // 3) Read config byte (0x20), set IRQ1-enable, write it back (0x60)
        write_command(0x20);
        let mut cfg = read_data();
        cfg |= 1 << 0; //bit 0 = IRQ1 enable
        write_data(0x60);
        write_data(cfg);

        // 4) Re-enable keyboard port (0xAE)
        write_command(0xAE);
    }

    /// Called from IRQ1 to read a scan-code byte and buffer it.
    pub fn handle_interrupt(&self) {
        // Pull the byte so the controller's output buffer clears
        let _sc = read_data();
        // (PIC EOI is already done by the interrupt stub)
    }
}