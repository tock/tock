//! Implementation of the GPIO controller.

use self::Pin::*;
use core::cell::Cell;
use core::mem;
use core::ops::{Index, IndexMut};
use kernel::common::VolatileCell;
use kernel::hil;
use rcc;

const SIZE: usize = 0x400;
const BASE_ADDRESS: usize = 0x40010800;

const CLOCKS: [rcc::APB2Clock; 7] = [rcc::APB2Clock::IOPA,
                                     rcc::APB2Clock::IOPB,
                                     rcc::APB2Clock::IOPC,
                                     rcc::APB2Clock::IOPD,
                                     rcc::APB2Clock::IOPE,
                                     rcc::APB2Clock::IOPF,
                                     rcc::APB2Clock::IOPG];

#[repr(C, packed)]
struct Registers {
    pub cr: [VolatileCell<u32>; 2],
    pub idr: VolatileCell<u32>,
    pub odr: VolatileCell<u32>,
    pub bsrr: VolatileCell<u32>,
    pub brr: VolatileCell<u32>,
    pub lckr: VolatileCell<u32>,
}

#[derive(Copy,Clone)]
pub enum Mode {
    Input(InputMode),
    Output10MHz(OutputMode),
    Output2MHz(OutputMode),
    Output50MHz(OutputMode),
}

#[derive(Copy,Clone)]
pub enum InputMode {
    Analog,
    Floating,
    PullUp,
    PullDown,
}

#[derive(Copy,Clone)]
pub enum OutputMode {
    PushPull,
    OpenDrain,
    AlternatePushPull,
    AlternateOpenDrain,
}

#[derive(Copy,Clone)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum Pin {
    PA00, PA01, PA02, PA03, PA04, PA05, PA06, PA07,
    PA08, PA09, PA10, PA11, PA12, PA13, PA14, PA15,

    PB00, PB01, PB02, PB03, PB04, PB05, PB06, PB07,
    PB08, PB09, PB10, PB11, PB12, PB13, PB14, PB15,

    PC00, PC01, PC02, PC03, PC04, PC05, PC06, PC07,
    PC08, PC09, PC10, PC11, PC12, PC13, PC14, PC15,

    PD00, PD01, PD02, PD03, PD04, PD05, PD06, PD07,
    PD08, PD09, PD10, PD11, PD12, PD13, PD14, PD15,

    PE00, PE01, PE02, PE03, PE04, PE05, PE06, PE07,
    PE08, PE09, PE10, PE11, PE12, PE13, PE14, PE15,

    PF00, PF01, PF02, PF03, PF04, PF05, PF06, PF07,
    PF08, PF09, PF10, PF11, PF12, PF13, PF14, PF15,

    PG00, PG01, PG02, PG03, PG04, PG05, PG06, PG07,
    PG08, PG09, PG10, PG11, PG12, PG13, PG14, PG15,
}

pub struct Port {
    pins: [GPIOPin; 16],
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

pub static mut PA: Port = Port {
    pins: [GPIOPin::new(PA00),
           GPIOPin::new(PA01),
           GPIOPin::new(PA02),
           GPIOPin::new(PA03),
           GPIOPin::new(PA04),
           GPIOPin::new(PA05),
           GPIOPin::new(PA06),
           GPIOPin::new(PA07),
           GPIOPin::new(PA08),
           GPIOPin::new(PA09),
           GPIOPin::new(PA10),
           GPIOPin::new(PA11),
           GPIOPin::new(PA12),
           GPIOPin::new(PA13),
           GPIOPin::new(PA14),
           GPIOPin::new(PA15)],
};

pub static mut PB: Port = Port {
    pins: [GPIOPin::new(PB00),
           GPIOPin::new(PB01),
           GPIOPin::new(PB02),
           GPIOPin::new(PB03),
           GPIOPin::new(PB04),
           GPIOPin::new(PB05),
           GPIOPin::new(PB06),
           GPIOPin::new(PB07),
           GPIOPin::new(PB08),
           GPIOPin::new(PB09),
           GPIOPin::new(PB10),
           GPIOPin::new(PB11),
           GPIOPin::new(PB12),
           GPIOPin::new(PB13),
           GPIOPin::new(PB14),
           GPIOPin::new(PB15)],
};

pub static mut PC: Port = Port {
    pins: [GPIOPin::new(PC00),
           GPIOPin::new(PC01),
           GPIOPin::new(PC02),
           GPIOPin::new(PC03),
           GPIOPin::new(PC04),
           GPIOPin::new(PC05),
           GPIOPin::new(PC06),
           GPIOPin::new(PC07),
           GPIOPin::new(PC08),
           GPIOPin::new(PC09),
           GPIOPin::new(PC10),
           GPIOPin::new(PC11),
           GPIOPin::new(PC12),
           GPIOPin::new(PC13),
           GPIOPin::new(PC14),
           GPIOPin::new(PC15)],
};

pub static mut PD: Port = Port {
    pins: [GPIOPin::new(PD00),
           GPIOPin::new(PD01),
           GPIOPin::new(PD02),
           GPIOPin::new(PD03),
           GPIOPin::new(PD04),
           GPIOPin::new(PD05),
           GPIOPin::new(PD06),
           GPIOPin::new(PD07),
           GPIOPin::new(PD08),
           GPIOPin::new(PD09),
           GPIOPin::new(PD10),
           GPIOPin::new(PD11),
           GPIOPin::new(PD12),
           GPIOPin::new(PD13),
           GPIOPin::new(PD14),
           GPIOPin::new(PD15)],
};

pub static mut PE: Port = Port {
    pins: [GPIOPin::new(PE00),
           GPIOPin::new(PE01),
           GPIOPin::new(PE02),
           GPIOPin::new(PE03),
           GPIOPin::new(PE04),
           GPIOPin::new(PE05),
           GPIOPin::new(PE06),
           GPIOPin::new(PE07),
           GPIOPin::new(PE08),
           GPIOPin::new(PE09),
           GPIOPin::new(PE10),
           GPIOPin::new(PE11),
           GPIOPin::new(PE12),
           GPIOPin::new(PE13),
           GPIOPin::new(PE14),
           GPIOPin::new(PE15)],
};

pub static mut PF: Port = Port {
    pins: [GPIOPin::new(PF00),
           GPIOPin::new(PF01),
           GPIOPin::new(PF02),
           GPIOPin::new(PF03),
           GPIOPin::new(PF04),
           GPIOPin::new(PF05),
           GPIOPin::new(PF06),
           GPIOPin::new(PF07),
           GPIOPin::new(PF08),
           GPIOPin::new(PF09),
           GPIOPin::new(PF10),
           GPIOPin::new(PF11),
           GPIOPin::new(PF12),
           GPIOPin::new(PF13),
           GPIOPin::new(PF14),
           GPIOPin::new(PF15)],
};

pub static mut PG: Port = Port {
    pins: [GPIOPin::new(PG00),
           GPIOPin::new(PG01),
           GPIOPin::new(PG02),
           GPIOPin::new(PG03),
           GPIOPin::new(PG04),
           GPIOPin::new(PG05),
           GPIOPin::new(PG06),
           GPIOPin::new(PG07),
           GPIOPin::new(PG08),
           GPIOPin::new(PG09),
           GPIOPin::new(PG10),
           GPIOPin::new(PG11),
           GPIOPin::new(PG12),
           GPIOPin::new(PG13),
           GPIOPin::new(PG14),
           GPIOPin::new(PG15)],
};

pub struct GPIOPin {
    port: *mut Registers,
    pin: usize,
    clock: usize,
    client: Cell<Option<&'static hil::gpio::Client>>,
    client_data: Cell<usize>,
}

impl GPIOPin {
    const fn new(pin: Pin) -> GPIOPin {
        GPIOPin {
            port: (BASE_ADDRESS + ((pin as usize) / 16) * SIZE) as *mut Registers,
            pin: (pin as usize) % 16,
            clock: (pin as usize) / 16,
            client: Cell::new(None),
            client_data: Cell::new(0),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    fn enable_afio(&self) {
        unsafe {
            rcc::enable_clock(rcc::Clock::APB2(rcc::APB2Clock::AFIO));
        }
    }

    fn set_cr(&self, mode: u32, config: u32) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        let bits = mode | (config << 2);
        let cr = &mut port.cr[self.pin / 8];
        let shift = 4 * (self.pin % 8);
        cr.set((cr.get() & !(0xf << shift)) | (bits << shift));
    }

    fn configure_input(&self, config: InputMode) {
        match config {
            InputMode::Analog => self.set_cr(0b00, 0b00),
            InputMode::Floating => self.set_cr(0b00, 0b01),
            InputMode::PullUp => {
                self.set_cr(0b00, 0b10);
                self.set()
            }
            InputMode::PullDown => {
                self.set_cr(0b00, 0b10);
                self.clear()
            }
        }
    }

    fn configure_output(&self, mode: u32, config: OutputMode) {
        match config {
            OutputMode::PushPull => self.set_cr(mode, 0b00),
            OutputMode::OpenDrain => self.set_cr(mode, 0b01),
            OutputMode::AlternatePushPull => {
                self.enable_afio();
                self.set_cr(mode, 0b10)
            }
            OutputMode::AlternateOpenDrain => {
                self.enable_afio();
                self.set_cr(mode, 0b11)
            }
        }
    }

    fn configure(&self, mode: Mode) {
        unsafe {
            rcc::enable_clock(rcc::Clock::APB2(CLOCKS[self.clock]));
        }
        match mode {
            Mode::Input(config) => self.configure_input(config),
            Mode::Output10MHz(config) => self.configure_output(0b01, config),
            Mode::Output2MHz(config) => self.configure_output(0b10, config),
            Mode::Output50MHz(config) => self.configure_output(0b11, config),
        }
    }

    pub fn read(&self) -> bool {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.idr.get() & (1 << self.pin) != 0
    }

    pub fn set(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.bsrr.set(1 << self.pin);
    }

    pub fn clear(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.brr.set(1 << self.pin);
    }

    pub fn toggle(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.odr.set(port.odr.get() ^ (1 << self.pin));
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
            hil::gpio::InputMode::PullNone => self.configure(Mode::Input(InputMode::Floating)),
            hil::gpio::InputMode::PullUp => self.configure(Mode::Input(InputMode::PullUp)),
            hil::gpio::InputMode::PullDown => self.configure(Mode::Input(InputMode::PullDown)),
        }
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn disable(&self) {
        self.configure(Mode::Input(InputMode::Floating));
    }

    fn make_output(&self) {
        self.configure(Mode::Output2MHz(OutputMode::PushPull));
    }

    fn make_input(&self) {
        self.configure(Mode::Input(InputMode::Floating));
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
        unimplemented!()
    }

    fn disable_interrupt(&self) {
        unimplemented!()
    }
}
