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
