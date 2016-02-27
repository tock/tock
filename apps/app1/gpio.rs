#![allow(dead_code)]

use super::syscalls::command;

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

