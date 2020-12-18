use core::fmt::Write;
use kernel::{self, static_init};
use std::{thread, time};

use crate::syscall::SysCall;
use crate::systick::SysTick;

use crate::emulation_config::Config;
use std::path::Path;

/// An generic `Chip` implementation
pub struct HostChip {
    systick: SysTick,
    syscall: SysCall,
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

    fn service_pending_interrupts(&self) {}

    fn has_pending_interrupts(&self) -> bool {
        false
    }

    fn sleep(&self) {
        let wait_until = self.systick.get_systick_left().unwrap();
        let wait_until = time::Duration::from_micros(wait_until as u64);
        thread::sleep(wait_until);
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
