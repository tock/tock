// pub mod ble;
pub mod rfc;
pub mod subghz;
// use cortexm4::{self, nvic};
// use peripheral_interrupts;

pub mod commands;

//pub static RFACK_NVIC: nvic::Nvic = unsafe { nvic::Nvic::new(peripheral_interrupts::RF_CMD_ACK) };
//pub static CPE0_NVIC: nvic::Nvic = unsafe { nvic::Nvic::new(peripheral_interrupts::RF_CORE_CPE0) };
pub static mut RFC: rfc::RFCore = rfc::RFCore::new();

pub static mut RADIO: subghz::Radio = unsafe { subghz::Radio::new(&RFC) };
// pub static mut BLE: ble::Ble = unsafe { ble::Ble::new(&RFC) };
