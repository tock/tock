//use core::cell::Cell;
//use dma::{DMAChannel};
use kernel::common::VolatileCell;
//use kernel::common::take_cell::TakeCell;
use kernel::ClockInterface;
use kernel::{MMIOInterface, MMIOManager};

//use kernel::hil;
use pm;





// NON GENERIC
struct TESTRegisterManager <'a> {
    registers: &'a TESTRegisters,
    clock: &'a ClockInterface<PlatformClockType=pm::Clock>,
}

impl<'a> TESTRegisterManager <'a> {
    fn new (hw: &'a TESTHw) -> TESTRegisterManager <'a> {
        let clock = &hw.clock;
        // If clock isn't enabled, lets enable it
        if clock.is_enabled() == false {
            debug!("TEST: Master clock on");
            clock.enable();
        }
        TESTRegisterManager {
            registers: unsafe { &*hw.registers },
            clock: clock,
        }
    }
}

impl<'a> Drop for TESTRegisterManager <'a> {
    fn drop(&mut self) {
        let mask = self.registers.interrupt_mask.get();
        if mask == 0 {
            debug!("TEST: Master clock off");
            self.clock.disable();
        }
        else {
            debug!("TEST: Master clock left on");
        }
    }
}
/////////////////////////



///// FAKE PERIPHERAL
#[repr(C, packed)]
#[allow(dead_code)]
pub struct TESTRegisters {
    control: VolatileCell<u32>,
    interrupt_mask: VolatileCell<u32>,
}

pub struct TESTHw {
    registers: *mut TESTRegisters,
    clock: pm::Clock,
    //dma: Cell<Option<&'static DMAChannel>>,
}

impl TESTHw {
    const fn new(base_addr: *mut TESTRegisters,
                 clock: pm::Clock,
                 ) -> TESTHw {
        TESTHw {
            registers: base_addr as *mut TESTRegisters,
            clock: clock,
        }
    }

    pub fn do_thing(&self) {
        let regs_manager = &TESTRegisterManager::new(&self); // use of non-gen
        let rm2 = &MMIOManager::new(self);                   // use of generic
        regs_manager.registers.control.get();
        rm2.registers.control.get();
    }
}
///////////////////////////////




impl MMIOInterface<pm::Clock> for TESTHw {
    type MMIORegisterType = TESTRegisters;
    type MMIOClockType = pm::Clock;

    fn get_hardware_address(&self) -> *mut TESTRegisters {
        self.registers
    }

    fn get_clock(&self) -> &pm::Clock {
        &self.clock
    }
}

const TEST_BASE_ADDR: *mut TESTRegisters = 0x40001000 as *mut TESTRegisters;
pub static mut TEST0: TESTHw = TESTHw::new(TEST_BASE_ADDR,
                                           pm::Clock::PBA(pm::PBAClock::TWIM0),
                                           );
