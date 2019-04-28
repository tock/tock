//! Generic support for all Cortex-M platforms.

#![crate_name = "cortexm"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, lang_items, naked_functions)]
#![no_std]

pub mod events;
pub mod nvic;
pub mod scb;
pub mod support;
pub mod syscall;
pub mod systick;

#[macro_export]
macro_rules! generic_isr {
    ($label:tt, $priority:expr) => {
        #[cfg(target_os = "none")]
        #[naked]
        unsafe extern "C" fn $label() {
            stash_process_state();
            events::set_event_flag_from_isr($priority as usize);
            set_privileged_thread();
        }
    };
}

#[macro_export]
macro_rules! custom_isr {
    ($label:tt, $priority:expr, $isr:ident) => {
        #[cfg(target_os = "none")]
        #[naked]
        unsafe extern "C" fn $label() {
            stash_process_state();
            events::set_event_flag_from_isr($priority);
            $isr();
            set_privileged_thread();
        }
    };
}
