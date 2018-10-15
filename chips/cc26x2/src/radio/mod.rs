// pub mod ble;
pub mod patch_cpe_prop;
pub mod patch_mce_genfsk;
pub mod patch_mce_longrange;
pub mod patch_rfe_genfsk;
pub mod rfc;
pub mod subghz;
pub mod test_settings;
//pub mod multimode;
use cortexm4::nvic;
use peripheral_interrupts;

pub mod commands;

const RF_ACK_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CMD_ACK as u32) };
const RF_CPE0_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CORE_CPE0 as u32) };
const RF_CPE1_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CORE_CPE1 as u32) };

pub static mut RFC: rfc::RFCore = rfc::RFCore::new(&RF_ACK_NVIC, &RF_CPE0_NVIC, &RF_CPE1_NVIC);
pub static mut RADIO: subghz::Radio = unsafe { subghz::Radio::new(&RFC) };
// pub static mut BLE: ble::Ble = unsafe { ble::Ble::new(&RFC) };
