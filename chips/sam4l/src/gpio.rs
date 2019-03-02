//! Implementation of the GPIO controller for the SAM4L.

use core::ops::{Index, IndexMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::gpio;

#[repr(C)]
struct Register {
    val: ReadWrite<u32>,
    set: WriteOnly<u32>,
    clear: WriteOnly<u32>,
    toggle: WriteOnly<u32>,
}

#[repr(C)]
struct RegisterRC {
    val: ReadOnly<u32>,
    reserved0: u32,
    clear: WriteOnly<u32>,
    reserved1: u32,
}

#[repr(C)]
struct GpioRegisters {
    gper: Register,
    pmr0: Register,
    pmr1: Register,
    pmr2: Register,
    oder: Register,
    ovr: Register,
    pvr: ReadOnly<u32>,
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
#[derive(Copy, Clone)]
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

/// Reference count for the number of GPIO interrupts currently active.
///
/// This is used to determine if it's possible for the SAM4L to go into
/// WAIT/RETENTION mode, since those modes will not be woken up by GPIO
/// interrupts.
///
/// This is an `AtomicUsize` because it has to be a `Sync` type to live in a
/// global---Rust has no way of knowing we're not going to use it across
/// threads. Use `Ordering::Relaxed` when reading/writing the value to get LLVM
/// to just use plain loads and stores instead of atomic operations.
pub static INTERRUPT_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Name of the GPIO pin on the SAM4L.
///
/// The "Package and Pinout" section[^1] of the SAM4L datasheet shows the
/// mapping between these names and hardware pins on different chip packages.
///
/// [^1]: Section 3.1, pages 10-18
#[derive(Copy,Clone)]
#[rustfmt::skip]
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
    port: StaticRef<GpioRegisters>,
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
        let port: &GpioRegisters = &*self.port;

        // Interrupt Flag Register (IFR) bits are only valid if the same bits
        // are enabled in Interrupt Enabled Register (IER).
        let mut fired = port.ifr.val.get() & port.ier.val.get();
        loop {
            let pin = fired.trailing_zeros() as usize;
            if pin < self.pins.len() {
                fired &= !(1 << pin);
                self.pins[pin].handle_interrupt();
                port.ifr.clear.set(1 << pin);
            } else {
                break;
            }
        }
    }
}

/// Port A
pub static mut PA: Port = Port {
    port: unsafe { StaticRef::new((BASE_ADDRESS + 0 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(Pin::PA00),
        GPIOPin::new(Pin::PA01),
        GPIOPin::new(Pin::PA02),
        GPIOPin::new(Pin::PA03),
        GPIOPin::new(Pin::PA04),
        GPIOPin::new(Pin::PA05),
        GPIOPin::new(Pin::PA06),
        GPIOPin::new(Pin::PA07),
        GPIOPin::new(Pin::PA08),
        GPIOPin::new(Pin::PA09),
        GPIOPin::new(Pin::PA10),
        GPIOPin::new(Pin::PA11),
        GPIOPin::new(Pin::PA12),
        GPIOPin::new(Pin::PA13),
        GPIOPin::new(Pin::PA14),
        GPIOPin::new(Pin::PA15),
        GPIOPin::new(Pin::PA16),
        GPIOPin::new(Pin::PA17),
        GPIOPin::new(Pin::PA18),
        GPIOPin::new(Pin::PA19),
        GPIOPin::new(Pin::PA20),
        GPIOPin::new(Pin::PA21),
        GPIOPin::new(Pin::PA22),
        GPIOPin::new(Pin::PA23),
        GPIOPin::new(Pin::PA24),
        GPIOPin::new(Pin::PA25),
        GPIOPin::new(Pin::PA26),
        GPIOPin::new(Pin::PA27),
        GPIOPin::new(Pin::PA28),
        GPIOPin::new(Pin::PA29),
        GPIOPin::new(Pin::PA30),
        GPIOPin::new(Pin::PA31),
    ],
};

/// Port B
pub static mut PB: Port = Port {
    port: unsafe { StaticRef::new((BASE_ADDRESS + 1 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(Pin::PB00),
        GPIOPin::new(Pin::PB01),
        GPIOPin::new(Pin::PB02),
        GPIOPin::new(Pin::PB03),
        GPIOPin::new(Pin::PB04),
        GPIOPin::new(Pin::PB05),
        GPIOPin::new(Pin::PB06),
        GPIOPin::new(Pin::PB07),
        GPIOPin::new(Pin::PB08),
        GPIOPin::new(Pin::PB09),
        GPIOPin::new(Pin::PB10),
        GPIOPin::new(Pin::PB11),
        GPIOPin::new(Pin::PB12),
        GPIOPin::new(Pin::PB13),
        GPIOPin::new(Pin::PB14),
        GPIOPin::new(Pin::PB15),
        GPIOPin::new(Pin::PB16),
        GPIOPin::new(Pin::PB17),
        GPIOPin::new(Pin::PB18),
        GPIOPin::new(Pin::PB19),
        GPIOPin::new(Pin::PB20),
        GPIOPin::new(Pin::PB21),
        GPIOPin::new(Pin::PB22),
        GPIOPin::new(Pin::PB23),
        GPIOPin::new(Pin::PB24),
        GPIOPin::new(Pin::PB25),
        GPIOPin::new(Pin::PB26),
        GPIOPin::new(Pin::PB27),
        GPIOPin::new(Pin::PB28),
        GPIOPin::new(Pin::PB29),
        GPIOPin::new(Pin::PB30),
        GPIOPin::new(Pin::PB31),
    ],
};

/// Port C
pub static mut PC: Port = Port {
    port: unsafe { StaticRef::new((BASE_ADDRESS + 2 * SIZE) as *const GpioRegisters) },
    pins: [
        GPIOPin::new(Pin::PC00),
        GPIOPin::new(Pin::PC01),
        GPIOPin::new(Pin::PC02),
        GPIOPin::new(Pin::PC03),
        GPIOPin::new(Pin::PC04),
        GPIOPin::new(Pin::PC05),
        GPIOPin::new(Pin::PC06),
        GPIOPin::new(Pin::PC07),
        GPIOPin::new(Pin::PC08),
        GPIOPin::new(Pin::PC09),
        GPIOPin::new(Pin::PC10),
        GPIOPin::new(Pin::PC11),
        GPIOPin::new(Pin::PC12),
        GPIOPin::new(Pin::PC13),
        GPIOPin::new(Pin::PC14),
        GPIOPin::new(Pin::PC15),
        GPIOPin::new(Pin::PC16),
        GPIOPin::new(Pin::PC17),
        GPIOPin::new(Pin::PC18),
        GPIOPin::new(Pin::PC19),
        GPIOPin::new(Pin::PC20),
        GPIOPin::new(Pin::PC21),
        GPIOPin::new(Pin::PC22),
        GPIOPin::new(Pin::PC23),
        GPIOPin::new(Pin::PC24),
        GPIOPin::new(Pin::PC25),
        GPIOPin::new(Pin::PC26),
        GPIOPin::new(Pin::PC27),
        GPIOPin::new(Pin::PC28),
        GPIOPin::new(Pin::PC29),
        GPIOPin::new(Pin::PC30),
        GPIOPin::new(Pin::PC31),
    ],
};
pub struct GPIOPin {
    port: StaticRef<GpioRegisters>,
    pin_mask: u32,
    client: OptionalCell<&'static hil::gpio::Client>,
}

impl GPIOPin {
    const fn new(pin: Pin) -> GPIOPin {
        GPIOPin {
            port: unsafe {
                StaticRef::new(
                    (BASE_ADDRESS + ((pin as usize) / 32) * SIZE) as *const GpioRegisters,
                )
            },
            pin_mask: 1 << ((pin as u32) % 32),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static gpio::Client) {
        self.client.set(client);
    }

    pub fn select_peripheral(&self, function: PeripheralFunction) {
        let f = function as u32;
        let (bit0, bit1, bit2) = (f & 0b1, (f & 0b10) >> 1, (f & 0b100) >> 2);
        let port: &GpioRegisters = &*self.port;

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
        let port: &GpioRegisters = &*self.port;
        port.gper.set.set(self.pin_mask);
    }

    pub fn disable(&self) {
        let port: &GpioRegisters = &*self.port;
        port.gper.clear.set(self.pin_mask);
    }

    pub fn is_pending(&self) -> bool {
        let port: &GpioRegisters = &*self.port;
        (port.ifr.val.get() & self.pin_mask) != 0
    }

    pub fn enable_output(&self) {
        let port: &GpioRegisters = &*self.port;
        port.oder.set.set(self.pin_mask);
    }

    pub fn disable_output(&self) {
        let port: &GpioRegisters = &*self.port;
        port.oder.clear.set(self.pin_mask);
    }

    pub fn enable_pull_down(&self) {
        let port: &GpioRegisters = &*self.port;
        port.pder.set.set(self.pin_mask);
    }

    pub fn disable_pull_down(&self) {
        let port: &GpioRegisters = &*self.port;
        port.pder.clear.set(self.pin_mask);
    }

    pub fn enable_pull_up(&self) {
        let port: &GpioRegisters = &*self.port;
        port.puer.set.set(self.pin_mask);
    }

    pub fn disable_pull_up(&self) {
        let port: &GpioRegisters = &*self.port;
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
        let port: &GpioRegisters = &*self.port;
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
        let port: &GpioRegisters = &*self.port;
        if port.ier.val.get() & self.pin_mask == 0 {
            INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
            port.ier.set.set(self.pin_mask);
        }
    }

    pub fn disable_interrupt(&self) {
        let port: &GpioRegisters = &*self.port;
        if port.ier.val.get() & self.pin_mask != 0 {
            INTERRUPT_COUNT.fetch_sub(1, Ordering::Relaxed);
            port.ier.clear.set(self.pin_mask);
        }
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| {
            client.fired();
        });
    }

    pub fn disable_schmidtt_trigger(&self) {
        let port: &GpioRegisters = &*self.port;
        port.ster.clear.set(self.pin_mask);
    }

    pub fn enable_schmidtt_trigger(&self) {
        let port: &GpioRegisters = &*self.port;
        port.ster.set.set(self.pin_mask);
    }

    pub fn read(&self) -> bool {
        let port: &GpioRegisters = &*self.port;
        (port.pvr.get() & self.pin_mask) > 0
    }

    pub fn toggle(&self) -> bool {
        let port: &GpioRegisters = &*self.port;
        port.ovr.toggle.set(self.pin_mask);
        self.read()
    }

    pub fn set(&self) {
        let port: &GpioRegisters = &*self.port;
        port.ovr.set.set(self.pin_mask);
    }

    pub fn clear(&self) {
        let port: &GpioRegisters = &*self.port;
        port.ovr.clear.set(self.pin_mask);
    }
}

impl hil::Controller for GPIOPin {
    type Config = Option<PeripheralFunction>;

    fn configure(&self, config: Self::Config) {
        match config {
            Some(c) => self.select_peripheral(c),
            None => self.enable(),
        }
    }
}

impl gpio::Pin for GPIOPin {}
impl gpio::InterruptPin for GPIOPin {}

impl gpio::Configure for GPIOPin {
    fn set_floating_state(&self, mode: gpio::FloatingState) {
        match mode {
            gpio::FloatingState::PullUp => {
                self.disable_pull_down();
                self.enable_pull_up();
            }
            gpio::FloatingState::PullDown => {
                self.disable_pull_up();
                self.enable_pull_down();
            }
            gpio::FloatingState::PullNone => {
                self.disable_pull_up();
                self.disable_pull_down();
            }
        }
    }

    fn low_power(&self) {
        GPIOPin::disable(self);
    }

    fn make_output(&self) -> gpio::Configuration {
        self.enable();
        GPIOPin::enable_output(self);
        self.disable_schmidtt_trigger();
        gpio::Configuration::Output
    }

    fn make_input(&self) -> gpio::Configuration {
        self.enable();
        GPIOPin::disable_output(self);
        self.enable_schmidtt_trigger();
        gpio::Configuration::Input
    }
    
    fn disable_output(&self) -> gpio::Configuration {
        let port: &GpioRegisters = &*self.port;
        port.oder.clear.set(self.pin_mask);
        self.configuration()
    }

    fn disable_input(&self) -> gpio::Configuration {
        self.configuration()
    }

    fn is_input(&self) -> bool {
        let port: &GpioRegisters = &*self.port;
        port.gper.val.get() & self.pin_mask != 0
    }

    fn is_output(&self) -> bool {
        let port: &GpioRegisters = &*self.port;
        port.oder.val.get() & self.pin_mask != 0
    }

    fn floating_state(&self) -> gpio::FloatingState {
        let port: &GpioRegisters = &*self.port;
        let down = (port.pder.val.get() & self.pin_mask) != 0;
        let up = (port.puer.val.get() &self.pin_mask) != 0;
        if down {
           gpio::FloatingState::PullDown
        } else if up {
           gpio::FloatingState::PullUp
        } else {
           gpio::FloatingState::PullNone 
        }
    }
    
    fn configuration(&self) -> gpio::Configuration {
        let input = self.is_input();
        let output = self.is_output();
        let config = (input, output);
        match config {
            (false, false) => gpio::Configuration::Unknown,
            (false, true)  => gpio::Configuration::Output,
            (true, false)  => gpio::Configuration::Input,
            (true, true)   => gpio::Configuration::InputOutput,
        } 
    } 
}

impl gpio::Input for GPIOPin {
    fn read(&self) -> bool {
        GPIOPin::read(self)
    }
}

impl gpio::Output for GPIOPin {
    fn toggle(&self) -> bool {
        GPIOPin::toggle(self)
    }

    fn set(&self) {
        GPIOPin::set(self);
    }

    fn clear(&self) {
        GPIOPin::clear(self);
    }
}

impl gpio::Interrupt for GPIOPin {
    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        let mode_bits = match mode {
            hil::gpio::InterruptEdge::EitherEdge => 0b00,
            hil::gpio::InterruptEdge::RisingEdge => 0b01,
            hil::gpio::InterruptEdge::FallingEdge => 0b10,
        };
        GPIOPin::set_interrupt_mode(self, mode_bits);
        GPIOPin::enable_interrupt(self);
    }

    fn disable_interrupts(&self) {
        GPIOPin::disable_interrupt(self);
    }

    fn set_client(&self, client: &'static gpio::Client) {
        GPIOPin::set_client(self, client);
    }

    fn is_pending(&self) -> bool {
        GPIOPin::is_pending(self)
    }
}
