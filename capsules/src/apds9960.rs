//! Proximity SyscallDriver for the Adafruit APDS9960 gesture/ambient light/proximity sensor.
//!
//! <https://content.arduino.cc/assets/Nano_BLE_Sense_av02-4191en_ds_apds-9960.pdf>   <-- Datasheet
//!
//! > The APDS-9960 device features advanced Gesture detection, Proximity detection, Digital Ambient Light Sense
//! > (ALS) and Color Sense (RGBC). The slim modular package,
//! > L 3.94 x W 2.36 x H 1.35 mm, incorporates an IR LED and
//! > factory calibrated LED driver for drop-in compatibility
//! > with existing footprints
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let apds9960_i2c = static_init!(
//!    capsules::virtual_i2c::I2CDevice,
//!    capsules::virtual_i2c::I2CDevice::new(sensors_i2c_bus, 0x39)
//!);
//!
//!let apds9960 = static_init!(
//!    capsules::apds9960::APDS9960<'static>,
//!    capsules::apds9960::APDS9960::new(
//!        apds9960_i2c,
//!        &nrf52840::gpio::PORT[APDS9960_PIN],
//!        &mut capsules::apds9960::BUFFER
//!    )
//!);
//!apds9960_i2c.set_client(apds9960);
//!nrf52840::gpio::PORT[APDS9960_PIN].set_client(apds9960);

//!let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//!
//!let proximity = static_init!(
//!    capsules::proximity::ProximitySensor<'static>,
//!    capsules::proximity::ProximitySensor::new(apds9960 , board_kernel.create_grant(&grant_cap)));

//!kernel::hil::sensors::ProximityDriver::set_client(apds9960, proximity);
//!
//! ```

use core::cell::Cell;
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

// I2C Buffer of 16 bytes
pub static mut BUFFER: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 175];

// BUFFER Layout:  [0,...  ,   12                            , 13               ,                   14                ,   15]
//                             ^take_meas() callback stored    ^take_meas_int callback stored       ^low thresh           ^high thresh

// Common Register Masks
const PON: u8 = 1 << 0; // Power-On
const SAI: u8 = 1 << 4; // Sleep after Interrupt
const PEN: u8 = 1 << 2; // Proximity Sensor Enable
const PIEN: u8 = 1 << 5; // Proximity Sensor Enable
const PVALID: u8 = 1 << 1; // Proximity Reading Valid Bit

// Default Proximity Int Persistence  (amount of times a prox reading can be within the interrupt-generating range before an int is actually fired;
// this is to prevent false triggers)
static PERS: u8 = 4;

// Device Registers
#[repr(u8)]
enum Registers {
    ENABLE = 0x80,
    ID = 0x92,
    PILT = 0x89,
    PIHT = 0x8B,
    CONFIG3 = 0x9f,
    PICCLR = 0xe5,
    PERS = 0x8c,
    PDATA = 0x9c,
    CONTROLREG1 = 0x8f,
    PROXPULSEREG = 0x8e,
    STATUS = 0x93,
}

// States
#[derive(Clone, Copy, PartialEq)]
enum State {
    ReadId,

    /// States visited in take_measurement_on_interrupt() function
    StartingProximity,
    ConfiguringProximity1,
    ConfiguringProximity2,
    ConfiguringProximity3,
    SendSAI,  // Send sleep-after-interrupt bit to Config3 reg
    PowerOn,  // Send sensor activation and power on info to device
    Idle,     // Waiting for Data (interrupt)
    PowerOff, // Sending power off command to device (to latch values in device data registers)
    ReadData, // Read data from reg

    /// States visited in take_measurement() function
    TakeMeasurement1,
    TakeMeasurement2,
    TakeMeasurement3,
    TakeMeasurement4,

    /// States for optional chip functionality
    SetPulse, // Set proximity pulse
    SetLdrive, // Set LED Current for Prox and ALS sensors
    Done,      // Final state for take_measurement() state sequence
}

pub struct APDS9960<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
    prox_callback: OptionalCell<&'a dyn kernel::hil::sensors::ProximityClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> APDS9960<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
        buffer: &'static mut [u8],
    ) -> APDS9960<'a> {
        // setup and return struct
        APDS9960 {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            prox_callback: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    // Read I2C-based ID of device (should be 0xAB)
    pub fn read_id(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                self.i2c.enable();

                buffer[0] = Registers::ID as u8;

                match self.i2c.write_read(buffer, 1, 1) {
                    Ok(()) => {
                        self.state.set(State::ReadId); // Reading ID
                        Ok(())
                    }
                    Err((err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.i2c.disable();
                        Err(err.into())
                    }
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    // Set Proximity Pulse Count and Length(1 = default)
    pub fn set_proximity_pulse(&self, mut length: u8, mut count: u8) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                self.i2c.enable();

                if length > 3 {
                    length = 3;
                }
                if count > 63 {
                    count = 63;
                }

                buffer[0] = Registers::PROXPULSEREG as u8;
                buffer[1] = (length << 6 | count) as u8;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::SetPulse); // Send pulse control command to device
                        Ok(())
                    }
                    Err((err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.i2c.disable();
                        Err(err.into())
                    }
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    // Set LED Current Strength (0 -> 100 mA , 3 --> 12.5 mA)
    pub fn set_ldrive(&self, mut ldrive: u8) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                self.i2c.enable();

                if ldrive > 3 {
                    ldrive = 3;
                }

                buffer[0] = Registers::CONTROLREG1 as u8;
                buffer[1] = (ldrive << 6) as u8;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::SetLdrive); // Send LED Current Control gain
                        Ok(())
                    }
                    Err((err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.i2c.disable();
                        Err(err.into())
                    }
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    // Take measurement immediately
    pub fn take_measurement(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            // Enable power and proximity sensor
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                self.i2c.enable();

                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = PON | PEN;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::TakeMeasurement1);
                        Ok(())
                    }
                    Err((err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.i2c.disable();
                        Err(err.into())
                    }
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    // Take Simple proximity measurement with interrupt persistence set to 4; `low` and `high` indicate upper interrupt threshold values
    // IC fires interrupt when (prox_reading < low) || (prox_reading > high)
    pub fn take_measurement_on_interrupt(&self, low: u8, high: u8) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            // Set threshold values
            self.buffer.take().map(|buffer| {
                // Save proximity thresholds to buffer unused space
                buffer[14] = low;
                buffer[15] = high;

                self.buffer.replace(buffer);
            });

            // Configure interrupt pin
            self.interrupt_pin.make_input();
            self.interrupt_pin
                .set_floating_state(gpio::FloatingState::PullUp);
            self.interrupt_pin.disable_interrupts();
            self.interrupt_pin
                .enable_interrupts(gpio::InterruptEdge::FallingEdge);

            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                // Set the device to Sleep-After-Interrupt Mode
                self.i2c.enable();

                buffer[0] = Registers::CONFIG3 as u8;
                buffer[1] = SAI;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::SendSAI);
                        Ok(())
                    }
                    Err((err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.i2c.disable();
                        Err(err.into())
                    }
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl i2c::I2CClient for APDS9960<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _status: Result<(), i2c::Error>) {
        match self.state.get() {
            State::ReadId => {
                // The ID is in `buffer[0]`, and should be 0xAB.
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::SendSAI => {
                // Set persistence to 4
                buffer[0] = Registers::PERS as u8;
                buffer[1] = (PERS) << 4;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::StartingProximity);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::StartingProximity => {
                // Set low prox thresh to value in buffer
                buffer[0] = Registers::PILT as u8;
                buffer[1] = buffer[14];

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::ConfiguringProximity1);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::ConfiguringProximity1 => {
                // Set high prox thresh to value in buffer
                buffer[0] = Registers::PIHT as u8;
                buffer[1] = buffer[15];

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::ConfiguringProximity2);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::ConfiguringProximity2 => {
                // Clear proximity interrupt.
                buffer[0] = Registers::PICCLR as u8;

                match self.i2c.write(buffer, 1) {
                    Ok(()) => {
                        self.state.set(State::ConfiguringProximity3);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::ConfiguringProximity3 => {
                // Enable Device
                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = PON | PEN | PIEN;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::PowerOn);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::PowerOn => {
                // Go into idle state and wait for interrupt for data
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadData => {
                // read prox_data from buffer and return it in callback
                buffer[13] = buffer[0]; // save callback to an unused place in buffer

                // Clear proximity interrupt
                buffer[0] = Registers::PICCLR as u8;

                match self.i2c.write(buffer, 1) {
                    Ok(()) => {
                        self.interrupt_pin.disable_interrupts();
                        self.state.set(State::PowerOff);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::PowerOff => {
                // Deactivate the device

                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = 0 as u8;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::Done);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::Done => {
                // Return to IDLE and perform callback
                let prox_data: u8 = buffer[13];

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);

                self.prox_callback.map(|cb| cb.callback(prox_data as u8));
            }
            State::TakeMeasurement1 => {
                // Read status reg
                buffer[0] = Registers::STATUS as u8;

                match self.i2c.write_read(buffer, 1, 1) {
                    Ok(()) => {
                        self.state.set(State::TakeMeasurement2);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::TakeMeasurement2 => {
                // Determine if prox data is valid by checking PVALID bit in status reg

                let status_reg: u8 = buffer[0];

                if status_reg & PVALID > 0 {
                    buffer[0] = Registers::PDATA as u8;

                    match self.i2c.write_read(buffer, 1, 1) {
                        Ok(()) => {
                            self.state.set(State::TakeMeasurement3);
                        }
                        Err((_err, buffer)) => {
                            self.buffer.replace(buffer);
                            self.state.set(State::Idle);
                            self.i2c.disable();
                        }
                    }
                } else {
                    // If not valid then keep rechecking status reg
                    buffer[0] = Registers::STATUS as u8;

                    match self.i2c.write_read(buffer, 1, 1) {
                        Ok(()) => {
                            self.state.set(State::TakeMeasurement2);
                        }
                        Err((_err, buffer)) => {
                            self.buffer.replace(buffer);
                            self.state.set(State::Idle);
                            self.i2c.disable();
                        }
                    }
                }
            }
            State::TakeMeasurement3 => {
                buffer[12] = buffer[0]; // Save callback value

                // Reset callback value
                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = 0;

                match self.i2c.write(buffer, 2) {
                    Ok(()) => {
                        self.state.set(State::TakeMeasurement4);
                    }
                    Err((_err, buffer)) => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                        self.i2c.disable();
                    }
                }
            }
            State::TakeMeasurement4 => {
                // Return to IDLE and perform callback

                let prox_data: u8 = buffer[12]; // Get callback value
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);

                self.prox_callback.map(|cb| cb.callback(prox_data as u8));
            }

            State::SetPulse => {
                // Return to IDLE
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::SetLdrive => {
                // Return to IDLE
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }

            _ => {}
        }
    }
}

/// Interrupt Service Routine
impl gpio::Client for APDS9960<'_> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            // Read value in PDATA reg
            self.i2c.enable();

            buffer[0] = Registers::PDATA as u8;

            match self.i2c.write_read(buffer, 1, 1) {
                Ok(()) => {
                    self.state.set(State::ReadData);
                }
                Err((_err, buffer)) => {
                    self.buffer.replace(buffer);
                    self.i2c.disable();
                }
            }
        });
    }
}

/// Proximity Driver Trait Implementation
impl<'a> kernel::hil::sensors::ProximityDriver<'a> for APDS9960<'a> {
    fn read_proximity(&self) -> Result<(), ErrorCode> {
        self.take_measurement()
    }

    fn read_proximity_on_interrupt(&self, low: u8, high: u8) -> Result<(), ErrorCode> {
        self.take_measurement_on_interrupt(low, high)
    }

    fn set_client(&self, client: &'a dyn kernel::hil::sensors::ProximityClient) {
        self.prox_callback.set(client);
    }
}
