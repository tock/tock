

use self::Pin::*;
use core::cell::Cell;
use core::mem;
use core::ops::{Index, IndexMut};
use kernel::common::volatile_cell::VolatileCell;
use kernel::hil;
use nvic;
use nvic::NvicIdx::*;

#[repr(C, packed)]
struct Register {
    val: VolatileCell<u32>,
    set: VolatileCell<u32>,
    clear: VolatileCell<u32>,
    toggle: VolatileCell<u32>,
}

#[repr(C, packed)]
struct RegisterRC {
    val: VolatileCell<u32>,
    reserved0: u32,
    clear: VolatileCell<u32>,
    reserved1: u32,
}

#[repr(C, packed)]
struct Registers {
    gper: Register,
    pmr0: Register,
    pmr1: Register,
    pmr2: Register,
    oder: Register,
    ovr: Register,
    pvr: VolatileCell<u32>,
    _reserved0: [u32; 3],
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
    A,
    B,
    C,
    D,
    E,
    F,
    G,
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
#[cfg_attr(rustfmt, rustfmt_skip)]
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

/// GPIO port that manages 32 pins.
///
/// The SAM4L divides GPIOs into _ports_ that each manage a group of 32
/// individual pins. There are up to three ports, depending particular chip
/// (see[^1]).
///
/// In general, the kernel and applications should care about individual
/// [GPIOPin](struct.GPIOPin.html)s. However, mirroring the hardware grouping in
/// Rust is useful, internally, for correctly handling and dispatching
/// interrupts.
///
/// The port itself is a set of 32-bit memory-mapped I/O registers. Each
/// register has a bit for each pin in the port. Pins are, thus, named by their
/// port and offset bit in each register that controls is. For example, the
/// first port has pins called "PA00" thru "PA31".
///
/// [^1]: SAM4L datasheet section 23.8 (page 573): "Module Configuration" for
///       GPIO
pub struct Port {
    port: *mut Registers,
    pins: [GPIOPin; 32],
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
        let port: &mut Registers = unsafe { mem::transmute(self.port) };

        // Interrupt Flag Register (IFR) bits are only valid if the same bits
        // are enabled in Interrupt Enabled Register (IER).
        let mut fired = port.ifr.val.get() & port.ier.val.get();

        // About to handle all the interrupts, so just clear them now to get
        // over with it.
        port.ifr.clear.set(!0);

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
    port: (BASE_ADDRESS + 0 * SIZE) as *mut Registers,
    pins: [GPIOPin::new(PA00, GPIO0),
           GPIOPin::new(PA01, GPIO0),
           GPIOPin::new(PA02, GPIO0),
           GPIOPin::new(PA03, GPIO0),
           GPIOPin::new(PA04, GPIO0),
           GPIOPin::new(PA05, GPIO0),
           GPIOPin::new(PA06, GPIO0),
           GPIOPin::new(PA07, GPIO0),
           GPIOPin::new(PA08, GPIO1),
           GPIOPin::new(PA09, GPIO1),
           GPIOPin::new(PA10, GPIO1),
           GPIOPin::new(PA11, GPIO1),
           GPIOPin::new(PA12, GPIO1),
           GPIOPin::new(PA13, GPIO1),
           GPIOPin::new(PA14, GPIO1),
           GPIOPin::new(PA15, GPIO1),
           GPIOPin::new(PA16, GPIO2),
           GPIOPin::new(PA17, GPIO2),
           GPIOPin::new(PA18, GPIO2),
           GPIOPin::new(PA19, GPIO2),
           GPIOPin::new(PA20, GPIO2),
           GPIOPin::new(PA21, GPIO2),
           GPIOPin::new(PA22, GPIO2),
           GPIOPin::new(PA23, GPIO2),
           GPIOPin::new(PA24, GPIO3),
           GPIOPin::new(PA25, GPIO3),
           GPIOPin::new(PA26, GPIO3),
           GPIOPin::new(PA27, GPIO3),
           GPIOPin::new(PA28, GPIO3),
           GPIOPin::new(PA29, GPIO3),
           GPIOPin::new(PA30, GPIO3),
           GPIOPin::new(PA31, GPIO3)],
};

/// Port B
pub static mut PB: Port = Port {
    port: (BASE_ADDRESS + 1 * SIZE) as *mut Registers,
    pins: [GPIOPin::new(PB00, GPIO4),
           GPIOPin::new(PB01, GPIO4),
           GPIOPin::new(PB02, GPIO4),
           GPIOPin::new(PB03, GPIO4),
           GPIOPin::new(PB04, GPIO4),
           GPIOPin::new(PB05, GPIO4),
           GPIOPin::new(PB06, GPIO4),
           GPIOPin::new(PB07, GPIO4),
           GPIOPin::new(PB08, GPIO5),
           GPIOPin::new(PB09, GPIO5),
           GPIOPin::new(PB10, GPIO5),
           GPIOPin::new(PB11, GPIO5),
           GPIOPin::new(PB12, GPIO5),
           GPIOPin::new(PB13, GPIO5),
           GPIOPin::new(PB14, GPIO5),
           GPIOPin::new(PB15, GPIO5),
           GPIOPin::new(PB16, GPIO6),
           GPIOPin::new(PB17, GPIO6),
           GPIOPin::new(PB18, GPIO6),
           GPIOPin::new(PB19, GPIO6),
           GPIOPin::new(PB20, GPIO6),
           GPIOPin::new(PB21, GPIO6),
           GPIOPin::new(PB22, GPIO6),
           GPIOPin::new(PB23, GPIO6),
           GPIOPin::new(PB24, GPIO7),
           GPIOPin::new(PB25, GPIO7),
           GPIOPin::new(PB26, GPIO7),
           GPIOPin::new(PB27, GPIO7),
           GPIOPin::new(PB28, GPIO7),
           GPIOPin::new(PB29, GPIO7),
           GPIOPin::new(PB30, GPIO7),
           GPIOPin::new(PB31, GPIO7)],
};

/// Port C
pub static mut PC: Port = Port {
    port: (BASE_ADDRESS + 2 * SIZE) as *mut Registers,
    pins: [GPIOPin::new(PC00, GPIO8),
           GPIOPin::new(PC01, GPIO8),
           GPIOPin::new(PC02, GPIO8),
           GPIOPin::new(PC03, GPIO8),
           GPIOPin::new(PC04, GPIO8),
           GPIOPin::new(PC05, GPIO8),
           GPIOPin::new(PC06, GPIO8),
           GPIOPin::new(PC07, GPIO8),
           GPIOPin::new(PC08, GPIO9),
           GPIOPin::new(PC09, GPIO9),
           GPIOPin::new(PC10, GPIO9),
           GPIOPin::new(PC11, GPIO9),
           GPIOPin::new(PC12, GPIO9),
           GPIOPin::new(PC13, GPIO9),
           GPIOPin::new(PC14, GPIO9),
           GPIOPin::new(PC15, GPIO9),
           GPIOPin::new(PC16, GPIO10),
           GPIOPin::new(PC17, GPIO10),
           GPIOPin::new(PC18, GPIO10),
           GPIOPin::new(PC19, GPIO10),
           GPIOPin::new(PC20, GPIO10),
           GPIOPin::new(PC21, GPIO10),
           GPIOPin::new(PC22, GPIO10),
           GPIOPin::new(PC23, GPIO10),
           GPIOPin::new(PC24, GPIO11),
           GPIOPin::new(PC25, GPIO11),
           GPIOPin::new(PC26, GPIO11),
           GPIOPin::new(PC27, GPIO11),
           GPIOPin::new(PC28, GPIO11),
           GPIOPin::new(PC29, GPIO11),
           GPIOPin::new(PC30, GPIO11),
           GPIOPin::new(PC31, GPIO11)],
};
pub struct GPIOPin {
    port: *mut Registers,
    nvic: nvic::NvicIdx,
    pin_mask: u32,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
}

impl GPIOPin {
    const fn new(pin: Pin, nvic: nvic::NvicIdx) -> GPIOPin {
        GPIOPin {
            port: (BASE_ADDRESS + ((pin as usize) / 32) * SIZE) as *mut Registers,
            nvic: nvic,
            pin_mask: 1 << ((pin as u32) % 32),
            client_data: Cell::new(0),
            client: Cell::new(None),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn select_peripheral(&self, function: PeripheralFunction) {
        let f = function as u32;
        let (bit0, bit1, bit2) = (f & 0b1, (f & 0b10) >> 1, (f & 0b100) >> 2);
        let port: &mut Registers = unsafe { mem::transmute(self.port) };

        // clear GPIO enable for pin
        port.gper.clear.set(self.pin_mask);

        // Set PMR0-2 according to passed in peripheral
        if bit0 == 0 {
            port.pmr0.clear.set(self.pin_mask);
        } else {
            port.pmr0.set.set(self.pin_mask);
        }
        if bit1 == 0 {
            port.pmr1.clear.set(self.pin_mask);
        } else {
            port.pmr1.set.set(self.pin_mask);
        }
        if bit2 == 0 {
            port.pmr2.clear.set(self.pin_mask);
        } else {
            port.pmr2.set.set(self.pin_mask);
        }
    }

    pub fn enable(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.gper.set.set(self.pin_mask);
    }

    pub fn disable(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.gper.clear.set(self.pin_mask);
    }

    pub fn enable_output(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.oder.set.set(self.pin_mask);
    }

    pub fn disable_output(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.oder.clear.set(self.pin_mask);
    }

    pub fn enable_pull_down(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.pder.set.set(self.pin_mask);
    }

    pub fn disable_pull_down(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.pder.clear.set(self.pin_mask);
    }

    pub fn enable_pull_up(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.puer.set.set(self.pin_mask);
    }

    pub fn disable_pull_up(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.puer.clear.set(self.pin_mask);
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
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        if mode & 0b01 != 0 {
            port.imr0.set.set(self.pin_mask);
        } else {
            port.imr0.clear.set(self.pin_mask);
        }

        if mode & 0b10 != 0 {
            port.imr1.set.set(self.pin_mask);
        } else {
            port.imr1.clear.set(self.pin_mask);
        }
    }

    pub fn enable_interrupt(&self) {
        unsafe {
            let port: &mut Registers = mem::transmute(self.port);
            nvic::enable(self.nvic);
            port.ier.set.set(self.pin_mask);
        }
    }

    pub fn disable_interrupt(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.ier.clear.set(self.pin_mask);
        if port.ier.val.get() == 0 {
            unsafe {
                nvic::disable(self.nvic);
            }
        }
    }

    pub fn handle_interrupt(&self) {
        self.client.get().map(|client| { client.fired(self.client_data.get()); });
    }

    pub fn disable_schmidtt_trigger(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.ster.clear.set(self.pin_mask);
    }

    pub fn enable_schmidtt_trigger(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.ster.set.set(self.pin_mask);
    }

    pub fn read(&self) -> bool {
        let port: &Registers = unsafe { mem::transmute(self.port) };
        (port.pvr.get() & self.pin_mask) > 0
    }

    pub fn toggle(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.ovr.toggle.set(self.pin_mask);
    }

    pub fn set(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.ovr.set.set(self.pin_mask);
    }

    pub fn clear(&self) {
        let port: &mut Registers = unsafe { mem::transmute(self.port) };
        port.ovr.clear.set(self.pin_mask);
    }
}

impl hil::Controller for GPIOPin {
    type Config = Option<PeripheralFunction>;


    fn configure(&self, config: Option<PeripheralFunction>) {
        config.map(|c| { self.select_peripheral(c); });
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
        GPIOPin::disable(self);
    }

    fn make_output(&self) {
        self.enable();
        GPIOPin::enable_output(self);
        self.disable_schmidtt_trigger();
    }

    fn make_input(&self) {
        self.enable();
        GPIOPin::disable_output(self);
        self.enable_schmidtt_trigger();
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

macro_rules! gpio_handler {
    ($num: ident) => {
        interrupt_handler!(concat_idents!(GPIO_, $num, _Handler), {
            use kernel::common::Queue;

            let nvic = concat_idents!(nvic::NvicIdx::GPIO, $num);
            nvic::disable(nvic);
            chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nvic);
        })
    }
}

interrupt_handler!(gpio0_handler, GPIO0);
interrupt_handler!(gpio1_handler, GPIO1);
interrupt_handler!(gpio2_handler, GPIO2);
interrupt_handler!(gpio3_handler, GPIO3);
interrupt_handler!(gpio4_handler, GPIO4);
interrupt_handler!(gpio5_handler, GPIO5);
interrupt_handler!(gpio6_handler, GPIO6);
interrupt_handler!(gpio7_handler, GPIO7);
interrupt_handler!(gpio8_handler, GPIO8);
interrupt_handler!(gpio9_handler, GPIO9);
interrupt_handler!(gpio10_handler, GPIO10);
interrupt_handler!(gpio11_handler, GPIO11);
