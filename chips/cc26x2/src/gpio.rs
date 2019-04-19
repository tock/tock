//! General Purpose Input Output (GPIO)
//!
//! For details see p.987 in the cc2650 technical reference manual.
//!
//! Configures the GPIO pins, and interfaces with the HIL for gpio.

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{FieldValue, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;

use crate::event;
use crate::ioc;
use crate::peripheral_interrupts;
use crate::pwm;
use cortexm4::nvic;

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
    fn standard_io(
        &self,
        port_id: FieldValue<u32, ioc::Config::Register>,
        io: FieldValue<u32, ioc::Config::Register>,
    ) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.write(
            port_id
                + ioc::Config::DRIVE_STRENGTH::Auto
                + ioc::Config::PULL::None
                + ioc::Config::SLEW_RED::CLEAR
                + ioc::Config::HYST_EN::CLEAR
                + io
                + ioc::Config::WAKEUP_CFG::CLEAR,
        );
    }

    // Rewrite of using the IOC_STD_OUTPUT macro
    fn standard_input(&self, port_id: FieldValue<u32, ioc::Config::Register>) {
        self.standard_io(port_id, ioc::Config::INPUT_EN::SET);
    }

    // Rewrite of using the IOC_STD_OUTPUT macro
    fn standard_output(&self, port_id: FieldValue<u32, ioc::Config::Register>) {
        self.standard_io(port_id, ioc::Config::INPUT_EN::CLEAR);
    }

    pub fn enable_gpio(&self) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];
        pin_ioc.modify(ioc::Config::PORT_ID::GPIO);
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

    pub fn enable_int(&self, mode: hil::gpio::InterruptMode) {
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

    fn set_i2c_input(&self, port_id: FieldValue<u32, ioc::Config::Register>) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.write(
            port_id
            + ioc::Config::DRIVE_STRENGTH::Auto
            + ioc::Config::PULL::None
            + ioc::Config::SLEW_RED::CLEAR
            + ioc::Config::HYST_EN::CLEAR
            + ioc::Config::IO_MODE::OpenDrain   // this is the special setting for I2C
            + ioc::Config::WAKEUP_CFG::CLEAR
            + ioc::Config::INPUT_EN::SET,
        );
    }

    /// Configures pin for I2C SDA
    pub fn enable_i2c_sda(&self) {
        self.set_i2c_input(ioc::Config::PORT_ID::I2C_MSSDA);
    }

    /// Configures pin for I2C SDA
    pub fn enable_i2c_scl(&self) {
        self.set_i2c_input(ioc::Config::PORT_ID::I2C_MSSCL);
    }

    fn pwm_output(&self, port_id: FieldValue<u32, ioc::Config::Register>) {
        let pin_ioc = &self.ioc_registers.cfg[self.pin];

        pin_ioc.write(
            port_id
                + ioc::Config::DRIVE_STRENGTH::Max
                + ioc::Config::PULL::None
                + ioc::Config::SLEW_RED::CLEAR
                + ioc::Config::HYST_EN::CLEAR
                + ioc::Config::IO_MODE::Normal
                + ioc::Config::WAKEUP_CFG::CLEAR
                + ioc::Config::INPUT_EN::CLEAR,
        );
    }

    // Configures pin for PWM
    // In addition, The PORT_EVENT must be connected to the timer periperhal
    pub fn enable_pwm(&self, pwm: pwm::Timer) {
        let port_id;
        match pwm {
            pwm::Timer::GPT0A => {
                event::REG.gpt0a_sel.write(event::Gpt0A::EVENT::PORT_EVENT0);
                port_id = ioc::Config::PORT_ID::PORT_EVENT0;
            }
            pwm::Timer::GPT0B => {
                event::REG.gpt0b_sel.write(event::Gpt0B::EVENT::PORT_EVENT1);
                port_id = ioc::Config::PORT_ID::PORT_EVENT1;
            }
            pwm::Timer::GPT1A => {
                event::REG.gpt1a_sel.write(event::Gpt1A::EVENT::PORT_EVENT2);
                port_id = ioc::Config::PORT_ID::PORT_EVENT2;
            }
            pwm::Timer::GPT1B => {
                event::REG.gpt1b_sel.write(event::Gpt1B::EVENT::PORT_EVENT3);
                port_id = ioc::Config::PORT_ID::PORT_EVENT3;
            }
            pwm::Timer::GPT2A => {
                event::REG.gpt2a_sel.write(event::Gpt2A::EVENT::PORT_EVENT4);
                port_id = ioc::Config::PORT_ID::PORT_EVENT4;
            }
            pwm::Timer::GPT2B => {
                event::REG.gpt2b_sel.write(event::Gpt2B::EVENT::PORT_EVENT5);
                port_id = ioc::Config::PORT_ID::PORT_EVENT5;
            }
            pwm::Timer::GPT3A => {
                event::REG.gpt3a_sel.write(event::Gpt3A::EVENT::PORT_EVENT6);
                port_id = ioc::Config::PORT_ID::PORT_EVENT6;
            }
            pwm::Timer::GPT3B => {
                event::REG.gpt3b_sel.write(event::Gpt3B::EVENT::PORT_EVENT7);
                port_id = ioc::Config::PORT_ID::PORT_EVENT7;
            }
        }
        self.pwm_output(port_id);
    }

    /// Configures pin for UART0 receive (RX).
    pub fn enable_uart0_rx(&self) {
        self.standard_input(ioc::Config::PORT_ID::UART0_RX);
    }

    // Configures pin for UART0 transmit (TX).
    pub fn enable_uart0_tx(&self) {
        self.standard_output(ioc::Config::PORT_ID::UART0_TX);
    }

    // Configures pin for UART1 receive (RX).
    pub fn enable_uart1_rx(&self) {
        self.standard_input(ioc::Config::PORT_ID::UART1_RX);
    }

    // Configures pin for UART1 transmit (TX).
    pub fn enable_uart1_tx(&self) {
        self.standard_output(ioc::Config::PORT_ID::UART1_TX);
    }

    pub fn enable_analog_input(&self) {
        self.standard_input(ioc::Config::PORT_ID::AUX_DOMAIN_IO);
    }

    pub fn enable_analog_output(&self) {
        self.standard_output(ioc::Config::PORT_ID::AUX_DOMAIN_IO);
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
        self.enable_int(mode);
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
    pub fn handle_interrupt(&self) {
        let regs = GPIO_BASE;
        let mut evflags = regs.evflags.get();
        // Clear all interrupts by setting their bits to 1 in evflags
        regs.evflags.set(evflags);

        let mut count = 0;
        while evflags != 0 && count < self.pins.len() {
            if (evflags & 0b1) != 0 {
                self.pins[count].handle_interrupt();
            }
            count += 1;
            evflags >>= 1;
        }

        self.nvic.clear_pending();
        self.nvic.enable();
    }
}

const GPIO_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NvicIrq::Gpio as u32) };

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
