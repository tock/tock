//! Traits and parameters for SPI master communication

use core::option::Option;

#[derive(Copy, Clone)]
pub enum Rate {
    MSBFirst,
    LSBFirst,
}
/// Values for the ordering of bits
#[derive(Copy, Clone)]
pub enum DataOrder {
    MSBFirst,
    LSBFirst,
}

/// Values for the clock polarity (idle state or CPOL)
#[derive(Copy, Clone)]
pub enum ClockPolarity {
    IdleHigh,
    IdleLow,
}
/// Values for the clock phase (CPHA), which defines when
/// values are sampled
#[derive(Copy, Clone)]
pub enum ClockPhase {
    SampleLeading,
    SampleTrailing,
}

pub trait SpiCallback {
    /// Called when a read/write operation finishes
    fn read_write_done(&'static self, 
                       read: Option<&'static mut[u8]>,
                       writer: Option<&'static mut[u8]>);
}

/// Using an SPI implementation normally involves three steps:
///
/// 1. Configure the SPI with the SpiConfig trait 
///   1a. Call set_chip_select to select which peripheral and
///       turn on SPI
///   1b. Call set operations as needed to configure bus
/// 2. Invoke read, write, read_write on SpiMaster 
/// 3a. Call clear_chip_select to turn off bus, or
/// 3b. Call set_chip_select to choose another peripheral 
///
pub trait SpiMaster {
    /// Configures an object for communication as an SPI master
    fn init(&self, client: &'static SpiCallback);
    fn read_write_bytes(&'static self, 
                        mut read: Option<&'static mut [u8]>, 
                        write: Option<&'static mut [u8]>) -> bool;
    fn write_byte(&'static self, val: u8);
    fn read_byte(&'static self) -> u8;
    fn read_write_byte(&'static self, val: u8) -> u8;

    fn set_chip_select(&'static self, cs: u8);
    fn clear_chip_select(&'static self);

    // Returns the actual rate set
    fn set_rate(&self, rate: u32) -> u32;
    fn get_rate(&self) -> u32;

    fn set_order(&self, order: DataOrder);
    fn get_order(&self) -> DataOrder;

    fn set_clock(&self, polarity: ClockPolarity);
    fn get_clock(&self) -> ClockPolarity;

    fn set_phase(&self, phase: ClockPhase);
    fn get_phase(&self) -> ClockPhase;
}
