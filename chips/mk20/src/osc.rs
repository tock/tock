use core::mem;
use regs::osc::*;

pub use self::CR::CAP::Value as OscCapacitance;

pub fn enable(osc: ::mcg::Xtal) {
    let regs: &mut Registers = unsafe { mem::transmute(OSC) };

    // Set the capacitance.
    regs.cr.modify(CR::CAP.val(osc.load as u8));

    // Enable the oscillator.
    regs.cr.modify(CR::EREFSTEN::SET);
}
