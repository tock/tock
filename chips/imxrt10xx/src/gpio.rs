//! i.MX RT GPIO driver
//!
//! # Design
//!
//! Allocate a [`Ports`] collection. Use this to access GPIO pins and GPIO ports.
//!
//! - A [`Port`] maps to a GPIO pin bank. `GPIO3` is a port. Ports may provide
//!   access to pins. Ports handle interrupts.
//! - A [`Pin`] maps to a physical GPIO pin. `GPIO3[17]` is a pin. It's identified
//!   by its [`PinId`], [`SdB0_05`][PinId::SdB0_05]. Pins can be set as inputs or
//!   outputs.
//!
//! # Example
//!
//! ```
//! use imxrt10xx::gpio::{Ports, Port, PinId};
//!
//! # let ccm = imxrt10xx::ccm::Ccm::new();
//! let ports = Ports::new(&ccm);
//! let pin_from_id = ports.pin(PinId::Emc25);
//! let pin_from_port = ports.gpio4.pin(25);
//! assert_eq!(pin_from_id as *const _, pin_from_port as *const _);
//! ```

use cortexm7::support::atomic;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::hil;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

use crate::ccm;

/// General-purpose I/Os
#[repr(C)]
struct GpioRegisters {
    // GPIO data register
    dr: ReadWrite<u32>,
    // GPIO direction register
    gdir: ReadWrite<u32>,
    // GPIO pad status register
    psr: ReadOnly<u32>,
    // GPIO Interrupt configuration register 1
    icr1: ReadWrite<u32>,
    // GPIO Interrupt configuration register 2
    icr2: ReadWrite<u32>,
    // GPIO interrupt mask register
    imr: ReadWrite<u32>,
    // GPIO interrupt status register -- W1C - Write 1 to clear
    isr: ReadWrite<u32>,
    // GPIO edge select register
    edge_sel: ReadWrite<u32>,
    _reserved1: [u8; 100],
    // GPIO data register set
    dr_set: WriteOnly<u32>,
    // GPIO data register clear
    dr_clear: WriteOnly<u32>,
    // GPIO data register toggle
    dr_toggle: WriteOnly<u32>,
}

const GPIO1_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401B8000 as *const GpioRegisters) };

const GPIO2_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401BC000 as *const GpioRegisters) };

const GPIO3_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401C0000 as *const GpioRegisters) };

const GPIO4_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401C4000 as *const GpioRegisters) };

const GPIO5_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x400C0000 as *const GpioRegisters) };

enum_from_primitive! {
    /// Imxrt1050-evkb has 5 GPIO ports labeled from 1-5 [^1]. This is represented
    /// by three bits.
    ///
    /// [^1]: 12.5.1 GPIO memory map, page 1009 of the Reference Manual.
    #[repr(u16)]
    enum GpioPort {
        GPIO1 = 0b000,
        GPIO2 = 0b001,
        GPIO3 = 0b010,
        GPIO4 = 0b011,
        GPIO5 = 0b100,
    }
}

/// Creates a GPIO ID
///
/// Low 6 bits are the GPIO offset; the '17' in GPIO2[17]
/// Next 3 bits are the GPIO port; the '2' in GPIO2[17] (base 0 index, 2 -> 1)
const fn gpio_id(port: GpioPort, offset: u16) -> u16 {
    ((port as u16) << 6) | offset & 0x3F
}

/// GPIO Pin Identifiers
#[repr(u16)]
#[derive(Copy, Clone)]
pub enum PinId {
    // GPIO1
    AdB0_00 = gpio_id(GpioPort::GPIO1, 0),
    AdB0_01 = gpio_id(GpioPort::GPIO1, 1),
    AdB0_02 = gpio_id(GpioPort::GPIO1, 2),
    AdB0_03 = gpio_id(GpioPort::GPIO1, 3),
    AdB0_04 = gpio_id(GpioPort::GPIO1, 4),
    AdB0_05 = gpio_id(GpioPort::GPIO1, 5),
    AdB0_06 = gpio_id(GpioPort::GPIO1, 6),
    AdB0_07 = gpio_id(GpioPort::GPIO1, 7),
    AdB0_08 = gpio_id(GpioPort::GPIO1, 8),
    AdB0_09 = gpio_id(GpioPort::GPIO1, 9),
    AdB0_10 = gpio_id(GpioPort::GPIO1, 10),
    AdB0_11 = gpio_id(GpioPort::GPIO1, 11),
    AdB0_12 = gpio_id(GpioPort::GPIO1, 12),
    AdB0_13 = gpio_id(GpioPort::GPIO1, 13),
    AdB0_14 = gpio_id(GpioPort::GPIO1, 14),
    AdB0_15 = gpio_id(GpioPort::GPIO1, 15),

    AdB1_00 = gpio_id(GpioPort::GPIO1, 16),
    AdB1_01 = gpio_id(GpioPort::GPIO1, 17),
    AdB1_02 = gpio_id(GpioPort::GPIO1, 18),
    AdB1_03 = gpio_id(GpioPort::GPIO1, 19),
    AdB1_04 = gpio_id(GpioPort::GPIO1, 20),
    AdB1_05 = gpio_id(GpioPort::GPIO1, 21),
    AdB1_06 = gpio_id(GpioPort::GPIO1, 22),
    AdB1_07 = gpio_id(GpioPort::GPIO1, 23),
    AdB1_08 = gpio_id(GpioPort::GPIO1, 24),
    AdB1_09 = gpio_id(GpioPort::GPIO1, 25),
    AdB1_10 = gpio_id(GpioPort::GPIO1, 26),
    AdB1_11 = gpio_id(GpioPort::GPIO1, 27),
    AdB1_12 = gpio_id(GpioPort::GPIO1, 28),
    AdB1_13 = gpio_id(GpioPort::GPIO1, 29),
    AdB1_14 = gpio_id(GpioPort::GPIO1, 30),
    AdB1_15 = gpio_id(GpioPort::GPIO1, 31),

    // GPIO2
    B0_00 = gpio_id(GpioPort::GPIO2, 0),
    B0_01 = gpio_id(GpioPort::GPIO2, 1),
    B0_02 = gpio_id(GpioPort::GPIO2, 2),
    B0_03 = gpio_id(GpioPort::GPIO2, 3),
    B0_04 = gpio_id(GpioPort::GPIO2, 4),
    B0_05 = gpio_id(GpioPort::GPIO2, 5),
    B0_06 = gpio_id(GpioPort::GPIO2, 6),
    B0_07 = gpio_id(GpioPort::GPIO2, 7),
    B0_08 = gpio_id(GpioPort::GPIO2, 8),
    B0_09 = gpio_id(GpioPort::GPIO2, 9),
    B0_10 = gpio_id(GpioPort::GPIO2, 10),
    B0_11 = gpio_id(GpioPort::GPIO2, 11),
    B0_12 = gpio_id(GpioPort::GPIO2, 12),
    B0_13 = gpio_id(GpioPort::GPIO2, 13),
    B0_14 = gpio_id(GpioPort::GPIO2, 14),
    B0_15 = gpio_id(GpioPort::GPIO2, 15),

    B1_00 = gpio_id(GpioPort::GPIO2, 16),
    B1_01 = gpio_id(GpioPort::GPIO2, 17),
    B1_02 = gpio_id(GpioPort::GPIO2, 18),
    B1_03 = gpio_id(GpioPort::GPIO2, 19),
    B1_04 = gpio_id(GpioPort::GPIO2, 20),
    B1_05 = gpio_id(GpioPort::GPIO2, 21),
    B1_06 = gpio_id(GpioPort::GPIO2, 22),
    B1_07 = gpio_id(GpioPort::GPIO2, 23),
    B1_08 = gpio_id(GpioPort::GPIO2, 24),
    B1_09 = gpio_id(GpioPort::GPIO2, 25),
    B1_10 = gpio_id(GpioPort::GPIO2, 26),
    B1_11 = gpio_id(GpioPort::GPIO2, 27),
    B1_12 = gpio_id(GpioPort::GPIO2, 28),
    B1_13 = gpio_id(GpioPort::GPIO2, 29),
    B1_14 = gpio_id(GpioPort::GPIO2, 30),
    B1_15 = gpio_id(GpioPort::GPIO2, 31),

    // GPIO3
    SdB1_00 = gpio_id(GpioPort::GPIO3, 0),
    SdB1_01 = gpio_id(GpioPort::GPIO3, 1),
    SdB1_02 = gpio_id(GpioPort::GPIO3, 2),
    SdB1_03 = gpio_id(GpioPort::GPIO3, 3),
    SdB1_04 = gpio_id(GpioPort::GPIO3, 4),
    SdB1_05 = gpio_id(GpioPort::GPIO3, 5),
    SdB1_06 = gpio_id(GpioPort::GPIO3, 6),
    SdB1_07 = gpio_id(GpioPort::GPIO3, 7),
    SdB1_08 = gpio_id(GpioPort::GPIO3, 8),
    SdB1_09 = gpio_id(GpioPort::GPIO3, 9),
    SdB1_10 = gpio_id(GpioPort::GPIO3, 10),
    SdB1_11 = gpio_id(GpioPort::GPIO3, 11),

    SdB0_00 = gpio_id(GpioPort::GPIO3, 12),
    SdB0_01 = gpio_id(GpioPort::GPIO3, 13),
    SdB0_02 = gpio_id(GpioPort::GPIO3, 14),
    SdB0_03 = gpio_id(GpioPort::GPIO3, 15),
    SdB0_04 = gpio_id(GpioPort::GPIO3, 16),
    SdB0_05 = gpio_id(GpioPort::GPIO3, 17),

    Emc32 = gpio_id(GpioPort::GPIO3, 18),
    Emc33 = gpio_id(GpioPort::GPIO3, 19),
    Emc34 = gpio_id(GpioPort::GPIO3, 20),
    Emc35 = gpio_id(GpioPort::GPIO3, 21),
    Emc36 = gpio_id(GpioPort::GPIO3, 22),
    Emc37 = gpio_id(GpioPort::GPIO3, 23),
    Emc38 = gpio_id(GpioPort::GPIO3, 24),
    Emc39 = gpio_id(GpioPort::GPIO3, 25),
    Emc40 = gpio_id(GpioPort::GPIO3, 26),
    Emc41 = gpio_id(GpioPort::GPIO3, 27),

    // GPIO4
    Emc00 = gpio_id(GpioPort::GPIO4, 0),
    Emc01 = gpio_id(GpioPort::GPIO4, 1),
    Emc02 = gpio_id(GpioPort::GPIO4, 2),
    Emc03 = gpio_id(GpioPort::GPIO4, 3),
    Emc04 = gpio_id(GpioPort::GPIO4, 4),
    Emc05 = gpio_id(GpioPort::GPIO4, 5),
    Emc06 = gpio_id(GpioPort::GPIO4, 6),
    Emc07 = gpio_id(GpioPort::GPIO4, 7),
    Emc08 = gpio_id(GpioPort::GPIO4, 8),
    Emc09 = gpio_id(GpioPort::GPIO4, 9),
    Emc10 = gpio_id(GpioPort::GPIO4, 10),
    Emc11 = gpio_id(GpioPort::GPIO4, 11),
    Emc12 = gpio_id(GpioPort::GPIO4, 12),
    Emc13 = gpio_id(GpioPort::GPIO4, 13),
    Emc14 = gpio_id(GpioPort::GPIO4, 14),
    Emc15 = gpio_id(GpioPort::GPIO4, 15),
    Emc16 = gpio_id(GpioPort::GPIO4, 16),
    Emc17 = gpio_id(GpioPort::GPIO4, 17),
    Emc18 = gpio_id(GpioPort::GPIO4, 18),
    Emc19 = gpio_id(GpioPort::GPIO4, 19),
    Emc20 = gpio_id(GpioPort::GPIO4, 20),
    Emc21 = gpio_id(GpioPort::GPIO4, 21),
    Emc22 = gpio_id(GpioPort::GPIO4, 22),
    Emc23 = gpio_id(GpioPort::GPIO4, 23),
    Emc24 = gpio_id(GpioPort::GPIO4, 24),
    Emc25 = gpio_id(GpioPort::GPIO4, 25),
    Emc26 = gpio_id(GpioPort::GPIO4, 26),
    Emc27 = gpio_id(GpioPort::GPIO4, 27),
    Emc28 = gpio_id(GpioPort::GPIO4, 28),
    Emc29 = gpio_id(GpioPort::GPIO4, 29),
    Emc30 = gpio_id(GpioPort::GPIO4, 30),
    Emc31 = gpio_id(GpioPort::GPIO4, 31),

    // GPIO5
    Wakeup = gpio_id(GpioPort::GPIO5, 0),
    PmicOnReq = gpio_id(GpioPort::GPIO5, 1),
    PmicStbyReq = gpio_id(GpioPort::GPIO5, 2),
}

impl PinId {
    /// Returns the port
    fn port(self) -> GpioPort {
        GpioPort::from_u16((self as u16) >> 6).unwrap()
    }
    /// Returns the pin offset, half-closed range [0, 32)
    const fn offset(self) -> usize {
        (self as usize) & 0x3F
    }
}

/// GPIO pin mode
///
/// This describes the pin direction when it's a _GPIO pin_.
/// It does not describe the direction for other I/O, like LPI2C
/// or LPUART.
///
/// In order to set alternate functions such as LPI2C or LPUART,
/// you will need to use iomuxc enable_sw_mux_ctl_pad_gpio with
/// the specific MUX_MODE according to the reference manual (Chapter 11).
/// For the gpio mode, input or output we set the GDIR pin accordingly [^1]
///
/// [^1]: 12.4.3. GPIO Programming, page 1008 of the Reference Manual
pub enum Mode {
    Input = 0b00,
    Output = 0b01,
}

/// A GPIO port, like `GPIO3`
///
/// `Port`s contain collections of pins. Use `Port`s to access pin by their
/// GPIO offset. See the module-level docs for an example.
pub struct Port<'a, const N: usize> {
    registers: StaticRef<GpioRegisters>,
    clock: PortClock<'a>,
    pins: [Pin<'a>; N],
}

/// Implementation of a port, generic over the number of
/// pins
impl<'a, const N: usize> Port<'a, N> {
    const fn new(
        registers: StaticRef<GpioRegisters>,
        clock: PortClock<'a>,
        pins: [Pin<'a>; N],
    ) -> Self {
        Self {
            registers,
            clock,
            pins,
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    /// Returns the GPIO pin in this port
    ///
    /// This is an alterative API to [`Ports::pin`] that maps more closely
    /// to the GPIO offset.
    pub const fn pin(&self, offset: usize) -> &Pin<'a> {
        &self.pins[offset]
    }

    pub fn handle_interrupt(&self) {
        let imr_val: u32 = self.registers.imr.get();

        // Read the `ISR` register and toggle the appropriate bits in
        // `isr`. Once that is done, write the value of `isr` back. We
        // can have a situation where memory value of `ISR` could have
        // changed due to an external interrupt. `ISR` is a read/clear write
        // 1 register (`rc_w1`). So, we only clear bits whose value has been
        // transferred to `isr`.
        let isr_val = unsafe {
            atomic(|| {
                let isr_val = self.registers.isr.get();
                self.registers.isr.set(isr_val);
                isr_val
            })
        };

        BitOffsets(isr_val)
            // Did we enable this interrupt?
            .filter(|offset| imr_val & (1 << offset) != 0)
            // Do we have a pin for that interrupt? (Likely)
            .filter_map(|offset| self.pins.get(offset as usize))
            // Call client
            .for_each(|pin| {
                pin.client.map(|client| client.fired());
            });
    }
}

type GPIO1<'a> = Port<'a, 32>;
type GPIO2<'a> = Port<'a, 32>;
type GPIO3<'a> = Port<'a, 28>;
type GPIO4<'a> = Port<'a, 32>;
type GPIO5<'a> = Port<'a, 3>;

impl<'a> Port<'a, 32> {
    const fn new_32(registers: StaticRef<GpioRegisters>, clock: PortClock<'a>) -> Self {
        Self::new(
            registers,
            clock,
            [
                Pin::new(registers, 00),
                Pin::new(registers, 01),
                Pin::new(registers, 02),
                Pin::new(registers, 03),
                Pin::new(registers, 04),
                Pin::new(registers, 05),
                Pin::new(registers, 06),
                Pin::new(registers, 07),
                Pin::new(registers, 08),
                Pin::new(registers, 09),
                Pin::new(registers, 10),
                Pin::new(registers, 11),
                Pin::new(registers, 12),
                Pin::new(registers, 13),
                Pin::new(registers, 14),
                Pin::new(registers, 15),
                Pin::new(registers, 16),
                Pin::new(registers, 17),
                Pin::new(registers, 18),
                Pin::new(registers, 19),
                Pin::new(registers, 20),
                Pin::new(registers, 21),
                Pin::new(registers, 22),
                Pin::new(registers, 23),
                Pin::new(registers, 24),
                Pin::new(registers, 25),
                Pin::new(registers, 26),
                Pin::new(registers, 27),
                Pin::new(registers, 28),
                Pin::new(registers, 29),
                Pin::new(registers, 30),
                Pin::new(registers, 31),
            ],
        )
    }
    const fn gpio1(ccm: &'a ccm::Ccm) -> GPIO1<'a> {
        Self::new_32(
            GPIO1_BASE,
            PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO1)),
        )
    }
    const fn gpio2(ccm: &'a ccm::Ccm) -> GPIO2<'a> {
        Self::new_32(
            GPIO2_BASE,
            PortClock(ccm::PeripheralClock::ccgr0(ccm, ccm::HCLK0::GPIO2)),
        )
    }
    const fn gpio4(ccm: &'a ccm::Ccm) -> GPIO4<'a> {
        Self::new_32(
            GPIO4_BASE,
            PortClock(ccm::PeripheralClock::ccgr3(ccm, ccm::HCLK3::GPIO4)),
        )
    }
}

impl<'a> Port<'a, 28> {
    const fn new_28(registers: StaticRef<GpioRegisters>, clock: PortClock<'a>) -> Self {
        Self::new(
            registers,
            clock,
            [
                Pin::new(registers, 00),
                Pin::new(registers, 01),
                Pin::new(registers, 02),
                Pin::new(registers, 03),
                Pin::new(registers, 04),
                Pin::new(registers, 05),
                Pin::new(registers, 06),
                Pin::new(registers, 07),
                Pin::new(registers, 08),
                Pin::new(registers, 09),
                Pin::new(registers, 10),
                Pin::new(registers, 11),
                Pin::new(registers, 12),
                Pin::new(registers, 13),
                Pin::new(registers, 14),
                Pin::new(registers, 15),
                Pin::new(registers, 16),
                Pin::new(registers, 17),
                Pin::new(registers, 18),
                Pin::new(registers, 19),
                Pin::new(registers, 20),
                Pin::new(registers, 21),
                Pin::new(registers, 22),
                Pin::new(registers, 23),
                Pin::new(registers, 24),
                Pin::new(registers, 25),
                Pin::new(registers, 26),
                Pin::new(registers, 27),
            ],
        )
    }
    const fn gpio3(ccm: &'a ccm::Ccm) -> GPIO3<'a> {
        Self::new_28(
            GPIO3_BASE,
            PortClock(ccm::PeripheralClock::ccgr2(ccm, ccm::HCLK2::GPIO3)),
        )
    }
}

impl<'a> Port<'a, 3> {
    const fn new_3(registers: StaticRef<GpioRegisters>, clock: PortClock<'a>) -> Self {
        Self::new(
            registers,
            clock,
            [
                Pin::new(registers, 00),
                Pin::new(registers, 01),
                Pin::new(registers, 02),
            ],
        )
    }
    const fn gpio5(ccm: &'a ccm::Ccm) -> GPIO5<'a> {
        Self::new_3(
            GPIO5_BASE,
            PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO5)),
        )
    }
}

/// All GPIO ports
///
/// Use [`new`](Ports::new) to create all GPIO ports, then use it to access GPIO
/// pins and individual ports. See the public members for the GPIO ports
#[non_exhaustive] // Fast GPIOs 6 through 9 not implemented
pub struct Ports<'a> {
    pub gpio1: GPIO1<'a>,
    pub gpio2: GPIO2<'a>,
    pub gpio3: GPIO3<'a>,
    pub gpio4: GPIO4<'a>,
    pub gpio5: GPIO5<'a>,
}

impl<'a> Ports<'a> {
    pub const fn new(ccm: &'a ccm::Ccm) -> Self {
        Self {
            gpio1: GPIO1::gpio1(ccm),
            gpio2: GPIO2::gpio2(ccm),
            gpio3: GPIO3::gpio3(ccm),
            gpio4: GPIO4::gpio4(ccm),
            gpio5: GPIO5::gpio5(ccm),
        }
    }

    /// Returns a GPIO pin
    ///
    /// For an interface that maps more closely to the numbers in
    /// `GPIO3[17]`, use a combination of the [`Ports`] members, and [`Port::pin()`].
    /// See the module-level docs for an example.
    pub fn pin(&self, pin: PinId) -> &Pin<'a> {
        match pin.port() {
            GpioPort::GPIO1 => &self.gpio1.pins[pin.offset()],
            GpioPort::GPIO2 => &self.gpio2.pins[pin.offset()],
            GpioPort::GPIO3 => &self.gpio3.pins[pin.offset()],
            GpioPort::GPIO4 => &self.gpio4.pins[pin.offset()],
            GpioPort::GPIO5 => &self.gpio5.pins[pin.offset()],
        }
    }
}

struct PortClock<'a>(ccm::PeripheralClock<'a>);

impl ClockInterface for PortClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}

/// A GPIO pin, like `GPIO3[17]`
///
/// `Pin` implements the `hil::gpio` traits. To acquire a `Pin`,
///
/// - use [`Ports::pin`] to reference a `Pin` by a [`PinId`], or
/// - use a combination of the ports on [`Ports`], and [`Port::pin`]
pub struct Pin<'a> {
    registers: StaticRef<GpioRegisters>,
    offset: usize,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
}

trait U32Ext {
    fn set_bit(self, offset: usize) -> Self;
    fn clear_bit(self, offset: usize) -> Self;
    fn is_bit_set(self, offset: usize) -> bool;
}

impl U32Ext for u32 {
    #[inline(always)]
    fn set_bit(self, offset: usize) -> u32 {
        self | (1 << offset)
    }
    #[inline(always)]
    fn clear_bit(self, offset: usize) -> u32 {
        self & !(1 << offset)
    }
    #[inline(always)]
    fn is_bit_set(self, offset: usize) -> bool {
        (self & (1 << offset)) != 0
    }
}

impl<'a> Pin<'a> {
    /// Fabricate a new `Pin` from a `PinId`
    pub fn from_pin_id(pin_id: PinId) -> Self {
        Self::new(
            match pin_id.port() {
                GpioPort::GPIO1 => GPIO1_BASE,
                GpioPort::GPIO2 => GPIO2_BASE,
                GpioPort::GPIO3 => GPIO3_BASE,
                GpioPort::GPIO4 => GPIO4_BASE,
                GpioPort::GPIO5 => GPIO5_BASE,
            },
            pin_id.offset(),
        )
    }
    const fn new(registers: StaticRef<GpioRegisters>, offset: usize) -> Self {
        Pin {
            registers,
            offset,
            client: OptionalCell::empty(),
        }
    }

    fn get_mode(&self) -> Mode {
        if self.registers.gdir.get().is_bit_set(self.offset) {
            Mode::Output
        } else {
            Mode::Input
        }
    }

    fn set_mode(&self, mode: Mode) {
        let gdir = self.registers.gdir.get();
        let gdir = match mode {
            Mode::Input => gdir.clear_bit(self.offset),
            Mode::Output => gdir.set_bit(self.offset),
        };
        self.registers.gdir.set(gdir);
    }

    fn set_output_high(&self) {
        self.registers.dr_set.set(1 << self.offset);
    }

    fn set_output_low(&self) {
        self.registers.dr_clear.set(1 << self.offset);
    }

    fn is_output_high(&self) -> bool {
        self.registers.dr.get().is_bit_set(self.offset)
    }

    fn toggle_output(&self) -> bool {
        self.registers.dr_toggle.set(1 << self.offset);
        self.is_output_high()
    }

    fn read_input(&self) -> bool {
        self.registers.psr.get().is_bit_set(self.offset)
    }

    fn mask_interrupt(&self) {
        let imr = self.registers.imr.get();
        let imr = imr.clear_bit(self.offset);
        self.registers.imr.set(imr);
    }

    fn unmask_interrupt(&self) {
        let imr = self.registers.imr.get();
        let imr = imr.set_bit(self.offset);
        self.registers.imr.set(imr);
    }

    fn clear_pending(&self) {
        self.registers.isr.set(1 << self.offset); // W1C
    }

    fn set_edge_sensitive(&self, sensitive: hil::gpio::InterruptEdge) {
        use hil::gpio::InterruptEdge::*;
        const RISING_EDGE_SENSITIVE: u32 = 0b10;
        const FALLING_EDGE_SENSITIVE: u32 = 0b11;

        let edge_sel = self.registers.edge_sel.get();
        let icr_offset = (self.offset % 16) * 2;

        let sensitive = match sensitive {
            EitherEdge => {
                let edge_sel = edge_sel.set_bit(self.offset);
                self.registers.edge_sel.set(edge_sel);
                // A high EDGE_SEL disregards the corresponding ICR[1|2] setting
                return;
            }
            RisingEdge => RISING_EDGE_SENSITIVE << icr_offset,
            FallingEdge => FALLING_EDGE_SENSITIVE << icr_offset,
        };

        let edge_sel = edge_sel.clear_bit(self.offset);
        self.registers.edge_sel.set(edge_sel);

        let icr_mask = 0b11 << icr_offset;
        if self.offset < 16 {
            let icr1 = self.registers.icr1.get();
            let icr1 = (icr1 & !icr_mask) | sensitive;
            self.registers.icr1.set(icr1);
        } else {
            let icr2 = self.registers.icr2.get();
            let icr2 = (icr2 & !icr_mask) | sensitive;
            self.registers.icr2.set(icr2);
        }
    }
}

impl hil::gpio::Configure for Pin<'_> {
    fn make_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Output);
        hil::gpio::Configuration::Output
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Input);
        hil::gpio::Configuration::Input
    }

    fn deactivate_to_low_power(&self) {
        // Not implemented yet
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        // Not implemented yet
        hil::gpio::Configuration::LowPower
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        // Not implemented yet
        hil::gpio::Configuration::LowPower
    }

    // PullUp or PullDown mode are set through the Iomux module
    fn set_floating_state(&self, _mode: hil::gpio::FloatingState) {}

    fn floating_state(&self) -> hil::gpio::FloatingState {
        hil::gpio::FloatingState::PullNone
    }

    fn configuration(&self) -> hil::gpio::Configuration {
        match self.get_mode() {
            Mode::Input => hil::gpio::Configuration::Input,
            Mode::Output => hil::gpio::Configuration::Output,
        }
    }
}

impl hil::gpio::Output for Pin<'_> {
    fn set(&self) {
        self.set_output_high();
    }

    fn clear(&self) {
        self.set_output_low();
    }

    fn toggle(&self) -> bool {
        self.toggle_output()
    }
}

impl hil::gpio::Input for Pin<'_> {
    fn read(&self) -> bool {
        self.read_input()
    }
}

impl<'a> hil::gpio::Interrupt<'a> for Pin<'a> {
    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        unsafe {
            atomic(|| {
                // disable the interrupt
                self.mask_interrupt();
                self.clear_pending();
                self.set_edge_sensitive(mode);

                self.unmask_interrupt();
            });
        }
    }

    fn disable_interrupts(&self) {
        unsafe {
            atomic(|| {
                self.mask_interrupt();
                self.clear_pending();
            });
        }
    }

    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn is_pending(&self) -> bool {
        self.registers.isr.get().is_bit_set(self.offset)
    }
}

/// An iterator that returns the offsets of each high bit
///
/// Each offset is returned only once. There is no guarantee
/// for iteration order.
struct BitOffsets(u32);

impl Iterator for BitOffsets {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let offset = self.0.trailing_zeros();
            self.0 &= self.0 - 1;
            Some(offset)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let popcnt = self.0.count_ones() as usize;
        (popcnt, Some(popcnt))
    }
}

impl ExactSizeIterator for BitOffsets {}

#[cfg(test)]
mod tests {
    use super::BitOffsets;

    #[test]
    fn bit_offsets() {
        fn check(word: u32, expected: impl ExactSizeIterator<Item = u32>) {
            let offsets = BitOffsets(word);
            assert_eq!(offsets.len(), expected.len());
            assert!(
                offsets.eq(expected),
                "Incorrect bit offsets for word {:#b}",
                word
            );
        }

        check(0, core::iter::empty());
        check(u32::max_value(), 0..32);
        check(u32::max_value() >> 1, 0..31);
        check(u32::max_value() << 1, 1..32);
        check(0x5555_5555, (0..32).step_by(2));
        check(0xAAAA_AAAA, (0..32).skip(1).step_by(2));
    }
}
