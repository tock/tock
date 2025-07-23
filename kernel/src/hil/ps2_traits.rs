// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use crate::ErrorCode;

/// A Tock HIL for talking to an 8042 PS/2 controller.
pub trait PS2Traits {
    /// Block until the controller input buffer is free.
    fn wait_input_ready();

    /// Block until the output buffer has data.
    fn wait_output_ready();

    /// Read a byte from data port.
    fn read_data() -> u8;

    /// Write a command byte to the command port.
    fn write_command(cmd: u8);

    /// Write a data byte to the data port.
    fn write_data(data: u8);

    /// Initialize the controller (self-test, config, enable IRQ, etc).
    fn init(&self);

    /// Called from your IRQ stub.  Should read one byte and
    /// return `Ok(())` or an error code.
    fn handle_interrupt(&self) -> Result<(), ErrorCode>;

    /// Pop one scan‐code out of the driver’s ring buffer.
    fn pop_scan_code(&self) -> Option<u8>;

    /// Push one scan‐code into the ring buffer.
    fn push_code(&self, code: u8) -> Result<(), ErrorCode>;
}
pub trait PS2Keyboard {
    /// Set keyboard LEDs: bit0=ScrollLock, bit1=NumLock, bit2=CapsLock.
    fn set_leds(&self, mask: u8) -> Result<(), ErrorCode>;

    /// Send Echo (0xEE) to keyboard and expect same byte back.
    fn probe_echo(&self) -> Result<(), ErrorCode>;

    /// Check if a keyboard is present (returns true on successful echo).
    fn is_present(&self) -> bool;

    /// Identify keyboard: send 0xF2, get up to 3 ID bytes and count.
    fn identify(&self) -> Result<([u8; 3], usize), ErrorCode>;

    /// Get or set current scan‑code set (0=Get, 1-3=Set).
    /// Returns the set number on success.
    fn scan_code_set(&self, subcmd: u8) -> Result<u8, ErrorCode>;

    /// Set typematic rate and delay (0xF3 + rate_delay byte).
    fn set_typematic(&self, rate_delay: u8) -> Result<(), ErrorCode>;

    /// Enable scan-code reporting (0xF4).
    fn enable_scanning(&self) -> Result<(), ErrorCode>;

    /// Disable scan-code reporting (0xF5).
    fn disable_scanning(&self) -> Result<(), ErrorCode>;

    /// Restore default keyboard parameters (0xF6).
    fn set_defaults(&self) -> Result<(), ErrorCode>;

    /// Set all keys to auto-repeat only (0xF7, scancode set 3 only).
    fn set_typematic_only(&self) -> Result<(), ErrorCode>;

    /// Set all keys to make + release (0xF8, scancode set 3 only).
    fn set_make_release(&self) -> Result<(), ErrorCode>;

    /// Set all keys to make-only (0xF9, scancode set 3 only).
    fn set_make_only(&self) -> Result<(), ErrorCode>;

    /// Set full-full-full mode (0xFA, scancode set 3 only).
    fn set_full_mode(&self) -> Result<(), ErrorCode>;

    /// Set a specific key to auto-repeat only (0xFB, scancode set 3 only).
    fn set_key_typematic_only(&self, scancode: u8) -> Result<(), ErrorCode>;

    /// Set a specific key to make + release (0xFC, scancode set 3 only).
    fn set_key_make_release(&self, scancode: u8) -> Result<(), ErrorCode>;

    /// Set a specific key to make-only (0xFD, scancode set 3 only).
    fn set_key_make_only(&self, scancode: u8) -> Result<(), ErrorCode>;

    /// Request keyboard to resend last byte (0xFE). Returns the resent byte.
    fn resend_last_byte(&self) -> Result<u8, ErrorCode>;

    /// Reset keyboard and run self-test (0xFF). Expects ACK (0xFA), then result (0xAA pass).
    fn reset_and_self_test(&self) -> Result<(), ErrorCode>;
}

/// Trait for consuming decoded key inputs (ASCII or keycodes).
pub trait KBReceiver {
    /// Called by consumers to fetch one decoded byte, `None` if none available.
    fn receive(&self) -> Option<u8>;
}

pub enum MousePacket {
    /// (left, right, middle buttons state, x_delta, y_delta)
    Relative { buttons: u8, dx: i8, dy: i8 },
    /// (optional) Wheel or 5‑button support, if we want to expand
    Extended {
        buttons: u8,
        dx: i8,
        dy: i8,
        wheel: i8,
    },
}

pub trait PS2Mouse {
    /// Reset + self‑test, returns OK or FAIL.
    fn set_scaling_1_1(&self) -> Result<(), ErrorCode>;
    fn set_scaling_2_1(&self) -> Result<(), ErrorCode>;
    fn status_request(&self) -> Result<[u8; 3], ErrorCode>;
    fn read_data(&self) -> Result<MouseEvent, ErrorCode>;
    fn reset(&self) -> Result<(), ErrorCode>;

    /// Enable streaming (data reporting).
    fn enable_streaming(&self) -> Result<(), ErrorCode>;

    /// Disable streaming.
    fn disable_streaming(&self) -> Result<(), ErrorCode>;

    /// Set sampling rate, protocol resolution, etc.
    fn set_sample_rate(&self, hz: u8) -> Result<(), ErrorCode>;
    fn set_resolution(&self, counts_per_mm: u8) -> Result<(), ErrorCode>;
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    pub buttons: u8,
    pub x_movement: i8,
    pub y_movement: i8,
}
