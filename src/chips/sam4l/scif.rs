/* scif.rs -- System control interface for SAM4L
 *
 * Author: Philip Levis
 * Date: Aug 2, 2015
 */

use core::prelude::*;
use core::intrinsics;
use nvic;
use chip;

#[repr(C, packed)]
#[allow(missing_copy_implementations)]
struct ScifRegisters {
    ier: u32,
    idr: u32,
    imr: u32,
    isr: u32,
    icr: u32,
    pclksr: u32,
    unlock: u32,
    cscr: u32,
    //0x20
    oscctrl0: u32,
    pll0: u32,
    dfll0conf: u32,
    dfll0val: u32,
    dfll0mul: u32,
    dfll0step: u32,
    dfll0ssg: u32,
    dfll0ratio: u32,
    //0x40
    dfll0sync: u32,
    rccr: u32,
    rcfastcfg: u32,
    rfcastsr: u32,
    rc80mcr: u32,
    reserved0: [u32; 4],
    // 0x64
    hrpcr: u32
    fpcr: u32
    fpmul: u32
    fpdiv: u32
    gcctrl0: u32
    gcctrl1: u32
    gcctrl2: u32
    // 0x80
    gcctrl3: u32
    gcctrl4: u32
    gcctrl5: u32
    gcctrl6: u32
    gcctrl7: u32
    gcctrl8: u32
    gcctrl9: u32
    gcctrl10: u32
    // 0xa0
    gcctrl11: u32
    //we leave out versions
}

pub const SCIF: isize = 0x400E0800;

#[allow(missing_copy_implementations)]
pub struct Scif {
    regs: &'static mut ScifRegisters,
}

#[repr(usize)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4
}

impl Scif {
    pub fn new() -> Scif {
        Scif {
            regs: unsafe { intrinsics::transmute(SCIF_BASE)},
        }
    }
}
