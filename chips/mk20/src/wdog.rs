//! Implementation of the MK20 hardware watchdog timer

use core::mem;
use kernel::hil;
use regs::wdog::*;

#[inline]
fn unlock() {
    let regs: &mut Registers = unsafe { mem::transmute(WDOG) };

    regs.unlock.write(UNLOCK::KEY::Key1);
    regs.unlock.write(UNLOCK::KEY::Key2);
    unsafe {
        asm!("nop" :::: "volatile");
        asm!("nop" :::: "volatile");
    }
}

#[allow(unused_variables)]
pub fn start(period: usize) {
    unimplemented!();
}

pub fn stop() {
    let regs: &mut Registers = unsafe { mem::transmute(WDOG) };

    // Must write the correct unlock sequence to the WDOG unlock register before reconfiguring
    // the module.
    unlock();

    // WDOG disabled in all power modes.
    // Allow future updates to the watchdog configuration.
    regs.stctrlh.modify(STCTRLH::ALLOWUPDATE::SET + 
                        STCTRLH::WAITEN::CLEAR +
                        STCTRLH::STOPEN::CLEAR +
                        STCTRLH::DBGEN::CLEAR +
                        STCTRLH::WDOGEN::CLEAR);
}

pub fn tickle() {
    let regs: &mut Registers = unsafe { mem::transmute(WDOG) };
    regs.refresh.write(REFRESH::KEY::Key1);
    regs.refresh.write(REFRESH::KEY::Key2);
}

pub struct Wdog;
impl hil::watchdog::Watchdog for Wdog {
    fn start(&self, period: usize) {
        start(period);
    }

    fn stop(&self) {
        stop();
    }

    fn tickle(&self) {
        tickle();
    }
}
