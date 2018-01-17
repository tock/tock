use core::cell::Cell;
use core::mem;
use kernel::common::VolatileCell;
use kernel::hil;
use nvic;
use rcc;

#[repr(C, packed)]
struct Registers {
    cr1: VolatileCell<u32>,
    cr2: VolatileCell<u32>,
    smcr: VolatileCell<u32>,
    dier: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    egr: VolatileCell<u32>,
    ccmr: [VolatileCell<u32>; 2],
    ccer: VolatileCell<u32>,
    cnt: VolatileCell<u32>,
    psc: VolatileCell<u32>,
    arr: VolatileCell<u32>,
    reserved0: VolatileCell<u32>,
    ccr: [VolatileCell<u32>; 4],
    reserved1: VolatileCell<u32>,
    dcr: VolatileCell<u32>,
    dmar: VolatileCell<u32>,
}

const TIMER2_ADDRESS: usize = 0x40000000;

pub static mut TIMER2: AlarmTimer = AlarmTimer::new(TIMER2_ADDRESS,
                                                    rcc::Clock::APB1(rcc::APB1Clock::TIM2),
                                                    nvic::NvicIdx::TIM2);

pub struct AlarmTimer {
    registers: *mut Registers,
    clock: rcc::Clock,
    nvic: nvic::NvicIdx,
    client: Cell<Option<&'static hil::time::Client>>,
}

impl AlarmTimer {
    const fn new(base_addr: usize, clock: rcc::Clock, nvic: nvic::NvicIdx) -> AlarmTimer {
        AlarmTimer {
            registers: base_addr as *mut Registers,
            clock: clock,
            nvic: nvic,
            client: Cell::new(None),
        }
    }

    fn disable_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.dier.set(regs.dier.get() & !(1 << 1)); // clear CC1IE
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        // check if caused by CC1IF
        if regs.sr.get() & (1 << 1) != 0 {
            self.disable_interrupts();
            regs.sr.set(regs.sr.get() & !(1 << 1)); // clear CC1IF
            self.client.get().map(|cb| { cb.fired(); });
        }
    }
}

impl hil::Controller for AlarmTimer {
    type Config = &'static hil::time::Client;

    fn configure(&self, client: &'static hil::time::Client) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        self.client.set(Some(client));
        unsafe {
            rcc::enable_clock(self.clock);
            nvic::enable(self.nvic);
        }
        regs.cr1.set(0);
        regs.cr2.set(0);
        regs.arr.set(0xffff);
        regs.psc.set((rcc::get_frequency(self.clock) / 16000) - 1);
        regs.egr.set(1 << 0); // UG
        regs.cr1.set(regs.cr1.get() | (1 << 0)); // CEN
    }
}

impl hil::time::Time for AlarmTimer {
    type Frequency = hil::time::Freq16KHz;

    fn disable(&self) {
        self.disable_interrupts();
    }

    fn is_armed(&self) -> bool {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.dier.get() & (1 << 1) != 0 // CC1IE
    }
}

impl hil::time::Alarm for AlarmTimer {
    fn now(&self) -> u32 {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.cnt.get()
    }

    fn set_alarm(&self, tics: u32) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.ccr[0].set(tics & 0xffff);
        regs.dier.set(regs.dier.get() | (1 << 1)); // CC1IE
    }

    fn get_alarm(&self) -> u32 {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.ccr[0].get()
    }
}

interrupt_handler!(timer2_handler, TIM2);
