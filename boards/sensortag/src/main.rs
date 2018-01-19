#![no_std]
#![no_main]
#![feature(lang_items, compiler_builtins_lib, asm)]

extern crate capsules;
extern crate compiler_builtins;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;

extern crate cc2650;

use core::fmt::{Arguments};
use kernel::common::VolatileCell;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
//
static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

#[cfg_attr(rustfmt, rustfmt_skip)]
#[no_mangle]
#[link_section = ".ccfg"]
pub static CCFG_CONF: [u32; 22] = [
        0x01800000,
        0xFF820010,
        0x0058FFFD,
        0xF3BFFF3A,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0x00FFFFFF,
        0xFFFFFFFF,
        0xFFFFFF00,
        0xFFC500C5,
        0xFF000000,
        0x00000000,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFF,
];

pub struct Platform {
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            // Todo, add drivers here
            _ => f(None),
        }
    }
}


#[repr(C)]
struct PRCM {
    _r0: [VolatileCell<u8>; 0x28],

    // Write 1 in order to load settings
    clk_load_ctl: VolatileCell<u32>,

    _r1: [VolatileCell<u8>; 0x1C],

    gpio_clk_gate_run: VolatileCell<u32>,
    gpio_clk_gate_sleep: VolatileCell<u32>,
    gpio_clk_gate_deep_sleep: VolatileCell<u32>,

    _r2: [VolatileCell<u8>; 0xD8],

    // Power domain control 0
    pd_ctl0: VolatileCell<u32>,
    _pd_ctl0_rfc: VolatileCell<u32>,
    _pd_ctl0_serial: VolatileCell<u32>,
    _pd_ctl0_peripheral: VolatileCell<u32>,

    _r3: [VolatileCell<u8>; 0x04],

    // Power domain status 0
    _pd_stat0: VolatileCell<u32>,
    _pd_stat0_rfc: VolatileCell<u32>,
    _pd_stat0_serial: VolatileCell<u32>,
    pd_stat0_periph: VolatileCell<u32>,
}

const PRCM_BASE: u32 = 0x40082000;

#[no_mangle]
pub unsafe fn reset_handler() {
    let prcm = &*(PRCM_BASE as *const PRCM);

    // PERIPH power domain on
    prcm.pd_ctl0.set(0x4);

    // Load values (peripherals should get power)
    prcm.clk_load_ctl.set(1);

    // Wait until peripheral power is on
    while (prcm.pd_stat0_periph.get() & 1) != 1  {
        asm!("nop;");
    }

    // Enable GPIO clocks
    prcm.gpio_clk_gate_run.set(1);
    prcm.gpio_clk_gate_sleep.set(1);
    prcm.gpio_clk_gate_deep_sleep.set(1);

    // Load values
    prcm.clk_load_ctl.set(1);

    // Setup DIO10
    let iocbase = 0x40081000;
    let iocfg10 = iocbase + 0x28;

    // Enable data output on DIO10
    let gpiobase = 0x40022000;
    let doe = gpiobase + 0xD0;

    // Set DIO10 to output
    *(iocfg10 as *mut u16) = 0x7000;
    // Set DataEnable to 1
    *(doe as *mut u32) = 0x400;

    loop {
        // Set DIO10
        *((gpiobase + 0x90) as *mut u32) |= 1 << 10;

        // Small delay
        for _i in 0..0x7FFFFF {
            asm!("nop;");
        }

        // Clear DIO10
        *((gpiobase + 0xA0) as *mut u32) |= 1 << 10;

        // Small delay
        for _i in 0..0x7FFFFF {
            asm!("nop;");
        }
    }

    let platform = Platform { };
    let mut chip = cc2650::chip::Cc2650::new();

    debug!("Initialization complete. Entering main loop\r");
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    kernel::process::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
    kernel::main(
        &platform,
        &mut chip,
        &mut PROCESSES,
        &kernel::ipc::IPC::new(),
    );
}

#[cfg(not(test))]
#[no_mangle]
#[lang = "panic_fmt"]
pub unsafe extern "C" fn panic_fmt(_args: Arguments, _file: &'static str, _line: u32) -> ! {
    loop { }
}
