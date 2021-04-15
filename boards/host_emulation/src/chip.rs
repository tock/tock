use core::fmt::Write;
use kernel::{self, static_init};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::vec::Vec;
use std::{thread, time};

use crate::syscall::SysCall;
use crate::systick::SysTick;

use crate::emulation_config::Config;
use std::path::Path;
pub trait Callback {
    fn execute(&self) -> ();
}

const SLEEP_DURATION_US: u128 = 1;

/// An generic `Chip` implementation
pub struct HostChip {
    systick: SysTick,
    syscall: SysCall,
    service_interrupts_callbacks: Vec<&'static dyn Callback>,
    terminate: Arc<AtomicBool>,
    terminate_callbacks: RefCell<Vec<&'static dyn Fn()>>,
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

        let terminate = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&terminate)).unwrap();
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate)).unwrap();

        HostChip {
            systick: SysTick::new(),
            syscall: syscall,
            service_interrupts_callbacks: Vec::new(),
            terminate,
            terminate_callbacks: RefCell::new(Vec::new()),
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

    pub fn add_service_interrupts_callback(&mut self, callback: &'static dyn Callback) {
        self.service_interrupts_callbacks.push(callback);
    }

    pub fn add_terminate_callback(&self, callback: &'static dyn Fn()) {
        self.terminate_callbacks.borrow_mut().push(callback);
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
        if self.terminate.load(Ordering::Relaxed) {
            println!("Terminated!");
            for callback in &*self.terminate_callbacks.borrow() {
                callback();
            }
            std::process::exit(0);
        }

        for callback in &self.service_interrupts_callbacks {
            callback.execute();
        }
        unsafe {
            super::UART0.handle_pending_requests();
            for i2cp in &super::I2CP {
                i2cp.handle_pending_requests();
            }
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
