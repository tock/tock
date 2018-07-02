//! Driver for the Microchip MCP23008 I2C GPIO extender.
//!
//! <http://www.microchip.com/wwwproducts/en/MCP23008>
//!
//! Paraphrased from the website:
//!
//! > The MCP23008 device provides 8-bit, general purpose, parallel I/O
//! > expansion for I2C bus applications. The MCP23008 has three address pins
//! > and consists of multiple 8-bit configuration registers for input, output
//! > and polarity selection. The system master can enable the I/Os as either
//! > inputs or outputs by writing the I/O configuration bits. The data for each
//! > input or output is kept in the corresponding Input or Output register. The
//! > polarity of the Input Port register can be inverted with the Polarity
//! > Inversion register. All registers can be read by the system master.
//!
//! Usage
//! -----
//! This capsule can either be used inside of the kernel or as an input to
//! the `gpio_async` capsule because it implements the `hil::gpio_async::Port`
//! trait.
//!
//! Example usage:
//!
//! ```rust
//! // Configure the MCP23008. Device address 0x20.
//! let mcp23008_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_mux, 0x20));
//! let mcp23008 = static_init!(
//!     capsules::mcp23008::MCP23008<'static>,
//!     capsules::mcp23008::MCP23008::new(mcp23008_i2c,
//!                                       Some(&sam4l::gpio::PA[04]),
//!                                       &mut capsules::mcp23008::BUFFER));
//! mcp23008_i2c.set_client(mcp23008);
//! sam4l::gpio::PA[04].set_client(mcp23008);
//!
//! // Create an array of the GPIO extenders so we can pass them to an
//! // administrative layer that provides a single interface to them all.
//! let async_gpio_ports = static_init!(
//!     [&'static capsules::mcp23008::MCP23008; 1],
//!     [mcp23008]);
//!
//! // `gpio_async` is the object that manages all of the extenders.
//! let gpio_async = static_init!(
//!     capsules::gpio_async::GPIOAsync<'static, capsules::mcp23008::MCP23008<'static>>,
//!     capsules::gpio_async::GPIOAsync::new(async_gpio_ports));
//! // Setup the clients correctly.
//! for port in async_gpio_ports.iter() {
//!     port.set_client(gpio_async);
//! }
//! ```
//!
//! Note that if interrupts are not needed, a `None` can be passed in when the
//! `mcp23008` object is created.

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::ReturnCode;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 7] = [0; 7];

#[allow(dead_code)]
#[derive(Debug)]
enum Registers {
    IoDir = 0x00,
    IPol = 0x01,
    GpIntEn = 0x02,
    DefVal = 0x03,
    IntCon = 0x04,
    IoCon = 0x05,
    GpPu = 0x06,
    IntF = 0x07,
    IntCap = 0x08,
    Gpio = 0x09,
    OLat = 0x0a,
}

/// States of the I2C protocol with the MCP23008.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Idle,

    // Setup input/output
    SelectIoDir(u8, Direction),
    ReadIoDir(u8, Direction),
    SelectGpPu(u8, bool),
    ReadGpPu(u8, bool),
    SetGpPu(u8),
    SelectGpio(u8, PinState),
    ReadGpio(u8, PinState),
    SelectGpioToggle(u8),
    ReadGpioToggle(u8),
    SelectGpioRead(u8),
    ReadGpioRead(u8),
    EnableInterruptSettings,
    ReadInterruptSetup,
    ReadInterruptValues,

    /// Disable I2C and release buffer
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Input = 0x01,
    Output = 0x00,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PinState {
    High = 0x01,
    Low = 0x00,
}

pub struct MCP23008<'a> {
    i2c: &'a hil::i2c::I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    interrupt_pin: Option<&'static hil::gpio::Pin>,
    interrupt_settings: Cell<u32>, // Whether the pin interrupt is enabled, and what mode it's in.
    identifier: Cell<usize>,
    client: Cell<Option<&'static hil::gpio_async::Client>>,
}

impl MCP23008<'a> {
    pub fn new(
        i2c: &'a hil::i2c::I2CDevice,
        interrupt_pin: Option<&'static hil::gpio::Pin>,
        buffer: &'static mut [u8],
    ) -> MCP23008<'a> {
        MCP23008 {
            i2c: i2c,
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            interrupt_pin: interrupt_pin,
            interrupt_settings: Cell::new(0),
            identifier: Cell::new(0),
            client: Cell::new(None),
        }
    }

    /// Set the client of this MCP23008 when commands finish or interrupts
    /// occur. The `identifier` is simply passed back with the callback
    /// so that the upper layer can keep track of which device triggered.
    pub fn set_client<C: hil::gpio_async::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    fn enable_host_interrupt(&self) -> ReturnCode {
        // We configure the MCP23008 to use an active high interrupt.
        // If we don't have an interrupt pin mapped to this driver then we
        // obviously can't do interrupts.
        self.interrupt_pin
            .map_or(ReturnCode::FAIL, |interrupt_pin| {
                interrupt_pin.make_input();
                interrupt_pin.enable_interrupt(0, hil::gpio::InterruptMode::RisingEdge);
                ReturnCode::SUCCESS
            })
    }

    fn set_direction(&self, pin_number: u8, direction: Direction) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::IoDir as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectIoDir(pin_number, direction));

            ReturnCode::SUCCESS
        })
    }

    /// Set the pull-up on the pin also configure it to be an input.
    fn configure_pullup(&self, pin_number: u8, enabled: bool) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::IoDir as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpPu(pin_number, enabled));

            ReturnCode::SUCCESS
        })
    }

    fn set_pin(&self, pin_number: u8, value: PinState) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::Gpio as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpio(pin_number, value));

            ReturnCode::SUCCESS
        })
    }

    fn toggle_pin(&self, pin_number: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::Gpio as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpioToggle(pin_number));

            ReturnCode::SUCCESS
        })
    }

    fn read_pin(&self, pin_number: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::Gpio as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpioRead(pin_number));

            ReturnCode::SUCCESS
        })
    }

    fn enable_interrupt_pin(
        &self,
        pin_number: u8,
        direction: hil::gpio::InterruptMode,
    ) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            // Mark the settings that we have for this interrupt.
            // Since the MCP23008 only seems to support level interrupts
            // and both edge interrupts, we choose to use both edge interrupts
            // and then filter here in the driver if the user only asked
            // for one direction interrupts. To do this, we need to know what
            // the user asked for.
            self.save_pin_interrupt_state(pin_number, true, direction);

            // Setup interrupt configs that are true of all interrupts
            buffer[0] = Registers::IntCon as u8;
            buffer[1] = 0; // Make all pins toggle on every change.
            buffer[2] = 0b00000010; // Make MCP23008 interrupt pin active high.
            self.i2c.write(buffer, 3);
            self.state.set(State::EnableInterruptSettings);

            ReturnCode::SUCCESS
        })
    }

    fn disable_interrupt_pin(&self, pin_number: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            // Clear this interrupt from our setup.
            self.remove_pin_interrupt_state(pin_number);

            // Just have to write the new interrupt settings.
            buffer[0] = Registers::GpIntEn as u8;
            buffer[1] = self.get_pin_interrupt_enabled_state();
            self.i2c.write(buffer, 2);
            self.state.set(State::Done);

            ReturnCode::SUCCESS
        })
    }

    /// Helper function for keeping track of which interrupts are currently
    /// enabled.
    fn save_pin_interrupt_state(
        &self,
        pin_number: u8,
        enabled: bool,
        direction: hil::gpio::InterruptMode,
    ) {
        let mut current_state = self.interrupt_settings.get();
        // Clear out existing settings
        current_state &= !(0x0F << (4 * pin_number));
        // Generate new settings
        let new_settings = (((enabled as u8) | ((direction as u8) << 1)) & 0x0F) as u32;
        // Update settings
        current_state |= new_settings << (4 * pin_number);
        self.interrupt_settings.set(current_state);
    }

    fn remove_pin_interrupt_state(&self, pin_number: u8) {
        let new_settings = self.interrupt_settings.get() & !(0x0F << (4 * pin_number));
        self.interrupt_settings.set(new_settings);
    }

    /// Create an 8 bit bitmask of which interrupts are enabled.
    fn get_pin_interrupt_enabled_state(&self) -> u8 {
        let current_state = self.interrupt_settings.get();
        let mut interrupts_enabled: u8 = 0;
        for i in 0..8 {
            if ((current_state >> (4 * i)) & 0x01) == 0x01 {
                interrupts_enabled |= 1 << i;
            }
        }
        interrupts_enabled
    }

    fn check_pin_interrupt_enabled(&self, pin_number: u8) -> bool {
        (self.interrupt_settings.get() >> (pin_number * 4)) & 0x01 == 0x01
    }

    fn get_pin_interrupt_direction(&self, pin_number: u8) -> hil::gpio::InterruptMode {
        let direction = self.interrupt_settings.get() >> ((pin_number * 4) + 1) & 0x03;
        match direction {
            0 => hil::gpio::InterruptMode::RisingEdge,
            1 => hil::gpio::InterruptMode::FallingEdge,
            _ => hil::gpio::InterruptMode::EitherEdge,
        }
    }
}

impl hil::i2c::I2CClient for MCP23008<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: hil::i2c::Error) {
        match self.state.get() {
            State::SelectIoDir(pin_number, direction) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadIoDir(pin_number, direction));
            }
            State::ReadIoDir(pin_number, direction) => {
                if direction == Direction::Input {
                    buffer[1] = buffer[0] | (1 << pin_number);
                } else {
                    buffer[1] = buffer[0] & !(1 << pin_number);
                }
                buffer[0] = Registers::IoDir as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectGpPu(pin_number, enabled) => {
                self.i2c.read(buffer, 7);
                self.state.set(State::ReadGpPu(pin_number, enabled));
            }
            State::ReadGpPu(pin_number, enabled) => {
                // Make sure the pin is enabled.
                buffer[1] = buffer[0] | (1 << pin_number);
                // Configure the pullup status and save it in the buffer.
                let pullup = match enabled {
                    true => buffer[6] | (1 << pin_number),
                    false => buffer[6] & !(1 << pin_number),
                };
                buffer[0] = Registers::IoDir as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::SetGpPu(pullup));
            }
            State::SetGpPu(pullup) => {
                // Now write the pull up settings to the chip.
                buffer[0] = Registers::GpPu as u8;
                buffer[1] = pullup;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectGpio(pin_number, value) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpio(pin_number, value));
            }
            State::ReadGpio(pin_number, value) => {
                buffer[1] = match value {
                    PinState::High => buffer[0] | (1 << pin_number),
                    PinState::Low => buffer[0] & !(1 << pin_number),
                };
                buffer[0] = Registers::Gpio as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectGpioToggle(pin_number) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpioToggle(pin_number));
            }
            State::ReadGpioToggle(pin_number) => {
                buffer[1] = buffer[0] ^ (1 << pin_number);
                buffer[0] = Registers::Gpio as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectGpioRead(pin_number) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpioRead(pin_number));
            }
            State::ReadGpioRead(pin_number) => {
                let pin_value = (buffer[0] >> pin_number) & 0x01;

                self.client.get().map(|client| {
                    client.done(pin_value as usize);
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::EnableInterruptSettings => {
                // Rather than read the current interrupts and write those
                // back, just write the entire register with our saved state.
                buffer[0] = Registers::GpIntEn as u8;
                buffer[1] = self.get_pin_interrupt_enabled_state();
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::ReadInterruptSetup => {
                // Now read the interrupt flags and the state of the lines
                self.i2c.read(buffer, 3);
                self.state.set(State::ReadInterruptValues);
            }
            State::ReadInterruptValues => {
                let interrupt_flags = buffer[0];
                let pins_status = buffer[2];
                // Check each bit to see if that pin triggered an interrupt.
                for i in 0..8 {
                    // Check that this pin is actually enabled.
                    if !self.check_pin_interrupt_enabled(i) {
                        continue;
                    }
                    if (interrupt_flags >> i) & 0x01 == 0x01 {
                        // Use the GPIO register to determine which way the
                        // interrupt went.
                        let pin_status = (pins_status >> i) & 0x01;
                        let interrupt_direction = self.get_pin_interrupt_direction(i);
                        // Check to see if this was an interrupt we want
                        // to report.
                        let fire_interrupt = match interrupt_direction {
                            hil::gpio::InterruptMode::EitherEdge => true,
                            hil::gpio::InterruptMode::RisingEdge => pin_status == 0x01,
                            hil::gpio::InterruptMode::FallingEdge => pin_status == 0x00,
                        };
                        if fire_interrupt {
                            // Signal this interrupt to the application.
                            self.client.get().map(|client| {
                                // Return both the pin that interrupted and
                                // the identifier that was passed for
                                // enable_interrupt.
                                client.fired(i as usize, self.identifier.get());
                            });
                            break;
                        }
                    }
                }
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::Done => {
                self.client.get().map(|client| {
                    client.done(0);
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl hil::gpio::Client for MCP23008<'a> {
    fn fired(&self, _: usize) {
        self.buffer.take().map(|buffer| {
            self.i2c.enable();

            // Need to read the IntF register which marks which pins
            // interrupted.
            buffer[0] = Registers::IntF as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::ReadInterruptSetup);
        });
    }
}

impl hil::gpio_async::Port for MCP23008<'a> {
    fn disable(&self, pin: usize) -> ReturnCode {
        // Best we can do is make this an input.
        self.set_direction(pin as u8, Direction::Input)
    }

    fn make_output(&self, pin: usize) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        self.set_direction(pin as u8, Direction::Output)
    }

    fn make_input(&self, pin: usize, mode: hil::gpio::InputMode) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        match mode {
            hil::gpio::InputMode::PullUp => self.configure_pullup(pin as u8, true),
            hil::gpio::InputMode::PullDown => {
                // No support for this
                self.configure_pullup(pin as u8, false)
            }
            hil::gpio::InputMode::PullNone => self.configure_pullup(pin as u8, false),
        }
    }

    fn read(&self, pin: usize) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        self.read_pin(pin as u8)
    }

    fn toggle(&self, pin: usize) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        self.toggle_pin(pin as u8)
    }

    fn set(&self, pin: usize) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        self.set_pin(pin as u8, PinState::High)
    }

    fn clear(&self, pin: usize) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        self.set_pin(pin as u8, PinState::Low)
    }

    fn enable_interrupt(
        &self,
        pin: usize,
        mode: hil::gpio::InterruptMode,
        identifier: usize,
    ) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        let ret = self.enable_host_interrupt();
        match ret {
            ReturnCode::SUCCESS => {
                self.identifier.set(identifier);
                self.enable_interrupt_pin(pin as u8, mode)
            }
            _ => ret,
        }
    }

    fn disable_interrupt(&self, pin: usize) -> ReturnCode {
        if pin > 7 {
            return ReturnCode::EINVAL;
        }
        self.disable_interrupt_pin(pin as u8)
    }
}
