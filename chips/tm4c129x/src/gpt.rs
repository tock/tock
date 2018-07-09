use kernel::common::cells::{OptionalCell, VolatileCell};
use kernel::common::StaticRef;
use kernel::hil;
use sysctl;

#[repr(C)]
struct GptRegisters {
    cfg: VolatileCell<u32>,
    tamr: VolatileCell<u32>,
    tbmr: VolatileCell<u32>,
    ctl: VolatileCell<u32>,
    sync: VolatileCell<u32>,
    _reserved0: [u32; 1],
    imr: VolatileCell<u32>,
    ris: VolatileCell<u32>,
    mis: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    tailr: VolatileCell<u32>,
    tbilr: VolatileCell<u32>,
    tamatchr: VolatileCell<u32>,
    tbmatchr: VolatileCell<u32>,
    tapr: VolatileCell<u32>,
    tbpr: VolatileCell<u32>,
    tapmr: VolatileCell<u32>,
    tbpmr: VolatileCell<u32>,
    tar: VolatileCell<u32>,
    tbr: VolatileCell<u32>,
    tav: VolatileCell<u32>,
    tbv: VolatileCell<u32>,
    rtcpd: VolatileCell<u32>,
    taps: VolatileCell<u32>,
    tbps: VolatileCell<u32>,
    _reserved1: [u32; 2],
    dmaev: VolatileCell<u32>,
    adcev: VolatileCell<u32>,
    _reserved2: [u32; 979],
    pp: VolatileCell<u32>,
    _reserved3: [u32; 1],
    cc: VolatileCell<u32>,
}

const TIMER0_BASE: StaticRef<GptRegisters> =
    unsafe { StaticRef::new(0x40030000 as *const GptRegisters) };

pub static mut TIMER0: AlarmTimer =
    AlarmTimer::new(TIMER0_BASE, sysctl::Clock::TIMER(sysctl::RCGCTIMER::TIMER0));

pub struct AlarmTimer {
    registers: StaticRef<GptRegisters>,
    clock: sysctl::Clock,
    client: OptionalCell<&'static hil::time::Client>,
}

impl AlarmTimer {
    const fn new(base_addr: StaticRef<GptRegisters>, clock: sysctl::Clock) -> AlarmTimer {
        AlarmTimer {
            registers: base_addr,
            clock: clock,
            client: OptionalCell::empty(),
        }
    }

    fn disable_interrupts(&self) {
        let regs = &*self.registers;
        regs.tamr.set(regs.tamr.get() & !(1 << 5)); // GPTM Timer A Match Interrupt
    }

    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        // check if caused by TAMMIS
        if regs.mis.get() & (1 << 4) != 0 {
            self.disable_interrupts();
            regs.icr.set(regs.icr.get() | (1 << 4));
            self.client.map(|cb| {
                cb.fired();
            });
        }
    }
}

impl hil::Controller for AlarmTimer {
    type Config = &'static hil::time::Client;

    fn configure(&self, client: &'static hil::time::Client) {
        unsafe {
            sysctl::enable_clock(self.clock);
        }

        self.client.set(client);
        let regs = &*self.registers;

        regs.ctl.set(0x0);
        regs.cfg.set(0x0);
        regs.tamr.set(regs.tamr.get() | 0x1012); // One-Shot count-up
        regs.tav.set(0x0);
        regs.tamatchr.set(0x0);
        regs.cc.set(0x1);
        regs.imr.set(regs.imr.get() | (1 << 4)); // TAMIM enable
        regs.ctl.set(regs.ctl.get() | (2 << 0));
        regs.ctl.set(regs.ctl.get() | (1 << 0)); // TAEN
    }
}

impl hil::time::Time for AlarmTimer {
    type Frequency = hil::time::Freq16MHz;

    fn disable(&self) {
        self.disable_interrupts();
    }

    fn is_armed(&self) -> bool {
        let regs = &*self.registers;
        regs.tamr.get() & (1 << 5) != 0
    }
}

impl hil::time::Alarm for AlarmTimer {
    fn now(&self) -> u32 {
        let regs = &*self.registers;
        regs.tar.get()
    }

    fn set_alarm(&self, tics: u32) {
        let regs = &*self.registers;
        regs.tamatchr.set(tics);
        regs.tamr.set(regs.tamr.get() | (1 << 5));
    }

    fn get_alarm(&self) -> u32 {
        let regs = &*self.registers;
        regs.tamatchr.get()
    }
}
