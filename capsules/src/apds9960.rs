// Driver for APDS9960 Gesture, Light, and Proximity Sensor for Arduino Nano33 BLE SENSE Board
// Note: Only Proximity Reads are enabled as per this implementation

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::{ ReturnCode };


// I2C Buffer of 16 bytes
pub static mut BUFFER: [u8; 16] = [0; 16];

#[allow(dead_code)]

// Bits to set in registers


const PON : u8 = 0b00000001; // Power-On
const SAI : u8 = 0b00010000; // Sleep after Interrupt
const PEN : u8 = 0b00000100; // Proximity Sensor Enable
const AEN : u8 = 0b00000010; // ALS Sensor Enable
const GEN : u8 = 0b01000000; // Gesture Sensor Enable

enum Registers {

    Enable = 0x80, // Enable register for all 3 sensors
    Configuration3Register = 0x9f, // SAI (Sleep after interrupt is set) bit in here

    PDATA = 0x9c, // Proximity Data

    CDATAL = 0x94, // RBGC Data (must read in 16-bit words as register pairs (low word then high word) starting with even addressed register)
    CDATAH = 0x95,
    RDATAL = 0x96,
    RDATAH = 0x97,
    GDATAL = 0x98,
    GDATAH = 0x99,
    BDATAL = 0x9A,
    BDATAH = 0x9B,

}

/// State Machine Diagram


///     SendSAI        -->    PowerOn     -->      Idle   -->   Int Received   --> PowerOff  --> RequestData ...
/// ^^^(send SAI bit)     Send PON/PEN bits     Wait For Int                      Send !PON       Request PDATA read       


///  --> ReadData   --> Idle
///  ^^^(Read PDATA)   Disable everything and put PDATA into callback()

#[derive(Clone, Copy, PartialEq)]
enum State {

    SendSAI, // Send sleep-after-interrupt bit to Config3 reg
    PowerOn, // Send sensor activation and power on info to device
    Idle, // Waiting for Data (interrupt)
    PowerOff, // Sending power off command to device (to latch values in device data registers)
    RequestData, // Request to read data from PDATA reg
    ReadData, // Read data from reg

   
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

    

    pub fn take_measurement(&self){

        // Enable interrupts
        self.interrupt_pin.make_input();
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::RisingEdge);

        self.buffer.take().map(|buffer|{
            
            // Send Sleep-After-Int bit to Config3Reg
            self.i2c.enable();

            buffer[0] = Registers::Configuration3Register as u8;
            buffer[1] = SAI;
            self.i2c.write(buffer , 2);

            self.state.set(State::SendSAI);

        });

    }

    
}

impl i2c::I2CClient for APDS9960<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::SendSAI => {
                // Send sensor enable and power on bits to enable reg
                buffer[0] = Registers::Enable as u8;
                buffer[1] = PEN |  PON;
                self.i2c.write(buffer , 2);
                self.state.set(State::PowerOn);
            }
            State::PowerOn => {
                // Go into idle state and wait for interrupt for data
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::PowerOff => {
                // Send request to read from PDATA reg
                buffer[0] = Registers::PDATA as u8;
                self.i2c.write(buffer,1);
                self.state.set(State::RequestData);
            }
            State::RequestData => {
                // Read PDATA
                self.i2c.read(buffer,1);
                self.state.set(State::ReadData);
            }
            State::ReadData => {
                // read prox_data from buffer and then disable everything
                let prox_data : u8 = buffer[0];
                self.prox_callback.map(|cb| cb.callback(prox_data as usize));
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.interrupt_pin.disable_interrupts();
                self.state.set(State::Idle);
            }

            _ => {}

        }
    }
}

impl<'a> kernel::hil::sensors::ProximityDriver<'a> for APDS9960<'a>{

    fn read_proximity(&self) -> kernel::ReturnCode {

        self.take_measurement();
        ReturnCode::SUCCESS

    }

    fn set_client(&self , client: &'a dyn kernel::hil::sensors::ProximityClient){
        self.prox_callback.set(client);
    }

}

/// Interrupt Service Routine
impl gpio::Client for APDS9960<'_> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            // Send power off command to device to latch data values
            self.i2c.enable();
            buffer[0] = Registers::Enable as u8;
            buffer[1] = PEN & !PON; // PON --> 1 to 0
            self.i2c.write(buffer,2);
            self.state.set(State::PowerOff);

        });
    }
}


