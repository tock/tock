/// This driver presents a simple interface to the APDS9960 Proximity, Gesture, and Color Sensor IC
/// This driver follows the generic proximity sensor interface for allowing user-space applications to read proximity measurements
/// and set the proximity gain on the IC
/// Function Implementations for setting the Proximity Pulse Count/Length and LED current are also present for reference but are not part of the driver interface


use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::ReturnCode;

// I2C Buffer of 16 bytes
pub static mut BUFFER: [u8; 16] = [0, 0, 0 , 0, 0, 0, 0 , 0, 0, 0, 0 , 0, 0, 0, 0 , 175 ];

#[allow(dead_code)]

// Common Register Masks
const PON: u8 = 1<<0; // Power-On
const SAI: u8 = 1<<4; // Sleep after Interrupt
const PEN: u8 = 1<<2; // Proximity Sensor Enable
const PIEN: u8 = 1<<5; // Proximity Sensor Enable


// Default Proximity Parameters (can be modified for user preference)
static LOW_THRESH : u8 = 0;
static HIGH_THRESH : u8 = 175;
static PERS : u8 = 4;


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

}

// States
#[derive(Clone, Copy, PartialEq)]
enum State {
    ReadId,
    StartingProximity,
    ConfiguringProximity1,
    ConfiguringProximity2,
    ConfiguringProximity3,
    SendSAI,     // Send sleep-after-interrupt bit to Config3 reg
    PowerOn,     // Send sensor activation and power on info to device
    Idle,        // Waiting for Data (interrupt)
    PowerOff,    // Sending power off command to device (to latch values in device data registers)
    ReadData,    // Read data from reg
    SetPgain, // Set Prox Gain
    SetPulse, // Set proximity pulse
    SetLdrive, // Set LED Current for Prox and ALS sensors
    Done, // Final state for take_measurement() state sequence

    TakeMeasurement1,
    TakeMeasurement2,
    TakeMeasurement3,
}

pub struct APDS9960<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin,
    prox_callback: OptionalCell<&'a dyn kernel::hil::sensors::ProximityClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> APDS9960<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin,
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
    pub fn read_id(&self) {
        self.buffer.take().map(|buffer| {
            
            self.i2c.enable();

            buffer[0] = Registers::ID as u8;
            self.i2c.write_read(buffer, 1, 1);

            self.state.set(State::ReadId); // Reading ID
        });
    }

    // Set Proximity Pulse Count and Length(1 = default)
    pub fn set_proximity_pulse(&self , mut length : u8 , mut count : u8){

        self.buffer.take().map(|buffer| {

            self.i2c.enable();

            if length > 3{
                length = 3;
            }
            if count > 63{
                count = 63;
            }


            buffer[0] = Registers::PROXPULSEREG as u8;
            buffer[1] = ( length<<6  | count  ) as u8;
            self.i2c.write(buffer , 2);

            self.state.set(State::SetPulse); // Send pulse control command to device

        });

    }

    // Set Proximity Gain (0 to 3)
    pub fn set_proximity_gain(&self , mut gain : u8){

        self.buffer.take().map(|buffer| {

            self.i2c.enable();

            if gain > 3{
                gain = 3;
            }

            buffer[0] = Registers::CONTROLREG1 as u8;
            buffer[1] = (gain<<2) as u8;
            self.i2c.write(buffer , 2);

            self.state.set(State::SetPgain); // Send gain command to device

        });

    }

    // Set LED Current Strength (0 -> 100 mA , 3 --> 12.5 mA)
    pub fn set_ldrive(&self , mut ldrive : u8){

        self.buffer.take().map(|buffer| {

            self.i2c.enable();

            if ldrive > 3{
                ldrive = 3;
            }

            buffer[0] = Registers::CONTROLREG1 as u8;
            buffer[1] = (ldrive<<6) as u8;
            self.i2c.write(buffer , 2);

            self.state.set(State::SetLdrive); // Send LED Current Control gain

        });


    }

    // Set proximity interrupt thresholds
    pub fn set_proximity_interrupt_thresholds(&self , low : u8 , high : u8){

        self.buffer.take().map(|buffer| {

            buffer[14] = low;
            buffer[15] = high;

        });


    }

    // Take measurement immediately
    pub fn take_measurement(&self){

        self.buffer.take().map(|buffer| {

            self.i2c.enable();

            buffer[0] = Registers::ENABLE as u8;
            buffer[1] = PON | PEN;

            self.i2c.write(buffer , 2);

            self.state.set(State::TakeMeasurement1);

        });

    }
    

    // Take Simple proximity measurement with persistence=4, low threshold=0, high threshold=175, and Sleep-After-Interrupt Mode enabled
    pub fn take_measurement_on_interrupt(&self) {
        
        // Configure interrupt pin
        self.interrupt_pin.make_input();
        self.interrupt_pin
            .set_floating_state(gpio::FloatingState::PullUp);
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::FallingEdge);

        self.buffer.take().map(|buffer| {
            // Set the device to Sleep-After-Interrupt Mode
            self.i2c.enable();

            buffer[0] = Registers::CONFIG3 as u8;
            buffer[1] = SAI;
            self.i2c.write(buffer, 2);

            self.state.set(State::SendSAI);
        });
    }
}

impl i2c::I2CClient for APDS9960<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {

        debug!("Reading Proximity Data: {:#x} , {:#x}", buffer[0] , buffer[1]);
        
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
                buffer[1] = (PERS)<<4;
                self.i2c.write(buffer,2);
                self.state.set(State::StartingProximity);
            }
            State::StartingProximity => {
                // Set low prox thresh to 0
                buffer[0] = Registers::PILT as u8;
                buffer[1] = buffer[14];
                self.i2c.write(buffer, 2);
                self.state.set(State::ConfiguringProximity1);
            }
            State::ConfiguringProximity1 => {
                // Set high prox thresh to 175
                buffer[0] = Registers::PIHT as u8;
                buffer[1] = buffer[15];
                self.i2c.write(buffer, 2);
                self.state.set(State::ConfiguringProximity2);
            }
            State::ConfiguringProximity2 => {
                // Clear proximity interrupt.
                buffer[0] = Registers::PICCLR as u8;
                self.i2c.write(buffer, 1);
                self.state.set(State::ConfiguringProximity3);
            }
            State::ConfiguringProximity3 => {
                // Enable Device
                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = PON | PEN | PIEN;
                self.i2c.write(buffer, 2);
                self.state.set(State::PowerOn);
            }
            State::PowerOn => {
                // Go into idle state and wait for interrupt for data
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadData => {
                // read prox_data from buffer and return it in callback
                debug!("Reading Proximity Data: {:#x}", buffer[0]);
                let prox_data: u8 = buffer[0];
                self.prox_callback.map(|cb| cb.callback(prox_data as usize));
                

                // Clear proximity interrupt
                buffer[0] = Registers::PICCLR as u8;
                self.i2c.write(buffer, 1);
                self.interrupt_pin.disable_interrupts();
                self.state.set(State::PowerOff); // Chane state transition to PowerOn for endless loop of interrupts to signal data values
            }
            State::PowerOff => {

                // Deactivate the device
        
                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = 0 as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);

            }
            State::Done => {
                // Return to IDLE
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);

            }
            State::TakeMeasurement1 => {

                buffer[0] = Registers::PDATA as u8;
                self.i2c.write_read(buffer,1,1);

                self.state.set(State::TakeMeasurement2);
            }
            State::TakeMeasurement2 => {

                // read prox_data from buffer and return it in callback
                debug!("Reading Proximity Data: {:#x}", buffer[0]);
                let prox_data: u8 = buffer[0];
                self.prox_callback.map(|cb| cb.callback(prox_data as usize));

                buffer[0] = Registers::ENABLE as u8;
                buffer[1] = 0 as u8;

                self.i2c.write(buffer , 2);

                self.state.set(State::TakeMeasurement3);

            }
            State::TakeMeasurement3 => {
                // Return to IDLE
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }

            State::SetPgain => {
                // Return to IDLE
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
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
        debug!("int fired");
        self.buffer.take().map(|buffer| {
            // Read value in PDATA reg
            self.i2c.enable();
            

            buffer[0] = Registers::PDATA as u8;
            self.i2c.write_read(buffer, 1, 1);
            self.state.set(State::ReadData);
        });
    }
} 

impl<'a> kernel::hil::sensors::ProximityDriver<'a> for APDS9960<'a> {
    fn read_proximity(&self) -> kernel::ReturnCode {
        self.take_measurement();
        ReturnCode::SUCCESS
    }

    fn read_proximity_on_interrupt(&self) -> kernel::ReturnCode {
        self.take_measurement_on_interrupt();
        ReturnCode::SUCCESS
    }

    fn set_proximity_interrupt_thresholds(&self , low : u8 , high : u8) -> kernel::ReturnCode {

        self.set_proximity_interrupt_thresholds(low , high);
        ReturnCode::SUCCESS

    }

    fn set_proximity_gain(&self , gain : u8) -> kernel::ReturnCode {
        self.set_proximity_gain(gain);
        ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'a dyn kernel::hil::sensors::ProximityClient) {
        self.prox_callback.set(client);
    }
}


