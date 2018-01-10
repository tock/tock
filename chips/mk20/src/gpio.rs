//! Implementation of the GPIO Controller
//! Resources:
//!     [1] Kinetis K66 Sub-Family Reference Manual
//!

use core::cell::Cell;
use core::sync::atomic::Ordering;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT};
use common::regs::{ReadWrite, WriteOnly, ReadOnly};
use core::mem;
use kernel::hil;
use nvic::{self, NvicIdx};

/// Register map for a single Port Control and Interrupt module
/// [^1]: Section 12.5
#[repr(C, packed)]
pub struct PortRegisters {
    pcr: [ReadWrite<u32, PinControl>; 32],
    gpclr: WriteOnly<u32>,
    gpchr: WriteOnly<u32>,
    _reserved0: [ReadOnly<u32>; 6],
    isfr: ReadWrite<u32>,
    _reserved1: [ReadOnly<u32>; 7],
    dfer: ReadWrite<u32>,
    dfcr: ReadWrite<u32>,
    dfwr: ReadWrite<u32>,
}

trace_macros!(true);
/// Look at the source for more documentation
/// would not compile with internal doc comments
bitfields! [ u32,
    /// this is pincontol
    PCR PinControl [

        // 31-25: reserved, read only, always value 0
        /// Interrupt Status Flag
        ///
        /// The pin interrupt configuration is valid in all digital pin muxing modes.
        ///
        /// When it in enabled:
        ///     If the pin is configured to generate a DMA request, then the
        ///	    corresponding flag will be cleared automatically at the completion of the requested DMA transfer.
        ///     Otherwise, the flag remains set until a logic one is written to the flag. If the pin is configured for a level
        ///     sensitive interrupt and the pin remains asserted, then the flag is set again immediately after it is cleared.
        InterruptStatusFlag 24 [
            /// Configured interrupt will not be triggered
            Unactive = 0,
            /// Configured interrupt will be triggered
            Active = 1
        ]
        ,
        // 23-20: reserved, read only, always value 0
        /// Interrupt Configuration
        ///
        /// The pin interrupt configuration is valid in all digital pin muxing modes. The corresponding pin is configured
        /// to generate interrupt/DMA request as follows:
        IRQConfiguration (16, Mask(0b1111)) [
            InterruptDisabled = 0,
            DmaRisingEdge = 1,
            DmaFallingEdge = 2,
            DmaEitherEdge = 3,
            InterruptLogicLow = 8,
            InterruptRisingEdge = 9,
            InterruptFallingEdge = 10,
            InterruptEitherEdge = 11,
            InterruptLogicHigh = 12
            // other values are reserved
        ],
        /// Lock the register fields until reset
        ///
        /// Lock the Pin Control Register fields ([15:0]) until next system reset
        LockRegister 15 [
            Unlocked = 0,
            Locked = 1
        ],
        // 14-11: reserved, read only, always value 0
        /// Mulitplex Control
        ///
        /// Not all pins support all pin muxing slots. Unimplemented pin muxing slots are reserved
        /// and may result in configuring the pin for a different pin muxing slot.
        PinMuxControl (8, Mask(0b111)) [
            /// pin disabled / analog
            PinDisabled = 0,
            /// GPIO
            Alternative1 = 1,
            /// chip specific
            Alternative2 = 2,
            /// chip specific
            Alternative3 = 3,
            /// chip specific
            Alternative4 = 4,
            /// chip specific
            Alternative5 = 5,
            /// chip specific
            Alternative6 = 6,
            /// chip specific
            Alternative7 = 7
        ],
        // 7: reserved, read only, always value 0
        /// Set Drive Strength
        ///
        /// This bit is read only for pins that do not support a configurable drive strength.
        /// Drive strength configuration is valid in all digital pin muxing modes.
        /// digital pins only
        DriveStrengthEnable 6 [
            Low = 0,
            High = 1
        ],
        /// Toggle Open Drain
        ///
        /// This bit is read only for pins that do not support a configurable open drain output.
        /// Open drain configuration is valid in all digital pin muxing modes.
        /// digital pins only
        OpenDrainEnable 5 [
            Disabled = 0,
            Enabled = 1
        ],
        /// Toggle Passive Filter
        ///
        /// Disable the passive input filter when high speed interfaces of more than 2 MHz are supported on the pin.
        /// digital pins only
        PassiveFilterEnable 4 [
            Disabled = 0,
            /// A low pass filter of 10 MHz to 30 MHz bandwidth is enabled on the digital input path
            Enabled = 1

        ],
        // 3: reserved, read only, always value 0
        /// Set Slew Rate
        ///
        /// This bit is read only for pins that do not support a configurable slew rate.
        /// Slew rate configuration is valid in all digital pin muxing modes.
        /// digital pins only
        SlewRateEnable 2 [
            Fast = 0,
            Slow = 1
        ],
        /// Toggle Pull
        /// This bit is read only for pins that do not support a configurable pull resistor.
        /// Pull configuration is valid in all digital pin muxing modes.
        /// `Enabled` only has effect on digital pins
        PullEnable 1 [
            Disabled = 0,
            Enabled = 1
        ],
        /// Set Push/Pull direction
        ///
        /// This bit is read only for pins that do not support a configurable pull resistor direction.
        /// Pull configuration is valid in all digital pin muxing modes.
        /// `PullEnable` needs to be enabled to take effect
        /// digital pins only
        PullSelect 0 [
            PullDown = 0,
            PullUp = 1
        ]
    ]
];
trace_macros!(false);

/// Register map for a single GPIO port--aliased to bitband region
/// [^1]: Section 63.3.1
#[repr(C, packed)]
pub struct GpioBitbandRegisters {
    output: [ReadWrite<u32>; 32],
    set: [ReadWrite<u32>; 32],
    clear: [ReadWrite<u32>; 32],
    toggle: [ReadWrite<u32>; 32],
    input: [ReadWrite<u32>; 32],
    direction: [ReadWrite<u32>; 32],
}

pub trait PinNum {
    const PIN: usize;
}

pub enum PeripheralFunction {
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
    Alt6,
    Alt7
}

pub struct Port<'a> {
    regs: *mut PortRegisters,
    bitband: *mut GpioBitbandRegisters,
    clients: [Cell<Option<&'a hil::gpio::Client>>; 32],
    client_data: [Cell<usize>; 32]
}

pub struct Pin<'a, P: PinNum> {
    gpio: Gpio<'a>,
    _pin: PhantomData<P>
}

pub struct Gpio<'a> {
    pin: usize,
    port: &'a Port<'a>,
    valid: AtomicBool
}

impl<'a> Port<'a> {
    const fn new(port: usize) -> Port<'a> {
        Port {
            regs: (PORT_BASE_ADDRESS + port*PORT_SIZE) as *mut PortRegisters,
            bitband: (GPIO_BASE_ADDRESS + port*GPIO_SIZE) as *mut GpioBitbandRegisters,
            clients: [Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None),
                      Cell::new(None), Cell::new(None)
                ],
            client_data: [Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0),
                          Cell::new(0), Cell::new(0)
                ]
        }
    }

    pub fn regs(&self) -> &mut PortRegisters {
        unsafe { mem::transmute(self.regs) }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.regs();
        let mut fired = regs.isfr.get();

        // Writing a logic 1 to the interrupt status flag register clears the interrupt
        regs.isfr.set(!0);

        loop {
            let pin = fired.trailing_zeros() as usize;
            if pin < self.clients.len() {
                fired &= !(1 << pin);
                self.clients[pin].get().map(|client| {
                    client.fired(self.client_data[pin].get());
                });
            } else {
                break;
            }
        }
    }
}

const PORT_BASE_ADDRESS: usize = 0x4004_9000;
const PORT_SIZE: usize = 0x1000;

const GPIO_BASE_ADDRESS: usize = 0x43FE_0000;
const GPIO_SIZE: usize = 0x800;

impl<'a, P: PinNum> Pin<'a, P> {
    const fn new(port: &'a Port<'a>) -> Pin<'a, P> {
        Pin {
            gpio: Gpio {
                pin: P::PIN,
                port: port,
                valid: ATOMIC_BOOL_INIT
            },
            _pin: PhantomData,
        }
    }

    pub fn claim_as_gpio(&self) -> &Gpio<'a> {
        let already_allocated = self.gpio.valid.swap(true, Ordering::Relaxed);
        if already_allocated {
            let port_name = match self.gpio.pin {
                0...31 => "A",
                32...63 => "B",
                64...95 => "C",
                96...127 => "D",
                _ => "E"
            };
            panic!("Requested GPIO pin P{}{} is already allocated.", port_name, self.gpio.index());
        }

        self.set_peripheral_function(PeripheralFunction::Alt1);

        &self.gpio
    }

    pub fn claim_as(&self, function: functions::Function<P>) {
        let already_allocated = self.gpio.valid.swap(true, Ordering::Relaxed);
        if already_allocated {
            return;
        }

        self.set_peripheral_function(function.val);
    }

    fn set_peripheral_function(&self, function: PeripheralFunction) {
        let port = self.gpio.port.regs();

        port.pcr[self.gpio.index()].modify(PCR::PinMuxControl.val(function as u32));
    }

    pub fn release_claim(&self) {
        self.gpio.clear_client();
        self.gpio.set_client_data(0);
        self.set_peripheral_function(PeripheralFunction::Alt0);
        self.gpio.valid.swap(false, Ordering::Relaxed);
    }
}

impl<'a> Gpio<'a> {
    pub fn regs(&self) -> &mut GpioBitbandRegisters {
        unsafe { mem::transmute(self.port.bitband) }
    }

    fn index(&self) -> usize {
        (self.pin % 32) as usize
    }

    pub fn disable_output(&self) {
        self.regs().direction[self.index()].set(0);
    }

    pub fn enable_output(&self) {
        self.regs().direction[self.index()].set(1);
    }

    pub fn read(&self) -> bool {
        self.regs().input[self.index()].get() > 0
    }

    pub fn toggle(&self) {
        self.regs().toggle[self.index()].set(1);
    }

    pub fn set(&self) {
        self.regs().set[self.index()].set(1);
    }

    pub fn clear(&self) {
        self.regs().clear[self.index()].set(1);
    }

    pub fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let config = match mode {
            hil::gpio::InputMode::PullUp => PCR::PullEnable::SET + PCR::PullSelect::PullUp,
            hil::gpio::InputMode::PullDown => PCR::PullEnable::SET + PCR::PullSelect::PullDown,
            hil::gpio::InputMode::PullNone => PCR::PullEnable::CLEAR,
        };

        self.port.regs().pcr[self.index()].modify(config);
    }

    pub fn set_interrupt_mode(&self, mode: hil::gpio::InterruptMode) {
        let config = match mode {
            hil::gpio::InterruptMode::RisingEdge => PCR::IRQConfiguration::InterruptRisingEdge,
            hil::gpio::InterruptMode::FallingEdge => PCR::IRQConfiguration::InterruptFallingEdge,
            hil::gpio::InterruptMode::EitherEdge => PCR::IRQConfiguration::InterruptEitherEdge
        };

        self.port.regs().pcr[self.index()].modify(config);
    }

    fn clear_interrupt_status_flag(&self) {
        self.port.regs().pcr[self.index()].modify(PCR::InterruptStatusFlag::SET);
    }

    fn enable_interrupt(&self) {
        unsafe {
            match self.pin {
                0...31 => nvic::enable(NvicIdx::PCMA),
                32...63 => nvic::enable(NvicIdx::PCMB),
                64...95 => nvic::enable(NvicIdx::PCMC),
                96...127 => nvic::enable(NvicIdx::PCMD),
                _ => nvic::enable(NvicIdx::PCME)
            };
        }
    }

    fn disable_interrupt(&self) {
        self.clear_interrupt_status_flag();
        self.port.regs().pcr[self.index()].modify(PCR::IRQConfiguration::InterruptDisabled);
    }

    pub fn clear_client(&self) {
        if !self.valid.load(Ordering::Relaxed) {
            return;
        }

        self.port.clients[self.index()].set(None);
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        if !self.valid.load(Ordering::Relaxed) {
            return;
        }

        self.port.clients[self.index()].set(Some(client));
    }

    pub fn set_client_data(&self, data: usize) {
        if !self.valid.load(Ordering::Relaxed) {
            return;
        }

        self.port.client_data[self.index()].set(data);
    }
}

impl<'a, P: PinNum> hil::Controller for Pin<'a, P> {
    type Config = functions::Function<P>;

    fn configure(&self, config: Self::Config) {
        self.claim_as(config);
    }
}

impl<'a> hil::Controller for Gpio<'a> {
    type Config = (hil::gpio::InputMode, hil::gpio::InterruptMode);

    fn configure(&self, config: Self::Config) {
        self.set_input_mode(config.0);
        self.set_interrupt_mode(config.1);
    }
}

impl<'a> hil::gpio::PinCtl for Gpio<'a> {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        Gpio::set_input_mode(self, mode);
    }
}

impl<'a> hil::gpio::Pin for Gpio<'a> {
    fn disable(&self) {
        unimplemented!();
    }

    fn make_output(&self) {
        self.enable_output();
    }

    fn make_input(&self) {
        self.disable_output();
    }

    fn read(&self) -> bool {
        self.read()
    }

    fn toggle(&self) {
        self.toggle();
    }

    fn set(&self) {
        self.set();
    }

    fn clear(&self) {
        self.clear();
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        Gpio::enable_interrupt(self);
        self.set_interrupt_mode(mode);
        self.set_client_data(client_data);
    }

    fn disable_interrupt(&self) {
        Gpio::disable_interrupt(self);
    }
}

macro_rules! pins {
    {$($port:ident [$($pintype:ident $pinname:ident $pinnum:expr),*]),*} => {
        $(
            $(
                pub struct $pintype;
                impl PinNum for $pintype {
                    const PIN: usize =  $pinnum;
                }
                pub static mut $pinname: Pin<$pintype> = Pin::new(unsafe { &$port });
            )*
        )*
    }
}

pub static mut PA: Port = Port::new(0);
pub static mut PB: Port = Port::new(1);
pub static mut PC: Port = Port::new(2);
pub static mut PD: Port = Port::new(3);
pub static mut PE: Port = Port::new(4);

pins! {
    PA [PinA00 PA00 0, PinA01 PA01 1, PinA02 PA02 2, PinA03 PA03 3,
        PinA04 PA04 4, PinA05 PA05 5, PinA06 PA06 6, PinA07 PA07 7,
        PinA08 PA08 8, PinA09 PA09 9, PinA10 PA10 10, PinA11 PA11 11,
        PinA12 PA12 12, PinA13 PA13 13, PinA14 PA14 14, PinA15 PA15 15,
        PinA16 PA16 16, PinA17 PA17 17, PinA18 PA18 18, PinA19 PA19 19,
        PinA20 PA20 20, PinA21 PA21 21, PinA22 PA22 22, PinA23 PA23 23,
        PinA24 PA24 24, PinA25 PA25 25, PinA26 PA26 26, PinA27 PA27 27,
        PinA28 PA28 28, PinA29 PA29 29, PinA30 PA30 30, PinA31 PA31 31],

    PB [PinB00 PB00 32, PinB01 PB01 33, PinB02 PB02 34, PinB03 PB03 35,
        PinB04 PB04 36, PinB05 PB05 37, PinB06 PB06 38, PinB07 PB07 39,
        PinB08 PB08 40, PinB09 PB09 41, PinB10 PB10 42, PinB11 PB11 43,
        PinB12 PB12 44, PinB13 PB13 45, PinB14 PB14 46, PinB15 PB15 47,
        PinB16 PB16 48, PinB17 PB17 49, PinB18 PB18 50, PinB19 PB19 51,
        PinB20 PB20 52, PinB21 PB21 53, PinB22 PB22 54, PinB23 PB23 55,
        PinB24 PB24 56, PinB25 PB25 57, PinB26 PB26 58, PinB27 PB27 59,
        PinB28 PB28 60, PinB29 PB29 61, PinB30 PB30 62, PinB31 PB31 63],

    PC [PinC00 PC00 64, PinC01 PC01 65, PinC02 PC02 66, PinC03 PC03 67,
        PinC04 PC04 68, PinC05 PC05 69, PinC06 PC06 70, PinC07 PC07 71,
        PinC08 PC08 72, PinC09 PC09 73, PinC10 PC10 74, PinC11 PC11 75,
        PinC12 PC12 76, PinC13 PC13 77, PinC14 PC14 78, PinC15 PC15 79,
        PinC16 PC16 80, PinC17 PC17 81, PinC18 PC18 82, PinC19 PC19 83,
        PinC20 PC20 84, PinC21 PC21 85, PinC22 PC22 86, PinC23 PC23 87,
        PinC24 PC24 88, PinC25 PC25 89, PinC26 PC26 90, PinC27 PC27 91,
        PinC28 PC28 92, PinC29 PC29 93, PinC30 PC30 94, PinC31 PC31 95],

    PD [PinD00 PD00 96, PinD01 PD01 97, PinD02 PD02 98, PinD03 PD03 99,
        PinD04 PD04 100, PinD05 PD05 101, PinD06 PD06 102, PinD07 PD07 103,
        PinD08 PD08 104, PinD09 PD09 105, PinD10 PD10 106, PinD11 PD11 107,
        PinD12 PD12 108, PinD13 PD13 109, PinD14 PD14 110, PinD15 PD15 111,
        PinD16 PD16 112, PinD17 PD17 113, PinD18 PD18 114, PinD19 PD19 115,
        PinD20 PD20 116, PinD21 PD21 117, PinD22 PD22 118, PinD23 PD23 119,
        PinD24 PD24 120, PinD25 PD25 121, PinD26 PD26 122, PinD27 PD27 123,
        PinD28 PD28 124, PinD29 PD29 125, PinD30 PD30 126, PinD31 PD31 127],

    PE [PinE00 PE00 128, PinE01 PE01 129, PinE02 PE02 130, PinE03 PE03 131,
        PinE04 PE04 132, PinE05 PE05 133, PinE06 PE06 134, PinE07 PE07 135,
        PinE08 PE08 136, PinE09 PE09 137, PinE10 PE10 138, PinE11 PE11 139,
        PinE12 PE12 140, PinE13 PE13 141, PinE14 PE14 142, PinE15 PE15 143,
        PinE16 PE16 144, PinE17 PE17 145, PinE18 PE18 146, PinE19 PE19 147,
        PinE20 PE20 148, PinE21 PE21 149, PinE22 PE22 150, PinE23 PE23 151,
        PinE24 PE24 152, PinE25 PE25 153, PinE26 PE26 154, PinE27 PE27 155,
        PinE28 PE28 156, PinE29 PE29 157, PinE30 PE30 158, PinE31 PE31 159]
}

pub mod functions {
    use gpio::*;
    use core::marker::PhantomData;
    use gpio::PeripheralFunction::*;

    pub struct Function<P: PinNum> {
        _pin: PhantomData<P>,
        pub val: PeripheralFunction,
    }

    impl<P: PinNum> Function<P> {
        const fn new(val: PeripheralFunction) -> Function<P> {
            Function {
                _pin: PhantomData,
                val: val
            }
        }
    }

    // Peripheral assignments
    // UART0: PB16, PB17
    pub const UART0_RX: Function<PinB16> = Function::new(Alt3);
    pub const UART0_TX: Function<PinB17> = Function::new(Alt3);

    // SPI0
    pub const SPI0_MOSI: Function<PinC06> = Function::new(Alt2);
    pub const SPI0_MISO: Function<PinC07> = Function::new(Alt2);
    pub const SPI0_SCK: Function<PinA15> = Function::new(Alt2);
    pub const SPI0_CS0: Function<PinC04> = Function::new(Alt2);

    // SPI1
    pub const SPI1_MOSI: Function<PinD06> = Function::new(Alt7);
    pub const SPI1_SCK: Function<PinD05> = Function::new(Alt7);
}

interrupt_handler!(porta_interrupt, PCMA);
interrupt_handler!(portb_interrupt, PCMB);
interrupt_handler!(portc_interrupt, PCMC);
interrupt_handler!(portd_interrupt, PCMD);
interrupt_handler!(porte_interrupt, PCME);
