//! Implementation of the GPIO controller.

use self::Pin::*;
use core::cell::Cell;
use core::ops::{Index, IndexMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;
use kernel::hil;
use sysctl;

const CLOCKS: [sysctl::RCGCGPIO; 15] = [
    sysctl::RCGCGPIO::GPIOA,
    sysctl::RCGCGPIO::GPIOB,
    sysctl::RCGCGPIO::GPIOC,
    sysctl::RCGCGPIO::GPIOD,
    sysctl::RCGCGPIO::GPIOE,
    sysctl::RCGCGPIO::GPIOF,
    sysctl::RCGCGPIO::GPIOG,
    sysctl::RCGCGPIO::GPIOH,
    sysctl::RCGCGPIO::GPIOJ,
    sysctl::RCGCGPIO::GPIOK,
    sysctl::RCGCGPIO::GPIOL,
    sysctl::RCGCGPIO::GPIOM,
    sysctl::RCGCGPIO::GPION,
    sysctl::RCGCGPIO::GPIOP,
    sysctl::RCGCGPIO::GPIOQ,
];

#[repr(C)]
struct GpioRegisters {
    _reserved0: [u32; 255],
    data: VolatileCell<u32>, //Verbesserungspotenzial Data Direction Operation
    dir: VolatileCell<u32>,
    is: VolatileCell<u32>,
    ibe: VolatileCell<u32>,
    iev: VolatileCell<u32>,
    im: VolatileCell<u32>,
    ris: VolatileCell<u32>,
    mis: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    afsel: VolatileCell<u32>,
    _reserved1: [u32; 55],
    dr2r: VolatileCell<u32>,
    dr4r: VolatileCell<u32>,
    dr8r: VolatileCell<u32>,
    odr: VolatileCell<u32>,
    pur: VolatileCell<u32>,
    pdr: VolatileCell<u32>,
    slr: VolatileCell<u32>,
    den: VolatileCell<u32>,
    lock: VolatileCell<u32>,
    cr: VolatileCell<u32>,
    amsel: VolatileCell<u32>,
    pctl: VolatileCell<u32>,
    adcctl: VolatileCell<u32>,
    dmactl: VolatileCell<u32>,
    si: VolatileCell<u32>,
    dr12r: VolatileCell<u32>,
    wakepen: VolatileCell<u32>,
    wakelvl: VolatileCell<u32>,
    wakestat: VolatileCell<u32>,
    _reserved2: [u32; 669],
    pp: VolatileCell<u32>,
    pc: VolatileCell<u32>,
}

#[derive(Copy, Clone)]
pub enum Mode {
    Input(InputMode),
    Output(OutputMode),
    InputOutput(InputOutputMode),
}

#[derive(Copy, Clone)]
pub enum InputMode {
    Digital,
    DigitalAfsel,
    Analog,
}

#[derive(Copy, Clone)]
pub enum OutputMode {
    Digital,
    DigitalAfsel,
    OpenDrain,
}

#[derive(Copy, Clone)]
pub enum InputOutputMode {
    DigitalAfsel,
    OpenDrainAfsel,
}

/// Peripheral functions that may be assigned to a `GPIOPin`.
#[derive(Copy, Clone)]
pub enum PeripheralFunction {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
    I = 9,
    J = 10,
    K = 11,
    L = 12,
    M = 13,
    N = 14,
    O = 15,
}

const BASE_ADDRESS: usize = 0x40058000;
const SIZE: usize = 0x00001000;

/// This is an `AtomicUsize` because it has to be a `Sync` type to live in a
/// global---Rust has no way of knowing we're not going to use it across
/// threads. Use `Ordering::Relaxed` when reading/writing the value to get LLVM
/// to just use plain loads and stores instead of atomic operations.
pub static INTERRUPT_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy,Clone)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum Pin {
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7,
    PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7,
    PC0, PC1, PC2, PC3, PC4, PC5, PC6, PC7,
    PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7,
    PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7,
    PF0, PF1, PF2, PF3, PF4, PF5, PF6, PF7,
    PG0, PG1, PG2, PG3, PG4, PG5, PG6, PG7,
    PH0, PH1, PH2, PH3, PH4, PH5, PH6, PH7,
    PJ0, PJ1, PJ2, PJ3, PJ4, PJ5, PJ6, PJ7,
    PK0, PK1, PK2, PK3, PK4, PK5, PK6, PK7,
    PL0, PL1, PL2, PL3, PL4, PL5, PL6, PL7,
    PM0, PM1, PM2, PM3, PM4, PM5, PM6, PM7,
    PN0, PN1, PN2, PN3, PN4, PN5, PN6, PN7,
    PP0, PP1, PP2, PP3, PP4, PP5, PP6, PP7,
    PQ0, PQ1, PQ2, PQ3, PQ4, PQ5, PQ6, PQ7,
}

pub struct Port {
    registers: StaticRef<GpioRegisters>,
    pins: [GPIOPin; 8],
}

impl Index<usize> for Port {
    type Output = GPIOPin;

    fn index(&self, index: usize) -> &GPIOPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GPIOPin {
        &mut self.pins[index]
    }
}

impl Port {
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;

        let mut fired = regs.ris.get() & regs.im.get();

        regs.icr.set(0xFF);

        loop {
            let pin = fired.trailing_zeros() as usize;
            if pin < self.pins.len() {
                fired &= !(1 << pin);
                self.pins[pin].handle_interrupt();
            } else {
                break;
            }
        }
    }
}

/// Port A
pub static mut PA: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 0 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PA0),
        GPIOPin::new(PA1),
        GPIOPin::new(PA2),
        GPIOPin::new(PA3),
        GPIOPin::new(PA4),
        GPIOPin::new(PA5),
        GPIOPin::new(PA6),
        GPIOPin::new(PA7),
    ],
};

/// Port B
pub static mut PB: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 1 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PB0),
        GPIOPin::new(PB1),
        GPIOPin::new(PB2),
        GPIOPin::new(PB3),
        GPIOPin::new(PB4),
        GPIOPin::new(PB5),
        GPIOPin::new(PB6),
        GPIOPin::new(PB7),
    ],
};

//// Port C
pub static mut PC: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 2 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PC0),
        GPIOPin::new(PC1),
        GPIOPin::new(PC2),
        GPIOPin::new(PC3),
        GPIOPin::new(PC4),
        GPIOPin::new(PC5),
        GPIOPin::new(PC6),
        GPIOPin::new(PC7),
    ],
};

//// Port D
pub static mut PD: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 3 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PD0),
        GPIOPin::new(PD1),
        GPIOPin::new(PD2),
        GPIOPin::new(PD3),
        GPIOPin::new(PD4),
        GPIOPin::new(PD5),
        GPIOPin::new(PD6),
        GPIOPin::new(PD7),
    ],
};

//// Port E
pub static mut PE: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 4 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PE0),
        GPIOPin::new(PE1),
        GPIOPin::new(PE2),
        GPIOPin::new(PE3),
        GPIOPin::new(PE4),
        GPIOPin::new(PE5),
        GPIOPin::new(PE6),
        GPIOPin::new(PE7),
    ],
};

//// Port F
pub static mut PF: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 5 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PF0),
        GPIOPin::new(PF1),
        GPIOPin::new(PF2),
        GPIOPin::new(PF3),
        GPIOPin::new(PF4),
        GPIOPin::new(PF5),
        GPIOPin::new(PF6),
        GPIOPin::new(PF7),
    ],
};

//// Port G
pub static mut PG: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 6 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PG0),
        GPIOPin::new(PG1),
        GPIOPin::new(PG2),
        GPIOPin::new(PG3),
        GPIOPin::new(PG4),
        GPIOPin::new(PG5),
        GPIOPin::new(PG6),
        GPIOPin::new(PG7),
    ],
};

//// Port H
pub static mut PH: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 7 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PH0),
        GPIOPin::new(PH1),
        GPIOPin::new(PH2),
        GPIOPin::new(PH3),
        GPIOPin::new(PH4),
        GPIOPin::new(PH5),
        GPIOPin::new(PH6),
        GPIOPin::new(PH7),
    ],
};

//// Port J
pub static mut PJ: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 8 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PJ0),
        GPIOPin::new(PJ1),
        GPIOPin::new(PJ2),
        GPIOPin::new(PJ3),
        GPIOPin::new(PJ4),
        GPIOPin::new(PJ5),
        GPIOPin::new(PJ6),
        GPIOPin::new(PJ7),
    ],
};

//// Port K
pub static mut PK: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 9 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PK0),
        GPIOPin::new(PK1),
        GPIOPin::new(PK2),
        GPIOPin::new(PK3),
        GPIOPin::new(PK4),
        GPIOPin::new(PK5),
        GPIOPin::new(PK6),
        GPIOPin::new(PK7),
    ],
};
//// Port L
pub static mut PL: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 10 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PL0),
        GPIOPin::new(PL1),
        GPIOPin::new(PL2),
        GPIOPin::new(PL3),
        GPIOPin::new(PL4),
        GPIOPin::new(PL5),
        GPIOPin::new(PL6),
        GPIOPin::new(PL7),
    ],
};
//// Port M
pub static mut PM: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 11 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PM0),
        GPIOPin::new(PM1),
        GPIOPin::new(PM2),
        GPIOPin::new(PM3),
        GPIOPin::new(PM4),
        GPIOPin::new(PM5),
        GPIOPin::new(PM6),
        GPIOPin::new(PM7),
    ],
};

//// Port N
pub static mut PN: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 12 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PN0),
        GPIOPin::new(PN1),
        GPIOPin::new(PN2),
        GPIOPin::new(PN3),
        GPIOPin::new(PN4),
        GPIOPin::new(PN5),
        GPIOPin::new(PN6),
        GPIOPin::new(PN7),
    ],
};

//// Port P
pub static mut PP: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 13 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PP0),
        GPIOPin::new(PP1),
        GPIOPin::new(PP2),
        GPIOPin::new(PP3),
        GPIOPin::new(PP4),
        GPIOPin::new(PP5),
        GPIOPin::new(PP6),
        GPIOPin::new(PP7),
    ],
};

//// Port Q
pub static mut PQ: Port = Port {
    registers: unsafe { StaticRef::new((BASE_ADDRESS + 14 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(PQ0),
        GPIOPin::new(PQ1),
        GPIOPin::new(PQ2),
        GPIOPin::new(PQ3),
        GPIOPin::new(PQ4),
        GPIOPin::new(PQ5),
        GPIOPin::new(PQ6),
        GPIOPin::new(PQ7),
    ],
};

pub struct GPIOPin {
    registers: StaticRef<GpioRegisters>,
    pin: usize,
    clock: usize,
    client: Cell<Option<&'static hil::gpio::Client>>,
    client_data: Cell<usize>,
}

impl GPIOPin {
    const fn new(pin: Pin) -> GPIOPin {
        GPIOPin {
            registers: unsafe {
                StaticRef::new((BASE_ADDRESS + ((pin as usize) / 8) * SIZE) as *const GpioRegisters)
            },
            pin: (pin as usize) % 8,
            clock: (pin as usize) / 8,
            client: Cell::new(None),
            client_data: Cell::new(0),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    fn configure_input(&self, config: InputMode) {
        match config {
            InputMode::Digital => self.enable_digital(),
            InputMode::DigitalAfsel => {
                self.enable_digital();
                self.enable_alternate()
            }
            InputMode::Analog => self.enable_analog(),
        }
    }

    fn configure_output(&self, config: OutputMode) {
        match config {
            OutputMode::Digital => {
                self.enable_output();
                self.enable_digital();
            }
            OutputMode::DigitalAfsel => {
                self.enable_digital();
                self.enable_alternate();
            }
            OutputMode::OpenDrain => {
                self.enable_output();
                self.enable_digital();
                self.enable_opendrain();
            }
        }
    }

    fn configure_inputoutput(&self, config: InputOutputMode) {
        match config {
            InputOutputMode::DigitalAfsel => {
                self.enable_digital();
                self.enable_alternate();
            }
            InputOutputMode::OpenDrainAfsel => {
                self.enable_alternate();
                self.enable_digital();
                self.enable_opendrain();
            }
        }
    }

    pub fn configure(&self, mode: Mode) {
        unsafe {
            sysctl::enable_clock(sysctl::Clock::GPIO(CLOCKS[self.clock]));
        }
        match mode {
            Mode::Input(config) => self.configure_input(config),
            Mode::Output(config) => self.configure_output(config),
            Mode::InputOutput(config) => self.configure_inputoutput(config),
        }
    }

    pub fn enable_analog(&self) {
        let regs = &*self.registers;
        regs.amsel.set(regs.amsel.get() | (1 << self.pin));
    }

    pub fn disable_analog(&self) {
        let regs = &*self.registers;
        regs.amsel.set(regs.amsel.get() & !(1 << self.pin));
    }

    pub fn enable_digital(&self) {
        let regs = &*self.registers;
        regs.den.set(regs.den.get() | (1 << self.pin));
    }

    pub fn disable_digital(&self) {
        let regs = &*self.registers;
        regs.den.set(regs.den.get() & !(1 << self.pin));
    }

    pub fn enable_opendrain(&self) {
        let regs = &*self.registers;
        regs.odr.set(regs.odr.get() | (1 << self.pin));
    }

    pub fn disable_opendrain(&self) {
        let regs = &*self.registers;
        regs.odr.set(regs.odr.get() & !(1 << self.pin));
    }

    pub fn enable_alternate(&self) {
        let regs = &*self.registers;
        regs.afsel.set(1 << self.pin);
        regs.pctl.set(regs.pctl.get() | (1 << self.pin * 4));
    }

    pub fn disable_alternate(&self) {
        let regs = &*self.registers;
        regs.afsel.set(regs.afsel.get() & !(1 << self.pin));
    }

    pub fn enable_output(&self) {
        let regs = &*self.registers;
        regs.dir.set(regs.dir.get() | (1 << self.pin));
    }

    pub fn disable_output(&self) {
        let regs = &*self.registers;
        regs.dir.set(regs.dir.get() & !(1 << self.pin));
    }

    pub fn enable_pull_down(&self) {
        let regs = &*self.registers;
        regs.pdr.set(regs.pdr.get() | (1 << self.pin));
    }

    pub fn disable_pull_down(&self) {
        let regs = &*self.registers;
        regs.pdr.set(regs.pdr.get() & !(1 << self.pin));
    }

    pub fn enable_pull_up(&self) {
        let regs = &*self.registers;
        regs.pur.set(regs.pur.get() | (1 << self.pin));
    }

    pub fn disable_pull_up(&self) {
        let regs = &*self.registers;
        regs.pur.set(regs.pur.get() & !(1 << self.pin));
    }

    /// | `mode` value |  Mode |
    /// | ------------ | -------------- |
    /// | 0b00         | Both edges     |
    /// | 0b01         | Rising edge    |
    /// | 0b10         | Falling edge   |

    pub fn set_interrupt_mode(&self, mode: u8) {
        let regs = &*self.registers;

        if mode == 0b00 {
            regs.is.set(0x0);
            regs.ibe.set(regs.ibe.get() | (1 << self.pin));
        } else if mode == 0b01 {
            regs.is.set(0x0);
            regs.iev.set(regs.iev.get() | (1 << self.pin));
        } else if mode == 0b10 {
            regs.is.set(0x0);
            regs.iev.set(regs.iev.get() & !(1 << self.pin));
        }
    }

    pub fn enable_interrupt(&self) {
        let regs = &*self.registers;
        if regs.im.get() & (1 << self.pin) == 0 {
            INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
            regs.im.set(regs.im.get() | (1 << self.pin));
        }
    }

    pub fn disable_interrupt(&self) {
        let regs = &*self.registers;
        if regs.im.get() & (1 << self.pin) != 0 {
            INTERRUPT_COUNT.fetch_sub(1, Ordering::Relaxed);
            regs.im.set(regs.iev.get() & !(1 << self.pin));
        }
    }

    pub fn handle_interrupt(&self) {
        self.client.get().map(|client| {
            client.fired(self.client_data.get());
        });
    }

    pub fn read(&self) -> bool {
        let regs = &*self.registers;
        regs.data.get() & (1 << self.pin) != 0
    }

    pub fn toggle(&self) {
        let regs = &*self.registers;
        regs.data.set(regs.data.get() ^ (1 << self.pin));
    }

    pub fn set(&self) {
        let regs = &*self.registers;
        regs.data.set(regs.data.get() | (1 << self.pin));
    }

    pub fn clear(&self) {
        let regs = &*self.registers;
        regs.data.set(regs.data.get() & !(1 << self.pin));
    }
}

impl hil::Controller for GPIOPin {
    type Config = Mode;

    fn configure(&self, config: Self::Config) {
        GPIOPin::configure(self, config)
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        match mode {
            hil::gpio::InputMode::PullUp => {
                self.disable_pull_down();
                self.enable_pull_up();
            }
            hil::gpio::InputMode::PullDown => {
                self.disable_pull_up();
                self.enable_pull_down();
            }

            hil::gpio::InputMode::PullNone => {
                self.disable_pull_up();
                self.disable_pull_down();
            }
        }
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn disable(&self) {
        self.configure(Mode::Input(InputMode::Analog));
    }

    fn make_output(&self) {
        self.configure(Mode::Output(OutputMode::Digital));
    }

    fn make_input(&self) {
        self.configure(Mode::Input(InputMode::Digital));
    }

    fn read(&self) -> bool {
        GPIOPin::read(self)
    }

    fn toggle(&self) {
        GPIOPin::toggle(self);
    }

    fn set(&self) {
        GPIOPin::set(self);
    }

    fn clear(&self) {
        GPIOPin::clear(self);
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        let mode_bits = match mode {
            hil::gpio::InterruptMode::EitherEdge => 0b00,
            hil::gpio::InterruptMode::RisingEdge => 0b01,
            hil::gpio::InterruptMode::FallingEdge => 0b10,
        };
        self.client_data.set(client_data);
        GPIOPin::set_interrupt_mode(self, mode_bits);
        GPIOPin::enable_interrupt(self);
    }

    fn disable_interrupt(&self) {
        GPIOPin::disable_interrupt(self);
    }
}
