pub mod rfc;
pub mod rfcore_driver;
pub mod rfcore_const;
pub mod commands;

pub static mut RFC: rfc::RFCore = rfc::RFCore::new();
pub static mut RADIO: rfcore_driver::Radio = unsafe { rfcore_driver::Radio::new(&RFC) };
