// General Purpose Input/Output (GPIO)

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

const GPIO_BASES: [StaticRef<GpioRegisters>; 6] = [
    unsafe { StaticRef::new(0x4000_4C00 as *const GpioRegisters) }, // PORT 1&2
    unsafe { StaticRef::new(0x4000_4C20 as *const GpioRegisters) }, // PORT 3&4
    unsafe { StaticRef::new(0x4000_4C40 as *const GpioRegisters) }, // PORT 5&6
    unsafe { StaticRef::new(0x4000_4C60 as *const GpioRegisters) }, // PORT 7&8
    unsafe { StaticRef::new(0x4000_4C80 as *const GpioRegisters) }, // PORT 9&10
    unsafe { StaticRef::new(0x4000_4D20 as *const GpioRegisters) }, // PORT J
];

const PINS_PER_PORT: u8 = 8;

#[repr(C)]
struct GpioRegisters {
    input: [ReadOnly<u8, PxIN::Register>; 2],
    out: [ReadWrite<u8, PxOUT::Register>; 2],
    dir: [ReadWrite<u8, PxDIR::Register>; 2],
    ren: [ReadWrite<u8, PxREN::Register>; 2],
    ds: [ReadWrite<u8, PxDS::Register>; 2],
    sel0: [ReadWrite<u8, PxSEL0::Register>; 2],
    sel1: [ReadWrite<u8, PxSEL1::Register>; 2],
    iv1: ReadWrite<u16, PxIV::Register>,
    _reserved: [u8; 6],
    selc: [ReadWrite<u8, PxSELC::Register>; 2],
    ies: [ReadWrite<u8, PxIES::Register>; 2],
    ie: [ReadWrite<u8, PxIE::Register>; 2],
    ifg: [ReadWrite<u8, PxIFG::Register>; 2],
    iv2: ReadWrite<u16, PxIV::Register>,
}

register_bitfields! [u8,
    // input-register, get input-status of pins
    PxIN [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // output-register, set output status of pins
    PxOUT [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // direction-register, set direction of pins
    PxDIR [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // pull-register, enable/disable pullup- or -down resistor
    PxREN [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // drive-strength register, select high(1) or low(0) drive-strength
    PxDS [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // function-selection register 0, combined with function-selection 1 the
    // module function is selected (GPIO, primary, secondary or tertiary)
    PxSEL0 [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // function-selection register 1, combined with function-selection 0 the
    // module function is selected (GPIO, primary, secondary or tertiary)
    PxSEL1 [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // complement selection, set a bit in PxSEL0 and PxSEL1 concurrently
    PxSELC [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // interrupt-edge selction, 0=rising-edge, 1=falling-edge
    PxIES [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // interrupt enable register
    PxIE [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ],
    // interrupt flag register
    PxIFG [
        PIN0 OFFSET(0) NUMBITS(1),
        PIN1 OFFSET(1) NUMBITS(1),
        PIN2 OFFSET(2) NUMBITS(1),
        PIN3 OFFSET(3) NUMBITS(1),
        PIN4 OFFSET(4) NUMBITS(1),
        PIN5 OFFSET(5) NUMBITS(1),
        PIN6 OFFSET(6) NUMBITS(1),
        PIN7 OFFSET(7) NUMBITS(1)
    ]
];

register_bitfields! [u16,
    // interrupt vector register
    PxIV [
        IV OFFSET(0) NUMBITS(16)
    ]
];

#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PinNr {
    P01_0, P01_1, P01_2, P01_3, P01_4, P01_5, P01_6, P01_7,
    P02_0, P02_1, P02_2, P02_3, P02_4, P02_5, P02_6, P02_7,
    P03_0, P03_1, P03_2, P03_3, P03_4, P03_5, P03_6, P03_7,
    P04_0, P04_1, P04_2, P04_3, P04_4, P04_5, P04_6, P04_7,
    P05_0, P05_1, P05_2, P05_3, P05_4, P05_5, P05_6, P05_7,
    P06_0, P06_1, P06_2, P06_3, P06_4, P06_5, P06_6, P06_7,
    P07_0, P07_1, P07_2, P07_3, P07_4, P07_5, P07_6, P07_7,
    P08_0, P08_1, P08_2, P08_3, P08_4, P08_5, P08_6, P08_7,
    P09_0, P09_1, P09_2, P09_3, P09_4, P09_5, P09_6, P09_7,
    P10_0, P10_1, P10_2, P10_3, P10_4, P10_5, P10_6, P10_7,
    PJ_0,  PJ_1,  PJ_2,  PJ_3,  PJ_4,  PJ_5,  PJ_6,  PJ_7,
}

pub struct Pin {
    pin: u8,
    port: u8,
    reg_idx: usize,
    registers: StaticRef<GpioRegisters>,
    client: OptionalCell<&'static dyn hil::gpio::Client>,
}

impl Pin {
    pub const fn new(pin: PinNr) -> Pin {
        let pin = (pin as u8) % PINS_PER_PORT;
        let port = pin / PINS_PER_PORT;
        Pin {
            pin: pin,
            port: port,
            reg_idx: (port % 2) as usize,
            registers: GPIO_BASES[(port / 2) as usize],
            client: OptionalCell::empty(),
        }
    }
}

impl hil::gpio::Input for Pin {
    fn read(&self) -> bool {
        (self.registers.input[self.reg_idx].get() & (1 << self.pin)) > 0
    }
}

impl hil::gpio::Output for Pin {
    fn set(&self) {
        let mut val = self.registers.out[self.reg_idx].get();
        val |= 1 << self.pin;
        self.registers.out[self.reg_idx].set(val);
    }

    fn clear(&self) {
        let mut val = self.registers.out[self.reg_idx].get();
        val &= !(1 << self.pin);
        self.registers.out[self.reg_idx].set(val);
    }

    fn toggle(&self) -> bool {
        let mut val = self.registers.out[self.reg_idx].get();
        val ^= 1 << self.pin;
        self.registers.out[self.reg_idx].set(val);
        (val & (1 << self.pin)) > 0
    }
}

impl hil::gpio::Configure for Pin {
    fn configuration(&self) -> hil::gpio::Configuration {
        let regs = &*self.registers;
        let dir = regs.dir[self.reg_idx].get();
        let mut sel = ((regs.sel0[self.reg_idx].get() & (1 << self.pin)) > 0) as u8;
        sel |= (((regs.sel1[self.reg_idx].get() & (1 << self.pin)) > 0) as u8) << 1;

        if sel > 0 {
            hil::gpio::Configuration::Function
        } else {
            if (dir & (1 << self.pin)) > 0 {
                hil::gpio::Configuration::Output
            } else {
                hil::gpio::Configuration::Input
            }
        }
    }

    fn make_output(&self) -> hil::gpio::Configuration {
        let mut val = self.registers.dir[self.reg_idx].get();
        val |= 1 << self.pin;
        self.registers.dir[self.reg_idx].set(val);
        hil::gpio::Configuration::Output
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        self.make_input()
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        let mut val = self.registers.dir[self.reg_idx].get();
        val &= !(1 << self.pin);
        self.registers.dir[self.reg_idx].set(val);
        hil::gpio::Configuration::Input
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        // it's not possible to deactivate the pin at all, so just return the
        // current configuration
        self.configuration()
    }

    fn deactivate_to_low_power(&self) {
        // the chip doesn't support any low-power, so set it to input with
        // a pullup resistor which should not consume much current
        self.make_input();
        self.set_floating_state(hil::gpio::FloatingState::PullUp);
    }

    fn set_floating_state(&self, state: hil::gpio::FloatingState) {
        let regs = &*self.registers;
        let mut ren = regs.ren[self.reg_idx].get();
        let mut out = regs.out[self.reg_idx].get();
        match state {
            hil::gpio::FloatingState::PullDown => {
                ren |= 1 << self.pin;
                out &= !(1 << self.pin);
            }
            hil::gpio::FloatingState::PullUp => {
                ren |= 1 << self.pin;
                out |= 1 << self.pin;
            }
            hil::gpio::FloatingState::PullNone => {
                ren &= !(1 << self.pin);
            }
            _ => {}
        }
        regs.ren[self.reg_idx].set(ren);
        regs.out[self.reg_idx].set(out);
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        let ren = self.registers.ren[self.reg_idx].get();
        let out = self.registers.out[self.reg_idx].get();

        if (ren & (1 << self.pin)) > 0 {
            if (out & (1 << self.pin)) > 0 {
                hil::gpio::FloatingState::PullUp
            } else {
                hil::gpio::FloatingState::PullDown
            }
        } else {
            hil::gpio::FloatingState::PullNone
        }
    }
}
