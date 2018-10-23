//! General Purpose Input Output (GPIO)
//!
//! For details see p.987 in the cc2650 technical reference manual.
//!
//! Configures the GPIO pins, and interfaces with the HIL for gpio.

//
// There are up to 32 signals (AUXIO0 to AUXIO31) in the sensor controller domain (AUX Domain). These
// signals can be routed to specific DIO pins given in Table 13-2. The signals AUXIO19 to AUXIO26 have
// analog capability, but can also be used as digital I/Os. All the other AUXIOn signals are digital only. The
// signals routed from the AUX Domain are configured differently than GPIO and other peripheral functions.
//
//

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::gpio::PinCtl;

use cortexm4::nvic;
use ioc;
use peripheral_interrupts;

pub const NUM_PINS: usize = 32;

const IOC_BASE: StaticRef<ioc::Registers> =
    unsafe { StaticRef::new(0x4008_1000 as *const ioc::Registers) };

const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40022000 as *const GpioRegisters) };

#[repr(C)]
struct GpioRegisters {
    _reserved0: [u8; 0x90],
    pub dout_set: WriteOnly<u32>,
    _reserved1: [u8; 0xC],
    pub dout_clr: WriteOnly<u32>,
    _reserved2: [u8; 0xC],
    pub dout_tgl: WriteOnly<u32>,
    _reserved3: [u8; 0xC],
    pub din: ReadWrite<u32>,
    _reserved4: [u8; 0xC],
    pub doe: ReadWrite<u32>,
    _reserved5: [u8; 0xC],
    pub evflags: ReadWrite<u32>,
}

pub struct GPIOPin {
    registers: StaticRef<GpioRegisters>,
    ioc_registers: StaticRef<ioc::Registers>,
    pin: usize,
    pin_mask: u32,
    client_data: Cell<usize>,
    client: OptionalCell<&'static hil::gpio::Client>,
}

impl GPIOPin {
    const fn new(pin: usize) -> GPIOPin {
        GPIOPin {
            registers: GPIO_BASE,
            ioc_registers: IOC_BASE,
            pin: pin,
            pin_mask: 1 << pin,
            client_data: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| {
            client.fired(self.client_data.get());
        });
    }
}

/// Pinmux implementation (IOC)
impl GPIOPin {
    pub fn enable_gpio(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        // In order to configure the pin for GPIO we need to clear
        // the lower 6 bits.
        pin_ioc.write(ioc::Config::PORT_ID::GPIO);
    }

    pub fn enable_output(&self) {
        // Enable by disabling input
        let pin_ioc = &self.ioc_registers.cfg[self.pin];
        pin_ioc.modify(ioc::Config::INPUT_EN::CLEAR);
    }

    pub fn enable_input(&self) {
        // Set IE (Input Enable) bit
        let pin_ioc = &self.ioc_registers.cfg[self.pin];
        pin_ioc.modify(ioc::Config::INPUT_EN::SET);
    }

    pub fn enable_interrupt(&self, mode: hil::gpio::InterruptMode) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        let ioc_edge_mode = match mode {
            hil::gpio::InterruptMode::FallingEdge => ioc::Config::EDGE_DET::FallingEdge,
            hil::gpio::InterruptMode::RisingEdge => ioc::Config::EDGE_DET::RisingEdge,
            hil::gpio::InterruptMode::EitherEdge => ioc::Config::EDGE_DET::BothEdges,
        };

        pin_ioc.modify(ioc_edge_mode + ioc::Config::EDGE_IRQ_EN::SET);
    }

    pub fn disable_interrupt(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];
        pin_ioc.modify(ioc::Config::EDGE_IRQ_EN::CLEAR);
    }

    /// Configures pin for I2C SDA
    pub fn enable_i2c_sda(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(
            ioc::Config::PORT_ID::I2C_MSSDA
                + ioc::Config::IO_MODE::OpenDrain
                + ioc::Config::PULL::Up,
        );
        self.enable_input();
    }

    /// Configures pin for I2C SDA
    pub fn enable_i2c_scl(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(
            ioc::Config::PORT_ID::I2C_MSSCL
                + ioc::Config::IO_MODE::OpenDrain
                + ioc::Config::PULL::Up,
        );
        // TODO(alevy): I couldn't find any justification for enabling input mode in the datasheet,
        // but I2C master seems not to work without it. Maybe it's important for multi-master mode,
        // or for allowing a slave to stretch the clock, but in any case, I2C master won't actually
        // output anything without this line.
        self.enable_input();
    }

    /// Configures pin for UART0 receive (RX).
    pub fn enable_uart0_rx(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(ioc::Config::PORT_ID::UART0_RX);
        self.set_input_mode(hil::gpio::InputMode::PullNone);
        self.enable_input();
    }

    // Configures pin for UART0 transmit (TX).
    pub fn enable_uart0_tx(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(ioc::Config::PORT_ID::UART0_TX);
        self.set_input_mode(hil::gpio::InputMode::PullNone);
        self.enable_output();
    }

    // Configures pin for UART1 receive (RX).
    pub fn enable_uart1_rx(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(ioc::Config::PORT_ID::UART1_RX);
        self.set_input_mode(hil::gpio::InputMode::PullNone);
        self.enable_input();
    }

    // Configures pin for UART1 transmit (TX).
    pub fn enable_uart1_tx(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(ioc::Config::PORT_ID::UART1_TX);
        self.set_input_mode(hil::gpio::InputMode::PullNone);
        self.enable_output();
    }

    // Rewrite of using the IOC_STD_OUTPUT macro
    pub fn enable_standard_output(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.modify(
            ioc::Config::CURRENT_MODE::Low
                + ioc::Config::DRIVE_STRENGTH::Auto
                + ioc::Config::PULL::None
                + ioc::Config::SLEW_RED::CLEAR
                + ioc::Config::HYST_EN::CLEAR
                + ioc::Config::IO_MODE::Normal
                + ioc::Config::WAKEUP_CFG::CLEAR
                + ioc::Config::INPUT_EN::CLEAR,
        );
    }

    // configure a pin as an input for 32kHz system clock
    pub fn enable_32khz_system_clock_input(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];
        pin_ioc.write(
            ioc::Config::PORT_ID::AON_CLK32K
                + ioc::Config::CURRENT_MODE::Low
                + ioc::Config::DRIVE_STRENGTH::Auto
                + ioc::Config::PULL::None
                + ioc::Config::SLEW_RED::CLEAR
                + ioc::Config::HYST_EN::SET
                + ioc::Config::IO_MODE::Normal
                + ioc::Config::WAKEUP_CFG::CLEAR
                + ioc::Config::INPUT_EN::SET,
        );
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        let field = match mode {
            hil::gpio::InputMode::PullDown => ioc::Config::PULL::Down,
            hil::gpio::InputMode::PullUp => ioc::Config::PULL::Up,
            hil::gpio::InputMode::PullNone => ioc::Config::PULL::None,
        };

        pin_ioc.modify(field);
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn make_output(&self) {
        self.enable_gpio();
        // Disable input in the io configuration
        self.enable_output();
        // Enable data output
        let regs = &*self.registers;
        regs.doe.set(regs.doe.get() | self.pin_mask);
    }

    fn make_input(&self) {
        self.enable_gpio();
        self.enable_input();
    }

    fn disable(&self) {
        hil::gpio::PinCtl::set_input_mode(self, hil::gpio::InputMode::PullNone);
    }

    fn set(&self) {
        let regs = &*self.registers;
        regs.dout_set.set(self.pin_mask);
    }

    fn clear(&self) {
        let regs = &*self.registers;
        regs.dout_clr.set(self.pin_mask);
    }

    fn toggle(&self) {
        let regs = &*self.registers;
        regs.dout_tgl.set(self.pin_mask);
    }

    fn read(&self) -> bool {
        let regs = &*self.registers;
        regs.din.get() & self.pin_mask != 0
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        self.client_data.set(client_data);
        self.enable_interrupt(mode);
    }

    fn disable_interrupt(&self) {
        self.disable_interrupt();
    }
}

pub struct Port {
    nvic: &'static nvic::Nvic,
    pins: [GPIOPin; NUM_PINS],
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
    pub fn handle_events(&self) {
        let regs = GPIO_BASE;
        let evflags = regs.evflags.get();
        // Clear all interrupts by setting their bits to 1 in evflags
        regs.evflags.set(evflags);

        // evflags indicate which pins has triggered an interrupt,
        // we need to call the respective handler for positive bit in evflags.
        let mut pin: usize = usize::max_value();
        while pin < self.pins.len() {
            pin = evflags.trailing_zeros() as usize;
            if pin >= self.pins.len() {
                break;
            }

            self.pins[pin].handle_interrupt();
        }
        self.nvic.clear_pending();
        self.nvic.enable();
    }
}

const GPIO_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::GPIO as u32) };

pub static mut PORT: Port = Port {
    nvic: &GPIO_NVIC,
    pins: [
        GPIOPin::new(0),
        GPIOPin::new(1),
        GPIOPin::new(2),
        GPIOPin::new(3),
        GPIOPin::new(4),
        GPIOPin::new(5),
        GPIOPin::new(6),
        GPIOPin::new(7),
        GPIOPin::new(8),
        GPIOPin::new(9),
        GPIOPin::new(10),
        GPIOPin::new(11),
        GPIOPin::new(12),
        GPIOPin::new(13),
        GPIOPin::new(14),
        GPIOPin::new(15),
        GPIOPin::new(16),
        GPIOPin::new(17),
        GPIOPin::new(18),
        GPIOPin::new(19),
        GPIOPin::new(20),
        GPIOPin::new(21),
        GPIOPin::new(22),
        GPIOPin::new(23),
        GPIOPin::new(24),
        GPIOPin::new(25),
        GPIOPin::new(26),
        GPIOPin::new(27),
        GPIOPin::new(28),
        GPIOPin::new(29),
        GPIOPin::new(30),
        GPIOPin::new(31),
    ],
};
