//! Always On Module (AON) management
//!
//! AON is a set of peripherals which is _always on_ (eg. the RTC, MCU, etc).
//!
//! The current configuration disables all wake-up selectors, since the
//! MCU never go to sleep and is always active.

use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;

#[repr(C)]
struct AonEventRegisters {
    mcu_wu_sel: VolatileCell<u32>,       // MCU Wake-up selector
    aux_wu_sel: VolatileCell<u32>,       // AUX Wake-up selector
    event_to_mcu_sel: VolatileCell<u32>, // Event selector for MCU Events
    rtc_sel: VolatileCell<u32>,          // RTC Capture event selector for AON_RTC
}

const AON_BASE: StaticRef<AonEventRegisters> =
    unsafe { StaticRef::new(0x40093000 as *const AonEventRegisters) };

pub struct AonEvent {
    registers: StaticRef<AonEventRegisters>,
}

pub static mut AON_EVENT: AonEvent = AonEvent::new();

impl AonEvent {
    const fn new() -> AonEvent {
        AonEvent {
            registers: AON_BASE,
        }
    }

    pub fn setup(&self) {
        let regs = &*self.registers;

        // Default to no events at all
        regs.aux_wu_sel.set(0x3F3F3F3F);
        regs.mcu_wu_sel.set(0x003F3F3F);
        regs.rtc_sel.set(0x0000003F);

        // The default reset value is 0x002B2B2B. However, 0x2b for each
        // programmable event corresponds to a JTAG event; which is fired
        // *all* the time during debugging through JTAG. It is better to
        // ignore it in this case.
        //      NOTE: the aon programmable interrupt will still be fired
        //            once a debugger is attached through JTAG.
        regs.event_to_mcu_sel.set(0x003F3F3F);
    }
}
