// On reset, MCGOUTCLK is sourced from the 32kHz internal reference clock 
// multiplied by the FLL, which has a default multiplier of 640.
static mut MCGOUTCLK: u32 = 20_480_000;

static mut CORECLK: u32 = 20_480_000;
static mut BUSCLK: u32 = 20_480_000;
static mut FLASHCLK: u32 = 10_240_000;

use osc;
use mcg;
use sim;

pub fn peripheral_clock_hz() -> u32 {
    unsafe { BUSCLK }
}

pub fn bus_clock_hz() -> u32 {
    unsafe { BUSCLK }
}

pub fn flash_clock_hz() -> u32 {
    unsafe { FLASHCLK }
}

pub fn core_clock_hz() -> u32 {
    unsafe { CORECLK }
}

#[allow(non_upper_case_globals)]
const MHz: u32 = 1_000_000;

pub fn configure(core_freq: u32) {
    if let mcg::State::Fei(fei) = mcg::state() {

        let (pll_mul, pll_div) = match core_freq {
            16 => (16, 8),
            20 => (20, 8),
            24 => (24, 8),
            28 => (28, 8),

            32 => (16, 4),
            36 => (18, 4),
            40 => (20, 4),
            44 => (22, 4),
            48 => (24, 4),
            52 => (26, 4),
            56 => (28, 4),
            60 => (30, 4),

            64 => (16, 2),
            68 => (17, 2),
            72 => (18, 2),
            76 => (19, 2),
            80 => (20, 2),
            84 => (21, 2),
            88 => (22, 2),
            92 => (23, 2),
            96 => (24, 2),
            100 => (25, 2),
            104 => (26, 2),
            108 => (27, 2),
            112 => (28, 2),
            116 => (29, 2),
            120 => (30, 2),
            _ => panic!("Invalid core frequency selected!")
        };

        let mut bus_div = 1;
        while core_freq / bus_div > 60 {
            bus_div += 1;
        }

        let mut flash_div = 1;
        while core_freq / flash_div > 28 {
            flash_div += 1;
        }

        osc::enable(mcg::xtals::Teensy16MHz);
        sim::set_dividers(1, bus_div, flash_div);

        let fbe = fei.use_xtal(mcg::xtals::Teensy16MHz);
        let pbe = fbe.enable_pll(pll_mul, pll_div);
        pbe.use_pll();

        unsafe {
            MCGOUTCLK = core_freq * MHz;
            CORECLK = core_freq * MHz;
            BUSCLK = (core_freq * MHz) / bus_div; 
            FLASHCLK = (core_freq * MHz) / flash_div;
        }

    } else {
        // We aren't in FEI mode, meaning that configuration has already occurred.
        // For now, just exit without changing the existing configuration.
        return;
    }
}
