//! Driver for the Microchip MCP230xx I2C GPIO extenders.
//!
//! - <https://www.microchip.com/wwwproducts/en/MCP23008>
//! - <https://www.microchip.com/wwwproducts/en/MCP23017>
//!
//! Paraphrased from the website for the MCP23008:
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
//! This driver can support the MCP230xx series GPIO extenders with a
//! configurable number of banks.
//!
//! Usage
//! -----
//! This capsule can either be used inside of the kernel or as an input to
//! the `gpio_async` capsule because it implements the `gpio_async::Port`
//! trait.
//!
//! Example usage:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! // Configure the MCP230xx. Device address 0x20.
//! let mcp230xx_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_mux, 0x20));
//! let mcp230xx = static_init!(
//!     capsules::mcp230xx::MCP230xx<'static>,
//!     capsules::mcp230xx::MCP230xx::new(mcp230xx_i2c,
//!                                       Some(&sam4l::gpio::PA[04]),
//!                                       None,
//!                                       &mut capsules::mcp230xx::BUFFER,
//!                                       8, // How many pins in a bank
//!                                       1, // How many pin banks on the chip
//!                                       ));
//! mcp230xx_i2c.set_client(mcp230xx);
//! sam4l::gpio::PA[04].set_client(mcp230xx);
//!
//! // Create an array of the GPIO extenders so we can pass them to an
//! // administrative layer that provides a single interface to them all.
//! let async_gpio_ports = static_init!(
//!     [&'static capsules::mcp230xx::MCP230xx; 1],
//!     [mcp230xx]);
//!
//! // `gpio_async` is the object that manages all of the extenders.
//! let gpio_async = static_init!(
//!     capsules::gpio_async::GPIOAsync<'static, capsules::mcp230xx::MCP230xx<'static>>,
//!     capsules::gpio_async::GPIOAsync::new(async_gpio_ports));
//! // Setup the clients correctly.
//! for port in async_gpio_ports.iter() {
//!     port.set_client(gpio_async);
//! }
//! ```
//!
//! Note that if interrupts are not needed, a `None` can be passed in when the
//! `mcp230xx` object is created.

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::gpio_async;
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

/// States of the I2C protocol with the MCP230xx.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Idle,

    // Setup input/output
    SelectIoDir(u8, Direction),
    ReadIoDir(u8, Direction),
    SelectIoDirForGpPu(u8, bool),
    ReadIoDirForGpPu(u8, bool),
    SetIoDirForGpPu(u8, bool),
    ReadGpPu(u8, bool),
    SelectGpio(u8, PinState),
    ReadGpio(u8, PinState),
    SelectGpioToggle(u8),
    ReadGpioToggle(u8),
    SelectGpioRead(u8),
    ReadGpioRead(u8),
    EnableInterruptSettings(u8),
    ReadInterruptSetup(u8),
    ReadInterruptValues(u8),

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

pub struct MCP230xx<'a> {
    i2c: &'a dyn hil::i2c::I2CDevice,
    state: Cell<State>,
    bank_size: u8,       // How many GPIO pins per bank (likely 8)
    number_of_banks: u8, // How many GPIO banks this extender has (likely 1 or 2)
    buffer: TakeCell<'static, [u8]>,
    interrupt_pin_a: Option<&'a dyn gpio::InterruptValuePin<'a>>,
    interrupt_pin_b: Option<&'a dyn gpio::InterruptValuePin<'a>>,
    interrupts_enabled: Cell<u32>, // Whether the pin interrupt is enabled
    interrupts_mode: Cell<u32>,    // What interrupt mode the pin is in
    client: OptionalCell<&'static dyn gpio_async::Client>,
}

impl<'a> MCP230xx<'a> {
    pub fn new(
        i2c: &'a dyn hil::i2c::I2CDevice,
        interrupt_pin_a: Option<&'a dyn gpio::InterruptValuePin<'a>>,
        interrupt_pin_b: Option<&'a dyn gpio::InterruptValuePin<'a>>,
        buffer: &'static mut [u8],
        bank_size: u8,
        number_of_banks: u8,
    ) -> MCP230xx<'a> {
        MCP230xx {
            i2c: i2c,
            state: Cell::new(State::Idle),
            bank_size: bank_size,
            number_of_banks: number_of_banks,
            buffer: TakeCell::new(buffer),
            interrupt_pin_a: interrupt_pin_a,
            interrupt_pin_b: interrupt_pin_b,
            interrupts_enabled: Cell::new(0),
            interrupts_mode: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }

    /// Set the client of this MCP230xx when commands finish or interrupts
    /// occur. The `identifier` is simply passed back with the callback
    /// so that the upper layer can keep track of which device triggered.
    pub fn set_client<C: gpio_async::Client>(&self, client: &'static C) {
        self.client.set(client);
    }

    fn enable_host_interrupt(&self) -> ReturnCode {
        // We configure the MCP230xx to use an active high interrupt.
        // If we don't have an interrupt pin mapped to this driver then we
        // obviously can't do interrupts.
        let first = self
            .interrupt_pin_a
            .map_or(ReturnCode::FAIL, |interrupt_pin| {
                interrupt_pin.make_input();
                interrupt_pin.enable_interrupts(gpio::InterruptEdge::RisingEdge);
                ReturnCode::SUCCESS
            });
        if first != ReturnCode::SUCCESS {
            return first;
        }
        // Also do the other interrupt pin if it exists.
        self.interrupt_pin_b.map(|interrupt_pin| {
            interrupt_pin.make_input();
            interrupt_pin.enable_interrupts(gpio::InterruptEdge::RisingEdge);
        });
        ReturnCode::SUCCESS
    }

    /// This calculates the actual register address to use based on the list of
    /// registers in the `Registers` enum definitions. This is needed because
    /// the addresses are different for single- and multi-port mcp230xx
    /// extenders.
    ///
    /// If this is a single port extender then the register index is the same as
    /// the `Registers` enum and what is passed in is returned. If the chip has
    /// multiple banks then the register address is shifted based on the number
    /// and size of the bank.
    fn calc_register_addr(&self, register: Registers, pin_number: u8) -> u8 {
        if self.number_of_banks == 1 {
            pin_number as u8
        } else {
            // Calculate an offset based on which bank this pin is in.
            let offset = pin_number / self.bank_size;
            // The register index is then the original value multiplied by
            // the number of banks, plus the offset.
            (register as u8 * self.number_of_banks) + offset
        }
    }

    fn set_direction(&self, pin_number: u8, direction: Direction) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = self.calc_register_addr(Registers::IoDir, pin_number);
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectIoDir(pin_number, direction));

            ReturnCode::SUCCESS
        })
    }

    /// Set the pull-up on the pin also configure it to be an input.
    fn configure_pullup(&self, pin_number: u8, enabled: bool) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = self.calc_register_addr(Registers::IoDir, pin_number);
            self.i2c.write(buffer, 1);
            self.state
                .set(State::SelectIoDirForGpPu(pin_number, enabled));

            ReturnCode::SUCCESS
        })
    }

    fn set_pin(&self, pin_number: u8, value: PinState) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = self.calc_register_addr(Registers::Gpio, pin_number);
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpio(pin_number, value));

            ReturnCode::SUCCESS
        })
    }

    fn toggle_pin(&self, pin_number: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = self.calc_register_addr(Registers::Gpio, pin_number);
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpioToggle(pin_number));

            ReturnCode::SUCCESS
        })
    }

    fn read_pin(&self, pin_number: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            buffer[0] = self.calc_register_addr(Registers::Gpio, pin_number);
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpioRead(pin_number));

            ReturnCode::SUCCESS
        })
    }

    fn enable_interrupt_pin(&self, pin_number: u8, direction: gpio::InterruptEdge) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            // Mark the settings that we have for this interrupt.
            // Since the MCP230xx only seems to support level interrupts
            // and both edge interrupts, we choose to use both edge interrupts
            // and then filter here in the driver if the user only asked
            // for one direction interrupts. To do this, we need to know what
            // the user asked for.
            self.save_pin_interrupt_state(pin_number, true, direction);

            // Setup interrupt configs that are true of all interrupts
            buffer[0] = self.calc_register_addr(Registers::IntCon, 0);
            // Set all of the IntCon registers to zero.
            let mut i: usize = 1;
            for _ in 0..(self.number_of_banks as usize) {
                buffer[i] = 0; // Make all pins toggle on every change.
                i += 1;
            }
            // The next register is the IoCon (configuration) register, which
            // we also want to set.
            buffer[i] = 0b00000010; // Make MCP230xx interrupt pin active high.
            self.i2c.write(buffer, (i + 1) as u8);
            self.state.set(State::EnableInterruptSettings(pin_number));

            ReturnCode::SUCCESS
        })
    }

    fn disable_interrupt_pin(&self, pin_number: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::EBUSY, |buffer| {
            self.i2c.enable();

            // Clear this interrupt from our setup.
            self.remove_pin_interrupt_state(pin_number);

            // Just have to write the new interrupt settings.
            buffer[0] = self.calc_register_addr(Registers::GpIntEn, pin_number);
            buffer[1] = self.get_pin_interrupt_enabled_state(pin_number);
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
        direction: gpio::InterruptEdge,
    ) {
        // Set the enabled bitmap.
        let mut current_enabled = self.interrupts_enabled.get();
        // Clear out existing settings
        current_enabled &= !(1 << pin_number);
        // Set new value
        current_enabled |= (enabled as u32) << pin_number;
        self.interrupts_enabled.set(current_enabled);

        // Set the direction bitmap.
        let mut current_mode = self.interrupts_mode.get();
        // Clear out existing settings
        current_mode &= !(0x03 << (2 * pin_number));
        // Generate new settings
        let new_settings = (direction as u32) & 0x03;
        // Update settings
        current_mode |= new_settings << (2 * pin_number);
        self.interrupts_mode.set(current_mode);
    }

    fn remove_pin_interrupt_state(&self, pin_number: u8) {
        let new_enabled = self.interrupts_enabled.get() & !(1 << pin_number);
        self.interrupts_enabled.set(new_enabled);
        let new_mode = self.interrupts_mode.get() & !(0x03 << (2 * pin_number));
        self.interrupts_mode.set(new_mode);
    }

    /// Create an 8 bit bitmask of which interrupts are enabled.
    fn get_pin_interrupt_enabled_state(&self, pin_number: u8) -> u8 {
        let offset = (pin_number / self.bank_size) * self.bank_size;
        let interrupts_enabled = self.interrupts_enabled.get();
        (interrupts_enabled >> offset) as u8
    }

    fn check_pin_interrupt_enabled(&self, pin_number: u8) -> bool {
        (self.interrupts_enabled.get() >> pin_number) & 0x01 == 0x01
    }

    fn get_pin_interrupt_direction(&self, pin_number: u8) -> gpio::InterruptEdge {
        let direction = self.interrupts_mode.get() >> (pin_number * 2) & 0x03;
        match direction {
            0 => gpio::InterruptEdge::RisingEdge,
            1 => gpio::InterruptEdge::FallingEdge,
            _ => gpio::InterruptEdge::EitherEdge,
        }
    }
}

impl hil::i2c::I2CClient for MCP230xx<'_> {
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
                buffer[0] = self.calc_register_addr(Registers::IoDir, pin_number);
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectIoDirForGpPu(pin_number, enabled) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadIoDirForGpPu(pin_number, enabled));
            }
            State::ReadIoDirForGpPu(pin_number, enabled) => {
                // Make sure the pin is enabled.
                buffer[1] = buffer[0] | (1 << pin_number);
                buffer[0] = self.calc_register_addr(Registers::IoDir, pin_number);
                self.i2c.write(buffer, 2);
                self.state.set(State::SetIoDirForGpPu(pin_number, enabled));
            }
            State::SetIoDirForGpPu(pin_number, enabled) => {
                buffer[0] = self.calc_register_addr(Registers::GpPu, pin_number);
                self.i2c.write(buffer, 1);
                self.state.set(State::ReadGpPu(pin_number, enabled));
            }
            State::ReadGpPu(pin_number, enabled) => {
                // Configure the pullup status and save it in the buffer.
                let pullup = match enabled {
                    true => buffer[0] | (1 << pin_number),
                    false => buffer[0] & !(1 << pin_number),
                };
                buffer[0] = self.calc_register_addr(Registers::GpPu, pin_number);
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
                buffer[0] = self.calc_register_addr(Registers::Gpio, pin_number);
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectGpioToggle(pin_number) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpioToggle(pin_number));
            }
            State::ReadGpioToggle(pin_number) => {
                buffer[1] = buffer[0] ^ (1 << pin_number);
                buffer[0] = self.calc_register_addr(Registers::Gpio, pin_number);
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::SelectGpioRead(pin_number) => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpioRead(pin_number));
            }
            State::ReadGpioRead(pin_number) => {
                let pin_value = (buffer[0] >> pin_number) & 0x01;

                self.client.map(|client| {
                    client.done(pin_value as usize);
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::EnableInterruptSettings(pin_number) => {
                // Rather than read the current interrupts and write those
                // back, just write the entire register with our saved state.
                buffer[0] = self.calc_register_addr(Registers::GpIntEn, pin_number);
                buffer[1] = self.get_pin_interrupt_enabled_state(pin_number);
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::ReadInterruptSetup(bank_number) => {
                // Now read the interrupt flags and the state of the lines
                self.i2c.read(buffer, 3);
                self.state.set(State::ReadInterruptValues(bank_number));
            }
            State::ReadInterruptValues(bank_number) => {
                let interrupt_flags = buffer[0];
                let pins_status = buffer[2];
                // Check each bit to see if that pin triggered an interrupt.
                for i in 0..8 {
                    // Calculate the actual pin number based on which bank we
                    // are examining.
                    let pin_number = i + (bank_number * self.bank_size);
                    // Check that this pin is actually enabled.
                    if !self.check_pin_interrupt_enabled(pin_number) {
                        continue;
                    }
                    if (interrupt_flags >> i) & 0x01 == 0x01 {
                        // Use the GPIO register to determine which way the
                        // interrupt went.
                        let pin_status = (pins_status >> i) & 0x01;
                        let interrupt_direction = self.get_pin_interrupt_direction(pin_number);
                        // Check to see if this was an interrupt we want
                        // to report.
                        let fire_interrupt = match interrupt_direction {
                            gpio::InterruptEdge::EitherEdge => true,
                            gpio::InterruptEdge::RisingEdge => pin_status == 0x01,
                            gpio::InterruptEdge::FallingEdge => pin_status == 0x00,
                        };
                        if fire_interrupt {
                            // Signal this interrupt to the application.
                            self.client.map(|client| {
                                // Return both the pin that interrupted and
                                // the identifier that was passed for
                                // enable_interrupt.
                                client.fired(pin_number as usize, 0);
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
                self.client.map(|client| {
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

impl gpio::ClientWithValue for MCP230xx<'_> {
    fn fired(&self, value: u32) {
        if value < 2 {
            return; // Error, value specifies which pin A=0, B=1
        }
        self.buffer.take().map(|buffer| {
            let bank_number = value;
            self.i2c.enable();

            // Need to read the IntF register which marks which pins
            // interrupted.
            buffer[0] =
                self.calc_register_addr(Registers::IntF, bank_number as u8 * self.bank_size);
            self.i2c.write(buffer, 1);
            self.state.set(State::ReadInterruptSetup(bank_number as u8));
        });
    }
}

impl gpio_async::Port for MCP230xx<'_> {
    fn disable(&self, pin: usize) -> ReturnCode {
        // Best we can do is make this an input.
        self.set_direction(pin as u8, Direction::Input)
    }

    fn make_output(&self, pin: usize) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        self.set_direction(pin as u8, Direction::Output)
    }

    fn make_input(&self, pin: usize, mode: gpio::FloatingState) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        match mode {
            gpio::FloatingState::PullUp => self.configure_pullup(pin as u8, true),
            gpio::FloatingState::PullDown => {
                // No support for this
                self.configure_pullup(pin as u8, false)
            }
            gpio::FloatingState::PullNone => self.configure_pullup(pin as u8, false),
        }
    }

    fn read(&self, pin: usize) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        self.read_pin(pin as u8)
    }

    fn toggle(&self, pin: usize) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        self.toggle_pin(pin as u8)
    }

    fn set(&self, pin: usize) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        self.set_pin(pin as u8, PinState::High)
    }

    fn clear(&self, pin: usize) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        self.set_pin(pin as u8, PinState::Low)
    }

    fn enable_interrupt(&self, pin: usize, mode: gpio::InterruptEdge) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        let ret = self.enable_host_interrupt();
        match ret {
            ReturnCode::SUCCESS => self.enable_interrupt_pin(pin as u8, mode),
            _ => ret,
        }
    }

    fn disable_interrupt(&self, pin: usize) -> ReturnCode {
        if pin > ((self.number_of_banks * self.bank_size) - 1) as usize {
            return ReturnCode::EINVAL;
        }
        self.disable_interrupt_pin(pin as u8)
    }

    fn is_pending(&self, _pin: usize) -> bool {
        false
    }
}
