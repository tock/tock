use helpers::*;

use core::mem;
use hil;

use self::Pin::*;

#[repr(C, packed)]
struct Register {
    val: u32,
    set: u32,
    clear: u32,
    toggle: u32
}

#[repr(C, packed)]
struct RegisterRC {
    val: u32,
    reserved0: u32,
    clear: u32,
    reserved1: u32
}

#[repr(C, packed)]
struct Registers {
    gper: Register,
    pmr0: Register,
    pmr1: Register,
    pmr2: Register,
    oder: Register,
    ovr: Register,
    pvr: u32,
    _reserverd0: [u32; 3],
    puer: Register,
    pder: Register,
    ier: Register,
    imr0: Register,
    imr1: Register,
    gfer: Register,
    ifr: RegisterRC,
    _reserved1: [u32; 8],
    ocdr0: Register,
    ocdr1: Register,
    _reserved2: [u32; 4],
    osrr0: Register,
    _reserved3: [u32; 8],
    ster: Register,
    _reserved4: [u32; 4],
    ever: Register,
    _reserved5: [u32; 26],
    parameter: u32,
    version: u32,
}

/// Peripheral functions that may be assigned to a `GPIOPin`.
///
/// GPIO pins on the SAM4L may serve multiple functions. In addition to the
/// default functionality, each pin can be assigned up to eight different
/// peripheral functions. The various functions for each pin are described in
/// "Peripheral Multiplexing I/O Lines" section of the SAM4L datasheet[^1].
///
/// [^1]: Section 3.2, pages 19-29
#[derive(Copy,Clone)]
pub enum PeripheralFunction {
    A, B, C, D, E, F, G
}


const BASE_ADDRESS: usize = 0x400E1000;
const SIZE: usize = 0x200;

/// Name of the GPIO pin on the SAM4L.
///
/// The "Package and Pinout" section[^1] of the SAM4L datasheet shows the mapping
/// between these names and hardware pins on different chip packages.
///
/// [^1]: Section 3.1, pages 10-18
#[derive(Copy,Clone)]
pub enum Pin {
    PA00, PA01, PA02, PA03, PA04, PA05, PA06, PA07,
    PA08, PA09, PA10, PA11, PA12, PA13, PA14, PA15,
    PA16, PA17, PA18, PA19, PA20, PA21, PA22, PA23,
    PA24, PA25, PA26, PA27, PA28, PA29, PA30, PA31,

    PB00, PB01, PB02, PB03, PB04, PB05, PB06, PB07,
    PB08, PB09, PB10, PB11, PB12, PB13, PB14, PB15,
    PB16, PB17, PB18, PB19, PB20, PB21, PB22, PB23,
    PB24, PB25, PB26, PB27, PB28, PB29, PB30, PB31,

    PC00, PC01, PC02, PC03, PC04, PC05, PC06, PC07,
    PC08, PC09, PC10, PC11, PC12, PC13, PC14, PC15,
    PC16, PC17, PC18, PC19, PC20, PC21, PC22, PC23,
    PC24, PC25, PC26, PC27, PC28, PC29, PC30, PC31,
}

pub struct GPIOPin {
    port: *mut Registers,
    pin_mask: u32
}

/// Port A `GPIOPin`s
pub static mut PA : [GPIOPin; 32] = [
    GPIOPin::new(PA00), GPIOPin::new(PA01), GPIOPin::new(PA02),
    GPIOPin::new(PA03), GPIOPin::new(PA04), GPIOPin::new(PA05),
    GPIOPin::new(PA06), GPIOPin::new(PA07), GPIOPin::new(PA08),
    GPIOPin::new(PA09), GPIOPin::new(PA10), GPIOPin::new(PA11),
    GPIOPin::new(PA12), GPIOPin::new(PA13), GPIOPin::new(PA14),
    GPIOPin::new(PA15), GPIOPin::new(PA16), GPIOPin::new(PA17),
    GPIOPin::new(PA18), GPIOPin::new(PA19), GPIOPin::new(PA20),
    GPIOPin::new(PA21), GPIOPin::new(PA22), GPIOPin::new(PA23),
    GPIOPin::new(PA24), GPIOPin::new(PA25), GPIOPin::new(PA26),
    GPIOPin::new(PA27), GPIOPin::new(PA28), GPIOPin::new(PA29),
    GPIOPin::new(PA30), GPIOPin::new(PA31)
];

/// Port B `GPIOPin`s
pub static mut PB : [GPIOPin; 32] = [
    GPIOPin::new(PB00), GPIOPin::new(PB01), GPIOPin::new(PB02),
    GPIOPin::new(PB03), GPIOPin::new(PB04), GPIOPin::new(PB05),
    GPIOPin::new(PB06), GPIOPin::new(PB07), GPIOPin::new(PB08),
    GPIOPin::new(PB09), GPIOPin::new(PB10), GPIOPin::new(PB11),
    GPIOPin::new(PB12), GPIOPin::new(PB13), GPIOPin::new(PB14),
    GPIOPin::new(PB15), GPIOPin::new(PB16), GPIOPin::new(PB17),
    GPIOPin::new(PB18), GPIOPin::new(PB19), GPIOPin::new(PB20),
    GPIOPin::new(PB21), GPIOPin::new(PB22), GPIOPin::new(PB23),
    GPIOPin::new(PB24), GPIOPin::new(PB25), GPIOPin::new(PB26),
    GPIOPin::new(PB27), GPIOPin::new(PB28), GPIOPin::new(PB29),
    GPIOPin::new(PB30), GPIOPin::new(PB31)
];

/// Port C `GPIOPin`s
pub static mut PC : [GPIOPin; 32] = [
    GPIOPin::new(PC00), GPIOPin::new(PC01), GPIOPin::new(PC02),
    GPIOPin::new(PC03), GPIOPin::new(PC04), GPIOPin::new(PC05),
    GPIOPin::new(PC06), GPIOPin::new(PC07), GPIOPin::new(PC08),
    GPIOPin::new(PC09), GPIOPin::new(PC10), GPIOPin::new(PC11),
    GPIOPin::new(PC12), GPIOPin::new(PC13), GPIOPin::new(PC14),
    GPIOPin::new(PC15), GPIOPin::new(PC16), GPIOPin::new(PC17),
    GPIOPin::new(PC18), GPIOPin::new(PC19), GPIOPin::new(PC20),
    GPIOPin::new(PC21), GPIOPin::new(PC22), GPIOPin::new(PC23),
    GPIOPin::new(PC24), GPIOPin::new(PC25), GPIOPin::new(PC26),
    GPIOPin::new(PC27), GPIOPin::new(PC28), GPIOPin::new(PC29),
    GPIOPin::new(PC30), GPIOPin::new(PC31)
];

impl GPIOPin {
    pub const fn new(pin: Pin) -> GPIOPin {
        GPIOPin {
            port: (BASE_ADDRESS + ((pin as usize) / 32) * SIZE) as *mut Registers,
            pin_mask: 1 << ((pin as u32) % 32)
        }
    }

    pub fn select_peripheral(&self, function: PeripheralFunction) {
        let f = function as u32;
        let (bit0, bit1, bit2) = (f & 0b1, (f & 0b10) >> 1, (f & 0b100) >> 2);
        let port : &mut Registers = unsafe { mem::transmute(self.port) };

        // clear GPIO enable for pin
        volatile_store(&mut port.gper.clear, self.pin_mask);

        // Set PMR0-2 according to passed in peripheral
        if bit0 == 0 {
            volatile_store(&mut port.pmr0.clear, self.pin_mask);
        } else {
            volatile_store(&mut port.pmr0.set, self.pin_mask);
        }
        if bit1 == 0 {
            volatile_store(&mut port.pmr1.clear, self.pin_mask);
        } else {
            volatile_store(&mut port.pmr1.set, self.pin_mask);
        }
        if bit2 == 0 {
            volatile_store(&mut port.pmr2.clear, self.pin_mask);
        } else {
            volatile_store(&mut port.pmr2.set, self.pin_mask);
        }
    }

    pub fn enable(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.gper.set, self.pin_mask);
    }

    pub fn disable(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.gper.clear, self.pin_mask);
    }

    pub fn enable_output(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.oder.set, self.pin_mask);
    }

    pub fn disable_output(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.oder.clear, self.pin_mask);
    }

    pub fn enable_pull_down(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.pder.set, self.pin_mask);
    }

    pub fn disable_pull_down(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.pder.clear, self.pin_mask);
    }

    pub fn enable_pull_up(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.puer.set, self.pin_mask);
    }

    pub fn disable_pull_up(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.puer.clear, self.pin_mask);
    }

    /// Sets the interrupt mode registers. Interrupts may fire on the rising or
    /// falling edge of the pin or on both.
    ///
    /// The mode is a two-bit value based on the mapping from section 23.7.13 of
    /// the SAM4L datasheet (page 563):
    ///
    /// | `mode` value | Interrupt Mode |
    /// | ------------ | -------------- |
    /// | 0b00         | Pin change     |
    /// | 0b01         | Rising edge    |
    /// | 0b10         | Falling edge   |
    ///
    pub fn set_interrupt_mode(&self, mode: u8) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        if mode & 0b01 != 0 {
            volatile_store(&mut port.imr0.set, self.pin_mask);
        } else {
            volatile_store(&mut port.imr0.clear, self.pin_mask);
        }

        if mode & 0b10 != 0 {
            volatile_store(&mut port.imr1.set, self.pin_mask);
        } else {
            volatile_store(&mut port.imr1.clear, self.pin_mask);
        }
    }

    pub fn disable_schmidtt_trigger(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.ster.clear, self.pin_mask);
    }

    pub fn enable_schmidtt_trigger(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.ster.set, self.pin_mask);
    }

    pub fn read(&self) -> bool {
        let port : &Registers = unsafe { mem::transmute(self.port) };
        (volatile_load(&port.pvr) & self.pin_mask) > 0
    }

    pub fn toggle(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.ovr.toggle, self.pin_mask);
    }

    pub fn set(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.ovr.set, self.pin_mask);
    }

    pub fn clear(&self) {
        let port : &mut Registers = unsafe { mem::transmute(self.port) };
        volatile_store(&mut port.ovr.clear, self.pin_mask);
    }
}

impl hil::Controller for GPIOPin {
    type Config = Option<PeripheralFunction>;


    fn configure(&self, config: Option<PeripheralFunction>) {
        config.map(|c| {
            self.select_peripheral(c);
        });
    }
}

impl hil::gpio::GPIOPin for GPIOPin {
    fn disable(&self) {
        GPIOPin::disable(self);
    }

    fn enable_output(&self) {
        self.enable();
        GPIOPin::enable_output(self);
        self.disable_schmidtt_trigger();
    }

    fn enable_input(&self, mode: hil::gpio::InputMode) {
        self.enable();
        GPIOPin::disable_output(self);
        self.enable_schmidtt_trigger();
        match mode {
            hil::gpio::InputMode::PullUp => {
                self.disable_pull_down();
                self.enable_pull_up();
            },
            hil::gpio::InputMode::PullDown => {
                self.disable_pull_up();
                self.enable_pull_down();
            }
        }
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

    fn set_interrupt_mode(&self, mode: hil::gpio::InterruptMode) {
        let mode_bits = match mode {
            hil::gpio::InterruptMode::Change => 0b00,
            hil::gpio::InterruptMode::RisingEdge => 0b01,
            hil::gpio::InterruptMode::FallingEdge => 0b10
        };
        GPIOPin::set_interrupt_mode(self, mode_bits);
    }
}

