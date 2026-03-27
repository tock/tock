use crate::chip_specifics::clock_constants::ClockConstants;
use crate::chip_specifics::flash::FlashChipSpecific;

pub trait ChipSpecs: ClockConstants + FlashChipSpecific {}

impl<T: ClockConstants + FlashChipSpecific> ChipSpecs for T {}
