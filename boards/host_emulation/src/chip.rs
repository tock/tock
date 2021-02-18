use core::fmt::Write;
use kernel::{self, static_init};
use std::{thread, time};

use crate::syscall::SysCall;
use crate::systick::SysTick;

use crate::emulation_config::Config;
use std::path::Path;

const SLEEP_DURATION_US: u128 = 1;

pub trait Callback {
    fn execute(&self) -> ();
}

/// An generic `Chip` implementation
pub struct HostChip {
    systick: SysTick,
    syscall: SysCall,
    service_interrupts_callback: Option<&'static dyn Callback>,
}

impl HostChip {
    /// Parse cmdline args to setup chip syscall interface
    /// Required to be called before any HostChip is created
    pub fn basic_setup() {
        unsafe {
            let cfg = static_init!(Config, Config::from_cmd_line_args().unwrap());
            let apps = cfg.apps();
            assert_eq!(apps.len(), 1);
            Config::set(cfg);
        }
    }
    pub fn new() -> HostChip {
        let cmd_info = Config::get();
        let syscall = match SysCall::try_new(cmd_info.syscall_rx_path(), cmd_info.syscall_tx_path())
        {
            Ok(syscall) => syscall,
            _ => panic!("Unable to create Syscall"),
        };
        HostChip {
            systick: SysTick::new(),
            syscall: syscall,
            service_interrupts_callback: None,
        }
    }

    #[allow(dead_code)]
    pub fn enable_interrupts(&self) {}

    #[allow(dead_code)]
    pub fn get_app_path() -> &'static Path {
        // let cmd_info = unsafe { HOST_CONFIG.unwrap() };
        let cmd_info = Config::get();
        cmd_info.apps()[0].bin_path()
    }

    pub fn set_service_interrupts_callback(&mut self, callback: &'static dyn Callback) {
        self.service_interrupts_callback = Some(callback);
    }
}

impl kernel::Chip for HostChip {
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;
    type SysTick = SysTick;

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.syscall
    }

    fn service_pending_interrupts(&self) {
        if let Some(callback) = &self.service_interrupts_callback {
            callback.execute();
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        false
    }

    fn sleep(&self) {
        // A temporary solution to wake kernel up periodically.
        // The proper solution would be to wait for an external event,
        // pending host emulation support of relevant peripherals.
        thread::sleep(time::Duration::from_micros(
            self.systick.get_systick_left().unwrap_or(SLEEP_DURATION_US) as u64,
        ));
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // It's ok to call f as irq and systick won't happen
        f()
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        writer
            .write_fmt(format_args!("print_state() not implemented."))
            .unwrap();
    }
}
